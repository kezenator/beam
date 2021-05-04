use crate::camera::Camera;
use crate::exec::{Context, ExecError, ExecResult, parse};
use crate::math::Scalar;
use crate::object::Object;
use crate::render::RenderOptions;
use crate::scene::Scene;
use crate::vec::Point3;

mod beam;
mod cornell;
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
    Exec(String),
}

#[derive(Clone)]
pub struct CameraDescription
{
    pub location: Point3,
    pub look_at: Point3,
    pub up: Point3,
    pub fov: Scalar,
}

#[derive(Clone)]
pub struct SceneDescription
{
    pub camera: CameraDescription,
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
            StandardScene::Furnace => Self::new_script(include_str!("furnace.beam").to_owned()).expect("Error in in-built script"),
            StandardScene::Veach => veach::generate_description(),
        }
    }

    pub fn new_script(script: String) -> ExecResult<Self>
    {
        let (camera, _objects) = run_script(&script)?;

        Ok(SceneDescription
        {
            camera,
            selection: SceneSelection::Exec(script),
        })
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
            SceneSelection::Exec(script) =>
            {
                let (camera, objects) = run_script(script).expect("Script execution failed");

                Scene::new(
                    options.sampling_mode,
                    Camera::new(camera.location, camera.look_at, camera.up, camera.fov, (options.width as f64) / (options.height as f64)),
                    vec![],
                    objects
                )
            },
        }
    }
}

fn run_script(script: &str) -> ExecResult<(CameraDescription, Vec<Object>)>
{
    let expressions = parse(script)?;

    let mut camera = CameraDescription
    {
        location: Point3::new(0.0, 0.0, 1.0),
        look_at: Point3::new(0.0, 0.0, 0.0),
        up: Point3::new(0.0, 1.0, 0.0),
        fov: 40.0,
    };
    let mut objects = Vec::new();

    let mut context = Context::new();

    for exp in expressions
    {
        let val = exp.evaluate(&mut context)?;
        let source = val.source_location();

        if let Ok(cam) = val.clone().into_camera()
        {
            camera = cam;
        }
        else if let Ok(obj) = val.into_object()
        {
            objects.push(obj);
        }
        else
        {
            return Err(ExecError::new(source, "Expect camera or object"));
        }
    }

    Ok((camera, objects))
}
