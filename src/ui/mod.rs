mod pixel;
mod system;

pub use system::System;
pub use pixel::PixelDisplay;

use crate::vec::Vec3;

pub trait UiApplication<T: 'static>
{
    fn handle_event(&mut self, event: winit::event::Event<T>) -> Option<winit::event_loop::ControlFlow>;
    fn render_background(&mut self, display: &glium::Display, frame: &mut glium::Frame);
    fn render_ui(&mut self, ui: &UiRenderer);
    fn idle(&mut self);
}

pub trait UiDisplay
{
    fn ui_display(&self, ui: &UiRenderer, label: &str);
}

pub trait UiEdit
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool;
}

pub trait UiTaggedEnum
{
    type TagEnum: PartialEq + Eq + Copy + 'static;

    fn all_tags() -> &'static [Self::TagEnum];
    fn display_for_tag(tag: Self::TagEnum) -> &'static str;
    fn default_val_for_tag(tag: Self::TagEnum) -> Self;
    fn get_tag(&self) -> Self::TagEnum;
}

pub struct UiRenderer<'a>
{
    pub imgui: &'a imgui::Ui
}

impl<'a> UiRenderer<'a>
{
    pub fn new(imgui: &'a imgui::Ui) -> Self
    {
        Self { imgui }
    }

    pub fn display_float(&self, label: &str, val: &f64)
    {
        self.imgui.label_text(label, format!("{}", val));
    }

    pub fn display_vec3(&self, label: &str, val: &Vec3)
    {
        self.imgui.label_text(label, format!("<{}, {}, {}>", val[0], val[1], val[2]));
    }

    pub fn display_tag<T: UiTaggedEnum>(&self, label: &str, val: &T)
    {
        self.imgui.label_text(label, T::display_for_tag(val.get_tag()));
    }

    pub fn edit_float(&self, label: &str, val: &mut f64) -> bool
    {
        let mut as_f32 = *val as f32;
        let result = self.imgui.input_float(label, &mut as_f32).build();

        if result
        {
            *val = as_f32 as f64;
        }
        
        result
    }

    pub fn edit_vec3(&self, label: &str, val: &mut Vec3) -> bool
    {
        let mut as_f32 = [val[0] as f32, val[1] as f32, val[2] as f32];
        let result = self.imgui.input_float3(label, &mut as_f32).build();

        if result
        {
            *val = Vec3::new(as_f32[0] as f64, as_f32[1] as f64, as_f32[2] as f64);
        }
        
        result
    }

    pub fn edit_tag<T: UiTaggedEnum>(&self, label: &str, val: &mut T) -> bool
    {
        let mut result = false;
        let cur_tag = val.get_tag();

        if let Some(_combo) = self.imgui.begin_combo(label, T::display_for_tag(cur_tag))
        {
            for tag in T::all_tags().iter()
            {
                let tag_str = T::display_for_tag(*tag);
                let selected = *tag == cur_tag;

                if selected
                {
                    self.imgui.set_item_default_focus();
                }

                if self.imgui.selectable_config(tag_str).selected(selected).build()
                {
                    *val = T::default_val_for_tag(*tag);
                    result = true;
                }
            }
        }

        result
    }
}