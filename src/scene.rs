use crate::bsdf::{Bsdf, Lambertian};
use crate::camera::Camera;
use crate::color::RGBA;
use crate::intersection::{Face, ObjectIntersection, SurfaceIntersection};
use crate::lighting::LightingRegion;
use crate::material::MaterialInteraction;
use crate::math::{EPSILON, Scalar};
use crate::object::Object;
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::vec::Dir3;

pub enum ScatteringResult
{
    Emit{ emitted_color: RGBA, probability: Scalar },
    Scatter{ attenuation_color: RGBA, scatter_dir: Dir3, probability: Scalar },
}

impl ScatteringResult
{
    pub fn emit(emitted_color: RGBA, probability: Scalar) -> Self
    {
        ScatteringResult::Emit{ emitted_color, probability }
    }

    pub fn scatter(attenuation_color: RGBA, scatter_dir: Dir3, probability: Scalar) -> Self
    {
        ScatteringResult::Scatter{ attenuation_color, scatter_dir, probability }
    }
}

pub trait ScatteringFunction
{
    fn max_rays() -> usize;
    fn scatter_ray<'r>(scene: &Scene, intersection: &'r SurfaceIntersection<'r>, material_interaction: MaterialInteraction, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> ScatteringResult;
    fn termination_contdition(attenuation: RGBA) -> RGBA;
}

#[derive(Clone, Copy)]
pub struct SceneSampleStats
{
    pub num_samples: u64,
    pub num_rays: u64,
    pub max_rays: usize,
    pub stopped_due_to_max_rays: u64,
    pub stopped_due_to_min_atten: u64,
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
        }
    }

    pub fn to_short_debug_string(&self) -> String
    {
        format!("Rays/Sample: [{:.2} avg, {:.2} max] Early-Exit: [{:.2}% max rays, {:.2}% min color]",
            (self.num_rays as Scalar) / (self.num_samples as Scalar),
            self.max_rays,
            100.0 * (self.stopped_due_to_max_rays as Scalar) / (self.num_samples as Scalar),
            100.0 * (self.stopped_due_to_min_atten as Scalar) / (self.num_samples as Scalar))
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
        }
    }
}

pub struct Scene
{
    camera: Camera,
    lighting_regions: Vec<LightingRegion>,
    objects: Vec<Object>,
}

impl Scene
{
    pub fn new(camera: Camera, lighting_regions: Vec<LightingRegion>, objects: Vec<Object>) -> Self
    {
        Scene { camera, lighting_regions, objects }
    }

    pub fn path_trace_global_lighting(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.path_trace::<GlobalLighting>(ray, sampler, stats)
    }

    pub fn path_trace_local_lighting(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.path_trace::<LocalLighting>(ray, sampler, stats)
    }

