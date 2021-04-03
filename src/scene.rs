use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;
use crate::light::Light;
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Sphere;

pub struct Scene
{
    camera: Camera,
    lights: Vec<Light>,
    objects: Vec<Sphere>,
}

impl Scene
{
    pub fn new(camera: Camera, lights: Vec<Light>, objects: Vec<Sphere>) -> Self
    {
        Scene { camera, lights, objects }
    }

    pub fn new_default() -> Self
    {
        Self::new(
            Camera::new(),
            vec![
                Light::ambient(RGBA::new(0.1, 0.1, 0.1, 1.0)),
                Light::directional(RGBA::new(0.9, 0.9, 0.9, 1.0), Dir3::new(0.0, 0.0, -1.0)),
            ],
            vec![
                // White sphere at the origin
                Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(0.8, 0.8, 0.8, 1.0)),

                // Red, green and blue ones around it
                Sphere::new(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(0.8, 0.0, 0.0, 1.0)),
                Sphere::new(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.8, 0.0, 1.0)),
                Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.0, 0.8, 1.0)),

                // Grey sphere below
                Sphere::new(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.5, 0.5, 0.5, 1.0)),

                // Ground
                Sphere::new(Point3::new(0.0, -100.0, 0.0), 95.0, RGBA::new(0.2, 0.2, 0.2, 1.0)),

                // Wall behind
                Sphere::new(Point3::new(0.0, 0.0, -13.0), 10.0, RGBA::new(0.5, 0.584, 0.929, 1.0)),
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
                // No intersections - query the lights
                // to see how much light they are providing

                let mut color = RGBA::new(0.0, 0.0, 0.0, 1.0);

                for light in self.lights.iter()
                {
                    color = color + light.get_light(ray);
                }
                
                color
            },
        }
    }
}
