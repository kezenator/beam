use std::collections::HashMap;

use crate::color::SRGB;
use crate::desc::edit::{Geom, Material, Object, Scene, Texture, Triangle, TriangleVertex};
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::MaterialIndex;
use crate::vec::Point3;


pub mod obj_file;
pub mod mtl_file;
mod parser;

pub fn import_obj_file(path: &str, scene: &mut Scene) -> Result<(), ImportError>
{
    let context = FileSystemContext::new();
    let (contents, sub_context) = context.load_file(path)?;
    let obj_file = obj_file::parse(&contents, path)?;

    let mut materials = MaterialLoader::new(&obj_file.material_library, &sub_context)?;

    for obj in obj_file.objects.iter()
    {
        for geom in obj.geometry.iter()
        {
            let material = materials.load(&geom.material_name, scene);

            let mut triangles = Vec::new();

            push_geom_triangles(&obj_file, &geom, &mut triangles);

            let geom = scene.geom.push(Geom::Mesh { triangles });

            scene.objects.push(Object { geom, material });
        }
    }

    Ok(())
}

pub fn import_obj_file_as_triangle_mesh(path: &str) -> Result<Geom, ImportError>
{
    let context = FileSystemContext::new();
    let (contents, _sub_context) = context.load_file(path)?;
    let obj_file = obj_file::parse(&contents, path)?;

    let mut triangles = Vec::new();

    for obj in obj_file.objects.iter()
    {
        for geom in obj.geometry.iter()
        {
            push_geom_triangles(&obj_file, geom, &mut triangles);
        }
    }

    Ok(Geom::Mesh{ triangles })
}

fn push_geom_triangles(obj_file: &obj_file::ObjFile, geom: &obj_file::Geometry, triangles: &mut Vec<Triangle>)
{
    for triangle in geom.triangles.iter()
    {
        triangles.push(Triangle{ vertices: [
            convert_vector(&obj_file.vertices[triangle[0].vertex_index]),
            convert_vector(&obj_file.vertices[triangle[1].vertex_index]),
            convert_vector(&obj_file.vertices[triangle[2].vertex_index]),
        ]});
    }
}

fn convert_vector(src: &obj_file::Vector) -> TriangleVertex
{
    TriangleVertex { location: Point3::new(src.0, src.1, src.2) }
}

struct MaterialLoader
{
    loaded: HashMap<String, mtl_file::Material>,
    imported: HashMap<Option<String>, MaterialIndex>,
}

impl MaterialLoader
{
    fn new(path: &Option<String>, fs_context: &FileSystemContext) -> Result<Self, ImportError>
    {
        let mut loaded = HashMap::new();

        if let Some(path) = path
        {
            let (contents, _sub_context) = fs_context.load_file(path)?;
            let mtl_file = mtl_file::parse(&contents, &path)?;

            for mtl in mtl_file.materials
            {
                loaded.insert(mtl.name.clone(), mtl);
            }
        }

        Ok(MaterialLoader
        {
            loaded,
            imported: HashMap::new(),
        })
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
                let texture = scene.textures.push(Texture::Solid(mtl.diffuse.into()));
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