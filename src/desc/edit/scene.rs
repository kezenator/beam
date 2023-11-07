use crate::indexed::{IndexedCollection, GeomIndex, ObjectIndex, TextureIndex, MaterialIndex};
use crate::desc::edit::{Camera, Object};
use crate::render::RenderOptions;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Clone)]
pub struct Scene
{
    pub camera: Camera,
    pub collection: IndexedCollection,
}

impl Scene
{
    pub fn new() -> Self
    {
        let camera = Camera::default();
        let mut collection = IndexedCollection::new();
        collection.add_index::<TextureIndex>("Textures");
        collection.add_index::<MaterialIndex>("Materials");
        collection.add_index::<GeomIndex>("Geometry");
        collection.add_index::<ObjectIndex>("Objects");

        Scene
        {
            camera,
            collection,
        }
    }

    pub fn build(&self, options: &RenderOptions, camera_override: Option<&Camera>) -> crate::scene::Scene
    {
        let objects = self.collection
            .map_all(|obj: &Object, collection| obj.build(collection));

        crate::scene::Scene::new(
            options.sampling_mode,
            camera_override.unwrap_or(&self.camera).build(options),
            Vec::new(),
            objects)
    }
}

impl UiDisplay for Scene
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        if let Some(_scene) = ui.imgui.tree_node_config(label)
            .frame_padding(true)
            .framed(true)
            .push()
        {
            self.camera.ui_display(ui, "Camera");
            self.collection.ui_display(ui, "Collections");
        }
    }
}

impl UiEdit for Scene
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;

        if let Some(_scene) = ui.imgui.tree_node_config(label)
            .frame_padding(true)
            .framed(true)
            .push()
        {
            result |= self.camera.ui_edit(ui, "Camera");
            result |= self.collection.ui_edit(ui, "Collections");
        }

        result
    }
}