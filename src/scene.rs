use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;
use crate::intersection::ObjectIntersection;
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::camera::Camera;
use crate::object::Object;
use crate::geom::{Plane, Sphere, Rectangle, Blob, BlobPart};
use crate::texture::Texture;

pub struct Scene
{
    camera: Camera,
    lights: Vec<Light>,
    local_light: Point3,
    objects: Vec<Object>,
}

impl Scene
{
    pub fn new(camera: Camera, lights: Vec<Light>, local_light: Point3, objects: Vec<Object>) -> Self
    {
        Scene { camera, lights, local_light, objects }
    }

    pub fn new_default(width: u32, height: u32) -> Self
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

        let rectangle = |point: Point3, u: Point3, v: Point3| -> Object
        {
            Object::new(
                Rectangle::new(point, u, v),
                Material::diffuse(Texture::solid(RGBA::new(0.7, 0.7, 0.7, 1.0))))
        };

        let plane = |point: Point3, normal: Dir3, color: RGBA| -> Object
        {
            Object::new(
                Plane::new(point, normal),
                Material::metal(Texture::checkerboard(color, RGBA::new(1.0, 1.0, 1.0, 1.0)), 0.2))
        };

        let cloud = |center: Point3, radius: Scalar, blob_rad: Scalar, num: usize, color: RGBA| -> Object
        {
            let mut sampler = Sampler::new_reproducable(0xBAD5EED5DEADBEEFu64);
            let mut merge = crate::geom::csg::Merge::new();

            for _ in 0..num
            {
                let blob_center = center + (radius - blob_rad) * sampler.uniform_dir_on_unit_sphere();
                
                merge.push(Sphere::new(blob_center, blob_rad));
            }

            let bounds = Sphere::new(center, radius);

            Object::new(
                crate::geom::bounds::BoundedSurface::new(bounds, merge),
                Material::diffuse(Texture::solid(color)))
        };

        let blob = |center: Point3, color: RGBA| -> Object
        {
            let spacing = 1.5;

            let parts = vec![
                BlobPart{ center: center, radius: 1.0 },
                BlobPart{ center: center + Point3::new(0.0, spacing, 0.0), radius: 1.0 },
                BlobPart{ center: center + Point3::new(0.707 * spacing, 0.5 * spacing, 0.0), radius: 1.0 },
            ];
            
            Object::new(
                Blob::new(parts, 0.25),
                Material::diffuse(Texture::solid(color)))
        };

        Self::new(
            Camera::new(width, height),
            vec![
                Light::ambient(RGBA::new(0.1, 0.1, 0.1, 1.0)),
                Light::directional(RGBA::new(0.9, 0.9, 0.9, 1.0), Dir3::new(0.0, 0.0, -1.0)),
            ],
            // Local light
            Point3::new(1.0, 6.0, 12.0),
            // Objects
            vec![
                // White sphere at the origin
                sphere(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(0.8, 0.8, 0.8, 1.0)),

                // Red, green and blue ones around it
                sphere(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(0.8, 0.0, 0.8, 1.0)),
                sphere(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.8, 0.0, 1.0)),
                sphere(Point3::new(-2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.0, 0.8, 1.0)),

                // Grey sphere below
                sphere(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.5, 0.5, 0.5, 1.0)),

                // Some more interesting stuff
                cloud(Point3::new(2.5, 2.5, 2.0), 1.2, 0.2, 400, RGBA::new(0.8, 0.6, 0.3, 1.0)),
                blob(Point3::new(-2.5, 2.5, 1.0), RGBA::new(0.8, 0.6, 0.3, 1.0)),

                // Rectangular "walls"
                rectangle(Point3::new(4.0, -3.0, -1.5), Point3::new(0.0, 0.0, 4.0), Point3::new(0.0, 4.0, 0.0)),
                rectangle(Point3::new(4.0, -3.0, -1.5), Point3::new(-4.0, 0.0, 0.0), Point3::new(0.0, 4.0, 0.0)),

                // Metal spheres
                metal_sphere(Point3::new(2.50, -2.0, 1.0), 1.25, RGBA::new(0.8, 0.5, 0.8, 1.0)),

                // Glass sphere
                glass_sphere(Point3::new(-1.5, -2.0, 1.5), 1.25),

                // Ground
                plane(Point3::new(0.0, -3.51, 0.0), Dir3::new(0.0, 1.0, 0.0), RGBA::new(0.2, 0.2, 0.2, 1.0)),

                // Wall behind
                sphere(Point3::new(0.0, 0.0, -13.0), 10.0, RGBA::new(0.5, 0.584, 0.929, 1.0)),
            ])
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
                // Use the object's material to calculate a
                // random scattering ray and attenuation.

                match intersection.material.scatter(sampler, &intersection.surface)
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

    fn cast_ray_local(&self, ray: &Ray, depth: usize, sampler: &mut Sampler) -> RGBA
    {
        match self.trace_intersection(ray)
        {
            Some(intersection) =>
            {
                // We've hit an object.
                // Use the object's material to get a
                // quick color

                let (color, onwards_ray) = intersection.material.local(&intersection.surface);

                if onwards_ray.is_none() || depth >= 5
                {
                    // Either no onwards ray, or we've already traced too many
                    // rays to get to this point - just return the color
                    // with a *really basic* lighting equation

                    // Ambient
                    let mut amount = 0.2;

                    // Basic diffuse
                    let int_location = intersection.surface.location();
                    let light_dir = (self.local_light - int_location).normalized();
                    let diffuse = intersection.surface.normal.dot(light_dir);
                    if diffuse > 0.0
                    {
                        if self.trace_intersection(&Ray::new(int_location, light_dir)).is_some()
                        {
                            // There's a object blocking the light
                            amount += 0.2 * diffuse;
                        }
                        else
                        {
                            // The object is illuminated by the light
                            amount += 0.8 * diffuse;
                        }
                    }

                    color.multiplied_by_scalar(amount)
                }
                else
                {
                    // Cast the onwards ray, and then combine it with
                    // the color of this surface

                    let onwards_color = self.cast_ray_local(&onwards_ray.unwrap(), depth + 1, sampler);

                    onwards_color.combined_with(&color)
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

        let found = intersections.drain(..).nth(0);

        found
    }
}
