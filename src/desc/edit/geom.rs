use std::collections::HashSet;

use crate::geom::Surface;
use crate::desc::edit::Color;
use crate::indexed::{IndexedValue, GeomIndex, AnyIndex, IndexedCollection};
use crate::ui::{UiDisplay, UiEdit, UiRenderer};
use crate::vec::{Dir3, Point3, Vec3};
use crate::math::Scalar;
use crate::desc::edit::Transform;

#[derive(Clone, Debug)]
pub struct TriangleVertex
{
    pub location: Point3,
    pub texture_coords: Point3,
    pub opt_color: Option<Color>,
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
        let mut opt_colors = None;

        if let (Some(c1), Some(c2), Some(c3)) = (self.vertices[0].opt_color, self.vertices[1].opt_color, self.vertices[2].opt_color)
        {
            opt_colors = Some([c1.into_linear(), c2.into_linear(), c3.into_linear()]);
        }

        crate::geom::Triangle::new(
            self.vertices[0].location,
            self.vertices[1].location,
            self.vertices[2].location,
            self.vertices[0].texture_coords,
            self.vertices[1].texture_coords,
            self.vertices[2].texture_coords,
            opt_colors)
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
                    opt_color: None,
                },
                TriangleVertex
                {
                    location: Point3::new(0.0, 1.0, 0.0),
                    texture_coords: Point3::new(0.0, 1.0, 0.0),
                    opt_color: None,
                },
                TriangleVertex
                {
                    location: Point3::new(0.0, 0.0, 1.0),
                    texture_coords: Point3::new(0.0, 0.0, 0.0),
                    opt_color: None,
                },
            ]
        }
    }
}

#[derive(Clone, Debug)]
pub struct Aabb
{
    pub min: Point3,
    pub max: Point3,
}

impl Default for Aabb
{
    fn default() -> Self
    {
        Aabb { min: Point3::new(0.0, 0.0, 0.0), max: Point3::new(1.0, 1.0, 1.0) }
    }
}

#[derive(Clone, Debug)]
pub enum Geom
{
    Sphere{center: Point3, radius: Scalar},
    Plane{point: Point3, normal: Dir3},
    Box{aabb: Aabb},
    Triangle{triangle: Triangle},
    Mesh{triangles: Vec<Triangle>, transform: Transform},
}

impl Geom
{
    pub fn build_surface(&self, collection: &IndexedCollection) -> Box<dyn Surface>
    {
        match self
        {
            Geom::Sphere{center, radius} => Box::new(crate::geom::Sphere::new(*center, *radius)),
            Geom::Plane{point, normal} => Box::new(crate::geom::Plane::new(*point, *normal)),
            Geom::Box{aabb} => Box::new(crate::geom::Aabb::new(aabb.min, aabb.max)),
            Geom::Triangle{triangle} => Box::new(triangle.build()),
            Geom::Mesh{triangles, transform} =>
            {
                let matrix = transform.build_matrix(collection);
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
            Geom::Box{..} => "Box",
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
                Geom::Box{aabb: Aabb::default() },
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
    type Index = GeomIndex;
    
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
            Geom::Box{aabb} =>
            {
                ui.imgui.label_text(label, "Box");
                ui.display_vec3("Min", &aabb.min);
                ui.display_vec3("Max", &aabb.max);
            }
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
            Geom::Box{aabb} =>
            {
                result |= ui.edit_vec3("Min", &mut aabb.min);
                result |= ui.edit_vec3("Max", &mut aabb.max);
            }
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