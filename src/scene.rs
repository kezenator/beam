use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Object;
use crate::geom::{Plane, Sphere};
use crate::texture::Texture;

pub struct Scene
{
    camera: Camera,
    lights: Vec<Light>,
    objects: Vec<Object>,
}

impl Scene
{
    pub fn new(camera: Camera, lights: Vec<Light>, objects: Vec<Object>) -> Self
    {
        Scene { camera, lights, objects }
    }

    pub fn new_default() -> Self
    {
        let sphere = |centre: Point3, radius: Scalar, color: RGBA| -> Object
        {
            Object::new(
                Sphere::new(centre, radius),
                Material::diffuse(Texture::solid(color)))
        };

        let metal_sphere = |centre: Point3, radius: Scalar, color: RGBA| -> Object
        {
            Object::new(
                Sphere::new(centre, radius),
                Material::metal(Texture::solid(color), 0.2))
        };

        let glass_sphere = |centre: Point3, radius: Scalar| -> Object
        {
            Object::new(
                Sphere::new(centre, radius),
                Material::dielectric(1.5))
        };

        let plane = |point: Point3, normal: Dir3, color: RGBA| -> Object
        {
            Object::new(
                Plane::new(point, normal),
                Material::metal(Texture::checkerboard(color, RGBA::new(1.0, 1.0, 1.0, 1.0)), 0.2))
        };

        Self::new(
            Camera::new(),
            vec![
                Light::ambient(RGBA::new(0.1, 0.1, 0.1, 1.0)),
                Light::directional(RGBA::new(0.9, 0.9, 0.9, 1.0), Dir3::new(0.0, 0.0, -1.0)),
            ],
            vec![
                // White sphere at the origin
                sphere(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(0.8, 0.8, 0.8, 1.0)),

                // Red, green and blue ones around it
                sphere(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(0.8, 0.0, 0.8, 1.0)),
                sphere(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.8, 0.0, 1.0)),
                sphere(Point3::new(-2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.0, 0.8, 1.0)),

                // Grey sphere below
                sphere(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.5, 0.5, 0.5, 1.0)),

                // Metal spheres
                metal_sphere(Point3::new(2.50, -2.0, 1.0), 1.25, RGBA::new(0.8, 0.5, 0.8, 1.0)),

                // Glass sphere
                glass_sphere(Point3::new(-1.5, -2.0, 1.5), 1.25),

                // Ground
                plane(Point3::new(0.0, -3.5, 0.0), Dir3::new(0.0, 1.0, 0.0), RGBA::new(0.2, 0.2, 0.2, 1.0)),

                // Wall behind
                sphere(Point3::new(0.0, 0.0, -13.0), 10.0, RGBA::new(0.5, 0.584, 0.929, 1.0)),
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
            .filter(|i| i.surface.distance >= EPSILON)
            .collect::<Vec<_>>();

        intersections.sort_unstable_by(|a, b| a.surface.distance.partial_cmp(&b.surface.distance).unwrap());

        match intersections.iter().nth(0)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material to calculate a
                // random scattering ray and attenuation.

                match intersection.material.scatter(sampler, &intersection.surface)
                {
                    Some((scatter_ray, attenuation_color)) =>
                    {
                        // There's a scattering ray - cast this ray
                        // to find the incoming light, and then attenuate
                        // based on the material's color

                        let scattering_light = self.cast_ray(&scatter_ray, depth + 1, sampler);

                        scattering_light.combined_with(&attenuation_color)
                    },
                    None =>
                    {
                        // This material doesn't scatter.
                        // There's no incoming light

                        RGBA::new(0.0, 0.0, 0.0, 1.0)
                    },
                }
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
