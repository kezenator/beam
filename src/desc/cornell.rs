use crate::camera::Camera;
use crate::color::{LinearRGB, SRGB};
use crate::desc::{SceneDescription, StandardScene};
use crate::geom::{Aabb, Sphere, Rectangle};
use crate::lighting::LightingRegion;
use crate::material::Material;
use crate::math::Scalar;
use crate::object::Object;
use crate::render::RenderOptions;
use crate::scene::Scene;
use crate::texture::Texture;
use crate::vec::Point3;

pub fn generate_description() -> SceneDescription
{
    SceneDescription
    {
        camera_location: Point3::new(277.5, 277.5, 2000.0),
        camera_look_at: Point3::new(277.5, 277.5, 555.0),
        camera_up: Point3::new(0.0, 1.0, 0.0),
        camera_fov: 40.0,
        scene: StandardScene::Cornell,
    }
}

pub fn generate_scene(desc: &SceneDescription, options: &RenderOptions) -> Scene
{
    let wall_rect = |point: Point3, u: Point3, v: Point3, color: SRGB| -> Object
    {
        Object::new(
            Rectangle::new(point, u, v),
            Material::diffuse(Texture::solid(color)))
    };

    let white_box = |min: Point3, max: Point3| -> Object
    {
        Object::new(
            Aabb::new(min, max),
            Material::diffuse(Texture::solid(LinearRGB::new(1.0, 1.0, 1.0))))
    };

    let glass_sphere = |pos: Point3, radius: Scalar| -> Object
    {
        Object::new(
            Sphere::new(pos, radius),
            Material::dielectric(1.5))
    };

    let metal_sphere = |pos: Point3, radius: Scalar, color: SRGB, roughness: Scalar| -> Object
    {
        Object::new(
            Sphere::new(pos, radius),
            Material::metal(Texture::solid(color), roughness))
    };

    let light_rect = |point: Point3, u: Point3, v: Point3| -> Object
    {
        Object::new(
            Rectangle::new(point, u, v),
            Material::emit_front_face_only(Texture::solid(SRGB::new(4.0, 4.0, 4.0))))
    };

    Scene::new(
        options.sampling_mode,
        Camera::new(desc.camera_location, desc.camera_look_at, desc.camera_up, desc.camera_fov, (options.width as f64) / (options.height as f64)),
        // Lighting regions
        vec![
            LightingRegion::new_2(
                Aabb::new(Point3::new(260.0, 164.0, 325.0), Point3::new(425.0, 166.0, 490.0)),
                Rectangle::new(Point3::new(213.0, 554.0, 227.0), Point3::new(130.0, 0.0, 0.0), Point3::new(0.0, 0.0, 105.0)),
                Sphere::new(Point3::new(342.5, 240.0, 407.5), 60.0),
                vec![
                    Point3::new(227.5, 554.0, 279.5),
                ]),
            LightingRegion::new_1(
                Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(556.0, 556.0, 556.0)),
                Rectangle::new(Point3::new(213.0, 554.0, 227.0), Point3::new(130.0, 0.0, 0.0), Point3::new(0.0, 0.0, 105.0)),
                vec![
                    Point3::new(227.5, 554.0, 279.5),
                ]),
        ],
        // Objects
        vec![
            // Walls - left(red), right(green), top, back floor
            wall_rect(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 555.0, 0.0), Point3::new(0.0, 0.0, 555.0), SRGB::new(1.0, 0.0, 0.0)),
            wall_rect(Point3::new(555.0, 0.0, 0.0), Point3::new(0.0, 555.0, 0.0), Point3::new(0.0, 0.0, 555.0), SRGB::new(0.0, 1.0, 0.0)),
            wall_rect(Point3::new(0.0, 555.0, 0.0), Point3::new(555.0, 0.0, 0.0), Point3::new(0.0, 0.0, 555.0), SRGB::new(1.0, 1.0, 1.0)),
            wall_rect(Point3::new(0.0, 0.0, 0.0), Point3::new(555.0, 0.0, 0.0), Point3::new(0.0, 555.0, 0.0), SRGB::new(1.0, 1.0, 1.0)),
            wall_rect(Point3::new(0.0, 0.0, 0.0), Point3::new(555.0, 0.0, 0.0), Point3::new(0.0, 0.0, 555.0), SRGB::new(1.0, 1.0, 1.0)),

            // Light
            light_rect(Point3::new(213.0, 554.0, 227.0), Point3::new(130.0, 0.0, 0.0), Point3::new(0.0, 0.0, 105.0)),

            // Objects
            white_box(Point3::new(260.0, 0.0, 325.0), Point3::new(425.0, 165.0, 490.0)),
            white_box(Point3::new(125.0, 0.0, 95.0), Point3::new(290.0, 330.0, 260.0)),
            glass_sphere(Point3::new(342.5, 240.0, 407.5), 60.0),
            metal_sphere(Point3::new(207.5, 405.0, 227.5), 60.0, SRGB::new(0.18, 0.18, 0.18), 0.1),
        ])
}