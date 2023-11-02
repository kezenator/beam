use std::collections::HashSet;

use crate::geom::Surface;
use crate::indexed::{IndexedValue, AnyIndex};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::{Dir3, Point3, Vec3};
use crate::math::Scalar;
use crate::desc::edit::Transform;

#[derive(Clone, Debug)]
pub struct TriangleVertex
{
    pub location: Point3,
    pub texture_coords: Point3,
}

#[derive(Clone, Debug)]
pub struct Triangle
{
    pub vertices: [TriangleVertex; 3],
}

impl Triangle
{
    fn build(&self) -> crate::geom::Triangle
    {
        crate::geom::Triangle::new(
            self.vertices[0].location,
            self.vertices[1].location,
            self.vertices[2].location,
            self.vertices[0].texture_coords,
            self.vertices[1].texture_coords,
            self.vertices[2].texture_coords)
    }
}

impl Default for Triangle
{
    fn default() -> Self
    {
        Triangle
        {
            vertices: [
                TriangleVertex
                {
                    location: Point3::new(1.0, 0.0, 0.0),
                    texture_coords: Point3::new(1.0, 0.0, 0.0),
                },
                TriangleVertex
                {
                    location: Point3::new(0.0, 1.0, 0.0),
                    texture_coords: Point3::new(0.0, 1.0, 0.0),
                },
                TriangleVertex
                {
                    location: Point3::new(0.0, 0.0, 1.0),
                    texture_coords: Point3::new(0.0, 0.0, 0.0),
                },
            ]
        }
    }
}

#[derive(Clone, Debug)]
pub enum Geom
{
    Sphere{center: Point3, radius: Scalar},
    Plane{point: Point3, normal: Dir3},
    Triangle{triangle: Triangle},
    Mesh{triangles: Vec<Triangle>, transform: Transform},
}

impl Geom
{
    pub fn build_surface(&self) -> Box<dyn Surface>
    {
        match self
        {
            Geom::Sphere{center, radius} => Box::new(crate::geom::Sphere::new(*center, *radius)),
            Geom::Plane{point, normal} => Box::new(crate::geom::Plane::new(*point, *normal)),
            Geom::Triangle{triangle} => Box::new(triangle.build()),
            Geom::Mesh{triangles, transform} =>
            {
                let matrix = transform.build_matrix();
                Box::new(crate::geom::Mesh::new(
                    triangles.iter()
                    .map(|t| t.build().transformed(&matrix)).collect()))
            },
        }
    }

    fn ui_tag(&self) -> &'static str
    {
        match self
        {
            Geom::Sphere{..} => "Sphere",
            Geom::Plane{..} => "Plane",
            Geom::Triangle{..} => "Triangle",
            Geom::Mesh{..} => "Mesh",
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
                Geom::Triangle{triangle: Triangle::default()},
                Geom::Mesh{triangles: vec![Triangle::default()], transform: Transform::new()},
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
            Geom::Triangle{triangle} =>
            {
                ui.imgui.label_text(label, "Triangle");
                ui.display_vec3("V1", &triangle.vertices[0].location);
                ui.display_vec3("V2", &triangle.vertices[1].location);
                ui.display_vec3("V3", &triangle.vertices[2].location);
                ui.display_vec3("T1", &triangle.vertices[0].texture_coords);
                ui.display_vec3("T2", &triangle.vertices[1].texture_coords);
                ui.display_vec3("T3", &triangle.vertices[2].texture_coords);
            },
            Geom::Mesh{ triangles, transform } =>
            {
                ui.imgui.label_text(label, "Mesh");
                ui.imgui.label_text("Triangles", triangles.len().to_string());
                transform.ui_display(ui, "Transform");
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
                result |= ui.edit_vec3("Point", point);
                result |= ui.edit_vec3("Normal", normal);
            },
            Geom::Triangle{triangle} =>
            {
                result |= ui.edit_vec3("V1", &mut triangle.vertices[0].location);
                result |= ui.edit_vec3("V2", &mut triangle.vertices[1].location);
                result |= ui.edit_vec3("V3", &mut triangle.vertices[2].location);
                result |= ui.edit_vec3("T1", &mut triangle.vertices[0].texture_coords);
                result |= ui.edit_vec3("T2", &mut triangle.vertices[1].texture_coords);
                result |= ui.edit_vec3("T3", &mut triangle.vertices[2].texture_coords);
            },
            Geom::Mesh{ triangles, transform } =>
            {
                ui.imgui.label_text("Triangles", triangles.len().to_string());
                result |= transform.ui_edit(ui, "Transform");
            },
        }

        ui.imgui.unindent();
        result
    }
}