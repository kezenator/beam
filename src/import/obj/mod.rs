use std::collections::HashMap;

use crate::color::SRGB;
use crate::desc::edit::transform::TransformStage;
use crate::desc::edit::{Geom, Material, Object, Scene, Texture, Transform, Triangle, TriangleVertex};
use crate::geom::Aabb;
use crate::import::{FileSystemContext, ImportError};
use crate::import::image::Image;
use crate::indexed::MaterialIndex;
use crate::vec::Point3;

pub mod obj_file;
pub mod mtl_file;
mod parser;

pub fn import_obj_file(path: &str, destination: &Aabb, scene: &mut Scene) -> Result<(), ImportError>
{
    let context = FileSystemContext::new();
    let (contents, sub_context) = context.load_text_file(path)?;
    let obj_file = obj_file::parse(&contents, path)?;

    let transform = calc_transform(&obj_file.vertices, destination);

    let mut resources = ResourceLoader::new(&obj_file.material_library, sub_context)?;

    for obj in obj_file.objects.iter()
    {
        for geom in obj.geometry.iter()
        {
            let material = resources.load_material(&geom.material_name, scene)?;

            let mut triangles = Vec::new();

            push_geom_triangles(&obj_file, &geom, &mut triangles);

            let geom = scene.collection.push(Geom::Mesh { triangles, transform: transform.clone() });

            scene.collection.push(Object { geom, material });
        }
    }

    Ok(())
}

pub fn import_obj_file_as_triangle_mesh(path: &str) -> Result<Geom, ImportError>
{
    let context = FileSystemContext::new();
    let (contents, _sub_context) = context.load_text_file(path)?;
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
            convert_vertex(&obj_file, &triangle[0]),
            convert_vertex(&obj_file, &triangle[1]),
            convert_vertex(&obj_file, &triangle[2]),
        ]});
    }
}

fn convert_vertex(file: &obj_file::ObjFile, triangle: &obj_file::Vertex) -> TriangleVertex
{
    let location = Point3::new(
        file.vertices[triangle.vertex_index].0,
        file.vertices[triangle.vertex_index].1,
        file.vertices[triangle.vertex_index].2);

    let mut texture_coords = location.clone();

    if let Some(ti) = triangle.texture_index
    {
        if ti < file.texture_coords.len()
        {
            // Not 100% sure about .OBJ file texture co-ordinates.
            // They seem backwards?
            // Just putting a 1.0 - y in here as it seems to have
            // some positive effects... TODO - sort out...

            texture_coords = Point3::new(
                file.texture_coords[ti].0,
                1.0 - file.texture_coords[ti].1,
                file.texture_coords[ti].2);
        }
    }

    TriangleVertex { location, texture_coords }
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

        result.stages.push(TransformStage::ShiftAndScale
            {
                from: crate::desc::edit::geom::Aabb{ min: Point3::new(min_src.0, min_src.1, min_src.2), max: Point3::new(max_src.0, max_src.1, max_src.2) },
                to: crate::desc::edit::geom::Aabb{ min: destination.min, max: destination.max },
                maintain_aspect: true,
            });
    }

    result
}

struct ResourceLoader
{
    fs_context: FileSystemContext,
    loaded_materials: HashMap<String, mtl_file::Material>,
    imported_materials: HashMap<Option<String>, MaterialIndex>,
    imported_images: HashMap<String, Image>,
}

impl ResourceLoader
{
    fn new(path: &Option<String>, fs_context: FileSystemContext) -> Result<Self, ImportError>
    {
        let mut loaded_materials = HashMap::new();

        if let Some(path) = path
        {
            let (contents, _sub_context) = fs_context.load_text_file(path)?;
            let mtl_file = mtl_file::parse(&contents, &path)?;

            for mtl in mtl_file.materials
            {
                loaded_materials.insert(mtl.name.clone(), mtl);
            }
        }

        Ok(ResourceLoader
        {
            fs_context,
            loaded_materials,
            imported_materials: HashMap::new(),
            imported_images: HashMap::new(),
        })
    }

    fn load_material(&mut self, name: &Option<String>, scene: &mut Scene) -> Result<MaterialIndex, ImportError>
    {
        // See if already loaded

        if let Some(result) = self.imported_materials.get(name)
        {
            return Ok(*result);
        }

        // Try and load

        if let Some(name) = name
        {
            if let Some(mtl) = self.loaded_materials.get(name).cloned()
            {
                if let (Some(disolve), Some(ior)) = (mtl.disolve, mtl.ior)
                {
                    if disolve < 1.0
                    {
                        // Use a dielectric
                        let result = scene.collection.push(Material::Dielectric { ior });
                        self.imported_materials.insert(Some(name.clone()), result);
                        return Ok(result);
                    }
                }

                // Create a diffuse material

                let texture = if let Some(path) = mtl.diffuse_map
                {
                    let image = self.load_image(&path)?;
                    scene.collection.push(Texture::Image(image))
                }
                else
                {
                    // Solid color
                    scene.collection.push(Texture::Solid(mtl.diffuse.into()))
                };

                let result = scene.collection.push(Material::Diffuse{ texture });
                self.imported_materials.insert(Some(name.clone()), result);
                return Ok(result);
            }

            // Return the cached none material
            return self.load_material(&None, scene);
        }

        // Return the 'none' material
        let texture = scene.collection.push(Texture::Solid(SRGB::new(1.0, 1.0, 1.0, 1.0).into()));
        let result = scene.collection.push(Material::Diffuse{ texture });
        self.imported_materials.insert(None, result);
        Ok(result)
    }

    fn load_image(&mut self, path: &str) -> Result<Image, ImportError>
    {
        if let Some(existing) = self.imported_images.get(path)
        {
            return Ok(existing.clone());
        }

        let image = crate::import::image::import_image(path, &mut self.fs_context)?;
        self.imported_images.insert(path.to_owned(), image.clone());
        Ok(image)
    }
}