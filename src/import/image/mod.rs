use std::sync::{Arc, RwLock};
use image::{ImageBuffer, Rgba};

use crate::color::SRGB;
use crate::import::{FileSystemContext, ImportError};
use crate::math::Scalar;

#[derive(Debug, Clone)]
pub struct Image
{
    data: Arc<RwLock<ImageBuffer<Rgba<f32>, Vec<f32>>>>
}

impl Image
{
    pub fn sample_at_uv(&self, u: Scalar, v: Scalar) -> SRGB
    {
        let image = self.data.read().unwrap();
        let (w, h) = image.dimensions();

        let x = (u * ((w - 1) as Scalar)).round() as u32;
        let y = (v * ((h - 1) as Scalar)).round() as u32;

        let x = x.clamp(0, w -1);
        let y = y.clamp(0, h - 1);

        let color = image.get_pixel(x, y);

        SRGB::new(color.0[0] as Scalar, color.0[1] as Scalar, color.0[2] as Scalar, color.0[3] as Scalar)
    }

    pub fn new_empty(w: u32, h: u32) -> Self
    {
        Image { data: Arc::new(RwLock::new(image::ImageBuffer::new(w, h))) }
    }
}

pub fn import_image(path: &str, context: &mut FileSystemContext) -> Result<Image, ImportError>
{
    let (contents, _sub_context) = context.load_binary_file(path)?;

    match image::load_from_memory(&contents)
    {
        Ok(image) => Ok(Image { data: Arc::new(RwLock::new(image.into_rgba32f())) }),
        Err(err) => Err(ImportError(err.to_string())),
    }
}