use crate::exec::{Context, ExecResult, parse};
use crate::render::RenderOptions;
use crate::scene::Scene;

mod beam;
mod cornell;
pub mod edit;
mod veach;

#[derive(Clone)]
pub enum StandardScene
{
    BeamExample,
    Cornell,
    Furnace,
    Veach,
}

#[derive(Clone)]
pub enum SceneSelection
{
    Standard(StandardScene),
    Edit(edit::Scene),
}

#[derive(Clone)]
pub struct SceneDescription
{
    pub camera: edit::Camera,
    pub selection: SceneSelection,
}

impl SceneDescription
{
    pub fn new_standard(scene: StandardScene) -> Self
    {
        match scene
        {
            StandardScene::BeamExample => beam::generate_description(),
            StandardScene::Cornell => cornell::generate_description(),
            StandardScene::Furnace => Self::new_edit(&run_script(include_str!("furnace.beam")).expect("Error in in-built script")),
            StandardScene::Veach => veach::generate_description(),
        }
    }

    pub fn new_edit(scene: &edit::Scene) -> Self
    {
        SceneDescription
        {
            camera: scene.camera.clone(),
            selection: SceneSelection::Edit(scene.clone()),
        }
    }

    pub fn build_scene(&self, options: &RenderOptions) -> Scene
    {
        match &self.selection
        {
            SceneSelection::Standard(standard) =>
            {
                match standard
                {
                    StandardScene::BeamExample => beam::generate_scene(self, options),
                    StandardScene::Cornell => cornell::generate_scene(self, options),
                    StandardScene::Furnace => panic!("Furnace is now a script"),
                    StandardScene::Veach => veach::generate_scene(self, options),
                }
            },
            SceneSelection::Edit(edit) =>
            {
                edit.build(options, Some(&self.camera))
            }
        }
    }
}

pub fn run_script(script: &str) -> ExecResult<edit::Scene>
{
    let expressions = parse(script)?;

    let mut context = Context::new_with_state(edit::Scene::new());

    for exp in expressions
    {
        exp.evaluate(&mut context)?;
    }

    let scene = context.with_app_state::<edit::Scene, _, _>(|scene| Ok(scene.clone()))?;

    Ok(scene)
}
