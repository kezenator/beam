use crate::material::LocalLightingModel;
use crate::math::{EPSILON, Scalar};
use crate::vec::Point3;
use crate::color::RGBA;
use crate::intersection::ObjectIntersection;
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Object;

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

    pub fn path_trace_pixel(&self, u: Scalar, v: Scalar, sampler: &mut Sampler) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.cast_ray_global(&ray, 0, sampler)
    }

    pub fn quick_trace_pixel(&self, u: Scalar, v: Scalar, sampler: &mut Sampler) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.cast_ray_local(&ray, 0, sampler)
    }

    fn cast_ray_global(&self, ray: &Ray, depth: usize, sampler: &mut Sampler) -> RGBA
    {
        if depth > 50
        {
            // We've recursed too deep - we've bounced around
            // so many times we can assume that none of the light
            // from here will make a visible difference to the final image

            return RGBA::new(0.0, 0.0, 0.0, 1.0);
        }

        match self.trace_intersection(ray)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material to calculate
                // 1) Any emmission (e.g. from a light source)
                // 2) A random scattering ray and attenuation

                let emission = intersection.material.emmission(sampler, &intersection.surface);

                let scatter = match intersection.material.scatter(sampler, &intersection.surface)
                {
                    Some((scatter_ray, attenuation_color)) =>
                    {
                        // There's a scattering ray - cast this ray
                        // to find the incoming light, and then attenuate
                        // based on the material's color

                        let scattering_light = self.cast_ray_global(&scatter_ray, depth + 1, sampler);

                        scattering_light.combined_with(&attenuation_color)
                    },
                    None =>
                    {
                        // This material doesn't scatter.
                        // There's no incoming light

                        RGBA::new(0.0, 0.0, 0.0, 1.0)
                    },
                };

                emission + scatter
            },
            None =>
            {
                // No intersections! It's dark here!

                RGBA::new(0.0, 0.0, 0.0, 1.0)
            },
        }
    }

    fn cast_ray_local(&self, ray: &Ray, depth: usize, sampler: &mut Sampler) -> RGBA
    {
        match self.trace_intersection(ray)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material to execute the local
                // lighting model

                match intersection.material.local(&intersection.surface)
                {
                    LocalLightingModel::Diffuse(color) =>
                    {
                        // A diffuse surface.
                        // Start with some ambient light, and cast shadow rays

                        // Ambient

                        let mut sum = color.multiplied_by_scalar(0.2);

                        // Shadow rays

                        let int_location = intersection.surface.location();
                        let one_on_num = (self.local_light_positions.len() as f64).recip();

                        for local_light_loc in self.local_light_positions.iter()
                        {
                            let light_dir = (local_light_loc - int_location).normalized();
                            let diffuse = intersection.surface.normal.dot(light_dir);
                            if diffuse > 0.0
                            {
                                if let Some(shadow_int) = self.trace_intersection(&Ray::new(int_location, light_dir))
                                {
                                    if let LocalLightingModel::Emit(emit) = shadow_int.material.local(&shadow_int.surface)
                                    {
                                        // Our shadow ray has hit an emitting surface - add a diffuse
                                        // component calculated from this light

                                        sum = sum + color.combined_with(&emit).multiplied_by_scalar(0.8 * one_on_num * diffuse);
                                    }
                                }
                            }
                        }

                        sum
                    },
                    LocalLightingModel::Attenuate(color, next_ray) =>
                    {
                        // Reflection/Refraction
                        // Just return the color if we've done too many rays,
                        // else cast another ray and attenuate

                        if depth >= 5
                        {
                            color
                        }
                        else
                        {
                            color.combined_with(&self.cast_ray_local(&next_ray, depth + 1, sampler))
                        }
                    },
                    LocalLightingModel::Emit(color) =>
                    {
                        // Just return the emitted color
                        color
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
