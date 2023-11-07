use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use crate::color::SRGB;
use crate::desc::edit::{Scene, Triangle, TriangleVertex, Geom, Transform, Object, Material, Texture};
use crate::geom::Aabb;
use crate::import;
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::{MaterialIndex, TextureIndex};
use crate::math::Scalar;
use crate::vec::Point3;

pub fn import_gltf_file(path: &str, _destination: &Aabb, scene: &mut Scene) -> Result<(), ImportError>
{
    let context = FileSystemContext::new();
    let filename = context.path_to_filename(path);
    let (contents, sub_context) = context.load_binary_file(path)?;
    let file_state = ScopedState::new(scene, sub_context, filename);

    let gltf = gltf::Gltf::from_slice(&contents)
        .map_err(|e| file_state.error(&format!("Decode Error: {:?}", e)))?;

    match gltf.default_scene()
    {
        None => Err(file_state.error("No default scene")),
        Some(gltf_scene) =>
        {
            let scene_state = file_state.sub_state("scene", gltf_scene.name(), gltf_scene.index());

            for node in gltf_scene.nodes()
            {
                import_node(&scene_state, &node)?;
            }

            Ok(())
        },
    }
}

fn import_node(parent_state: &ScopedState, node: &gltf::Node) -> Result<(), ImportError>
{
    let node_state = parent_state.sub_state("node", node.name(), node.index());

    if let Some(camera) = node.camera()
    {
        let _camera_state = node_state.sub_state("camera", camera.name(), camera.index());
    }

    if let Some(mesh) = node.mesh()
    {
        let mesh_state = node_state.sub_state("mesh", mesh.name(), mesh.index());

        for primitive in mesh.primitives()
        {
            let primitive_state = mesh_state.sub_state("primitive", None, primitive.index());

            let indexes = primitive_state.decode_accessor_required_vector_u32(primitive.indices())?;

            match primitive.mode()
            {
                gltf::mesh::Mode::Triangles =>
                {
                    let positions = primitive_state.decode_accessor_required_vector_vec3_f32(primitive.get(&gltf::mesh::Semantic::Positions))?;
                    let texture_coords = primitive_state.decode_accessor_optional_vector_vec2_f32(primitive.get(&gltf::mesh::Semantic::TexCoords(0)))?
                        .unwrap_or_else(|| positions.clone());

                    let max_index = *indexes.iter().max().ok_or_else(|| primitive_state.error("Primitive must have at least one index"))?;

                    if max_index > positions.len()
                    {
                        return Err(primitive_state.error(&format!("Primitive index {} is larger than provided position count {}", max_index, positions.len())));
                    }
                    else if (indexes.len() % 3) != 0
                    {
                        return Err(primitive_state.error(&format!("Primitive triangles expects multiple of 3 indexes, but got {}", indexes.len())));
                    }
                    else
                    {
                        let num_tri = indexes.len() / 3;

                        let mut triangles = Vec::new();

                        for i in 0..num_tri
                        {
                            let x = positions[indexes[3 * i + 0]];
                            let y = positions[indexes[3 * i + 1]];
                            let z = positions[indexes[3 * i + 2]];

                            let u = texture_coords[indexes[3 * i + 0]];
                            let v = texture_coords[indexes[3 * i + 1]];
                            let w = texture_coords[indexes[3 * i + 2]];

                            triangles.push(Triangle { vertices: [
                                TriangleVertex{ location: x, texture_coords: u },
                                TriangleVertex{ location: y, texture_coords: v },
                                TriangleVertex{ location: z, texture_coords: w },
                            ]});
                        }

                        let material = import_material(&primitive_state, primitive.material())?;

                        let mut state = primitive_state.state.borrow_mut();
                        let geom = state.scene.collection.push(Geom::Mesh{ triangles, transform: Transform::new() });
                        let _obj = state.scene.collection.push(Object{ geom, material });
                    }
                },
                _ =>
                {
                    return Err(primitive_state.error(&format!("Unsupported primitive mode {:?}", primitive.mode())));
                },
            }
        }
    }

    for child in node.children()
    {
        import_node(&node_state, &child)?;
    }

    Ok(())
}

