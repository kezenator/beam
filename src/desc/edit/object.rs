use std::collections::HashSet;
use crate::{indexed::{IndexedValue, GeomIndex, MaterialIndex}, ui::{UiDisplay, UiRenderer}, ui::UiEdit};
use super::Scene;

#[derive(Clone, Debug, Default)]
pub struct Object
{
    geom: GeomIndex,
    material: MaterialIndex,
}

impl Object
{
    pub fn build(&self, scene: &Scene) -> crate::object::Object
    {
        crate::object::Object::new_boxed(
            scene.build_surface(self.geom),
            scene.build_material(self.material))
    }
}

impl IndexedValue for Object
{
    fn collect_indexes(&self, indexes: &mut HashSet<crate::indexed::AnyIndex>)
    {
    }

    fn summary(&self) -> String
    {
        "Object".into()
    }
}

impl UiDisplay for Object
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        self.geom.ui_display(ui, "Geom");
        self.material.ui_display(ui, "Material");
    }
}

impl UiEdit for Object
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        result |= self.geom.ui_edit(ui, "Geom");
        result |= self.material.ui_edit(ui, "Material");
        result
    }
}