use crate::color;
use crate::desc::SceneDescription;
use crate::math::Scalar;
use crate::scene::{SamplingMode, Scene, SceneSampleStats};
use crate::sample::Sampler;

use std::time::{Instant, Duration};
use std::thread::JoinHandle;
use crossbeam::channel::Sender;
use itertools::Itertools;
use rand::{thread_rng, seq::SliceRandom};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderIlluminationMode
{
    Local,
    Global,
}

#[derive(Clone)]
pub struct RenderOptions
{
    pub width: u32,
    pub height: u32,
    pub illumination_mode: RenderIlluminationMode,
    pub sampling_mode: SamplingMode,
    pub max_blockiness: u32,
}

impl RenderOptions
{
    pub fn new(width: u32, height: u32) -> Self
    {
        let illumination_mode = RenderIlluminationMode::Global;
        let sampling_mode = SamplingMode::BsdfAndLights;
        let max_blockiness = 1024;

        RenderOptions { width, height, illumination_mode, sampling_mode, max_blockiness }
    }
}

#[derive(Clone)]
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
    pub color: color::LinearRGB
}

pub struct RenderProgress
{
    pub actions: String,
    pub total_duration: Duration,
    pub avg_duration_per_sample: Duration,
    pub stats: SceneSampleStats,
}