fn import_material(parent_state: &ScopedState, material: gltf::Material) -> Result<MaterialIndex, ImportError>
{
    let index = material.index().unwrap_or(usize::MAX);
    let material_state = parent_state.sub_state("material", material.name(), index);

    let existing_index = material_state.state.borrow().materials.get(&index).cloned();
    match existing_index
    {
        None =>
        {
            if let Some(spec_glossy) = material.pbr_specular_glossiness()
            {
                let diffuse = spec_glossy.diffuse_factor();
                let diffuse = SRGB::new(diffuse[0] as Scalar, diffuse[1] as Scalar, diffuse[2] as Scalar, diffuse[3] as Scalar);

                let texture = match spec_glossy.diffuse_texture()
                {
                    None =>
                    {
                        let mut state = material_state.state.borrow_mut();
                        Ok(state.scene.collection.push(Texture::Solid(diffuse.into())))
                    },
                    Some(image_info) =>
                    {
                        if image_info.texture_transform().is_some()
                        {
                            return Err(material_state.error("Texture transforms are not supported"));
                        }

                        import_image(&material_state, image_info.texture().source())
                    },
                }?;
                let mut state = material_state.state.borrow_mut();
                let added_index = state.scene.collection.push(Material::Diffuse{ texture });
                state.materials.insert(index, added_index.clone());
                Ok(added_index)
            }
            else
            {
                Err(material_state.error("Unsupported material"))
            }
        },
        Some(existing) =>
        {
            Ok(existing)
        }
    }        
}

fn import_image(parent_state: &ScopedState, image: gltf::Image) -> Result<TextureIndex, ImportError>
{
    let texture_state = parent_state.sub_state("image", image.name(), image.index());

    match image.source()
    {
        gltf::image::Source::View { .. } => Err(texture_state.error("Loading images from a view not supported")),
        gltf::image::Source::Uri { uri, .. } =>
        {
            let mut state = texture_state.state.borrow_mut();
            let image = import::image::import_image(uri, &mut state.fs_context)?;

            Ok(state.scene.collection.push(Texture::Image(image)))
        },
    }
}

struct ImportState<'a>
{
    scene: &'a mut Scene,
    fs_context: FileSystemContext,
    blobs: HashMap<Option<String>, Vec<u8>>,
    materials: HashMap<usize, MaterialIndex>,
}

struct ScopedState<'a>
{
    state: Rc<RefCell<ImportState<'a>>>,
    path: String,    
}

