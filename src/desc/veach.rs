use crate::camera::Camera;
use crate::color::SRGB;
use crate::desc::{SceneDescription, SceneSelection, StandardScene};
use crate::geom::{Aabb, Rectangle, Sphere, bounds::BoundedSurface, csg::Merge, csg::Difference};
use crate::lighting::LightingRegion;
use crate::math::Scalar;
use crate::material::Material;
use crate::object::Object;
use crate::render::RenderOptions;
use crate::scene::Scene;
use crate::texture::Texture;
use crate::vec::{Dir3, Point3};

pub fn generate_description() -> SceneDescription
{
    SceneDescription
    {
        camera: super::edit::Camera
        {
            location: Point3::new(-12.360750, -22.707277, 35.0),
            look_at: Point3::new(-0.390985, 10.182305, 0.0),
            up: Point3::new(0.0, 0.0, 1.0),
            fov: 45.0,
        },
        selection: SceneSelection::Standard(StandardScene::Veach),
    }
}

pub fn generate_scene(desc: &SceneDescription, options: &RenderOptions) -> Scene
{
    let mut lighting_region = LightingRegion::new(Aabb::new(Point3::new(-50.0, -50.0, -50.0), Point3::new(50.0, 50.0, 50.0)));
    let mut objects = Vec::new();

    // Walls
    {
        let mut walls = Merge::new();

        walls.push(Rectangle::new(Point3::new(-40.0, 10.0, 0.0), Dir3::new(80.0, 0.0, 0.0), Dir3::new(0.0, 0.0, 40.0)));  // Back
        walls.push(Rectangle::new(Point3::new(-40.0, 10.0, 0.0), Dir3::new(80.0, 0.0, 0.0), Dir3::new(0.0, -40.0, 0.0))); // Floor
        walls.push(Rectangle::new(Point3::new(-40.0, 10.0, 0.0), Dir3::new(0.0, -40.0, 0.0), Dir3::new(0.0, 0.0, 40.0))); // Left
        walls.push(Rectangle::new(Point3::new(40.0, 10.0, 0.0), Dir3::new(0.0, -40.0, 0.0), Dir3::new(0.0, 0.0, 40.0))); // Right
        walls.push(Rectangle::new(Point3::new(-40.0, 10.0, 40.0), Dir3::new(80.0, 0.0, 0.0), Dir3::new(0.0, -40.0, 0.0))); // Top
        walls.push(Rectangle::new(Point3::new(-40.0, -30.0, 0.0), Dir3::new(80.0, 0.0, 0.0), Dir3::new(0.0, 0.0, 40.0))); // Back

        objects.push(Object::new(
            walls,
            Material::diffuse(Texture::solid(SRGB::new(1.0, 1.0, 1.0)))));
    }

    // Light
    {
        let pos = Point3::new(10.0, -5.0, 30.0);
        let d1 = Dir3::new(5.0, 0.0, 0.0);
        let d2 = Dir3::new(0.0, -5.0, 0.0);

        objects.push(Object::new(
            Rectangle::new(pos - d1 - d2, 2.0 * d1, 2.0 * d2),
            Material::emit_front_face_only(Texture::solid(SRGB::new(4.0, 4.0, 4.0)))));

        lighting_region.global_surfaces.push(Box::new(Rectangle::new(pos - d1 - d2, 2.0 * d1, 2.0 * d2)));
        lighting_region.local_points.push(pos);
    }

    // Colored lights
    const LIGHT_Y: Scalar = 7.0;
    const LIGHT_Z: Scalar = 10.0;

    {
        let mut light = |x: Scalar, y: Scalar, z: Scalar, radius: Scalar, color: SRGB|
        {
            let color: crate::color::LinearRGB = color.into();
            let color = color.multiplied_by_scalar(5.0);

            lighting_region.global_surfaces.push(Box::new(Sphere::new(Point3::new(x, y, z), radius)));
            lighting_region.local_points.push(Point3::new(x, y, z));
            objects.push(Object::new(
                Sphere::new(Point3::new(x, y, z), radius),
                Material::emit(Texture::solid(color))));
        };

        // The four colored lights
        light(-10.0, LIGHT_Y, LIGHT_Z, 0.2, SRGB::new(1.0, 0.0, 0.0));
        light(-5.0, LIGHT_Y, LIGHT_Z, 1.0, SRGB::new(0.0, 1.0, 0.0));
        light(2.0, LIGHT_Y, LIGHT_Z, 2.0, SRGB::new(0.0, 0.0, 1.0));
        light(12.0, LIGHT_Y, LIGHT_Z, 4.0, SRGB::new(1.0, 1.0, 0.0));

        // The wall light
        light(0.0, 9.5, 12.0, 0.3, SRGB::new(10.0, 10.0, 10.0));

        objects.push(Object::new(
            BoundedSurface::new(
                Aabb::new(Point3::new(-1.0, 9.0, 11.0), Point3::new(1.0, 10.0, 13.0)),
                Difference::new(
                    Aabb::new(Point3::new(-1.0, 9.0, 11.0), Point3::new(1.0, 10.0, 13.0)),
                    Aabb::new(Point3::new(-0.8, 9.2, 10.5), Point3::new(0.8, 10.1, 13.5)))),
            Material::diffuse(Texture::solid(SRGB::new(0.5, 0.5, 0.5)))));
    }

    // Metal bars
    {
        let mut metal_bar = |y: Scalar, z: Scalar, fuzz: Scalar|
        {
            let pos = Point3::new(0.0, y, z);
            let to_camera = (Point3::new(0.0, desc.camera.location.y, desc.camera.location.z) - pos).normalized();
            let to_light = (Point3::new(0.0, LIGHT_Y, LIGHT_Z) - pos).normalized();

            let ny = (to_light.z - to_camera.z) / (to_camera.y - to_light.y);
            let nz = ny * (to_light.y - to_camera.y) / (to_camera.z - to_light.z);

            let normal = Point3::new(0.0, ny, nz);

            let dir = 0.9 * Point3::new(1.0, 0.0, 0.0).cross(normal);

            let y1 = pos.y - dir.y;
            let z1 = pos.z - dir.z;
            let dy = 2.0 * dir.y;
            let dz = 2.0 * dir.z;

            objects.push(Object::new(
                Rectangle::new(Point3::new(-14.0, y1, z1), Dir3::new(28.0, 0.0, 0.0), Dir3::new(0.0, dy, dz)),
                Material::metal(Texture::solid(SRGB::new(0.5, 0.5, 0.5)), fuzz)));
        };

        metal_bar(0.0, 1.0, 0.1);
        metal_bar(2.0, 2.0, 0.05);
        metal_bar(4.0, 3.0, 0.01);
        metal_bar(6.0, 4.0, 0.0001);
    }

    Scene::new(
        options.sampling_mode,
        Camera::new(desc.camera.location, desc.camera.look_at, desc.camera.up, desc.camera.fov, (options.width as f64) / (options.height as f64)),
        vec![
            lighting_region,
        ],
        objects)
}
