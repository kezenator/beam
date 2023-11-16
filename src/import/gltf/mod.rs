use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use crate::color::SRGB;
use crate::desc::edit::transform::TransformStage;
use crate::desc::edit::{Scene, Triangle, TriangleVertex, Geom, Transform, Object, Material, Texture};
use crate::geom::Aabb;
use crate::import;
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::{MaterialIndex, TextureIndex};
use crate::math::Scalar;
use crate::vec::{Point3, Mat4};

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
            let identity_matrix = Mat4::identity();

            for node in gltf_scene.nodes()
            {
                import_node(&scene_state, &node, &identity_matrix)?;
            }

            Ok(())
        },
    }
}

fn import_node(parent_state: &ScopedState, node: &gltf::Node, parent_matrix: &Mat4) -> Result<(), ImportError>
{
    let node_state = parent_state.sub_state("node", node.name(), node.index());

    if let Some(camera) = node.camera()
    {
        let _camera_state = node_state.sub_state("camera", camera.name(), camera.index());
    }

    let local_matrix = Mat4::from_col_arrays(node.transform().matrix().map(|r| r.map(|e| e as f64)));
    let node_matrix = *parent_matrix * local_matrix;

    if let Some(mesh) = node.mesh()
    {
        let mesh_state = node_state.sub_state("mesh", mesh.name(), mesh.index());

        let single_primitive = mesh.primitives().len() == 1;

        for primitive in mesh.primitives()
        {
            let primitive_state = mesh_state.sub_state("primitive", None, primitive.index());

            let primitive_name = if single_primitive { mesh_state.collection_name() } else { format!("{}-{}", mesh_state.collection_name(), primitive.index()) };

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
                        println!("{:?}", indexes);
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

                        let mut geom_transform = Transform::new();
                        geom_transform.stages.push(TransformStage::Matrix(node_matrix.clone()));

                        // TODO - different co-ordinate systems?
                        geom_transform.stages.push(TransformStage::Matrix(Mat4::scaling_3d(Point3::new(1.0, 1.0, -1.0))));

                        let mut state = primitive_state.state.borrow_mut();
                        let geom = state.scene.collection.push_named(Geom::Mesh{ triangles, transform: geom_transform }, primitive_name.clone());
                        let _obj = state.scene.collection.push_named(Object{ geom, material }, primitive_name);
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
        import_node(&node_state, &child, &node_matrix)?;
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
                        Ok(state.scene.collection.push_named(Texture::Solid(diffuse.into()), material_state.collection_name()))
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
                let added_index = state.scene.collection.push_named(Material::Diffuse{ texture }, material_state.collection_name());
                state.materials.insert(index, added_index.clone());
                Ok(added_index)
            }
            else
            {
                let mr = material.pbr_metallic_roughness();

                let base_color_factor = mr.base_color_factor();
                let base_color_factor = SRGB::new(base_color_factor[0] as Scalar, base_color_factor[1] as Scalar, base_color_factor[2] as Scalar, base_color_factor[3] as Scalar);

                let texture = match mr.base_color_texture()
                {
                    None =>
                    {
                        let mut state = material_state.state.borrow_mut();
                        Ok(state.scene.collection.push_named(Texture::Solid(base_color_factor.into()), material_state.collection_name()))
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

                if mr.metallic_factor() == 0.0
                {
                    let added_index = state.scene.collection.push_named(Material::Diffuse{ texture }, material_state.collection_name());
                    state.materials.insert(index, added_index.clone());
                    Ok(added_index)
                }
                else if mr.metallic_factor() == 1.0
                {
                    let added_index = state.scene.collection.push_named(Material::Metal{ texture, fuzz: mr.roughness_factor().powf(2.0) as f64 }, material_state.collection_name());
                    state.materials.insert(index, added_index.clone());
                    Ok(added_index)
                }
                else
                {
                    Err(material_state.error("Unsupported material - PBR Metallic/Roughness metallic-factor only 0.0 or 1.0 supported"))
                }
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

            Ok(state.scene.collection.push_named(Texture::Image(image), texture_state.collection_name()))
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
    collection_name: String,
}

impl<'a> ScopedState<'a>
{
    fn new(scene: &'a mut Scene, fs_context: FileSystemContext, filename: String) -> Self
    {
        let blobs = HashMap::new();
        let materials = HashMap::new();
        let state = Rc::new(RefCell::new(ImportState { scene, fs_context, blobs, materials }));
        ScopedState { state, path: filename.clone(), collection_name: filename.clone() }
    }

    fn sub_state(&self, kind: &str, name: Option<&str>, index: usize) -> Self
    {
        let path = format!("{}/{}-{}", self.path, kind, name.map(|s| s.to_string()).unwrap_or_else(|| index.to_string()));
        let collection_name = name.map(|s| s.to_string()).unwrap_or_else(|| format!("{}-{}", kind, index));
        println!("Entering: {}: {} ({})", kind, path, collection_name);
        ScopedState { state: self.state.clone(), path, collection_name }
    }

    fn collection_name(&self) -> String
    {
        self.collection_name.clone()
    }

    fn error(&self, msg: &str) -> ImportError
    {
        ImportError(format!("GLTF Error: {}: {}", self.path, msg))
    }

    fn with_view_data<F>(&self, accessor: &gltf::Accessor, view: &gltf::buffer::View, accessor_len: usize, func: F) -> Result<(), ImportError>
        where F: FnOnce(&[u8], Option<usize>) -> Result<(), ImportError>
    {
        let view_state = self.sub_state("view", view.name(), view.index());

        let buffer = view.buffer();
        let buffer_state = view_state.sub_state("buffer", buffer.name(), buffer.index());

        let mut state = self.state.borrow_mut();

        // Load the buffer

        let buffer_vector = match buffer.source()
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

        let buffer_vec_len = buffer_vector.len();
        let buffer_len = buffer.length();

        if buffer_vec_len > buffer_len
        {
            return Err(buffer_state.error(&format!("Buffer expected to be {} bytes, but found/loaded {} bytes", buffer_len, buffer_vec_len)));
        }

        // Check the view is contained in the buffer

        let view_offset = view.offset();
        let view_len = view.length();
        let view_stride = view.stride();

        let accessor_offset = accessor.offset();

        println!("Buffer: {:?} => buffer len {} => try range accessor[offset={}] view[offset={}, len={}, stride={:?}]", buffer.source(), buffer_vec_len, accessor_offset, view_offset, view_len, view_stride);

        let view_end = view_offset + view_len;

        if (view_end < view_offset)
            || (view_offset >= buffer_vec_len)
            || (view_end > buffer_vec_len)
        {
            Err(buffer_state.error(&format!(
                "View[index={}, offset={}, len={}] not valid within Buffer[index={}, source={:?}, len={}]",
                view.index(), view_offset, view_len, buffer.index(), buffer.source(), buffer_vec_len)))
        }
        else
        {
            // Now check the accessor is contained within the view

            let accessor_offset = view_offset + accessor_offset;
            let accessor_end = accessor_offset + accessor_len;

            if (accessor_end < accessor_offset)
                || (accessor_offset < view_offset)
                || (accessor_offset >= view_end)
                || (accessor_end > view_end)
            {
                Err(buffer_state.error(&format!(
                    "Accessor[index={}, offset={}, len={}], View[index={}, offset={}, len={}] not valid within Buffer[index={}, source={:?}, len={}]",
                    accessor.index(), accessor_offset, accessor_len, view.index(), view_offset, view_len, buffer.index(), buffer.source(), buffer_vec_len)))
            }
            else
            {
                func(&buffer_vector[accessor_offset..accessor_end], view_stride)
            }
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
                            let expected = L * count;

                            accessor_state.with_view_data(
                                &accessor,
                                &view,
                                expected,
                                |slice, stride|
                                {
                                    if slice.len() < expected
                                    {
                                        Err(accessor_state.error(&format!("Expected {} bytes of data for {} x {:?} x {:?}, but got {} bytes",
                                            expected, count, dimensions, data_type, slice.len())))
                                    }
                                    else if stride.unwrap_or(L) != L
                                    {
                                        Err(accessor_state.error(&format!("Expected stride of {} for {} x {:?} x {:?}, but got {:?}",
                                            L, count, dimensions, data_type, stride)))
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
