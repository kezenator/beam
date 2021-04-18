use crate::math::Scalar;
use crate::render::RenderOptions;
use crate::scene::Scene;
use crate::vec::Point3;

mod beam;
mod cornell;
mod furnace;

#[derive(Clone)]
pub enum StandardScene
{
    BeamExample,
    Cornell,
    Furnace,
}

#[derive(Clone)]
pub struct SceneDescription
{
    pub camera_location: Point3,
    pub camera_look_at: Point3,
    pub camera_up: Point3,
    pub camera_fov: Scalar,
    pub scene: StandardScene,
}

impl SceneDescription
{
    pub fn new(scene: StandardScene) -> Self
    {
        match scene
        {
            StandardScene::BeamExample => beam::generate_description(),
            StandardScene::Cornell => cornell::generate_description(),
            StandardScene::Furnace => furnace::generate_description(),
        }
    }

    pub fn build_scene(&self, options: &RenderOptions) -> Scene
    {
        match self.scene
        {
            StandardScene::BeamExample => beam::generate_scene(self, options),
            StandardScene::Cornell => cornell::generate_scene(self, options),
            StandardScene::Furnace => furnace::generate_scene(self, options),
        }
    }
}