    pub fn path_trace<S: ScatteringFunction>(&self, ray: Ray, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        stats.num_samples += 1;

        let mut cur_ray = ray;
        let mut cur_attenuation = RGBA::new(1.0, 1.0, 1.0, 1.0);
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
                    let material_interaction = intersection.material.get_surface_interaction(&intersection.surface);

                    match S::scatter_ray(&self, &intersection.surface, material_interaction, sampler, stats)
                    {
                        ScatteringResult::Scatter{ attenuation_color, scatter_dir, probability } =>
                        {
                            cur_ray = Ray::new(intersection.surface.location(), scatter_dir);
                            cur_attenuation = cur_attenuation.combined_with(&attenuation_color);
                            cur_probability *= probability;

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

                                return RGBA::new(0.0, 0.0, 0.0, 1.0);
                            }

                            continue;
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

                    return RGBA::new(0.0, 0.0, 0.0, 1.0);
                },
            }
        }

        // We've traced down too many levels of rays!
        // Ask the scattering function what
        // termination condition they want

        stats.stopped_due_to_max_rays += 1;

        S::termination_contdition(cur_attenuation).divided_by_scalar(cur_probability)
    }

    fn trace_intersection<'r, 'm>(&'m self, ray: &'r Ray) -> Option<ObjectIntersection<'r, 'm>>
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

    fn scatter_ray<'r>(scene: &Scene, intersection: &'r SurfaceIntersection<'r>, material_interaction: MaterialInteraction, sampler: &mut Sampler, _stats: &mut SceneSampleStats) -> ScatteringResult
    {
        match material_interaction
        {
            MaterialInteraction::Diffuse{ diffuse_color } =>
            {
                let lambertian = Lambertian::new(intersection.normal);

                let location = intersection.location();

                let (scatter_dir, probability) = match scene.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(location)).nth(0)
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

                            let (dir, mut prob) = lighting_region.global_surfaces[sampled_index].generate_random_sample_direction_from_and_calc_pdf(location, sampler);

                            let sampled_ray = Ray::new(intersection.location(), dir);

                            for sum_index in 0..num_lights
                            {
                                if sum_index != sampled_index
                                {
                                    prob += lighting_region.global_surfaces[sum_index].calculate_pdf_for_ray(&sampled_ray);
                                }
                            }

                            let prob = (light_prob * prob / (num_lights as Scalar))
                                + (bsdf_prob * lambertian.calculate_pdf_for_dir(dir));

                            (dir, prob)
                        }
                        else
                        {
                            // Sample in the direction suggested by the BSDF

                            let (dir, prob) = lambertian.generate_random_sample_direction_and_calc_pdf(sampler);

                            let sampled_ray = Ray::new(intersection.location(), dir);

                            let prob = (bsdf_prob * prob)
                                + (light_prob * lighting_region.global_surfaces.iter().map(|s| s.calculate_pdf_for_ray(&sampled_ray)).sum::<Scalar>());

                            (dir, prob)
                        }
                    },
                    None =>
                    {
                        // No light sampling information - we can only
                        // sample by the surface

                        lambertian.generate_random_sample_direction_and_calc_pdf(sampler)
                    }
                };

                if scatter_dir.dot(intersection.normal) <= 0.0
                {
                    ScatteringResult::emit(
                        RGBA::new(0.0, 0.0, 0.0, 1.0),
                        probability)
                }
                else
                {
                    ScatteringResult::scatter(
                        diffuse_color.multiplied_by_scalar(scatter_dir.dot(intersection.normal)),
                        scatter_dir,
                        probability)
                }
            },
            MaterialInteraction::Reflection{ attenuate_color, fuzz } =>
            {
                // Reflect incoming ray about the normal

                let reflect_dir = reflect(intersection.ray.dir, intersection.normal);

                // Add in some fuzzyness to the reflected ray

                let reflect_dir =
                    reflect_dir.normalized()
                    + (fuzz * sampler.uniform_point_in_unit_sphere());

                // With this fuzzyness, glancing rays or large
                // fuzzyness can cause the reflected ray to point
                // inside the object.

                if reflect_dir.dot(intersection.normal) > EPSILON
                {
                    ScatteringResult::scatter(attenuate_color, reflect_dir, 1.0)
                }
                else
                {
                    // Degenerate reflection
                    ScatteringResult::emit(RGBA::new(0.0, 0.0, 0.0, 1.0), 1.0)
                }
            },
            MaterialInteraction::Refraction{ ior } =>
            {
                let refraction_ratio = if intersection.face == Face::FrontFace
                {
                    1.0 / ior
                }
                else
                {
                    ior
                };

                let unit_direction = intersection.ray.dir.normalized();

                match refract_or_reflect(unit_direction, intersection.normal, refraction_ratio)
                {
                    RefractResult::TotalInternalReflection{ reflect_dir } =>
                    {
                        ScatteringResult::scatter(RGBA::new(1.0, 1.0, 1.0, 1.0), reflect_dir, 1.0)
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

                        ScatteringResult::scatter(RGBA::new(probability, probability, probability, 1.), dir, probability)
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

    fn termination_contdition(_attenuation: RGBA) -> RGBA
    {
        // This ray has not been able to find an emitting surface -
        // we can't return any light to the camera

        RGBA::new(0.0, 0.0, 0.0, 1.0)
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

    fn scatter_ray<'r>(scene: &Scene, intersection: &'r SurfaceIntersection<'r>, material_interaction: MaterialInteraction, _sampler: &mut Sampler, stats: &mut SceneSampleStats) -> ScatteringResult
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

                ScatteringResult::emit(phong(scene, intersection, diffuse_color, 0.1, 0.6, diffuse_color, 0.3, 20.0, stats), 1.0)
            },
            MaterialInteraction::Reflection{ attenuate_color, .. } =>
            {
                ScatteringResult::scatter(attenuate_color, reflect(intersection.ray.dir, intersection.normal), 1.0)
            },
            MaterialInteraction::Refraction{ ior } =>
            {
                let refraction_ratio = if intersection.face == Face::FrontFace
                {
                    1.0 / ior
                }
                else
                {
                    ior
                };

                let unit_direction = intersection.ray.dir.normalized();

                let new_dir = match refract_or_reflect(unit_direction, intersection.normal, refraction_ratio)
                {
                    RefractResult::TotalInternalReflection{ reflect_dir } => reflect_dir,
                    RefractResult::ReflectOrRefract{ refract_dir, .. } => refract_dir,
                };

                ScatteringResult::scatter(RGBA::new(1.0, 1.0, 1.0, 1.0), new_dir, 1.0)
            },
            MaterialInteraction::Emit{ emitted_color } =>
            {
                ScatteringResult::emit(emitted_color, 1.0)
            },
        }
    }

    fn termination_contdition(attenuation: RGBA) -> RGBA
    {
        // For local lighting, best results are obtained
        // through multiple reflections if we assume
        // the final ray hits a fully lit surface.
        // Otherwise complicated areas just go black

        attenuation
    }
}

