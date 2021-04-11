use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use beam::render::{Renderer, RenderOptions};

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

    let renderer = Renderer::new(RenderOptions::new(WIDTH, HEIGHT));

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
                _ => {},
            }
        }

        if let Some(update) = renderer.get_update()
        {
            for pixel in update.pixels
            {
                surface.fill_rect(
                    sdl2::rect::Rect::new(pixel.rect.x as i32, pixel.rect.y as i32, pixel.rect.width, pixel.rect.height),
                    sdl2::pixels::Color::from(pixel.color.to_u8_tuple()))?;
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
