use std::sync::{Arc, RwLock};
use image::{ImageBuffer, Rgba};

use crate::color::SRGB;
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::{IndexedValue, ImageIndex};
use crate::math::Scalar;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Debug, Clone)]
pub struct Image
{
    data: Arc<RwLock<ImageBuffer<Rgba<f32>, Vec<f32>>>>
}

impl Image
{
    pub fn dimensions(&self) -> (u32, u32)
    {
        let image = self.data.read().unwrap();
        image.dimensions()
    }

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

impl IndexedValue for Image
{
    type Index = ImageIndex;

    fn collect_indexes(&self, _indexes: &mut std::collections::HashSet<crate::indexed::AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        let dimensions = self.dimensions();
        format!("{} x {} pixels", dimensions.0, dimensions.1)
    }
}

impl UiDisplay for Image
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.summary());
    }
}

impl UiEdit for Image
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        ui.imgui.label_text(label, self.summary());
        false
    }
}

impl Default for Image
{
    fn default() -> Self
    {
        Image::new_empty(1, 1)
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