fn phong<'r>(scene: &Scene, intersection: &'r SurfaceIntersection<'r>, diffuse_color: RGBA, ka: Scalar, kd: Scalar, specular_color: RGBA, ks: Scalar, shininess: Scalar, stats: &mut SceneSampleStats) -> RGBA
{
    let mut result = diffuse_color.multiplied_by_scalar(ka);

    // Shadow rays, for lights in the first lighting region that
    // covers the intersection location

    let int_location = intersection.location();

    if let Some(lighting_region) = scene.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(int_location)).nth(0)
    {
        // Scale effects by the number of lights

        let lights_factor = (lighting_region.local_points.len() as f64).recip();
        let kd = kd * lights_factor;
        let ks = ks * lights_factor;

        for local_light_loc in lighting_region.local_points.iter()
        {
            let light_dir = local_light_loc - int_location;

            if intersection.normal.dot(light_dir) > 0.0
            {
                // The light is in the same direction as the normal - this light
                // can contribute

                let light_dir = light_dir.normalized();

                stats.num_rays += 1;

                if let Some(shadow_int) = scene.trace_intersection(&Ray::new(int_location, light_dir))
                {
                    if let MaterialInteraction::Emit{ emitted_color } = shadow_int.material.get_surface_interaction(&shadow_int.surface)
                    {
                        // Our shadow ray has hit an emitting surface:
                        // 1) Clamp the emitted color - global illumination can need lights "brighter" than 1.0
                        // 2) Add diffuse and specular components as required

                        let emitted_color = emitted_color.clamped();

                        if kd > 0.0
                        {
                            result = result + diffuse_color.combined_with(&emitted_color).multiplied_by_scalar(kd * light_dir.dot(intersection.normal));
                        }

                        if ks > 0.0
                        {
                            let reflected = reflect(light_dir, intersection.normal);

                            let r_dot_v = reflected.dot(intersection.ray.dir.normalized());

                            if r_dot_v > 0.0
                            {
                                result = result + specular_color.combined_with(&emitted_color).multiplied_by_scalar(ks * r_dot_v.powf(shininess));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn reflect(incoming: Dir3, normal: Dir3) -> Dir3
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#metal
    //
    // incoming is the ray coming in.
    // normal is the surface normal.
    //
    // Both must be normalized, and must
    // be in opposite directions (i.e. dot product is negative).
    //
    // incoming.dot(normal) * normal is the component that brings the
    // incoming ray back to perpendicular with the normal.
    // Adding twice this will give the reflection
    
    incoming - ((2.0 * incoming.dot(normal)) * normal)
}

enum RefractResult
{
    ReflectOrRefract{ reflect_dir: Dir3, refract_dir: Dir3, reflect_probability: Scalar },
    TotalInternalReflection{ reflect_dir: Dir3 }
}

fn refract_or_reflect(incoming: Dir3, normal: Dir3, refraction_ratio: Scalar) -> RefractResult
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics

    let cos_theta = normal.dot(-incoming).min(1.0);

    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    if cannot_refract
    {
        RefractResult::TotalInternalReflection
        {
            reflect_dir: reflect(incoming, normal),
        }
    }
    else
    {
        let r_out_perp =  refraction_ratio * (incoming + cos_theta*normal);
        let r_out_parallel = -(1.0 - r_out_perp.magnitude_squared()).abs().sqrt() * normal;

        let refract_dir = r_out_perp + r_out_parallel;
        let reflect_dir = reflect(incoming, normal);
        let reflect_probability = reflectance(cos_theta, refraction_ratio);

        RefractResult::ReflectOrRefract
        {
            reflect_dir,
            refract_dir,
            reflect_probability
        }
    }
}


fn reflectance(cosine: Scalar, ref_idx: Scalar) -> Scalar
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
    // Use Schlick's approximation for reflectance.
    
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
