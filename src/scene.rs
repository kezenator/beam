use crate::material::MaterialInteraction;
use crate::math::{EPSILON, Scalar};
use crate::vec::Dir3;
use crate::color::RGBA;
use crate::intersection::{Face, ObjectIntersection, SurfaceIntersection};
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Object;
use crate::lighting::LightingRegion;

pub enum ScatteringResult
{
    Emit(RGBA),
    AttenuateNext(RGBA, Dir3),
}

pub trait ScatteringFunction
{
    fn max_rays() -> usize;
    fn scatter_ray<'r>(scene: &Scene, intersection: &'r SurfaceIntersection<'r>, material_interaction: MaterialInteraction, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> ScatteringResult;
    fn termination_contdition(attenuation: RGBA) -> RGBA;
}

pub struct SceneSampleStats
{
    pub num_rays: u64,
}

impl SceneSampleStats
{
    pub fn new() -> Self
    {
        SceneSampleStats { num_rays: 0 }
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
        let mut cur_ray = ray;
        let mut cur_attenuation = RGBA::new(1.0, 1.0, 1.0, 1.0);

        for _ in 0..S::max_rays()
        {
            stats.num_rays += 1;

            match self.trace_intersection(&cur_ray)
            {
                Some(intersection) =>
                {
                    let material_interaction = intersection.material.get_surface_interaction(&intersection.surface);

                    match S::scatter_ray(&self, &intersection.surface, material_interaction, sampler, stats)
                    {
                        ScatteringResult::AttenuateNext(attenuation_color, next_dir) =>
                        {
                            cur_ray = Ray::new(intersection.surface.location(), next_dir);
                            cur_attenuation = cur_attenuation.combined_with(&attenuation_color);

                            continue;
                        },
                        ScatteringResult::Emit(emitted_color) =>
                        {
                            // We've reached an emitting surface - return
                            // the total contribution

                            return emitted_color.combined_with(&cur_attenuation);
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

        S::termination_contdition(cur_attenuation)
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

    fn scatter_ray<'r>(_scene: &Scene, intersection: &'r SurfaceIntersection<'r>, material_interaction: MaterialInteraction, sampler: &mut Sampler, _stats: &mut SceneSampleStats) -> ScatteringResult
    {
        match material_interaction
        {
            MaterialInteraction::Diffuse{ diffuse_color } =>
            {
                let scatter_dir = intersection.normal + sampler.uniform_dir_on_unit_sphere();

                ScatteringResult::AttenuateNext(diffuse_color, scatter_dir)
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
                    ScatteringResult::AttenuateNext(attenuate_color, reflect_dir)
                }
                else
                {
                    // Degenerate reflection
                    ScatteringResult::Emit(RGBA::new(0.0, 0.0, 0.0, 1.0))
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

                let new_dir = refract_or_reflect(unit_direction, intersection.normal, refraction_ratio, sampler.uniform_scalar_unit());

                ScatteringResult::AttenuateNext(RGBA::new(1.0, 1.0, 1.0, 1.0), new_dir)
            },
            MaterialInteraction::Emit{ emited_color } =>
            {
                // The object is emitting light - return it and no scattering
                // is required

                ScatteringResult::Emit(emited_color)
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
                // must raster to render.
                // Instead of scattering from diffuse surfaces,
                // we explicitly see which lights are visible,
                // apply a basic Phong local lighting model,
                // and then Emit that light back towards the camera.

                // Ambient

                let mut sum = diffuse_color.multiplied_by_scalar(0.2);

                // Shadow rays, for lights in the first lighting region that
                // covers the intersection location

                let int_location = intersection.location();

                if let Some(lighting_region) = scene.lighting_regions.iter().filter(|lr| lr.covered_volume.is_point_inside(int_location)).nth(0)
                {
                    // The factor for each light is 0.8 (totall diffuse component) / num lights

                    let diffuse_factor = 0.8 / (lighting_region.local_points.len() as f64);

                    for local_light_loc in lighting_region.local_points.iter()
                    {
                        let light_dir = (local_light_loc - int_location).normalized();
                        let geom_factor = intersection.normal.dot(light_dir);
                        if geom_factor > 0.0
                        {
                            stats.num_rays += 1;

                            if let Some(shadow_int) = scene.trace_intersection(&Ray::new(int_location, light_dir))
                            {
                                if let MaterialInteraction::Emit{ emited_color } = shadow_int.material.get_surface_interaction(&shadow_int.surface)
                                {
                                    // Our shadow ray has hit an emitting surface - add a diffuse
                                    // component calculated from this light.
                                    // Clamp color as the extra light required by the global mode is not needed.

                                    sum = sum + diffuse_color.combined_with(&emited_color.clamped()).multiplied_by_scalar(diffuse_factor * geom_factor);
                                }
                            }
                        }
                    }
                }

                ScatteringResult::Emit(sum)
            },
            MaterialInteraction::Reflection{ attenuate_color, .. } =>
            {
                ScatteringResult::AttenuateNext(attenuate_color, reflect(intersection.ray.dir, intersection.normal))
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

                let new_dir = refract_or_reflect(unit_direction, intersection.normal, refraction_ratio, 1.0);

                ScatteringResult::AttenuateNext(RGBA::new(1.0, 1.0, 1.0, 1.0), new_dir)
            },
            MaterialInteraction::Emit{ emited_color } =>
            {
                ScatteringResult::Emit(emited_color)
            },
        }
    }

    fn termination_contdition(attenuation: RGBA) -> RGBA
    {
        // For local coloring, best results are obtained for
        // reflective surfaces if we just assume they are fully lit.

        attenuation
    }
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

fn refract_or_reflect(incoming: Dir3, normal: Dir3, refraction_ratio: Scalar, no_reflection_chance: Scalar) -> Dir3
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics

    let cos_theta = normal.dot(-incoming).min(1.0);

    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    if cannot_refract
        || reflectance(cos_theta, refraction_ratio) > no_reflection_chance
    {
        reflect(incoming, normal)
    }
    else
    {
        let r_out_perp =  refraction_ratio * (incoming + cos_theta*normal);
        let r_out_parallel = -(1.0 - r_out_perp.magnitude_squared()).abs().sqrt() * normal;

        r_out_perp + r_out_parallel
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
