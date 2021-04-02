use crate::math::{EPSILON, Scalar};
use crate::vec::Point3;
use crate::color::RGBA;
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Sphere;

pub struct Scene
{
    camera: Camera,
    objects: Vec<Sphere>,
}

impl Scene
{
    pub fn new(camera: Camera, objects: Vec<Sphere>) -> Self
    {
        Scene { camera, objects }
    }

    pub fn new_default() -> Self
    {
        Self::new(
            Camera::new(),
            vec![
                Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(1.0, 1.0, 1.0, 1.0)),
                Sphere::new(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 1.0, 0.0, 1.0)),
                Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.0, 1.0, 1.0)),
                Sphere::new(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(1.0, 0.0, 0.0, 1.0)),
                Sphere::new(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.5, 0.5, 0.5, 1.0)),
                Sphere::new(Point3::new(0.0, 0.0, -10.0), 5.0, RGBA::new(0.5, 0.584, 0.929, 1.0)),
            ])
    }

    pub fn sample_pixel(&self, u: Scalar, v: Scalar, sampler: &mut Sampler) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        self.cast_ray(&ray, 0, sampler)
    }

    fn cast_ray(&self, ray: &Ray, depth: usize, sampler: &mut Sampler) -> RGBA
    {
        if depth > 50
        {
            // We've recursed too deep - we've bounced around
            // so many times we can assume that none of the light
            // from here will make a visible difference to the final image

            return RGBA::new(0.0, 0.0, 0.0, 1.0);
        }

        let mut intersections = Vec::new();

        for obj in self.objects.iter()
        {
            obj.get_intersections(&ray, &mut intersections);
        }

        let mut intersections = intersections
            .drain(..)
            .filter(|i| i.distance >= EPSILON)
            .collect::<Vec<_>>();

        intersections.sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        match intersections.iter().nth(0)
        {
            Some(intersection) =>
            {
                // We've hit an object. Calculate a random
                // diffuse scattering ray, calculcate the light
                // from this ray, and then absorb light based
                // on the object's color.

                let scatter_dir = intersection.normal + sampler.uniform_dir_on_unit_sphere();

                let scatter_ray = Ray::new(intersection.location, scatter_dir);

                let scatter_color = self.cast_ray(&scatter_ray, depth + 1, sampler);

                intersection.color.combined_with(&scatter_color)
            },
            None =>
            {
                // No intersections - for now we have no lights,
                // so we need to assume that there's infinite amounts
                // of white light coming in from all directions
                
                RGBA::new(1.0, 1.0, 1.0, 1.0)
            },
        }
    }
}
