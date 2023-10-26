use crate::math::Scalar;
use crate::ui::{UiDisplay, UiEdit, UiTaggedEnum};
use crate::vec::{Vec3, Mat4};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TransformStageTag
{
    Scale,
    Translate,
}

#[derive(Debug, Clone)]
pub enum TransformStage
{
    Scale(Scalar),
    Translate(Vec3),
}

#[derive(Debug, Clone)]
pub struct Transform
{
    pub stages: Vec<TransformStage>
}

impl Transform
{
    pub fn new() -> Self
    {
        Transform { stages: Vec::new() }
    }

    pub fn build_matrix(&self) -> Mat4
    {
        let mut result = Mat4::identity();

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
        }
    }

    fn default_val_for_tag(tag: Self::TagEnum) -> Self
    {
        match tag
        {
            TransformStageTag::Scale => TransformStage::Scale(1.0),
            TransformStageTag::Translate => TransformStage::Translate(Vec3::new(0.0, 0.0, 0.0)),
        }
    }

    fn get_tag(&self) -> Self::TagEnum
    {
        match self
        {
            TransformStage::Scale(_) => TransformStageTag::Scale,
            TransformStage::Translate(_) => TransformStageTag::Translate,
        }
    }
}

impl UiDisplay for TransformStage
{
    fn ui_display(&self, ui: &crate::ui::UiRenderer, label: &str)
    {
        let _label = ui.imgui.push_id(label);
        match self
        {
            TransformStage::Scale(scale) => ui.display_float("Scale", scale),
            TransformStage::Translate(offset) => ui.display_vec3("Translate", offset),
        }
    }
}

impl UiEdit for TransformStage
{
    fn ui_edit(&mut self, ui: &crate::ui::UiRenderer, label: &str) -> bool
    {
        let _label = ui.imgui.push_id(label);
        let mut result = ui.edit_tag("Type", self);

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
        }

        result
    }
}

impl UiDisplay for Transform
{
    fn ui_display(&self, ui: &crate::ui::UiRenderer, label: &str)
    {
        let _label = ui.imgui.push_id(label);
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

        for (i, stage) in self.stages.iter_mut().enumerate()
        {
            result |= stage.ui_edit(ui, &i.to_string());
        }

        result
    }
}
