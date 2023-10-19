use std::collections::HashSet;

use crate::indexed::{Index, IndexedValue, AnyIndex, TextureIndex};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use super::Scene;

#[derive(Clone, Debug)]
pub enum Material
{
    Phong{ color: TextureIndex },
    Emit{ color: TextureIndex },
}

impl Material
{
    pub fn build(&self, scene: &Scene) -> crate::material::Material
    {
        match self
        {
            Material::Phong{color} => crate::material::Material::Diffuse(scene.build_texture(*color)),
            Material::Emit{color} => crate::material::Material::Emit(scene.build_texture(*color)),
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Material::Phong{..} => "Phong",
            Material::Emit{..} => "Emit",
        }
    }

    fn ui_render_combo(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        let cur_tag = self.ui_tag();
        if let Some(_) = ui.imgui.begin_combo(label, cur_tag)
        {
            for entry in [
                Material::Phong{ color: TextureIndex::from_usize(0) },
                Material::Emit{ color: TextureIndex::from_usize(0) },
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
        Material::Phong{ color: TextureIndex::from_usize(0) }
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
            Material::Phong{ color } =>
            {
                ui.imgui.label_text(label, "Phong");
                ui.imgui.label_text("Color", color.to_usize().to_string());
            },
            Material::Emit{ color } =>
            {
                ui.imgui.label_text(label, "Emit");
                ui.imgui.label_text("Color", color.to_usize().to_string());
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
            Material::Phong{ color } =>
            {
                result |= color.ui_edit(ui, "Color");
            },
            Material::Emit{ color } =>
            {
                result |= color.ui_edit(ui, "Color");
            },
        }

        ui.imgui.unindent();
        result
    }
}