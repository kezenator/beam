use std::collections::HashSet;

use crate::indexed::{Index, IndexedValue, AnyIndex, TextureIndex};
use crate::math::Scalar;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use super::Scene;

#[derive(Clone, Debug)]
pub enum Material
{
    Dielectric { ior: Scalar },
    Diffuse{ texture: TextureIndex },
    Emit{ texture: TextureIndex },
    Metal{ texture: TextureIndex, fuzz: Scalar },
}

impl Material
{
    pub fn build(&self, scene: &Scene) -> crate::material::Material
    {
        match self
        {
            Material::Dielectric{ior} => crate::material::Material::Dielectric(*ior),
            Material::Diffuse{texture} => crate::material::Material::Diffuse(scene.build_texture(*texture)),
            Material::Emit{texture} => crate::material::Material::Emit(scene.build_texture(*texture)),
            Material::Metal{texture, fuzz} => crate::material::Material::Metal(scene.build_texture(*texture), *fuzz),
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Material::Dielectric{..} => "Dielectric",
            Material::Diffuse{..} => "Diffuse",
            Material::Emit{..} => "Emit",
            Material::Metal{..} => "Metal",
        }
    }

    fn ui_render_combo(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        let cur_tag = self.ui_tag();
        if let Some(_) = ui.imgui.begin_combo(label, cur_tag)
        {
            for entry in [
                Material::Dielectric{ ior: 1.5 },
                Material::Diffuse{ texture: TextureIndex::from_usize(0) },
                Material::Emit{ texture: TextureIndex::from_usize(0) },
                Material::Metal{ texture: TextureIndex::from_usize(0), fuzz: 0.0 },
            ]
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

impl Default for Material
{
    fn default() -> Self
    {
        Material::Diffuse{ texture: TextureIndex::from_usize(0) }
    }
}

impl IndexedValue for Material
{
    fn collect_indexes(&self, _indexes: &mut HashSet<AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        self.ui_tag().into()
    }
}

impl UiDisplay for Material
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        match self
        {
            Material::Dielectric{ ior } =>
            {
                ui.imgui.label_text(label, "Dielectric");
                ui.display_float("IOR", ior);
            },
            Material::Diffuse{ texture } =>
            {
                ui.imgui.label_text(label, "Diffuse");
                ui.imgui.label_text("Texture", texture.to_usize().to_string());
            },
            Material::Emit{ texture } =>
            {
                ui.imgui.label_text(label, "Emit");
                ui.imgui.label_text("Texture", texture.to_usize().to_string());
            },
            Material::Metal{ texture, fuzz } =>
            {
                ui.imgui.label_text(label, "Metal");
                ui.imgui.label_text("Texture", texture.to_usize().to_string());
                ui.display_float("Fuzz", fuzz);
            },
        }
    }
}

impl UiEdit for Material
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = self.ui_render_combo(ui, label);
        ui.imgui.indent();

        match self
        {
            Material::Dielectric{ ior } =>
            {
                result |= ui.edit_float("IOR", ior);
            },
            Material::Diffuse{ texture } =>
            {
                result |= texture.ui_edit(ui, "Texture");
            },
            Material::Emit{ texture } =>
            {
                result |= texture.ui_edit(ui, "Texture");
            },
            Material::Metal{ texture, fuzz } =>
            {
                result |= texture.ui_edit(ui, "Texture");
                result |= ui.edit_float("Fuzz", fuzz);
            },
        }

        ui.imgui.unindent();
        result
    }
}