use std::collections::HashSet;

use crate::desc::edit::Color;
use crate::indexed::{IndexedValue, IndexedCollection, AnyIndex, ImageIndex, TextureIndex};
use crate::math::Scalar;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::{Mat4, Point3};

#[derive(Clone, Debug)]
pub enum Texture
{
    Solid(Color),
    Checkerboard(Color, Color),
    Image
    {
        base_color: Color,
        image: ImageIndex,
        scale: Point3,
        rotate: Scalar,
        translate: Point3,
    },
}

impl Texture
{
    pub fn build(&self, collection: &IndexedCollection) -> crate::texture::Texture
    {
        match self
        {
            Texture::Solid(color) => crate::texture::Texture::Solid(color.into_linear()),
            Texture::Checkerboard(a, b) => crate::texture::Texture::Checkerboard(a.into_linear(), b.into_linear()),
            Texture::Image{base_color, image, scale, rotate, translate} =>
            {
                let image = collection.map_item(*image, |i, _| i.clone());

                let mut transform = Mat4::scaling_3d(*scale);
                transform.rotate_3d(*rotate, Point3::new(0.0, 0.0, 1.0));
                transform.translate_3d(*translate);
                crate::texture::Texture::image(base_color.into_linear(), image, transform)
            },
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Texture::Solid(_) => "Solid",
            Texture::Checkerboard(_,_) => "Checkerboard",
            Texture::Image{..} => "Image",
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
                Texture::Image{
                    base_color: Color::default(),
                    image: ImageIndex::default(),
                    scale: Point3::new(1.0, 1.0, 1.0),
                    rotate: 0.0,
                    translate: Point3::new(0.0, 0.0, 0.0)} ]
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
            Texture::Image{base_color, image, scale, rotate, translate } =>
            {
                ui.imgui.label_text(label, "Image");
                base_color.ui_display(ui, "Base Color");
                image.ui_display(ui, "Image");
                ui.display_vec3("Scale", scale);
                ui.display_angle("Rotate", rotate);
                ui.display_vec3("Translate", translate);
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
            Texture::Image{ base_color, image, scale, rotate, translate, } =>
            {
                result |= base_color.ui_edit(ui, "Base Color");
                result |= image.ui_edit(ui, "Image");
                result |= ui.edit_vec3("Scale", scale);
                result |= ui.edit_angle("Rotate", rotate);
                result |= ui.edit_vec3("Translate", translate);
            }
        }

        ui.imgui.unindent();
        result
    }
}