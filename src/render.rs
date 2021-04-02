use crate::color;
use crate::math::Scalar;
use crate::scene::Scene;
use crate::sample::Sampler;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use rand::{thread_rng, seq::SliceRandom};
use rayon::prelude::*;

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

const PIXELS_PER_UPDATE: usize = 10000;

impl Renderer
{
    pub fn new(options: RenderOptions) -> Self
    {
        let (sender, receiver) = std::sync::mpsc::sync_channel(PIXELS_PER_UPDATE);

        let state = RenderState
        {
            options,
            sender,
            progress: "First pass".to_owned(),
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

    pub fn get_progress_str(&self) -> String
    {
        let state = self.arc.lock().unwrap();
        state.progress.clone()
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
    progress: String,
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

    // First, do a quick pass at decreasing sizes

    const MAX_STEP_SIZE: u32 = 1024;

    let mut step = MAX_STEP_SIZE;

    while step > 0
    {
        if !render_pass(&scene, width, height, step, step == MAX_STEP_SIZE, 1, sender.clone())
        {
            return;
        }

        step /= 2;
    }

    {
        let mut state = arc.lock().unwrap();
        state.progress = "Completed 1 sample per pixel".to_owned();
    }

    // Now, do a single pass with 64 x multi-sampling

    if !render_pass(&scene, width, height, 1, true, 64, sender.clone())
    {
        return;
    }

    {
        let mut state = arc.lock().unwrap();
        state.progress = "Completed 64 samples per pixel".to_owned();
    }

    // Finally update to 1000 x multi-sampling

    if !render_pass(&scene, width, height, 1, true, 1000, sender.clone())
    {
        return;
    }

    {
        let mut state = arc.lock().unwrap();
        state.progress = "Completed 1000 samples per pixel".to_owned();
    }

    // We've rendered in as much detail as we want for now

    {
        let mut state = arc.lock().unwrap();

        state.complete = true;
    }
}

fn render_pass(scene: &Scene, width: u32, height: u32, step: u32, all_pixels: bool, samples_per_pixel: u32, sender: std::sync::mpsc::SyncSender<PixelUpdate>) -> bool
{
    // Work out which pixels we need to update, and the size
    // that they are drawn at

    let mut updates = Vec::new();

    for x in (0..width).step_by(step as usize)
    {
        for y in (0..height).step_by(step as usize)
        {
            let x_mod = x % (step * 2);
            let y_mod = y % (step * 2);

            if all_pixels || (x_mod != 0) || (y_mod != 0)
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

                let update = PixelUpdate{
                    x: x,
                    y: y,
                    width: w_step,
                    height: h_step,
                    color: color::RGBA::new(0.0, 0.0, 0.0, 1.0),
                };

                updates.push(update);
            }
        }
    }

    // Shuffle the updates so they occur in a more random order.

    updates.shuffle(&mut thread_rng());

    // Now - in parallel, try and multi-sample each pixel
    // the required number of times.
    // If any send fails (if the render has been aborted),
    // then the "all" operation will short-circuit and return
    // immediately.

    updates.par_iter().all(|update|
    {
        let mut sampler = Sampler::new();
    
        let mut color = color::RGBA::new(0.0, 0.0, 0.0, 1.0);

        for _ in 0..samples_per_pixel
        {
            let u = ((update.x as Scalar) + sampler.uniform_scalar_unit()) / (width as Scalar);
            let v = ((update.y as Scalar) + sampler.uniform_scalar_unit()) / (height as Scalar);

            color = color + scene.sample_pixel(u, v, &mut sampler);
        }

        let update =  PixelUpdate
        {
            x: update.x,
            y: update.y,
            width: update.width,
            height: update.height,
            color: color.divided_by_scalar(samples_per_pixel as Scalar).gamma_corrected_2(),
        };

        if sender.send(update).is_err()
        {
            return false;
        }

        return true;
    })
}
