use std::collections::HashMap;
use wavefront_obj::{obj, mtl};

use crate::color::SRGB;
use crate::desc::edit::{Geom, Material, Object, Scene, Texture, Triangle, TriangleVertex};
use crate::indexed::MaterialIndex;
use crate::vec::Point3;

pub fn import_obj_file(path: &str, scene: &mut Scene)
{
    let obj_set = obj::parse(std::fs::read_to_string(path).expect("Can't read .obj file")).expect("Can't parse .obj file");

    let mut materials = MaterialLoader::new(&obj_set.material_library);

    for obj in obj_set.objects
    {
        for geom in obj.geometry.iter()
        {
            let material = materials.load(&geom.material_name, scene);

            let mut triangles = Vec::new();

            push_geom_triangles(&obj, &geom.shapes, &mut triangles);

            let geom = scene.geom.push(Geom::Mesh { triangles });

            scene.objects.push(Object { geom, material });
        }
    }
}

pub fn import_obj_file_as_triangle_mesh(path: &str) -> Geom
{
    let obj_set = obj::parse(std::fs::read_to_string(path).expect("Can't read .obj file")).expect("Can't parse .obj file");

    let mut triangles = Vec::new();

    for obj in obj_set.objects.iter()
    {
        for geom in obj.geometry.iter()
        {
            push_geom_triangles(&obj, &geom.shapes, &mut triangles);
        }
    }

    Geom::Mesh{ triangles }
}

fn push_geom_triangles(obj: &obj::Object, shapes: &Vec<obj::Shape>, triangles: &mut Vec<Triangle>)
{
    for shape in shapes.iter()
    {
        if let obj::Primitive::Triangle(v0, v1, v2) = shape.primitive
        {
            triangles.push(Triangle{ vertices: [
                convert_vertex(&obj.vertices[v0.0]),
                convert_vertex(&obj.vertices[v1.0]),
                convert_vertex(&obj.vertices[v2.0]),
            ]});
        }
    }
}

fn convert_vertex(src: &obj::Vertex) -> TriangleVertex
{
    TriangleVertex { location: Point3::new(src.x, src.y, src.z) }
}

struct MaterialLoader
{
    loaded: HashMap<String, mtl::Material>,
    imported: HashMap<Option<String>, MaterialIndex>,
}

impl MaterialLoader
{
    fn new(path: &Option<String>) -> Self
    {
        let mut loaded = HashMap::new();

        if let Some(path) = path
        {
            let mtl_set = mtl::parse(std::fs::read_to_string(path).expect("Can't read .mtl file")).expect("Can't parse .mtl file");

            for mtl in mtl_set.materials
            {
                println!("Loaded {} material", mtl.name);
                loaded.insert(mtl.name.clone(), mtl);
            }
            println!("Loaded {} materials", loaded.len());
        }

        MaterialLoader
        {
            loaded,
            imported: HashMap::new(),
        }
    }

    fn load(&mut self, name: &Option<String>, scene: &mut Scene) -> MaterialIndex
    {
        // See if already loaded

        if let Some(result) = self.imported.get(name)
        {
            return *result;
        }

        // Try and load

        if let Some(name) = name
        {
            if let Some(mtl) = self.loaded.get(name)
            {
                let texture = scene.textures.push(Texture::Solid(SRGB::new(
                    mtl.color_diffuse.r,
                    mtl.color_diffuse.g,
                    mtl.color_diffuse.b).into()));
                let result = scene.materials.push(Material::Diffuse{ texture });
                self.imported.insert(Some(name.clone()), result);
                return result
            }

            // Return the cached none material
            return self.load(&None, scene);
        }

        // Return the 'none' material
        let texture = scene.textures.push(Texture::Solid(SRGB::new(1.0, 1.0, 1.0).into()));
        let result = scene.materials.push(Material::Diffuse{ texture });
        self.imported.insert(None, result);
        result
    }
}