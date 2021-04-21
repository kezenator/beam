use crate::bsdf::{Bsdf, Lambertian, Phong};
use crate::camera::Camera;
use crate::color::LinearRGB;
use crate::intersection::{Face, ObjectIntersection, ShadingIntersection};
use crate::lighting::LightingRegion;
use crate::material::MaterialInteraction;
use crate::math::{EPSILON, Scalar, ScalarConsts};
use crate::object::Object;
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::vec::{Dir3, Point3, RefractResult, bsdf_reflect, bsdf_refract_or_reflect};

#[derive(Copy, Clone)]
pub enum SamplingMode
{
    Uniform,
    BsdfOnly,
    LightsOnly,
    BsdfAndLights,
}

pub enum ScatteringResult
{
    Emit{ emitted_color: LinearRGB, probability: Scalar },
    Trace{ attenuation_color: LinearRGB, next_dir: Dir3, probability: Scalar },
    Scatter{ attenuation_color: LinearRGB, bsdf: Box<dyn Bsdf> },
}

impl ScatteringResult
{
    pub fn emit(emitted_color: LinearRGB, probability: Scalar) -> Self
    {
        ScatteringResult::Emit{ emitted_color, probability }
    }

    pub fn trace(attenuation_color: LinearRGB, next_dir: Dir3, probability: Scalar) -> Self
    {
        ScatteringResult::Trace{ attenuation_color, next_dir, probability }
    }

    pub fn scatter(attenuation_color: LinearRGB, bsdf: Box<dyn Bsdf>) -> Self
    {
        ScatteringResult::Scatter{ attenuation_color, bsdf }
    }
}

pub trait ScatteringFunction
{
    fn max_rays() -> usize;
    fn scatter_ray(scene: &Scene, intersection: &ShadingIntersection, material_interaction: MaterialInteraction, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> ScatteringResult;
    fn termination_contdition(attenuation: LinearRGB) -> LinearRGB;
}

#[derive(Clone, Copy)]
pub struct SceneSampleStats
{
    pub num_samples: u64,
    pub num_rays: u64,
    pub max_rays: usize,
    pub stopped_due_to_max_rays: u64,
    pub stopped_due_to_min_atten: u64,
    pub stopped_due_to_min_prob: u64,
}

impl SceneSampleStats
{
    pub fn new() -> Self
    {
        SceneSampleStats
        {
            num_samples: 0,
            num_rays: 0,
            max_rays: 0,
            stopped_due_to_max_rays: 0,
            stopped_due_to_min_atten: 0,
            stopped_due_to_min_prob: 0,
        }
    }

    pub fn to_short_debug_string(&self) -> String
    {
        format!("Rays/Sample: [{:.2} avg, {:.2} max] Early-Exit: [{:.2}% max rays, {:.2}% min color, {:.2}% min prob]",
            (self.num_rays as Scalar) / (self.num_samples as Scalar),
            self.max_rays,
            100.0 * (self.stopped_due_to_max_rays as Scalar) / (self.num_samples as Scalar),
            100.0 * (self.stopped_due_to_min_atten as Scalar) / (self.num_samples as Scalar),
            100.0 * (self.stopped_due_to_min_prob as Scalar) / (self.num_samples as Scalar))
    }
}

impl std::ops::Add for SceneSampleStats
{
    type Output = SceneSampleStats;

    fn add(self, rhs: SceneSampleStats) -> Self::Output
    {
        SceneSampleStats
        {
            num_samples: self.num_samples + rhs.num_samples,
            num_rays: self.num_rays + rhs.num_rays,
            max_rays: self.max_rays.max(rhs.max_rays),
            stopped_due_to_max_rays: self.stopped_due_to_max_rays + rhs.stopped_due_to_max_rays,
            stopped_due_to_min_atten: self.stopped_due_to_min_atten + rhs.stopped_due_to_min_atten,
            stopped_due_to_min_prob: self.stopped_due_to_min_prob + rhs.stopped_due_to_min_prob,
        }
    }
}

pub struct Scene
{
    sampling_mode: SamplingMode,
    camera: Camera,
    lighting_regions: Vec<LightingRegion>,
    objects: Vec<Object>,
}

impl Scene
{
    pub fn new(sampling_mode: SamplingMode, camera: Camera, lighting_regions: Vec<LightingRegion>, objects: Vec<Object>) -> Self
    {
        Scene { sampling_mode, camera, lighting_regions, objects }
    }

