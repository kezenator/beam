use crate::material::MaterialInteraction;
use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;
use crate::intersection::{Face, ObjectIntersection};
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Object;

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
    local_light_positions: Vec<Point3>,
    objects: Vec<Object>,
}

impl Scene
{
    pub fn new(camera: Camera, local_light_positions: Vec<Point3>, objects: Vec<Object>) -> Self
    {
        Scene { camera, local_light_positions, objects }
    }

    pub fn path_trace_pixel(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.cast_ray_global(&ray, 0, sampler, stats)
    }

    pub fn quick_trace_pixel(&self, u: Scalar, v: Scalar, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.cast_ray_local(&ray, 0, sampler, stats)
    }

    fn cast_ray_global(&self, ray: &Ray, depth: usize, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        if depth > 50
        {
            // We've recursed too deep - we've bounced around
            // so many times we can assume that none of the light
            // from here will make a visible difference to the final image

            return RGBA::new(0.0, 0.0, 0.0, 1.0);
        }

        stats.num_rays += 1;

        match self.trace_intersection(ray)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material restults to execute
                // the local lighting model

                match intersection.material.get_surface_interaction(&intersection.surface)
                {
                    MaterialInteraction::Diffuse{ diffuse_color } =>
                    {
                        let location = intersection.surface.location();

                        let scatter_dir = intersection.surface.normal + sampler.uniform_dir_on_unit_sphere();
        
                        let scatter_ray = Ray::new(location, scatter_dir);
        
                        diffuse_color.combined_with(&self.cast_ray_global(&scatter_ray, depth + 1, sampler, stats))
                    },
                    MaterialInteraction::Reflection{ attenuate_color, fuzz } =>
                    {
                        let location = intersection.surface.location();

                        // Reflect incoming ray about the normal
        
                        let reflect_dir = reflect(intersection.surface.ray.dir, intersection.surface.normal);
        
                        // Add in some fuzzyness to the reflected ray
        
                        let reflect_dir =
                            reflect_dir.normalized()
                            + (fuzz * sampler.uniform_point_in_unit_sphere());
        
                        // With this fuzzyness, glancing rays or large
                        // fuzzyness can cause the reflected ray to point
                        // inside the object.
        
                        if reflect_dir.dot(intersection.surface.normal) > EPSILON
                        {
                            let scatter_ray = Ray::new(location, reflect_dir);

                            attenuate_color.combined_with(&self.cast_ray_global(&scatter_ray, depth + 1, sampler, stats))
                        }
                        else
                        {
                            // Degenerate reflection
                            RGBA::new(0.0, 0.0, 0.0, 1.0)
                        }
                    },
                    MaterialInteraction::Refraction{ ior } =>
                    {
                        let refraction_ratio = if intersection.surface.face == Face::FrontFace
                        {
                            1.0 / ior
                        }
                        else
                        {
                            ior
                        };
        
                        let unit_direction = intersection.surface.ray.dir.normalized();
        
                        let new_dir = refract_or_reflect(unit_direction, intersection.surface.normal, refraction_ratio, sampler.uniform_scalar_unit());
        
                        let new_ray = Ray::new(intersection.surface.location(), new_dir);
        
                        self.cast_ray_local(&new_ray, depth + 1, sampler, stats)
                    },
                    MaterialInteraction::Emit{ emited_color } =>
                    {
                        // The object is emitting light - return it and no scattering
                        // is required

                        emited_color
                    },
                }
            },
            None =>
            {
                // No intersections! It's dark here!

                RGBA::new(0.0, 0.0, 0.0, 1.0)
            },
        }
    }

    fn cast_ray_local(&self, ray: &Ray, depth: usize, sampler: &mut Sampler, stats: &mut SceneSampleStats) -> RGBA
    {
        stats.num_rays += 1;

        match self.trace_intersection(ray)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material restults to execute
                // the local lighting model

                match intersection.material.get_surface_interaction(&intersection.surface)
                {
                    MaterialInteraction::Diffuse{ diffuse_color } =>
                    {
                        // A diffuse surface.
                        // Start with some ambient light, and cast shadow rays

                        // Ambient

                        let mut sum = diffuse_color.multiplied_by_scalar(0.2);

                        // Shadow rays

                        let int_location = intersection.surface.location();
                        let one_on_num = (self.local_light_positions.len() as f64).recip();

                        for local_light_loc in self.local_light_positions.iter()
                        {
                            let light_dir = (local_light_loc - int_location).normalized();
                            let diffuse_factor = intersection.surface.normal.dot(light_dir);
                            if diffuse_factor > 0.0
                            {
                                stats.num_rays += 1;

                                if let Some(shadow_int) = self.trace_intersection(&Ray::new(int_location, light_dir))
                                {
                                    if let MaterialInteraction::Emit{ emited_color } = shadow_int.material.get_surface_interaction(&shadow_int.surface)
                                    {
                                        // Our shadow ray has hit an emitting surface - add a diffuse
                                        // component calculated from this light.
                                        // Clamp color as the extra light required by the global mode is not needed.

                                        sum = sum + diffuse_color.combined_with(&emited_color.clamped()).multiplied_by_scalar(0.8 * one_on_num * diffuse_factor);
                                    }
                                }
                            }
                        }

                        sum
                    },
                    MaterialInteraction::Reflection{ attenuate_color, .. } =>
                    {
                        // Just return the color if we've done too many rays,
                        // else cast another ray and attenuate.

                        if depth >= 5
                        {
                            attenuate_color
                        }
                        else
                        {
                            let reflected_dir = reflect(intersection.surface.ray.dir, intersection.surface.normal);
                            let reflected_ray = Ray::new(intersection.surface.location(), reflected_dir);

                            let reflected_color = self.cast_ray_local(&reflected_ray, depth + 1, sampler, stats);

                            attenuate_color.combined_with(&reflected_color)
                        }
                    },
                    MaterialInteraction::Refraction{ ior } =>
                    {
                        // Just return black if we've done too many rays,
                        // else cast another ray

                        if depth >= 5
                        {
                            RGBA::new(0.0, 0.0, 0.0, 1.0)
                        }
                        else
                        {
                            let refraction_ratio = if intersection.surface.face == Face::FrontFace
                            {
                                1.0 / ior
                            }
                            else
                            {
                                ior
                            };
            
                            let unit_direction = intersection.surface.ray.dir.normalized();
            
                            let new_dir = refract_or_reflect(unit_direction, intersection.surface.normal, refraction_ratio, 1.0);
            
                            let new_ray = Ray::new(intersection.surface.location(), new_dir);
            
                            self.cast_ray_local(&new_ray, depth + 1, sampler, stats)
                        }
                    },
                    MaterialInteraction::Emit{ emited_color } =>
                    {
                        emited_color
                    },
                }
            },
            None =>
            {
                // No intersections - just return a
                // dull background

                RGBA::new(0.2, 0.2, 0.2, 1.0)
            },
        }
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
