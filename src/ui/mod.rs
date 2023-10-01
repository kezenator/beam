mod pixel;
mod system;

pub use system::System;
pub use pixel::PixelDisplay;

pub trait UiApplication<T: 'static>
{
    fn handle_event(&mut self, event: winit::event::Event<T>) -> Option<winit::event_loop::ControlFlow>;
    fn render_background(&mut self, display: &glium::Display, frame: &mut glium::Frame);
    fn render_ui(&mut self, ui: &imgui::Ui);
    fn idle(&mut self);
}