    pub fn path_trace_global_lighting(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> LinearRGB
    {
        let ray = self.camera.get_ray(u, v);

        self.path_trace::<GlobalLighting>(ray, sampler, stats)
    }

    pub fn path_trace_local_lighting(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> LinearRGB
    {
        let ray = self.camera.get_ray(u, v);

        self.path_trace::<LocalLighting>(ray, sampler, stats)
    }

    pub fn path_trace<S: ScatteringFunction>(&self, ray: Ray, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> LinearRGB
    {
        stats.num_samples += 1;

        let mut cur_ray = ray;
        let mut cur_attenuation = LinearRGB::white();
        let mut cur_probability = 1.0;

        for ray_num in 0..S::max_rays()
        {
            stats.num_rays += 1;

            if (ray_num + 1) > stats.max_rays
            {
                stats.max_rays = ray_num + 1;
            }

            match self.trace_intersection(&cur_ray)
            {
                Some(intersection) =>
                {
                    let shading_intersection = intersection.surface.into();
                    let material_interaction = intersection.material.get_surface_interaction(&shading_intersection);

                    match S::scatter_ray(&self, &shading_intersection, material_interaction, sampler, stats)
                    {
                        ScatteringResult::Scatter{ attenuation_color, bsdf } =>
                        {
                            let (scatter_dir, reflectance, probability) = self.scatter(&shading_intersection, bsdf, sampler);

                            cur_ray = Ray::new(shading_intersection.location, scatter_dir);
                            cur_attenuation = cur_attenuation.combined_with(&attenuation_color.multiplied_by_scalar(reflectance));
                            cur_probability *= probability;
                        },
                        ScatteringResult::Trace{ attenuation_color, next_dir, probability } =>
                        {
                            cur_ray = Ray::new(shading_intersection.location, next_dir);
                            cur_attenuation = cur_attenuation.combined_with(&attenuation_color);
                            cur_probability *= probability;
                        },
                        ScatteringResult::Emit{ emitted_color, probability } =>
                        {
                            // We've reached an emitting surface - return
                            // the total contribution

                            let final_probability = cur_probability * probability;

                            return emitted_color.combined_with(&cur_attenuation).divided_by_scalar(final_probability);
                        },
                    }
                },
                None =>
                {
                    // This ray doens't hit any objects -
                    // there's nothing to see

                    return LinearRGB::black();
                },
            }

            // Check for some extra termination conditions

            if cur_attenuation.max_color_component() < 1.0e-4
            {
                // The current attenuation has dropped below what can be seen
                // even with a 12-bit HDR monitor.
                // What may be occuring is that a ray keeps bouncing around,
                // the attenuation keeps building, the probability keeps getting
                // lower. Eventually we have a sample that's a random color
                // close to black, with a very low probability.
                // This gets divided to a very bright color.
                // Instead - just terminate early and return a real black.

                stats.stopped_due_to_min_atten += 1;

                return LinearRGB::black();
            }

            if cur_probability < 1.0e-6
            {
                // The current probability is getting really small.
                // This means this sample gets multipled by a big amount
                // and has a big effect on the final image. This creates
                // bright specks of noise.

                stats.stopped_due_to_min_prob += 1;

                return LinearRGB::black();
            }
        }

        // We've traced down too many levels of rays!
        // Ask the scattering function what
        // termination condition they want

        stats.stopped_due_to_max_rays += 1;

        S::termination_contdition(cur_attenuation).divided_by_scalar(cur_probability)
    }

    pub fn trace_intersection<'r, 'm>(&'m self, ray: &'r Ray) -> Option<ObjectIntersection<'r, 'm>>
    {
        let mut range = RayRange::new(EPSILON, Scalar::MAX);
        let mut closest = None;

        for obj in self.objects.iter()
        {
            if let Some(intersection) = obj.closest_intersection_in_range(ray, &range)
            {
                range.set_max(intersection.surface.distance);
                closest = Some(intersection);
            }
        }

        closest
    }

    pub fn get_lighting_region_at(&self, location: Point3) -> Option<&LightingRegion>
    {
        self.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(location)).nth(0)
    }

    fn scatter(&self, intersection: &ShadingIntersection, bsdf: Box<dyn Bsdf>, sampler: &mut Sampler) -> (Dir3, Scalar, Scalar)
    {
        let (scatter_dir, probability) = match self.sampling_mode
        {
            SamplingMode::Uniform =>
            {
                (sampler.uniform_dir_on_unit_sphere(), 0.25 * ScalarConsts::FRAC_1_PI)
            },
            SamplingMode::BsdfOnly =>
            {
                bsdf.generate_random_sample_dir_and_calc_pdf(sampler)
            },
            SamplingMode::LightsOnly =>
            {
                match self.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(intersection.location)).nth(0)
                {
                    Some(lighting_region) =>
                    {
                        let num_lights = lighting_region.global_surfaces.len();
            
                        let sampled_index = sampler.uniform_index(num_lights);
        
                        let (dir, mut prob) = lighting_region.global_surfaces[sampled_index].generate_random_sample_direction_from_and_calc_pdf(intersection.location, sampler);
        
                        let sampled_ray = Ray::new(intersection.location, dir);
        
                        for sum_index in 0..num_lights
                        {
                            if sum_index != sampled_index
                            {
                                prob += lighting_region.global_surfaces[sum_index].calculate_pdf_for_ray(&sampled_ray);
                            }
                        }
        
                        (dir, prob / (num_lights as Scalar))
                    },
                    None =>
                    {
                        // No light sampling information - revert to uniform sampling
            
                        (sampler.uniform_dir_on_unit_sphere(), 0.25 * ScalarConsts::FRAC_1_PI)
                    },
                }
            },
            SamplingMode::BsdfAndLights =>
            {
                match self.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(intersection.location)).nth(0)
                {
                    Some(lighting_region) =>
                    {
                        let light_prob = 0.5;
                        let bsdf_prob = 1.0 - light_prob;
            
                        let num_lights = lighting_region.global_surfaces.len();
            
                        if sampler.uniform_scalar_unit() < light_prob
                        {
                            // Sample in the direction the lights suggest
            
                            let sampled_index = sampler.uniform_index(num_lights);
            
                            let (dir, mut prob) = lighting_region.global_surfaces[sampled_index].generate_random_sample_direction_from_and_calc_pdf(intersection.location, sampler);
            
                            let sampled_ray = Ray::new(intersection.location, dir);
            
                            for sum_index in 0..num_lights
                            {
                                if sum_index != sampled_index
                                {
                                    prob += lighting_region.global_surfaces[sum_index].calculate_pdf_for_ray(&sampled_ray);
                                }
                            }
            
                            let prob = (light_prob * prob / (num_lights as Scalar))
                                + (bsdf_prob * bsdf.calculate_pdf_for_dir(dir));
            
                            (dir, prob)
                        }
                        else
                        {
                            // Sample in the direction suggested by the BSDF
            
                            let (dir, prob) = bsdf.generate_random_sample_dir_and_calc_pdf(sampler);
            
                            let sampled_ray = Ray::new(intersection.location, dir);
            
                            let prob = (bsdf_prob * prob)
                                + (light_prob * lighting_region.global_surfaces.iter().map(|s| s.calculate_pdf_for_ray(&sampled_ray)).sum::<Scalar>());
            
                            (dir, prob)
                        }
                    },
                    None =>
                    {
                        // No light sampling information - revert to BSDF sampling
            
                        bsdf.generate_random_sample_dir_and_calc_pdf(sampler)
                    }
                }
            },
        };
    
        let reflectance = bsdf.reflectance(scatter_dir);

        (scatter_dir, reflectance, probability)
    }
}

struct GlobalLighting
{
}

impl ScatteringFunction for GlobalLighting
{
    fn max_rays() -> usize
    {
        50
    }

