use crate::indexed::{IndexedValue, TransformIndex, IndexedCollection};
use crate::math::Scalar;
use crate::desc::edit::geom::Aabb;
use crate::ui::{UiDisplay, UiEdit, UiTaggedEnum};
use crate::vec::{Vec3, Mat4, Point3};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TransformStageTag
{
    Scale,
    Translate,
    ShiftAndScale,
    Matrix,
}

#[derive(Debug, Clone)]
pub enum TransformStage
{
    Scale(Scalar),
    Translate(Vec3),
    ShiftAndScale{from: Aabb, to: Aabb, maintain_aspect: bool},
    Matrix(Mat4),
}

#[derive(Debug, Clone)]
pub struct Transform
{
    pub base: Option<TransformIndex>,
    pub stages: Vec<TransformStage>,
}

impl Default for Transform
{
    fn default() -> Self
    {
        Transform::new()
    }
}

impl IndexedValue for Transform
{
    type Index = TransformIndex;

    fn collect_indexes(&self, _indexes: &mut std::collections::HashSet<crate::indexed::AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        format!("{} stages", self.stages.len())
    }
}

impl Transform
{
    pub fn new() -> Self
    {
        Transform { base: None, stages: Vec::new() }
    }

    pub fn build_matrix(&self, collection: &IndexedCollection) -> Mat4
    {
        let mut result = Mat4::identity();

        if let Some(base_index) = self.base
        {
            result = collection.map_item(base_index, |t, collection| t.build_matrix(collection)) * result;
        }

        for stage in self.stages.iter()
        {
            match stage
            {
                TransformStage::Scale(scale) =>
                {
                    result.scale_3d(Vec3::new(*scale, *scale, *scale));
                },
                TransformStage::Translate(offset) =>
                {
                    result.translate_3d(*offset);
                },
                TransformStage::ShiftAndScale { from, to, maintain_aspect } =>
                {
                    let from_min = Point3::new(from.min.x.min(from.max.x), from.min.y.min(from.max.y), from.min.z.min(from.max.z));
                    let from_max = Point3::new(from.min.x.max(from.max.x), from.min.y.max(from.max.y), from.min.z.max(from.max.z));
                    let from_dim = Point3::new(from_max.x - from_min.x, from_max.y - from_min.y, from_max.z - from_min.z);

                    let to_min = Point3::new(to.min.x.min(to.max.x), to.min.y.min(to.max.y), to.min.z.min(to.max.z));
                    let to_max = Point3::new(to.min.x.max(to.max.x), to.min.y.max(to.max.y), to.min.z.max(to.max.z));
                    let to_dim = Point3::new(to_max.x - to_min.x, to_max.y - to_min.y, to_max.z - to_min.z);

                    let mut scale = Point3::new(to_dim.x / from_dim.x, to_dim.y / from_dim.y, to_dim.z / from_dim.z);

                    if *maintain_aspect
                    {
                        let scale_min = scale.x.min(scale.y).min(scale.z);

                        scale = Point3::new(scale_min, scale_min, scale_min);
                    }

                    result.translate_3d(-from_min - (0.5 * from_dim));
                    result.scale_3d(scale);
                    result.translate_3d(to_min + (0.5 * to_dim));
                }
                TransformStage::Matrix(m) =>
                {
                    result = result * *m;
                },
            }
        }

        result
    }
}

impl UiTaggedEnum for TransformStage
{
    type TagEnum = TransformStageTag;

    fn all_tags() -> &'static [Self::TagEnum]
    {
        &[TransformStageTag::Scale, TransformStageTag::Translate]
    }

    fn display_for_tag(tag: Self::TagEnum) -> &'static str
    {
        match tag
        {
            TransformStageTag::Scale => "Scale",
            TransformStageTag::Translate => "Translate",
            TransformStageTag::ShiftAndScale => "Shift and Scale",
            TransformStageTag::Matrix => "Matrix",
        }
    }

    fn default_val_for_tag(tag: Self::TagEnum) -> Self
    {
        match tag
        {
            TransformStageTag::Scale => TransformStage::Scale(1.0),
            TransformStageTag::Translate => TransformStage::Translate(Vec3::new(0.0, 0.0, 0.0)),
            TransformStageTag::ShiftAndScale => TransformStage::ShiftAndScale { from: Aabb::default(), to: Aabb::default(), maintain_aspect: true },
            TransformStageTag::Matrix => TransformStage::Matrix(Mat4::identity()),
        }
    }

