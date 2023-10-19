use std::collections::HashSet;

use crate::geom::Surface;
use crate::indexed::{IndexedValue, AnyIndex};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::{Point3, Vec3};
use crate::math::Scalar;

#[derive(Clone, Debug)]
pub enum Geom
{
    Sphere{center: Point3, radius: Scalar},
}

impl Geom
{
    pub fn build_surface(&self) -> Box<dyn Surface>
    {
        match self
        {
            Geom::Sphere{center, radius} => Box::new(crate::geom::Sphere::new(*center, *radius)),
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Geom::Sphere{..} => "Sphere",
        }
    }

    fn ui_render_combo(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        let cur_tag = self.ui_tag();
        if let Some(_) = ui.imgui.begin_combo(label, cur_tag)
        {
            for entry in [
                Geom::Sphere{center: Point3::new(0.0, 0.0, 0.0), radius: 0.0},
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

impl Default for Geom
{
    fn default() -> Self
    {
        Geom::Sphere{ center: Vec3::new(0.0, 0.0, 0.0), radius: 1.0, }
    }
}

impl IndexedValue for Geom
{
    fn collect_indexes(&self, _indexes: &mut HashSet<AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        self.ui_tag().into()
    }
}

impl UiDisplay for Geom
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        match self
        {
            Geom::Sphere{center, radius} =>
            {
                ui.imgui.label_text(label, "Sphere");
                ui.display_vec3("Center", center);
                ui.display_float("Radius", radius);
            },
        }
    }
}

impl UiEdit for Geom
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = self.ui_render_combo(ui, label);
        ui.imgui.indent();

        match self
        {
            Geom::Sphere{ center, radius} =>
            {
                result |= ui.edit_vec3("Center", center);
                result |= ui.edit_float("Radius", radius);
            },
        }

        ui.imgui.unindent();
        result
    }
}