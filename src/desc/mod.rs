use crate::math::Scalar;
use crate::scene::Scene;
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;
use crate::lighting::LightingRegion;
use crate::material::Material;
use crate::camera::Camera;
use crate::object::Object;
use crate::geom::{AABB, Plane, Sphere, Rectangle, Blob, BlobPart, BoundedSurface, csg};
use crate::texture::Texture;
use crate::sample::Sampler;
use crate::render::RenderOptions;

#[derive(Clone)]
pub struct SceneDescription
{
    pub camera_location: Point3,
    pub camera_look_at: Point3,
    pub camera_up: Point3,
    pub camera_fov: Scalar,
}

impl SceneDescription
{
    pub fn new() -> Self
    {
        SceneDescription
        {
            camera_location: Point3::new(-3.0, 12.0, 12.0),
            camera_look_at: Point3::new(0.0, -1.0, 0.0),
            camera_up: Point3::new(0.0, 1.0, 0.0),
            camera_fov: 40.0,
        }
    }

    pub fn build_scene(&self, options: &RenderOptions) -> Scene
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

        let light_sphere = |centre: Point3, radius: Scalar, brightness: Scalar| -> Object
        {
            Object::new(
                Sphere::new(centre, radius),
                Material::emit(Texture::solid(RGBA::new(brightness, brightness, brightness, 1.0))))
        };

        let rectangle = |point: Point3, u: Point3, v: Point3| -> Object
        {
            Object::new(
                Rectangle::new(point, u, v),
                Material::diffuse(Texture::solid(RGBA::new(0.7, 0.7, 0.7, 1.0))))
        };

        let aabb = |min: Point3, max: Point3, color: RGBA| -> Object
        {
            Object::new(
                AABB::new(min, max),
                Material::diffuse(Texture::solid(color)))
        };

        let plane = |point: Point3, normal: Dir3, color: RGBA| -> Object
        {
            Object::new(
                Plane::new(point, normal),
                Material::metal(Texture::checkerboard(color, RGBA::new(1.0, 1.0, 1.0, 1.0)), 0.1))
        };

        let wall_plane = |point: Point3, normal: Dir3, color: RGBA| -> Object
        {
            Object::new(
                Plane::new(point, normal),
                Material::diffuse(Texture::solid(color)))
        };