    fn scatter_ray(_scene: &Scene, intersection: &ShadingIntersection, material_interaction: MaterialInteraction, sampler: &mut Sampler, _stats: &mut SceneSampleStats) -> ScatteringResult
    {
        match material_interaction
        {
            MaterialInteraction::Diffuse{ diffuse_color } =>
            {
                ScatteringResult::scatter(
                    diffuse_color,
                    Box::new(Lambertian::new(intersection)))
            },
            MaterialInteraction::Reflection{ attenuate_color, fuzz } =>
            {
                ScatteringResult::scatter(
                    attenuate_color,
                    Box::new(Phong::new(intersection, 0.2, 0.8, 5.0 / fuzz)))
            },
            MaterialInteraction::Refraction{ ior } =>
            {
                let refraction_ratio = if intersection.face == Face::Front
                {
                    1.0 / ior
                }
                else
                {
                    ior
                };

                match bsdf_refract_or_reflect(intersection.incoming, intersection.normal, refraction_ratio)
                {
                    RefractResult::TotalInternalReflection{ reflect_dir } =>
                    {
                        ScatteringResult::trace(LinearRGB::white(), reflect_dir, 1.0)
                    },
                    RefractResult::ReflectOrRefract{ refract_dir, reflect_dir, reflect_probability } =>
                    {
                        let (dir, probability) = if sampler.uniform_scalar_unit() < reflect_probability
                        {
                            (reflect_dir, reflect_probability)
                        }
                        else
                        {
                            (refract_dir, 1.0 - reflect_probability)
                        };

                        ScatteringResult::trace(LinearRGB::grey(probability), dir, probability)
                    },
                }
            },
            MaterialInteraction::Emit{ emitted_color } =>
            {
                // The object is emitting light - return it and no scattering
                // is required

                ScatteringResult::emit(emitted_color, 1.0)
            },
        }
    }