pub struct RenderUpdate
{
    pub progress: RenderProgress,
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
    pub fn new(options: RenderOptions, desc: SceneDescription) -> Self
    {
        let (sender, receiver) = crossbeam::channel::bounded(1);

        let thread = Some(std::thread::spawn(move || render_thread(options, desc, sender)));
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

struct SampleUpdate
{
    pub rect: PixelRect,
    pub collector: SampleCollector
}

struct SampleResult
{
    pixels: Vec<SampleUpdate>,
    stats: SceneSampleStats,
    duration: Duration,
}

#[derive(Clone)]
struct SampleCollector
{
    sum: color::LinearRGB,
    samples: u64,
}

impl SampleCollector
{
    pub fn new() -> Self
    {
        SampleCollector
        {
            sum: color::LinearRGB::black(),
            samples: 0,
        }
    }

    pub fn add_sample(&mut self, color: color::LinearRGB, probability: Scalar)
    {
        self.sum = self.sum + color.divided_by_scalar(probability);
        self.samples += 1;
    }

    pub fn add_collection(&mut self, collector: &SampleCollector)
    {
        self.sum = self.sum + collector.sum;
        self.samples += collector.samples;
    }

    pub fn result(&self) -> color::LinearRGB
    {
        self.sum.divided_by_scalar(self.samples as Scalar)
    }
}

struct RenderState
{
    options: RenderOptions,
    scene: Scene,
    stats: SceneSampleStats,
    total_duration: Duration,
    pixels: Vec<SampleCollector>,
}

impl RenderState
{
    fn new(options: RenderOptions, desc: SceneDescription) -> Self
    {
        let num_pixels = (options.width as usize) * (options.height as usize);
        let scene = desc.build_scene(&options);

        RenderState
        {
            options,
            scene,
            stats: SceneSampleStats::new(),
            total_duration: Duration::default(),
            pixels: vec![SampleCollector::new(); num_pixels],
        }
    }
}

fn render_thread(options: RenderOptions, desc: SceneDescription, sender: Sender<RenderUpdate>)
{
    let mut state = RenderState::new(options, desc);

    // First, do a quick pass with local lighting
    // down to half the resolution

    let mut first_local_pass = true;

    {
        const MAX_STEP_SIZE: u32 = 1024;

        let mut step = MAX_STEP_SIZE;

        while (step > 1) && (step > state.options.width) && (step > state.options.height)
        {
            step /= 2;
        }

        while step >= 2
        {
            if step <= state.options.max_blockiness
            {
                if !render_pass(&mut state, step, first_local_pass, 1, 1, &sender)
                {
                    return;
                }
                first_local_pass = false;
            }

            step /= 2;
        }
    }

    // Ensure all pixels have at least one sample taken

    if !render_pass(&mut state, 1, first_local_pass, 1, 1, &sender)
    {
        return;
    }

    if state.options.illumination_mode == RenderIlluminationMode::Global
    {
        // Sample all pixels with additional samples

        let mut completed_samples = 1;

        for requested_samples in [8, 32, 128, 512, 2048, 8096].iter()
        {
            let new_samples = requested_samples - completed_samples;

            if !render_pass(&mut state, 1, true, new_samples, *requested_samples, &sender)
            {
                return;
            }

            completed_samples = *requested_samples;
        }
    }

    // Mark that we're completed

    let final_update = RenderUpdate
    {
        progress: RenderProgress
            {
                actions: "Complete".to_owned(),
                total_duration: state.total_duration,
                avg_duration_per_sample: time_per_sample(&state.total_duration, &state.stats.num_samples),
                stats: state.stats.clone(),
            },
        complete: true,
        pixels: Vec::new(),
    };

    let _ = sender.send(final_update);
}

fn render_pass(state: &mut RenderState, step: u32, all_pixels: bool, new_samples_per_pixel: usize, total_samples_per_pixel: usize, sender: &Sender<RenderUpdate>) -> bool
{
    // Work out which pixels we need to update, and the size
    // that they are drawn at

    let width = state.options.width;
    let height = state.options.height;

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

    let spawn_thread = |chunks: Vec<Vec<PixelRect>>| -> JoinHandle<()>
    {
        let thread_sender = sub_sender.clone();
        let thread_options = state.options.clone();
        let thread_scene = state.scene.clone();

        std::thread::spawn(move || render_pixel_thread(thread_options, thread_scene, new_samples_per_pixel, chunks, thread_sender))
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
            state.stats = state.stats + chunk.stats;
            state.total_duration = state.total_duration + chunk.duration;

            for pixel in chunk.pixels.iter()
            {
                let x = pixel.rect.x;
                let y = pixel.rect.y;
                let index = (y * state.options.width + x) as usize;

                state.pixels[index].add_collection(&pixel.collector);

                pixels.push(PixelUpdate
                {
                    rect: pixel.rect.clone(),
                    color: state.pixels[index].result(),
                });
            }

            collected_chunks += 1;
        }

        let actions = if step > 1
        {
            format!("Preview")
        }
        else
        {
            format!("Rendering {} sample{}/pixel, {:.1}%",
                total_samples_per_pixel,
                if total_samples_per_pixel == 1 { "" } else { "s" },
                100.0 * (collected_chunks as f64) / (num_chunks as f64))
        };

        let progress = RenderProgress
        {
            actions,
            total_duration: state.total_duration,
            avg_duration_per_sample: time_per_sample(&state.total_duration, &state.stats.num_samples),
            stats: state.stats.clone(),
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

fn time_per_sample(duration: &Duration, samples: &u64) -> Duration
{
    if *samples == 0
    {
        Duration::ZERO
    }
    else
    {
        Duration::from_secs_f64(duration.as_secs_f64() / (*samples as f64))
    }
}

fn render_pixel_thread(options: RenderOptions, scene: Scene, new_samples_per_pixel: usize, updates: Vec<Vec<PixelRect>>, sender: Sender<SampleResult>)
{
    let mut sampler = Sampler::new();

    for updates in updates.into_iter()
    {
        let mut stats = SceneSampleStats::new();
        let now = Instant::now();

        let pixels = updates
            .into_iter()
            .map(|update| calculate_update(&options, &scene, &mut sampler, new_samples_per_pixel, &mut stats, update))
            .collect::<Vec<SampleUpdate>>();

        let duration = now.elapsed();

        let result = SampleResult
        {
            pixels,
            stats,
            duration,
        };

        if !sender.send(result).is_ok()
        {
            // The render has been cancelled
            return;
        }
    }
}

fn calculate_update(options: &RenderOptions, scene: &Scene, sampler: &mut Sampler, new_samples_per_pixel: usize, stats: &mut SceneSampleStats, update: PixelRect) -> SampleUpdate
{
    let mut collector = SampleCollector::new();

    match options.illumination_mode
    {
        RenderIlluminationMode::Local =>
        {
            let u = (update.x as Scalar) / (options.width as Scalar);
            let v = (update.y as Scalar) / (options.height as Scalar);

            collector.add_sample(scene.path_trace_local_lighting(u, v, sampler, stats).0, 1.0);
        },
        RenderIlluminationMode::Global =>
        {
            for _ in 0..new_samples_per_pixel
            {
                let u = ((update.x as Scalar) + sampler.uniform_scalar_unit()) / (options.width as Scalar);
                let v = ((update.y as Scalar) + sampler.uniform_scalar_unit()) / (options.height as Scalar);

                let (color, probability) = scene.path_trace_global_lighting(u, v, sampler, stats);
                collector.add_sample(color, probability);
            }
        },
    };

    SampleUpdate
    {
        rect: update,
        collector: collector,
    }
}