        let cloud = |center: Point3, radius: Scalar, blob_rad: Scalar, num: usize, color: RGBA| -> Object
        {
            let mut sampler = Sampler::new_reproducable(0xBAD5EED5DEADBEEFu64);
            let mut merge = csg::Merge::new();

            for _ in 0..num
            {
                let blob_center = center + (radius - blob_rad) * sampler.uniform_dir_on_unit_sphere();
                
                merge.push(Sphere::new(blob_center, blob_rad));
            }

            let bounds = Sphere::new(center, radius);

            Object::new(
                BoundedSurface::new(bounds, merge),
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

        let cut_box = |center: Point3, radius: Scalar, color: RGBA| -> Object
        {
            let min = Point3::new(center.x - radius, center.y - radius, center.z - radius);
            let max = Point3::new(center.x + radius, center.y + radius, center.z + radius);

            let diff = csg::Difference::new(AABB::new(min, max), Sphere::new(center, 1.3 * radius));
            let aabb = AABB::new(min, max);

            Object::new(
                BoundedSurface::new(aabb, diff),
                Material::diffuse(Texture::solid(color)))
        };

        Scene::new(
            Camera::new(self.camera_location, self.camera_look_at, self.camera_up, self.camera_fov, (options.width as f64) / (options.height as f64)),
            // Lighting regions
            vec![
                LightingRegion::new_2(
                    Sphere::new(Point3::new(0.0, 0.0, 0.0), 100.0),
                    Rectangle::new(Point3::new(-50.0, 50.0, 50.0), Dir3::new(0.0, 0.0, -100.0), Dir3::new(100.0, 0.0, 0.0)),
                    Sphere::new(Point3::new(-2.0, 1.0, 1.0), 0.2),
                    vec![
                        // Small sphere
                        Point3::new(-2.0, 1.0, 1.0),
                        Point3::new(-2.0, 1.2, 1.0),
                        Point3::new(-2.0, 1.0, 1.2),
                        Point3::new(-2.0, 1.0, 0.8),
                        // Top
                        Point3::new(0.0, 50.0, 0.0),
                    ]),
            ],
            // Objects
            vec![
                // White sphere at the origin
                sphere(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(0.8, 0.8, 0.8, 1.0)),

                // Red, green and blue ones around it
                sphere(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(0.8, 0.0, 0.8, 1.0)),
                sphere(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.8, 0.0, 1.0)),
                sphere(Point3::new(-2.0, 0.0, -0.5), 1.0, RGBA::new(0.0, 0.0, 0.8, 1.0)),

                // Grey sphere below
                sphere(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.5, 0.5, 0.5, 1.0)),

                // Some more interesting stuff
                cloud(Point3::new(2.5, 2.5, 2.0), 1.2, 0.2, 400, RGBA::new(0.8, 0.6, 0.3, 1.0)),
                blob(Point3::new(-2.5, 2.5, 1.0), RGBA::new(0.8, 0.6, 0.3, 1.0)),

                // Rectangular "walls"
                rectangle(Point3::new(4.0, -3.0, -1.5), Point3::new(0.0, 0.0, 4.0), Point3::new(0.0, 4.0, 0.0)),
                rectangle(Point3::new(4.0, -3.0, -1.5), Point3::new(-4.0, 0.0, 0.0), Point3::new(0.0, 4.0, 0.0)),

                // Real walls to enclose the scene
                wall_plane(Point3::new(-50.0, 0.0, 0.0), Dir3::new(1.0, 0.0, 0.0), RGBA::new(1.0, 1.0, 1.0, 1.0)),
                wall_plane(Point3::new(50.0, 0.0, 0.0), Dir3::new(-1.0, 0.0, 0.0), RGBA::new(1.0, 1.0, 1.0, 1.0)),
                wall_plane(Point3::new(0.0, 0.0, 50.0), Dir3::new(0.0, 0.0, -1.0), RGBA::new(1.0, 1.0, 1.0, 1.0)),
                wall_plane(Point3::new(0.0, 0.0, -50.0), Dir3::new(0.0, 0.0, 1.0), RGBA::new(1.0, 1.0, 1.0, 1.0)),

                // Metal spheres
                metal_sphere(Point3::new(2.50, -2.0, 1.0), 1.25, RGBA::new(0.8, 0.5, 0.8, 1.0)),

                // Glass spheres and a floor for a caustic
                glass_sphere(Point3::new(-1.5, -2.0, 1.5), 1.25),
                glass_sphere(Point3::new(-4.0, -2.2, 0.5), 0.75),
                aabb(Point3::new(-6.0, -3.4, -3.0), Point3::new(-2.0, -3.2, 1.0), RGBA::new(0.7, 0.7, 0.7, 1.0)),

                // A cut box on the caustic floor
                cut_box(Point3::new(-4.0, -2.2, -1.5), 0.75, RGBA::new(0.9, 0.5, 0.2, 1.0)),

                // Lights
                light_sphere(Point3::new(-2.0, 1.0, 1.0), 0.2, 4.0),
                light_sphere(Point3::new(0.0, 500.0, 0.0), 450.0, 0.5),

                // Ground
                plane(Point3::new(0.0, -3.51, 0.0), Dir3::new(0.0, 1.0, 0.0), RGBA::new(0.2, 0.2, 0.2, 1.0)),

                // Wall behind
                sphere(Point3::new(0.0, 0.0, -13.0), 10.0, RGBA::new(0.5, 0.584, 0.929, 1.0)),
            ])
        }
    }