    fn get_tag(&self) -> Self::TagEnum
    {
        match self
        {
            TransformStage::Scale(_) => TransformStageTag::Scale,
            TransformStage::Translate(_) => TransformStageTag::Translate,
            TransformStage::ShiftAndScale{..} => TransformStageTag::ShiftAndScale,
            TransformStage::Matrix(_) => TransformStageTag::Matrix,
        }
    }
}

impl UiDisplay for TransformStage
{
    fn ui_display(&self, ui: &crate::ui::UiRenderer, label: &str)
    {
        let _label = ui.imgui.push_id(label);
        ui.display_tag("Stage", self);
        match self
        {
            TransformStage::Scale(scale) => ui.display_float("Scale", scale),
            TransformStage::Translate(offset) => ui.display_vec3("Translate", offset),
            TransformStage::ShiftAndScale{ from, to, maintain_aspect } =>
            {
                ui.display_vec3("Source Aabb (min)", &from.min);
                ui.display_vec3("Source Aabb (max)", &from.max);
                ui.display_vec3("Dest Aabb (min)", &to.min);
                ui.display_vec3("Dest Aabb (max)", &to.max);

                let mut maintain_aspect = *maintain_aspect;
                ui.imgui.checkbox("Maintain Aspect Ratio", &mut maintain_aspect);
            },
            TransformStage::Matrix(m) =>
            {
                let mut rows = m.into_row_arrays().map(|r| r.map(|c| c as f32));
                ui.imgui.input_float4("R1", &mut rows[0]).build();
                ui.imgui.input_float4("R2", &mut rows[1]).build();
                ui.imgui.input_float4("R3", &mut rows[2]).build();
                ui.imgui.input_float4("R4", &mut rows[3]).build();
            },
        }
    }
}

impl UiEdit for TransformStage
{
    fn ui_edit(&mut self, ui: &crate::ui::UiRenderer, label: &str) -> bool
    {
        let _label = ui.imgui.push_id(label);
        let mut result = ui.edit_tag("Stage", self);

        match self
        {
            TransformStage::Scale(scale) =>
            {
                result |= ui.edit_float("Scale", scale);
            },
            TransformStage::Translate(offset) =>
            {
                result |= ui.edit_vec3("Offset", offset);
            },
            TransformStage::ShiftAndScale{ from, to, maintain_aspect } =>
            {
                result |= ui.edit_vec3("Source Aabb (min)", &mut from.min);
                result |= ui.edit_vec3("Source Aabb (max)", &mut from.max);
                result |= ui.edit_vec3("Dest Aabb (min)", &mut to.min);
                result |= ui.edit_vec3("Dest Aabb (max)", &mut to.max);
                result |= ui.imgui.checkbox("Maintain Aspect Ratio", maintain_aspect);
            },
            TransformStage::Matrix(m) =>
            {
                let mut rows = m.into_row_arrays().map(|r| r.map(|c| c as f32));
                result |= ui.imgui.input_float4("R1", &mut rows[0]).build();
                result |= ui.imgui.input_float4("R2", &mut rows[1]).build();
                result |= ui.imgui.input_float4("R3", &mut rows[2]).build();
                result |= ui.imgui.input_float4("R4", &mut rows[3]).build();
                
                if result
                {
                    *m = Mat4::from_row_arrays(rows.map(|r| r.map(|c| c as f64)));
                }
            }
        }

        result
    }
}

impl UiDisplay for Transform
{
    fn ui_display(&self, ui: &crate::ui::UiRenderer, label: &str)
    {
        let _label = ui.imgui.push_id(label);
        self.base.ui_display(ui, "Base Transform");
        for (i, stage) in self.stages.iter().enumerate()
        {
            stage.ui_display(ui, &i.to_string());
        }
    }
}

impl UiEdit for Transform
{
    fn ui_edit(&mut self, ui: &crate::ui::UiRenderer, label: &str) -> bool
    {
        let _label = ui.imgui.push_id(label);
        let mut result = false;

        result |= self.base.ui_edit(ui, "Base Transform");

        for (i, stage) in self.stages.iter_mut().enumerate()
        {
            result |= stage.ui_edit(ui, &i.to_string());
        }

        result
    }
}
