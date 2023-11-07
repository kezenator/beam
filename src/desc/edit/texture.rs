use std::collections::HashSet;

use crate::desc::edit::Color;
use crate::indexed::{IndexedValue, AnyIndex, TextureIndex};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::import::image::Image;

#[derive(Clone, Debug)]
pub enum Texture
{
    Solid(Color),
    Checkerboard(Color, Color),
    Image(Image),
}

impl Texture
{
    pub fn build(&self) -> crate::texture::Texture
    {
        match self
        {
            Texture::Solid(color) => crate::texture::Texture::Solid(color.into_linear()),
            Texture::Checkerboard(a, b) => crate::texture::Texture::Checkerboard(a.into_linear(), b.into_linear()),
            Texture::Image(image) => crate::texture::Texture::image(image.clone()),
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Texture::Solid(_) => "Solid",
            Texture::Checkerboard(_,_) => "Checkerboard",
            Texture::Image(_) => "Image",
        }
    }

    fn ui_render_combo(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        let cur_tag = self.ui_tag();
        if let Some(_) = ui.imgui.begin_combo(label, cur_tag)
        {
            for entry in [
                Texture::Solid(Color::default()),
                Texture::Checkerboard(Color::default(), Color::default()),
                Texture::Image(Image::new_empty(10, 10)) ]
            {
                let entry_tag = entry.ui_tag();
                let selected = entry_tag == cur_tag;

                if selected
                {
                    ui.imgui.set_item_default_focus();
                }

                if ui.imgui.selectable_config(entry_tag).selected(selected).build()
                {
                    *self = entry;
                    result = true;
                }
            }
        }
        result
    }
}

impl Default for Texture
{
    fn default() -> Self
    {
        Texture::Solid(Color::default())
    }
}

impl IndexedValue for Texture
{
    type Index = TextureIndex;
    
    fn collect_indexes(&self, _indexes: &mut HashSet<AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        self.ui_tag().into()
    }
}

impl UiDisplay for Texture
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        match self
        {
            Texture::Solid(color) =>
            {
                ui.imgui.label_text(label, "Solid");
                color.ui_display(ui, "Color");
            },
            Texture::Checkerboard(a, b) =>
            {
                ui.imgui.label_text(label, "Checkerboard");
                ui.imgui.text("Checkboard");
                a.ui_display(ui, "A");
                b.ui_display(ui, "B");
            },
            Texture::Image(_image) =>
            {
                ui.imgui.label_text(label, "Checkerboard");
                ui.imgui.text("Image");
            },
        }
    }
}

impl UiEdit for Texture
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = self.ui_render_combo(ui, label);
        ui.imgui.indent();

        match self
        {
            Texture::Solid(color) =>
            {
                result |= color.ui_edit(ui, "Color");
            },
            Texture::Checkerboard(a, b) =>
            {
                result |= a.ui_edit(ui, "Color A");
                result |= b.ui_edit(ui, "Color B");
            },
            Texture::Image(_image) =>
            {
                ui.imgui.text("Image");
            }
        }

        ui.imgui.unindent();
        result
    }
}