use std::collections::HashSet;
use crate::{indexed::{IndexedValue, GeomIndex, MaterialIndex, ObjectIndex, IndexedCollection}, ui::{UiDisplay, UiRenderer}, ui::UiEdit};

#[derive(Clone, Debug, Default)]
pub struct Object
{
    pub geom: GeomIndex,
    pub material: MaterialIndex,
}

impl Object
{
    pub fn build(&self, collection: &IndexedCollection) -> crate::object::Object
    {
        crate::object::Object::new_boxed(
            collection.map_item(self.geom, |geom, _| geom.build_surface()),
            collection.map_item(self.material, |material, collection| material.build(collection)))
    }
}

impl IndexedValue for Object
{
    type Index = ObjectIndex;
    
    fn collect_indexes(&self, _indexes: &mut HashSet<crate::indexed::AnyIndex>)
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
        let _id = ui.imgui.push_id(label);

        self.geom.ui_display(ui, "Geom");
        self.material.ui_display(ui, "Material");
    }
}

impl UiEdit for Object
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let _id = ui.imgui.push_id(label);

        let mut result = false;
        result |= self.geom.ui_edit(ui, "Geom");
        result |= self.material.ui_edit(ui, "Material");
        result
    }
}