    fn termination_contdition(_attenuation: LinearRGB) -> LinearRGB
    {
        // This ray has not been able to find an emitting surface -
        // we can't return any light to the camera

        LinearRGB::black()
    }
}

struct LocalLighting
{
}

impl ScatteringFunction for LocalLighting
{
    fn max_rays() -> usize
    {
        5
    }

    fn scatter_ray(scene: &Scene, intersection: &ShadingIntersection, material_interaction: MaterialInteraction, _sampler: &mut Sampler, stats: &mut SceneSampleStats) -> ScatteringResult
    {
        match material_interaction
        {
            MaterialInteraction::Diffuse{ diffuse_color } =>
            {
                // This is the main approximation to make "local lighting"
                // muct raster to render.
                // Instead of scattering from diffuse surfaces,
                // we apply the local Phong model,
                // and then Emit that light back towards the camera.

                ScatteringResult::emit(Phong::local_shading(scene, intersection, diffuse_color, 0.1, 0.6, diffuse_color, 0.3, 20.0, stats), 1.0)
            },
            MaterialInteraction::Reflection{ attenuate_color, .. } =>
            {
                ScatteringResult::trace(attenuate_color, bsdf_reflect(intersection.incoming, intersection.normal), 1.0)
            },
            MaterialInteraction::Refraction{ ior } =>
            {
                let refraction_ratio = if intersection.face == Face::Front
                {
                    1.0 / ior
                }
                else
                {
                    ior
                };

                let new_dir = match bsdf_refract_or_reflect(intersection.incoming, intersection.normal, refraction_ratio)
                {
                    RefractResult::TotalInternalReflection{ reflect_dir } => reflect_dir,
                    RefractResult::ReflectOrRefract{ refract_dir, .. } => refract_dir,
                };

                ScatteringResult::trace(LinearRGB::white(), new_dir, 1.0)
            },
            MaterialInteraction::Emit{ emitted_color } =>
            {
                ScatteringResult::emit(emitted_color, 1.0)
            },
        }
    }

    fn termination_contdition(attenuation: LinearRGB) -> LinearRGB
    {
        // For local lighting, best results are obtained
        // through multiple reflections if we assume
        // the final ray hits a fully lit surface.
        // Otherwise complicated areas just go black

        attenuation
    }
}
