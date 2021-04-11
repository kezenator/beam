use crate::color;
use crate::math::Scalar;
use crate::scene::Scene;
use crate::sample::Sampler;

use std::time::{Instant, Duration};
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

pub struct PixelRect
{
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct PixelUpdate
{
    pub rect: PixelRect,
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

struct SampleResult
{
    pixels: Vec<PixelUpdate>,
    duration: Duration,
}

struct RenderState
{
    local_samples: u64,
    local_duration: Duration,
    global_samples: u64,
    global_duration: Duration,
    completed_global_samples_per_pixel: usize,
    pixels: Vec<color::RGBA>,
}

impl RenderState
{
    fn new(options: &RenderOptions) -> Self
    {
        RenderState
        {
            local_samples: 0,
            local_duration: Duration::default(),
            global_samples: 0,
            global_duration: Duration::default(),
            completed_global_samples_per_pixel: 0,
            pixels: vec![color::RGBA::new(0.0, 0.0, 0.0, 1.0); (options.width as usize) * (options.height as usize)],
        }
    }
}

fn render_thread(options: RenderOptions, sender: Sender<RenderUpdate>)
{
    let mut state = RenderState::new(&options);

    // First, do a quick pass with local lighting
    // down to half the resolution

    const MAX_STEP_SIZE: u32 = 1024;

    let mut step = MAX_STEP_SIZE;

    while step >= 2
    {
        if !render_pass(&options, &mut state, step, step == MAX_STEP_SIZE, 1, false, &sender)
        {
            return;
        }

        step /= 2;
    }

    // Now sample all pixels, with global lighting
    // at increasing samples per pixel

    for samples in [1, 8, 32, 128, 512, 2048].iter()
    {
        if !render_pass(&options, &mut state, 1, true, *samples, true, &sender)
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

fn render_pass(options: &RenderOptions, state: &mut RenderState, step: u32, all_pixels: bool, samples_per_pixel: usize, global_illum: bool, sender: &Sender<RenderUpdate>) -> bool
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

                let update = PixelRect{
                    x: x,
                    y: y,
                    width: w_step,
                    height: h_step,
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

    let chunks: Vec<Vec<PixelRect>> = updates
        .into_iter()
        .chunks(updates_per_chunk)
        .into_iter()
        .map(|ch| ch.collect::<Vec<_>>())
        .collect();

    let num_chunks = chunks.len();

    // Break this into threads, and start threads to run them

    let chunks_per_thread = (chunks.len() + num_threads - 1) / num_threads;

    let (sub_sender, sub_receiver) = crossbeam::channel::unbounded();

    let new_samples_per_pixel = if global_illum { samples_per_pixel - state.completed_global_samples_per_pixel } else { samples_per_pixel };

    let spawn_thread = |chunks: Vec<Vec<PixelRect>>| -> JoinHandle<()>
    {
        let thread_sender = sub_sender.clone();
        let thread_options = options.clone();

        std::thread::spawn(move || render_pixel_thread(thread_options, new_samples_per_pixel, global_illum, chunks, thread_sender))
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
            let duration = chunk.duration;
            let mut new_pixels = chunk.pixels;

            if global_illum
            {
                state.global_samples += (new_pixels.len() as u64) * (new_samples_per_pixel as u64);
                state.global_duration += duration;

                for mut pixel in new_pixels.iter_mut()
                {
                    let x = pixel.rect.x;
                    let y = pixel.rect.y;
                    let index = (y * options.width + x) as usize;

                    let sum = state.pixels[index].clone();
                    let sum = sum + pixel.color.clone();

                    state.pixels[index] = sum.clone();

                    pixel.color = sum.divided_by_scalar(samples_per_pixel as Scalar).gamma_corrected_2();
                }
            }
            else
            {
                state.local_samples += (new_pixels.len() as u64) * (new_samples_per_pixel as u64);
                state.local_duration += duration;

                for mut pixel in new_pixels.iter_mut()
                {
                    pixel.color = pixel.color.gamma_corrected_2();
                }
            }

            collected_chunks += 1;
            pixels.extend(new_pixels);
        }

        let timing = format!(" [{} global, {} local]",
            time_per_sample(&state.global_duration, &state.global_samples),
            time_per_sample(&state.local_duration, &state.local_samples));

        let actions = if global_illum
        {
            format!("Rendering {} samples/pixel, {:.1}%",
                samples_per_pixel,
                100.0 * (collected_chunks as f64) / (num_chunks as f64))
        }
        else
        {
            "Preview".to_owned()
        };

        let progress = format!("{}{}", actions, timing);

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

    // Mark how many samples have been completed

    if global_illum
    {
        state.completed_global_samples_per_pixel = samples_per_pixel;
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

fn time_per_sample(duration: &Duration, samples: &u64) -> String
{
    let tps = if *samples == 0
    {
        0.0
    }
    else
    {
        duration.as_secs_f64() / (*samples as f64)
    };

    format!("{:.2} us/sample", tps * 1000000.0)
}

fn render_pixel_thread(options: RenderOptions, new_samples_per_pixel: usize, global_illum: bool, updates: Vec<Vec<PixelRect>>, sender: Sender<SampleResult>)
{
    let scene = Scene::new_default(options.width, options.height);
    let mut sampler = Sampler::new_reproducable(thread_rng().next_u64());

    for updates in updates.into_iter()
    {
        let now = Instant::now();

        let pixels = updates
            .into_iter()
            .map(|update| calculate_update(&options, &scene, &mut sampler, new_samples_per_pixel, global_illum, update))
            .collect::<Vec<PixelUpdate>>();

        let duration = now.elapsed();

        let result = SampleResult
        {
            pixels,
            duration,
        };

        if !sender.send(result).is_ok()
        {
            // The render has been cancelled
            return;
        }
    }
}

fn calculate_update(options: &RenderOptions, scene: &Scene, sampler: &mut Sampler, new_samples_per_pixel: usize, global_illum: bool, update: PixelRect) -> PixelUpdate
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
        for _ in 0..new_samples_per_pixel
        {
            let u = ((update.x as Scalar) + sampler.uniform_scalar_unit()) / (options.width as Scalar);
            let v = ((update.y as Scalar) + sampler.uniform_scalar_unit()) / (options.height as Scalar);

            color = color + scene.path_trace_pixel(u, v, sampler);
        }
    }

    PixelUpdate
    {
        rect: update,
        color: color,
    }
}
