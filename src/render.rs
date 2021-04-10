use crate::color;
use crate::math::Scalar;
use crate::scene::Scene;
use crate::sample::Sampler;

use std::thread::JoinHandle;
use crossbeam::channel::{Sender};
use itertools::Itertools;
use rand::{thread_rng, RngCore, seq::SliceRandom};

#[derive(Clone)]
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

pub struct RenderUpdate
{
    pub progress: String,
    pub complete: bool,
    pub pixels: Vec<PixelUpdate>,
}

pub struct Renderer
{
    thread: Option<JoinHandle<()>>,
    receiver: Option<crossbeam::channel::Receiver<RenderUpdate>>,
}

impl Renderer
{
    pub fn new(options: RenderOptions) -> Self
    {
        let (sender, receiver) = crossbeam::channel::bounded(1);

        let thread = Some(std::thread::spawn(move || render_thread(options, sender)));
        let receiver = Some(receiver);

        Renderer { thread, receiver }
    }

    pub fn get_update(&self) -> Option<RenderUpdate>
    {
        self.receiver.as_ref().unwrap().try_recv().ok()
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

fn render_thread(options: RenderOptions, sender: Sender<RenderUpdate>)
{
    // First, do a quick pass with local lighting
    // down to half the resolution

    const MAX_STEP_SIZE: u32 = 1024;

    let mut step = MAX_STEP_SIZE;

    while step >= 2
    {
        if !render_pass(&options, step, step == MAX_STEP_SIZE, 1, false, &sender)
        {
            return;
        }

        step /= 2;
    }

    // Now sample all pixels, with global lighting
    // at increasing samples per pixel

    for samples in [1, 8, 64, 1000].iter()
    {
        if !render_pass(&options, 1, true, *samples, true, &sender)
        {
            return;
        }
    }

    // Mark that we're completed

    let final_update = RenderUpdate
    {
        progress: "Completed".to_owned(),
        complete: true,
        pixels: Vec::new(),
    };

    let _ = sender.send(final_update);
}

fn render_pass(options: &RenderOptions, step: u32, all_pixels: bool, samples_per_pixel: usize, global_illum: bool, sender: &Sender<RenderUpdate>) -> bool
{
    // Work out which pixels we need to update, and the size
    // that they are drawn at

    let width = options.width;
    let height = options.height;

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

    // Break the updates into chunks of updates

    let num_updates = updates.len();
    let num_threads = num_cpus::get();

    let updates_per_chunk = (num_updates / num_threads)
        .max(1000)
        .min(1);

    let chunks: Vec<Vec<PixelUpdate>> = updates
        .into_iter()
        .chunks(updates_per_chunk)
        .into_iter()
        .map(|ch| ch.collect::<Vec<_>>())
        .collect();

    let num_chunks = chunks.len();

    // Break this into threads, and start threads to run them

    let chunks_per_thread = (chunks.len() + num_threads - 1) / num_threads;

    let (sub_sender, sub_receiver) = crossbeam::channel::unbounded();

    let spawn_thread = |chunks: Vec<Vec<PixelUpdate>>| -> JoinHandle<()>
    {
        let thread_sender = sub_sender.clone();
        let thread_options = options.clone();

        std::thread::spawn(move || render_pixel_thread(thread_options, samples_per_pixel, global_illum, chunks, thread_sender))
    };

    let join_handles: Vec<JoinHandle<()>> = chunks
        .into_iter()
        .chunks(chunks_per_thread)
        .into_iter()
        .map(|i| i.collect::<Vec<_>>())
        .map(|chunks| spawn_thread(chunks))
        .collect::<Vec<_>>();

    // Receive updates from the threads and aggregate these
    // into the completed results

    let mut collected_chunks = 0;

    while collected_chunks < num_chunks
    {
        let mut pixels = Vec::new();

        while let Ok(chunk) = sub_receiver.try_recv()
        {
            collected_chunks += 1;
            pixels.extend(chunk);
        }

        let progress = if global_illum
        {
            format!("Rendering {} samples per pixel", samples_per_pixel)
        }
        else
        {
            "Preview (Local Illumination)".to_owned()
        };

        let complete = false;

        let render_update = RenderUpdate
        {
            progress,
            complete,
            pixels,
        };

        if !sender.send(render_update).is_ok()
        {
            // The overall renderer has been closed - 
            // abort

            return false;
        }
    }

    // All results collected - wait for the
    // threads to complete and return that it was
    // completed successfully.

    let _ = join_handles
        .into_iter()
        .map(|jh| jh.join().unwrap())
        .last();

    true
}

fn render_pixel_thread(options: RenderOptions, samples_per_pixel: usize, global_illum: bool, updates: Vec<Vec<PixelUpdate>>, sender: Sender<Vec<PixelUpdate>>)
{
    let scene = Scene::new_default();
    let mut sampler = Sampler::new_reproducable(thread_rng().next_u64());

    for updates in updates.into_iter()
    {
        let results = updates
            .into_iter()
            .map(|update| calculate_update(&options, &scene, &mut sampler, samples_per_pixel, global_illum, update))
            .collect::<Vec<PixelUpdate>>();

        if !sender.send(results).is_ok()
        {
            // The render has been cancelled
            return;
        }
    }
}

fn calculate_update(options: &RenderOptions, scene: &Scene, sampler: &mut Sampler, samples_per_pixel: usize, global_illum: bool, update: PixelUpdate) -> PixelUpdate
{
    let mut color = color::RGBA::new(0.0, 0.0, 0.0, 1.0);

    if !global_illum
    {
        let u = (update.x as Scalar) / (options.width as Scalar);
        let v = (update.y as Scalar) / (options.height as Scalar);

        color = scene.quick_trace_pixel(u, v, sampler);
    }
    else
    {
        for _ in 0..samples_per_pixel
        {
            let u = ((update.x as Scalar) + sampler.uniform_scalar_unit()) / (options.width as Scalar);
            let v = ((update.y as Scalar) + sampler.uniform_scalar_unit()) / (options.height as Scalar);

            color = color + scene.path_trace_pixel(u, v, sampler);
        }
    }

    PixelUpdate
    {
        x: update.x,
        y: update.y,
        width: update.width,
        height: update.height,
        color: color.divided_by_scalar(samples_per_pixel as Scalar).gamma_corrected_2(),
    }
}
