use crate::camera::Camera;
use crate::color::RGBA;
use crate::desc::{SceneDescription, StandardScene};
use crate::geom::Sphere;
use crate::material::Material;
use crate::object::Object;
use crate::render::RenderOptions;
use crate::scene::Scene;
use crate::texture::Texture;
use crate::vec::Point3;

pub fn generate_description() -> SceneDescription
{
    SceneDescription
    {
        camera_location: Point3::new(0.0, 0.0, 9.0),
        camera_look_at: Point3::new(0.0, 0.0, 0.0),
        camera_up: Point3::new(0.0, 1.0, 0.0),
        camera_fov: 40.0,
        scene: StandardScene::Furnace,
    }
}

pub fn generate_scene(desc: &SceneDescription, options: &RenderOptions) -> Scene
{
    Scene::new(
        Camera::new(desc.camera_location, desc.camera_look_at, desc.camera_up, desc.camera_fov, (options.width as f64) / (options.height as f64)),
        // Lighting regions
        vec![
        ],
        // Objects
        vec![
            Object::new(
                Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0),
                Material::diffuse(Texture::solid(RGBA::new(0.5, 0.5, 0.5, 1.0)))),

            Object::new(
                Sphere::new(Point3::new(0.0, 0.0, 0.0), 10.0),
                Material::emit(Texture::solid(RGBA::new(1.0, 1.0, 1.0, 1.0)))),
        ])
}