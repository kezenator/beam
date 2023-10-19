use crate::color::{SRGB, LinearRGB};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Clone, Copy, Debug)]
pub struct Color
{
    linear: LinearRGB,
}

impl Color
{
    pub fn new() -> Self
    {
        Color
        {
            linear: LinearRGB::new(0.0, 0.0, 0.0),
        }
    }

    pub fn into_linear(&self) -> LinearRGB
    {
        self.linear
    }
}

impl From<LinearRGB> for Color
{
    fn from(linear: LinearRGB) -> Self
    {
        Color { linear }
    }
}

impl From<SRGB> for Color
{
    fn from(srgb: SRGB) -> Self
    {
        Color { linear: srgb.into() }
    }
}

impl Default for Color
{
    fn default() -> Self
    {
        Self { linear: LinearRGB::new(0.0, 0.0, 0.0), }
    }
}

impl UiDisplay for Color
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        let mut slice: [f32; 3] = self.linear.to_srgb().into();
        ui.imgui.color_edit3_config(label, &mut slice).inputs(false).build();
    }
}

impl UiEdit for Color
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut slice: [f32; 3] = self.linear.to_srgb().into();
        if ui.imgui.color_edit3(label, &mut slice)
        {
            let srgb: SRGB = slice.into();
            (*self).linear = srgb.into();
            return true;
        }
        false
    }
}