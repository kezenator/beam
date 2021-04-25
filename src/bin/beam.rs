use std::io::prelude::*;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use std::time::Duration;

use beam::color::LinearRGB;
use beam::desc::{SceneDescription, StandardScene};
use beam::math::Scalar;
use beam::render::{Renderer, RenderOptions, RenderIlluminationMode};
use beam::sample::Sampler;
use beam::scene::{SamplingMode, SceneSampleStats};
use beam::vec::{Mat4, Vec3, Vec4};

fn main() -> Result<(), String>
{
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    unsafe
    {
        const PROCESS_SYSTEM_DPI_AWARE: u32 = 1;
        winapi::um::shellscalingapi::SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut surface = sdl2::surface::Surface::new(WIDTH, HEIGHT, sdl2::pixels::PixelFormatEnum::RGBA8888)?;

    let window = video_subsystem.window("Beam - Rendering...", WIDTH, HEIGHT)
        .allow_highdpi()
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();

    let mut app_state = AppState::new(WIDTH, HEIGHT);
    let mut renderer = app_state.new_renderer();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop
    {
        for event in event_pump.poll_iter()
        {
            match event
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(keycode), keymod, .. } =>
                {
                    if app_state.handle_keycode(keycode, keymod)
                    {
                        renderer = app_state.new_renderer();
                    }
                },
                Event::MouseButtonDown { x, y, .. } =>
                {
                    app_state.samples_to_csv(x, y);
                },
                _ => {},
            }
        }

        if let Some(update) = renderer.get_update()
        {
            for pixel in update.pixels
            {
                surface.fill_rect(
                    sdl2::rect::Rect::new(pixel.rect.x as i32, pixel.rect.y as i32, pixel.rect.width, pixel.rect.height),
                    sdl2::pixels::Color::from(pixel.color.to_srgb().to_u8_rgba_tuple()))?;
            }

            canvas.window_mut().set_title(&format!("Beam - {}", update.progress)).expect("Could not set window title");
        }


        let texture = surface.as_texture(&texture_creator).unwrap();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

struct AppState
{
    options: RenderOptions,
    desc: SceneDescription,
}

impl AppState
{
    pub fn new(width: u32, height: u32) -> Self
    {
        AppState
        {
            options: RenderOptions::new(width, height),
            desc: SceneDescription::new(StandardScene::Cornell),
        }
    }

    pub fn new_renderer(&self) -> Renderer
    {
        Renderer::new(self.options.clone(), self.desc.clone())
    }

    pub fn handle_keycode(&mut self, keycode: Keycode, keymod: Mod) -> bool
    {
        let ctrl = keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
            || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD);

        let handled = match keycode
        {
            Keycode::Num1 =>
            {
                self.desc = SceneDescription::new(StandardScene::BeamExample);
                true
            },
            Keycode::Num2 =>
            {
                self.desc = SceneDescription::new(StandardScene::Cornell);
                true
            },
            Keycode::Num3 =>
            {
                self.desc = SceneDescription::new(StandardScene::Furnace);
                true
            },
            Keycode::Num4 =>
            {
                self.desc = SceneDescription::new(StandardScene::Veach);
                true
            },
            Keycode::F1 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::BsdfAndLights;
                true
            }
            Keycode::F2 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::LightsOnly;
                true
            }
            Keycode::F3 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::BsdfOnly;
                true
            }
            Keycode::F4 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Global;
                self.options.sampling_mode = SamplingMode::Uniform;
                true
            }
            Keycode::F5 =>
            {
                self.options.illumination_mode = RenderIlluminationMode::Local;
                true
            }
            Keycode::C =>
            {
                println!("Camera Location: {:?}", self.desc.camera_location);
                println!("Camera Look-at:  {:?}", self.desc.camera_look_at);
                println!("Camera Up:       {:?}", self.desc.camera_up);
                println!("Camera FOV:      {:?}", self.desc.camera_fov);
                false
            },
            Keycode::L =>
            {
                self.options.illumination_mode = match self.options.illumination_mode
                {
                    RenderIlluminationMode::Local => RenderIlluminationMode::Global,
                    RenderIlluminationMode::Global => RenderIlluminationMode::Local,
                };
                self.options.sampling_mode = SamplingMode::BsdfAndLights;
                true
            },
            Keycode::Left =>
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
            Keycode::Right =>
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
            Keycode::Up =>
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
            Keycode::Down =>
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
            Keycode::KpPlus =>
            {
                self.desc.camera_fov = (self.desc.camera_fov - 5.0).clamp(1.0, 175.0);
                true
            },
            Keycode::KpMinus =>
            {
                self.desc.camera_fov = (self.desc.camera_fov + 5.0).clamp(1.0, 175.0);
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
        let look = self.desc.camera_look_at - self.desc.camera_location;
        let right = look.cross(self.desc.camera_up).normalized();
        let back = right.cross(self.desc.camera_up).normalized();
        let up = self.desc.camera_up.normalized();
        
        // Don't move in the up/down direction
        let right = right - (up.dot(right) * up);
        let back = back - (up.dot(right) * up);

        // Work out how far to move
        let dist_factor = 0.05 * look.magnitude();

        let dir = dist_factor * (factor_left_right * right + factor_forward_back * back);

        self.desc.camera_location += dir;
        self.desc.camera_look_at += dir;
    }

    fn rotate_around(&mut self, degrees: Scalar)
    {
        let dir = self.desc.camera_location - self.desc.camera_look_at;

        let rot = Mat4::rotation_3d(degrees.to_radians(), self.desc.camera_up);

        let new_dir: Vec3 = (rot * Vec4::from_direction(dir)).into();

        self.desc.camera_location = new_dir + self.desc.camera_look_at;
    }

    fn tilt(&mut self, degrees: Scalar)
    {
        let dir = self.desc.camera_location - self.desc.camera_look_at;

        let right = dir.cross(self.desc.camera_up);

        let rot = Mat4::rotation_3d(degrees.to_radians(), right);

        let new_dir: Vec3 = (rot * Vec4::from_direction(dir)).into();

        self.desc.camera_location = new_dir + self.desc.camera_look_at;
    }

    fn samples_to_csv(&self, x: i32, y: i32)
    {
        const NUM_SAMPLES: usize = 32;

        let scene = self.desc.build_scene(&self.options);
        let mut sampler = Sampler::new();
        let mut stats = SceneSampleStats::new();

        let mut color = LinearRGB::black();
        let mut csv = "R,G,B,Probability\n".to_owned();
        
        for _ in 0..NUM_SAMPLES
        {
            let u = ((x as Scalar) + sampler.uniform_scalar_unit()) / (self.options.width as Scalar);
            let v = ((y as Scalar) + sampler.uniform_scalar_unit()) / (self.options.height as Scalar);

            let (sample, probability) = scene.path_trace_global_lighting(u, v, &mut sampler, &mut stats);

            color = color + sample;

            let line: String = format!("{},{},{},{}\n", sample.r, sample.g, sample.b, probability);
            csv += &line;
        }

        let mut file = std::fs::File::create("samples.csv").expect("Could not write CSV file");
        file.write_all(csv.as_bytes()).expect("Could not write CSV file");

        println!("Wrote \"samples.csv\"");
        println!("Result = {:?}", color.divided_by_scalar(NUM_SAMPLES as Scalar));
    }
}
