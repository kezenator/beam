use std::collections::HashSet;

use crate::geom::Surface;
use crate::indexed::{IndexedValue, AnyIndex};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::{Dir3, Point3, Vec3};
use crate::math::Scalar;

#[derive(Clone, Debug)]
pub struct TriangleVertex
{
    pub location: Point3,
}

#[derive(Clone, Debug)]
pub enum Geom
{
    Sphere{center: Point3, radius: Scalar},
    Plane{point: Point3, normal: Dir3},
    Triangle([TriangleVertex;3]),
}

impl Geom
{
    pub fn build_surface(&self) -> Box<dyn Surface>
    {
        match self
        {
            Geom::Sphere{center, radius} => Box::new(crate::geom::Sphere::new(*center, *radius)),
            Geom::Plane{point, normal} => Box::new(crate::geom::Plane::new(*point, *normal)),
            Geom::Triangle(vertices) => Box::new(crate::geom::Triangle::new(vertices[0].location, vertices[1].location, vertices[2].location)),
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Geom::Sphere{..} => "Sphere",
            Geom::Plane{..} => "Plane",
            Geom::Triangle(_) => "Triangle",
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
                Geom::Plane{point: Point3::new(0.0, 0.0, 0.0), normal: Dir3::new(0.0, 1.0, 0.0)},
                Geom::Triangle([TriangleVertex{location: Point3::new(0.0, 0.0, 0.0)}, TriangleVertex{location: Point3::new(1.0, 0.0, 0.0)}, TriangleVertex{location: Point3::new(0.0, 1.0, 0.0)}]),
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
            Geom::Plane{point, normal} =>
            {
                ui.imgui.label_text(label, "Plane");
                ui.display_vec3("Point", point);
                ui.display_vec3("Normal", normal);
            },
            Geom::Triangle(vertices) =>
            {
                ui.imgui.label_text(label, "Triangle");
                ui.display_vec3("V1", &vertices[0].location);
                ui.display_vec3("V2", &vertices[1].location);
                ui.display_vec3("V3", &vertices[2].location);
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
            Geom::Plane{point, normal} =>
            {
                ui.edit_vec3("Point", point);
                ui.edit_vec3("Normal", normal);
            },
            Geom::Triangle(vertices) =>
            {
                ui.imgui.label_text(label, "Triangle");
                ui.edit_vec3("V1", &mut vertices[0].location);
                ui.edit_vec3("V2", &mut vertices[1].location);
                ui.edit_vec3("V3", &mut vertices[2].location);
            },
        }

        ui.imgui.unindent();
        result
    }
}