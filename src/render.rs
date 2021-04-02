use crate::color;
use crate::math::Float;
use crate::scene::Scene;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use rand::{thread_rng, seq::SliceRandom};

pub struct RenderOptions
{
    width: u32,
    height: u32,
}

impl RenderOptions
{
    pub fn new(width: u32, height: u32) -> Self
    {
        RenderOptions { width, height }
    }
}

pub struct PixelUpdate
{
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: color::RGBA,
}

pub struct Renderer
{
    arc: Arc<Mutex<RenderState>>,
    thread: Option<JoinHandle<()>>,
    receiver: Option<std::sync::mpsc::Receiver<PixelUpdate>>,
}

const PIXELS_PER_UPDATE: usize = 1000;

impl Renderer
{
    pub fn new(options: RenderOptions) -> Self
    {
        let (sender, receiver) = std::sync::mpsc::sync_channel(PIXELS_PER_UPDATE);

        let state = RenderState
        {
            options,
            sender,
            complete: false,
        };

        let arc = Arc::new(Mutex::new(state));
        let thread_arc = arc.clone();
        let thread = Some(std::thread::spawn(move || render_thread(thread_arc)));
        let receiver = Some(receiver);

        Renderer { arc, thread, receiver }
    }

    pub fn get_updates(&mut self) -> Vec<PixelUpdate>
    {
        let mut result = Vec::new();

        for _ in 0..PIXELS_PER_UPDATE
        {
            if let Ok(update) = self.receiver.as_mut().unwrap().try_recv()
            {
                result.push(update);
            }
            else
            {
                break;
            }
        }

        result
    }

    pub fn is_complete(&self) -> bool
    {
        let state = self.arc.lock().unwrap();
        state.complete
    }
}

impl Drop for Renderer
{
    fn drop(&mut self)
    {
        drop(self.receiver.take());
        self.thread.take().unwrap().join().unwrap();
    }
}

struct RenderState
{
    options: RenderOptions,
    sender: std::sync::mpsc::SyncSender<PixelUpdate>,
    complete: bool,
}

fn render_thread(arc: Arc<Mutex<RenderState>>)
{
    let (width, height, sender) =
    {
        let state = arc.lock().unwrap();
        (state.options.width, state.options.height, state.sender.clone())
    };

    let scene = Scene::new_default();

    const MAX_STEP_SIZE: u32 = 1024;

    let mut step = MAX_STEP_SIZE;

    while step > 0
    {
        let mut updates = Vec::new();

        for x in (0..width).step_by(step as usize)
        {
            for y in (0..height).step_by(step as usize)
            {
                let x_mod = x % (step * 2);
                let y_mod = y % (step * 2);

                if (step == MAX_STEP_SIZE) || (x_mod != 0) || (y_mod != 0)
                {
                    let mut w_step = step;
                    let mut h_step = step;

                    if (x + w_step) > width
                    {
                        w_step = width - x;
                    }

                    if (y + h_step) > height
                    {
                        h_step = height - y;
                    }

                    let u = (x as Float) / (width as Float);
                    let v = (y as Float) / (height as Float);

                    let update = PixelUpdate{
                        x: x,
                        y: y,
                        width: w_step,
                        height: h_step,
                        color: scene.sample_pixel(u, v),
                    };

                    updates.push(update);
                }
            }
        }

        updates.shuffle(&mut thread_rng());

        for update in updates.drain(..)
        {
            if sender.send(update).is_err()
            {
                return;
            }
        }

        step /= 2;
    }

    {
        let mut state = arc.lock().unwrap();

        state.complete = true;
    }
}