impl<'a> ScopedState<'a>
{
    fn new(scene: &'a mut Scene, fs_context: FileSystemContext, filename: String) -> Self
    {
        let blobs = HashMap::new();
        let materials = HashMap::new();
        let state = Rc::new(RefCell::new(ImportState { scene, fs_context, blobs, materials }));
        ScopedState { state, path: filename }
    }

    fn sub_state(&self, kind: &str, name: Option<&str>, index: usize) -> Self
    {
        let sub_path = format!("{}/{}", self.path, name.map(|s| s.to_string()).unwrap_or_else(|| index.to_string()));
        println!("Entering: {}: {}", kind, sub_path);
        ScopedState { state: self.state.clone(), path: sub_path }
    }

    fn error(&self, msg: &str) -> ImportError
    {
        ImportError(format!("GLTF Error: {}: {}", self.path, msg))
    }

    fn with_view_data<F>(&self, view: &gltf::buffer::View, func: F) -> Result<(), ImportError>
        where F: FnOnce(&[u8]) -> Result<(), ImportError>
    {
        let view_state = self.sub_state("view", view.name(), view.index());

        let buffer = view.buffer();
        let buffer_state = view_state.sub_state("buffer", buffer.name(), buffer.index());

        let mut state = self.state.borrow_mut();

        let vector = match buffer.source()
        {
            gltf::buffer::Source::Bin =>
            {
                Err(buffer_state.error("TODO: GLTF-internal binary blobs not supported yet"))
            },
            gltf::buffer::Source::Uri(uri) =>
            {
                let blob_key = Some(uri.to_string());
                let existing = state.blobs.get(&blob_key);
                match existing
                {
                    None =>
                    {
                        let (contents, _) = state.fs_context.load_binary_file(uri)?;
                        state.blobs.insert(blob_key.clone(), contents);
                        Ok(state.blobs.get(&blob_key).unwrap())
                    },
                    Some(existing) =>
                    {
                        Ok(existing)
                    },
                }
            },
        }?;

        let vec_len = vector.len();
        let buffer_len = buffer.length();

        if vec_len != buffer_len
        {
            return Err(buffer_state.error(&format!("Buffer expected to be {} bytes, but found/loaded {} bytes", buffer_len, vec_len)));
        }

        let view_offset = view.offset();
        let view_len = view.length();

        println!("Buffer: {:?} => vector len {} => try range offset={}, len={}", buffer.source(), vec_len, view_offset, view_len);

        if (view_offset >= vec_len)
            || (view_len > vec_len)
            || ((view_offset + view_len) > vec_len)
        {
            Err(buffer_state.error(&format!(
                "View offset/len {}/{} not valid within buffer {:?} of length  {}",
                view_offset, view_len, buffer.source(), vec_len)))
        }
        else
        {
            func(&vector[view_offset..(view_offset+view_len)])
        }
    }

    fn decode_accessor_required_vector_u32(&self, accessor: Option<gltf::Accessor>) -> Result<Vec<usize>, ImportError>
    {
        self.decode_accessor_required_vector(accessor, gltf::accessor::Dimensions::Scalar, gltf::accessor::DataType::U32, |v| u32::from_ne_bytes(v) as usize)
    }

    fn decode_accessor_required_vector_vec3_f32(&self, accessor: Option<gltf::Accessor>) -> Result<Vec<Point3>, ImportError>
    {
        self.decode_accessor_required_vector(accessor, gltf::accessor::Dimensions::Vec3, gltf::accessor::DataType::F32,
            |v: [u8; 12]|
            {
                let x = f32::from_ne_bytes([v[0], v[1], v[2], v[3]]);
                let y = f32::from_ne_bytes([v[4], v[5], v[6], v[7]]);
                let z = f32::from_ne_bytes([v[8], v[9], v[10], v[11]]);
                Point3::new(x as Scalar, y as Scalar, z as Scalar)
            })
    }

    fn decode_accessor_optional_vector_vec2_f32(&self, accessor: Option<gltf::Accessor>) -> Result<Option<Vec<Point3>>, ImportError>
    {
        self.decode_accessor_optional_vector(accessor, gltf::accessor::Dimensions::Vec2, gltf::accessor::DataType::F32,
            |v: [u8; 8]|
            {
                let x = f32::from_ne_bytes([v[0], v[1], v[2], v[3]]);
                let y = f32::from_ne_bytes([v[4], v[5], v[6], v[7]]);
                Point3::new(x as Scalar, y as Scalar, 0.0)
            })
    }

    fn decode_accessor_optional_vector<const L: usize, T, F>(&self, accessor: Option<gltf::Accessor>, dimensions: gltf::accessor::Dimensions, data_type: gltf::accessor::DataType, convert: F) -> Result<Option<Vec<T>>, ImportError>
        where F: Fn([u8; L]) -> T + 'static
    {
        match accessor
        {
            None => Ok(None),
            Some(_) => self.decode_accessor_required_vector(accessor, dimensions, data_type, convert).map(|v| Some(v)),
        }
    }

    fn decode_accessor_required_vector<const L: usize, T, F>(&self, accessor: Option<gltf::Accessor>, dimensions: gltf::accessor::Dimensions, data_type: gltf::accessor::DataType, convert: F) -> Result<Vec<T>, ImportError>
        where F: Fn([u8; L]) -> T + 'static
    {
        match accessor
        {
            None => Err(self.error("Missing accessor")),
            Some(accessor) =>
            {
                let accessor_state = self.sub_state("accessor", accessor.name(), accessor.index());

                if (accessor.dimensions() != dimensions)
                    || (accessor.data_type() != data_type)
                {
                    Err(accessor_state.error(&format!("Expected Vec3/F32 but got {:?}/{:?}", accessor.dimensions(), accessor.data_type())))
                }
                else
                {
                    let count = accessor.count();

                    let mut result = Vec::new();
                    result.reserve(count);

                    println!("Accessor: Trying to decode {} items of type {:?}/{:?}", count, dimensions, data_type);

                    match accessor.view()
                    {
                        None => Err(accessor_state.error("No view provided")),
                        Some(view) =>
                        {
                            accessor_state.with_view_data(
                                &view,
                                |slice|
                                {
                                    let expected = L * count;
                                    if slice.len() < expected
                                    {
                                        Err(accessor_state.error(&format!("Expected {} bytes of data for {} x {:?} x {:?}, but got {} bytes",
                                            expected, count, dimensions, data_type, slice.len())))
                                    }
                                    else
                                    {
                                        for i in 0..count
                                        {
                                            let offset = i * L;
                                            let item = convert(slice[offset..(offset+L)].try_into().unwrap());
                                            result.push(item);
                                        }
                                        Ok(())
                                    }
                                })?;
        
                            Ok(result)
                        },
                    }
                }
            },
        }
    }
}
