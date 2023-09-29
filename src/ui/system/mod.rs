use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::Display;
use imgui::{Context, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

mod clipboard;

// A system that runs an application
// Adapted from the imgui-rs example:
// https://github.com/imgui-rs/imgui-rs/blob/main/imgui-examples/examples/support/mod.rs
// as accessed on 2023-09-29
pub struct System
{
    event_loop: EventLoop<()>,
    display: glium::Display,
    imgui: Context,
    platform: WinitPlatform,
    renderer: Renderer,
}

impl System
{
    pub fn init(title: &str) -> System
    {
        let event_loop = EventLoop::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let builder = WindowBuilder::new()
            .with_title(title.to_owned())
            .with_inner_size(glutin::dpi::LogicalSize::new(1024f64, 768f64));
        let display =
            Display::new(builder, context, &event_loop).expect("Failed to initialize display");

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        if let Some(backend) = clipboard::init()
        {
            imgui.set_clipboard_backend(backend);
        }
        else
        {
            eprintln!("Failed to initialize clipboard");
        }

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();

            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
        }

        imgui.fonts().add_font(&[
            FontSource::DefaultFontData { config: None },
        ]);

        let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        System
        {
            event_loop,
            display,
            imgui,
            platform,
            renderer,
        }
    }

    pub fn display(&self) -> &glium::Display
    {
        &self.display
    }

    pub fn main_loop<App>(self, mut app: App) -> !
        where App: super::UiApplication + 'static
    {
        let System
        {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;
        let mut last_frame = Instant::now();

        event_loop.run(move |event, _, control_flow| match event
        {
            Event::NewEvents(_) =>
            {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared =>
            {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) =>
            {
                let ui = imgui.frame();

                app.render_ui(ui);

                let gl_window = display.gl_window();
                let mut target = display.draw();
                app.render_background(&mut target);
                platform.prepare_render(ui, gl_window.window());
                let draw_data = imgui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            }
            event =>
            {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}
