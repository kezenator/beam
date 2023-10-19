use crate::geom::Surface;
use crate::indexed::{IndexedVec, GeomIndex, ObjectIndex, TextureIndex, MaterialIndex};
use crate::desc::edit::{Camera, Geom, Material, Object, Texture};
use crate::render::RenderOptions;
use crate::scene::SamplingMode;
use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Clone)]
pub struct Scene
{
    camera: Camera,
    textures: IndexedVec<TextureIndex, Texture>,
    materials: IndexedVec<MaterialIndex, Material>,
    geom: IndexedVec<GeomIndex, Geom>,
    objects: IndexedVec<ObjectIndex, Object>,
}

impl Scene
{
    pub fn new() -> Self
    {
        let camera = Camera::default();
        let textures = IndexedVec::new();
        let materials = IndexedVec::new();
        let geom = IndexedVec::new();
        let objects = IndexedVec::new();

        Scene
        {
            camera,
            textures,
            materials,
            geom,
            objects,
        }
    }

    pub fn build(&self, options: &RenderOptions) -> crate::scene::Scene
    {
        let mut objects = Vec::new();

        for obj in self.objects.iter()
        {
            objects.push(obj.build(self));
        }

        crate::scene::Scene::new(
            options.sampling_mode,
            self.camera.build(options),
            Vec::new(),
            objects)
    }

    pub fn build_texture(&self, index: TextureIndex) -> crate::texture::Texture
    {
        self.textures.get(index).build()
    }

    pub fn build_material(&self, index: MaterialIndex) -> crate::material::Material
    {
        self.materials.get(index).build(self)
    }

    pub fn build_surface(&self, index: GeomIndex) -> Box<dyn Surface>
    {
        self.geom.get(index).build_surface()
    }

    pub fn build_obj(&self, index: ObjectIndex) -> crate::object::Object
    {
        self.objects.get(index).build(self)
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
            self.textures.ui_display(ui, "Textures");
            self.materials.ui_display(ui, "Materials");
            self.geom.ui_display(ui, "Geometry");
            self.objects.ui_display(ui, "Objects");
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
            result |= self.textures.ui_edit(ui, "Textures");
            result |= self.materials.ui_edit(ui, "Materials");
            result |= self.geom.ui_edit(ui, "Geometry");
            result |= self.objects.ui_edit(ui, "Objects");
        }

        result
    }
}