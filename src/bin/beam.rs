use std::time::Duration;

use glium::Surface;
use winit::event::{ElementState, Event, ModifiersState, VirtualKeyCode, WindowEvent};

use beam::desc::{SceneDescription, StandardScene};
use beam::math::Scalar;
use beam::render::{Renderer, RenderOptions, RenderIlluminationMode};
use beam::scene::SamplingMode;
use beam::vec::{Mat4, Vec3, Vec4};


fn main() -> Result<(), String>
{
    let system = beam::ui::System::init("Beam");
    let app_state = AppState::new(&system, 128, 128);
    system.main_loop(app_state);
}

struct AppState
{
    filename: Option<String>,
    options: RenderOptions,
    desc: SceneDescription,
    renderer: Renderer,
    pixels: beam::ui::PixelDisplay,
    progress: Option<beam::render::RenderProgress>,
    keyboard_modifiers: winit::event::ModifiersState,
}

impl AppState
{
    pub fn new(system: &beam::ui::System<()>, width: u32, height: u32) -> Self
    {
        let filename = None;
        let options = RenderOptions::new(width, height);
        let desc = SceneDescription::new_standard(StandardScene::Cornell);
        let renderer = Renderer::new(options.clone(), desc.clone());
        let pixels = beam::ui::PixelDisplay::new(system.display(), width, height);
        let progress = None;
        let keyboard_modifiers = ModifiersState::empty();

        AppState
        {
            filename,
            options,
            desc,
            renderer,
            pixels,
            progress,
            keyboard_modifiers
        }
    }

    pub fn new_renderer(&self) -> Renderer
    {
        Renderer::new(self.options.clone(), self.desc.clone())
    }

    pub fn load_file(&mut self, filename: &str)
    {
        self.filename = Some(filename.to_owned());

        match std::fs::read_to_string(filename)
        {
            Ok(text) =>
            {
                match SceneDescription::new_script(text)
                {
                    Ok(desc) =>
                    {
                        self.desc = desc;
                        return;
                    },
                    Err(err) =>
                    {
                        println!("Error: Could not execute script: {:?}", err);
                    },
                }
            },
            Err(err) =>
            {
                println!("Error: Could not load file: {:?}", err);
            },
        }

        self.desc = SceneDescription::new_standard(StandardScene::Cornell);
    }

