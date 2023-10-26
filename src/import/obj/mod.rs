use std::collections::HashMap;

use crate::color::SRGB;
use crate::desc::edit::transform::TransformStage;
use crate::desc::edit::{Geom, Material, Object, Scene, Texture, Transform, Triangle, TriangleVertex};
use crate::geom::Aabb;
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::MaterialIndex;
use crate::vec::Point3;


pub mod obj_file;
pub mod mtl_file;
mod parser;

pub fn import_obj_file(path: &str, destination: &Aabb, scene: &mut Scene) -> Result<(), ImportError>
{
    let context = FileSystemContext::new();
    let (contents, sub_context) = context.load_file(path)?;
    let obj_file = obj_file::parse(&contents, path)?;

    let transform = calc_transform(&obj_file.vertices, destination);

    let mut materials = MaterialLoader::new(&obj_file.material_library, &sub_context)?;

    for obj in obj_file.objects.iter()
    {
        for geom in obj.geometry.iter()
        {
            let material = materials.load(&geom.material_name, scene);

            let mut triangles = Vec::new();

            push_geom_triangles(&obj_file, &geom, &mut triangles);

            let geom = scene.geom.push(Geom::Mesh { triangles, transform: transform.clone() });

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

    Ok(Geom::Mesh{ triangles, transform: Transform::new() })
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

fn calc_transform(vertices: &Vec<obj_file::Vector>, destination: &Aabb) -> Transform
{
    let mut result = Transform::new();

    if !vertices.is_empty()
    {
        let min_src = vertices.iter().fold(vertices[0], |a, b|
        {
            (a.0.min(b.0), a.1.min(b.1), a.2.min(b.2))
        });
        let max_src = vertices.iter().fold(vertices[0], |a, b|
        {
            (a.0.max(b.0), a.1.max(b.1), a.2.max(b.2))
        });

        let dim_src = (max_src.0 - min_src.0, max_src.1 - min_src.1, max_src.2 - min_src.2);
        let max_dim_src = dim_src.0.max(dim_src.1).max(dim_src.2);

        let dim_dest = destination.max - destination.min;

        let min_dim_dest = dim_dest.x.min(dim_dest.y).min(dim_dest.z);

        result.stages.push(TransformStage::Translate(Point3::new(-min_src.0, -min_src.1, -min_src.2)));
        result.stages.push(TransformStage::Scale(1.0 / max_dim_src));
        result.stages.push(TransformStage::Scale(min_dim_dest));
        result.stages.push(TransformStage::Translate(destination.min));
    }

    result
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