use crate::render::RenderOptions;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::Point3;

#[derive(Clone, Debug)]
pub struct Camera
{
    location: Point3,
    look_at: Point3,
    up: Point3,
    fov: f64,
}

impl Camera
{
    pub fn build(&self, options: &RenderOptions) -> crate::camera::Camera
    {
        let aspect_ratio = (options.width as f64) / (options.height as f64);
        
        crate::camera::Camera::new(
            self.location,
            self.look_at,
            self.up,
            self.fov,
            aspect_ratio)
    }
}

impl Default for Camera
{
    fn default() -> Self
    {
        Camera
        {
            location: Point3::new(0.0, 1.0, -1.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            up: Point3::new(0.0, 1.0, 0.0),
            fov: 30.0,
        }
    }
}

impl UiDisplay for Camera
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.display_vec3("Location", &self.location);
        ui.display_vec3("Look At", &self.look_at);
        ui.display_vec3("Up", &self.up);
        ui.display_float("FOV", &self.fov);
    }
}

impl UiEdit for Camera
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        result |= ui.edit_vec3("Location", &mut self.location);
        result |= ui.edit_vec3("Look At", &mut self.look_at);
        result |= ui.edit_vec3("Up", &mut self.up);
        result |= ui.edit_float("FOV", &mut self.fov);
        result
    }
}