    pub fn handle_keycode(&mut self, keycode: VirtualKeyCode, keymod: ModifiersState) -> bool
    {
        let ctrl = keymod.ctrl();

        let handled = match keycode
        {
            VirtualKeyCode::Key1 =>
            {
                self.desc = SceneDescription::new_standard(StandardScene::BeamExample);
                true
            },
            VirtualKeyCode::Key2 =>
            {
                self.desc = SceneDescription::new_standard(StandardScene::Cornell);
                true
            },
            VirtualKeyCode::Key3 =>
            {
                self.desc = SceneDescription::new_standard(StandardScene::Furnace);
                true
            },
            VirtualKeyCode::Key4 =>
            {
                self.desc = SceneDescription::new_standard(StandardScene::Veach);
                true
            },
            VirtualKeyCode::Key0 =>
            {
                if let Some(filename) = self.filename.clone()
                {
                    self.load_file(&filename);
                    true
                }
                else
                {
                    false
                }
            },
            VirtualKeyCode::F1 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::BsdfAndLights;
                true
            }
            VirtualKeyCode::F2 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::LightsOnly;
                true
            }
            VirtualKeyCode::F3 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::BsdfOnly;
                true
            }
            VirtualKeyCode::F4 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::Uniform;
                true
            }
            VirtualKeyCode::F5 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Local;
                true
            }
            VirtualKeyCode::C =>
            {
                println!("Camera Location: {:?}", self.desc.camera.location);
                println!("Camera Look-at:  {:?}", self.desc.camera.look_at);
                println!("Camera Up:       {:?}", self.desc.camera.up);
                println!("Camera FOV:      {:?}", self.desc.camera.fov);
                false
            },
            VirtualKeyCode::L =>
            {
                self.options.illumination_mode = match self.options.illumination_mode
                {
                    RenderIlluminationMode::Local => RenderIlluminationMode::Global,
                    RenderIlluminationMode::Global => RenderIlluminationMode::Local,
                };
                self.options.sampling_mode = SamplingMode::BsdfAndLights;
                true
            },
            VirtualKeyCode::Left =>
            {
                if ctrl
                {
                    self.rotate_around(-10.0);
                }
                else
                {
                    self.move_around(-1.0, 0.0);
                }
                true
            },
            VirtualKeyCode::Right =>
            {
                if ctrl
                {
                    self.rotate_around(10.0);
                }
                else
                {
                    self.move_around(1.0, 0.0);
                }
                true
            },
            VirtualKeyCode::Up =>
            {
                if ctrl
                {
                    self.tilt(10.0);
                }
                else
                {
                    self.move_around(0.0, -1.0);
                }
                true
            },
            VirtualKeyCode::Down =>
            {
                if ctrl
                {
                    self.tilt(-10.0);
                }
                else
                {
                    self.move_around(0.0, 1.0);
                }
                true
            },
            VirtualKeyCode::NumpadAdd =>
            {
                self.desc.camera.fov = (self.desc.camera.fov - 5.0).clamp(1.0, 175.0);
                true
            },
            VirtualKeyCode::NumpadSubtract =>
            {
                self.desc.camera.fov = (self.desc.camera.fov + 5.0).clamp(1.0, 175.0);
                true
            },
            _ =>
            {
                false
            },
        };

        if handled
        {
            self.options.max_blockiness = 8;
        }

        handled
    }

    fn move_around(&mut self, factor_left_right: Scalar, factor_forward_back: Scalar)
    {
        let look = self.desc.camera.look_at - self.desc.camera.location;
        let right = look.cross(self.desc.camera.up).normalized();
        let back = right.cross(self.desc.camera.up).normalized();
        let up = self.desc.camera.up.normalized();
        
        // Don't move in the up/down direction
        let right = right - (up.dot(right) * up);
        let back = back - (up.dot(right) * up);

        // Work out how far to move
        let dist_factor = 0.05 * look.magnitude();

        let dir = dist_factor * (factor_left_right * right + factor_forward_back * back);

        self.desc.camera.location += dir;
        self.desc.camera.look_at += dir;
    }

    fn rotate_around(&mut self, degrees: Scalar)
    {
        let dir = self.desc.camera.location - self.desc.camera.look_at;

        let rot = Mat4::rotation_3d(degrees.to_radians(), self.desc.camera.up);

        let new_dir: Vec3 = (rot * Vec4::from_direction(dir)).into();

        self.desc.camera.location = new_dir + self.desc.camera.look_at;
    }

    fn tilt(&mut self, degrees: Scalar)
    {
        let dir = self.desc.camera.location - self.desc.camera.look_at;

        let right = dir.cross(self.desc.camera.up);

        let rot = Mat4::rotation_3d(degrees.to_radians(), right);

        let new_dir: Vec3 = (rot * Vec4::from_direction(dir)).into();

        self.desc.camera.location = new_dir + self.desc.camera.look_at;
    }
}

