mod pixel;
mod system;

pub use system::System;
pub use pixel::PixelDisplay;

pub trait UiApplication
{
    fn render_background(&mut self, frame: &mut glium::Frame);
    fn render_ui(&mut self, ui: &imgui::Ui);
}