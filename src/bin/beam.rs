use std::time::Duration;

use glium::Surface;
use winit::event::{ElementState, Event, ModifiersState, VirtualKeyCode, WindowEvent};

use beam::desc::{SceneDescription, StandardScene};
use beam::math::Scalar;
use beam::render::{Renderer, RenderOptions, RenderIlluminationMode};
use beam::scene::SamplingMode;
use beam::ui::{UiDisplay, UiEdit, UiRenderer};
use beam::vec::{Mat4, Vec3, Vec4};


fn main() -> Result<(), String>
{
    let filename = std::env::args().nth(1);
    let filecontents = filename.map(|filename| std::fs::read_to_string(filename).unwrap());
    let system = beam::ui::System::init("Beam");
    let app_state = AppState::new(&system, 128, 128, filecontents);
    system.main_loop(app_state);
}

struct AppState
{
    filename: Option<String>,
    downscale: u32,
    options: RenderOptions,
    desc: SceneDescription,
    renderer: Renderer,
    pixels: beam::ui::PixelDisplay,
    progress: Option<beam::render::RenderProgress>,
    keyboard_modifiers: winit::event::ModifiersState,
    scene: beam::desc::edit::Scene,
    source: String,
}

impl AppState
{
    pub fn new(system: &beam::ui::System<()>, width: u32, height: u32, default_file: Option<String>) -> Self
    {
        let filename = None;
        let downscale = 1;
        let options = RenderOptions::new(width, height);
        let mut desc = SceneDescription::new_standard(StandardScene::Cornell);
        let renderer = Renderer::new(options.clone(), desc.clone());
        let pixels = beam::ui::PixelDisplay::new(system.display(), width, height);
        let progress = None;
        let keyboard_modifiers = ModifiersState::empty();
        let mut scene = beam::desc::edit::Scene::new();
        let mut source = "camera{\n   location: <0.0, 0.0, 9.0>,\n   look_at: <0.0, 0.0, 0.0>,\n   up: <0.0, 1.0, 0.0>,\n   fov: 40.0,\n}".to_owned();

        if let Some(default_file) = default_file
        {
            scene = beam::desc::run_script(&default_file).unwrap();
            source = default_file;
            desc = SceneDescription::new_script(source.clone()).unwrap();
        }

        AppState
        {
            filename,
            downscale,
            options,
            desc,
            renderer,
            pixels,
            progress,
            keyboard_modifiers,
            scene,
            source,
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
        let frame_dimensions = frame.get_dimensions();
        let desired_dimensions = (frame_dimensions.0 / self.downscale, frame_dimensions.1 / self.downscale);
        if desired_dimensions != self.pixels.dimensions()
        {
            let (width, height) = desired_dimensions;
            self.pixels.resize(width, height);
            self.options.width = width;
            self.options.height = height;
            self.renderer = self.new_renderer();
        }
        self.pixels.render(display, frame);
    }

    fn render_ui(&mut self, ui: &UiRenderer)
    {
        if let Some(progress) = &self.progress
        {
            if let Some(_progress_window) = ui.imgui.window("Progress").begin()
            {
                if render_progress(ui.imgui, &mut self.downscale, &mut self.options, progress)
                {
                    self.renderer = self.new_renderer();
                }
            }
        }
        
        if let Some(_editor_window) = ui.imgui.window("Editor Demo").begin()
        {
            {
                let _script = ui.imgui.push_id("script");

                ui.imgui.input_text_multiline("source", &mut self.source, [300.0, 100.0])
                    .build();

                if ui.imgui.button("Run")
                {
                    if let Ok(scene) = beam::desc::run_script(&self.source)
                    {
                        self.scene = scene;
                    }
                }
            }

            self.scene.ui_display(ui, "Display");
            self.scene.ui_edit(ui, "Edit");

            if ui.imgui.button("Build")
            {
                self.desc = SceneDescription::new_edit(&self.scene);
                self.renderer = self.new_renderer();                
            }
        }

        ui.imgui.show_metrics_window(&mut true);
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

fn render_progress(ui: &imgui::Ui, downscale: &mut u32, options: &mut RenderOptions, progress: &beam::render::RenderProgress) -> bool
{
    let mut changed = false;

    changed |= ui.input_scalar("Downscale", downscale).build();

    if let Some(_) = ui.begin_combo("Illumination", format!("{:?}", options.illumination_mode))
    {
        if ui.selectable(format!("{:?}", beam::render::RenderIlluminationMode::Global))
        {
            changed = true;
            options.illumination_mode = beam::render::RenderIlluminationMode::Global;
        }
        if ui.selectable(format!("{:?}", beam::render::RenderIlluminationMode::Local))
        {
            changed = true;
            options.illumination_mode = beam::render::RenderIlluminationMode::Local;
        }
    }

    if options.illumination_mode == beam::render::RenderIlluminationMode::Global
    {
        if let Some(_) = ui.begin_combo("Sampling", format!("{:?}", options.sampling_mode))
        {
            if ui.selectable(format!("{:?}", beam::scene::SamplingMode::Uniform))
            {
                changed = true;
                options.sampling_mode = beam::scene::SamplingMode::Uniform;
            }
            if ui.selectable(format!("{:?}", beam::scene::SamplingMode::BsdfOnly))
            {
                changed = true;
                options.sampling_mode = beam::scene::SamplingMode::BsdfOnly;
            }
            if ui.selectable(format!("{:?}", beam::scene::SamplingMode::LightsOnly))
            {
                changed = true;
                options.sampling_mode = beam::scene::SamplingMode::LightsOnly;
            }
            if ui.selectable(format!("{:?}", beam::scene::SamplingMode::BsdfAndLights))
            {
                changed = true;
                options.sampling_mode = beam::scene::SamplingMode::BsdfAndLights;
            }
        }
    }

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

    changed
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