impl beam::ui::UiApplication<()> for AppState
{
    fn handle_event(&mut self, event: winit::event::Event<()>) -> Option<winit::event_loop::ControlFlow>
    {
        match event
        {
            Event::WindowEvent{ event, .. } =>
            {
                match event
                {
                    WindowEvent::ModifiersChanged(modifiers) =>
                    {
                        self.keyboard_modifiers = modifiers;
                    },
                    WindowEvent::KeyboardInput { input, .. } =>
                    {
                        if input.state == ElementState::Pressed
                        {
                            if let Some(virtual_keycode) = input.virtual_keycode
                            {
                                if self.handle_keycode(virtual_keycode, self.keyboard_modifiers)
                                {
                                    self.renderer = self.new_renderer();
                                }
                            }
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }

        None
    }

    fn render_background(&mut self, display: &glium::Display, frame: &mut glium::Frame)
    {
        if frame.get_dimensions() != self.pixels.dimensions()
        {
            let (width, height) = frame.get_dimensions();
            self.pixels.resize(width, height);
            self.options.width = width;
            self.options.height = height;
            self.renderer = self.new_renderer();
        }
        self.pixels.render(display, frame);
    }

    fn render_ui(&mut self, ui: &imgui::Ui)
    {
        if let Some(progress) = &self.progress
        {
            if let Some(_) = ui.window("Progress").begin()
            {
                render_progress(ui, progress)
            }
        }
    }

    fn idle(&mut self)
    {
        if let Some(update) = self.renderer.get_update()
        {
            for pixel in update.pixels
            {
                self.pixels.set_pixel(
                    pixel.rect.x,
                    pixel.rect.y,
                    image::Rgba([
                        (pixel.color.r * 255.0) as u8,
                        (pixel.color.g * 255.0) as u8,
                        (pixel.color.b * 255.0) as u8,
                        255,
                    ]));
            }

            self.progress = Some(update.progress);
        }
    }
}

fn render_progress(ui: &imgui::Ui, progress: &beam::render::RenderProgress)
{
    ui.text(&progress.actions);
    ui.text("Total Duration:");
    ui.text(duration_to_str(&progress.total_duration));
    ui.text("Avg Sample Duration:");
    ui.text(duration_to_str(&progress.avg_duration_per_sample));
    ui.text("Total samples:");
    ui.text(progress.stats.num_samples.to_string());
    ui.text("Total  Rays:");
    ui.text(progress.stats.num_rays.to_string());
    ui.text("Max Rays:");
    ui.text(progress.stats.max_rays.to_string());

    if let Some(_) = ui.begin_table("stats", 3)
    {
        ui.table_next_row_with_flags(imgui::TableRowFlags::HEADERS);
        ui.table_next_column();
        ui.text("Termination");
        ui.table_next_column();
        ui.text("Count");
        ui.table_next_column();
        ui.text("Percent");

        ui.table_next_row();
        ui.table_next_column();
        ui.text("Max Rays");
        ui.table_next_column();
        ui.text(progress.stats.stopped_due_to_max_rays.to_string());
        ui.table_next_column();
        ui.text(percent_to_str(progress.stats.stopped_due_to_max_rays, progress.stats.num_samples));

        ui.table_next_row();
        ui.table_next_column();
        ui.text("Min Color");
        ui.table_next_column();
        ui.text(progress.stats.stopped_due_to_min_atten.to_string());
        ui.table_next_column();
        ui.text(percent_to_str(progress.stats.stopped_due_to_min_atten, progress.stats.num_samples));

        ui.table_next_row();
        ui.table_next_column();
        ui.text("Min Probability");
        ui.table_next_column();
        ui.text(progress.stats.stopped_due_to_min_prob.to_string());
        ui.table_next_column();
        ui.text(percent_to_str(progress.stats.stopped_due_to_min_prob, progress.stats.num_samples));
    }
}

fn percent_to_str(num: u64, den: u64) -> String
{
    let percent = 100.0 * (num as f64) / (den as f64);
    format!("{:.3}%", percent)
}

fn duration_to_str(duration: &Duration) -> String
{
    let secs = duration.as_secs_f64();

    if secs > 10.0
    {
        let secs = secs as u64;
        if secs > 3600
        {
            format!("{}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
        }
        else if secs > 60
        {
            format!("{}:{:02}", secs / 60, secs % 60)
        }
        else
        {
            format!("{} s", secs)
        }
    }
    else if secs > 1.0
    {
        format!("{:.3} s", duration.as_secs_f64())
    }
    else if secs > 0.001
    {
        format!("{:.3} ms", duration.as_secs_f64() * 1000.0)
    }
    else
    {
        format!("{:.3} us", duration.as_secs_f64() * 1000000.0)
    }
}