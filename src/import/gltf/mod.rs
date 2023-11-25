use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use crate::color::SRGB;
use crate::desc::edit::transform::TransformStage;
use crate::desc::edit::{Scene, Triangle, TriangleVertex, Geom, Transform, Object, Material, Texture, Color};
use crate::geom::{Aabb, AabbBuilder};
use crate::import;
use crate::import::{FileSystemContext, ImportError};
use crate::indexed::{ImageIndex, MaterialIndex, TextureIndex, TransformIndex};
use crate::math::Scalar;
use crate::vec::{Point3, Mat4, Vec3, Quaternion};

pub fn import_gltf_file(path: &str, destination: &Aabb, scene: &mut Scene) -> Result<(), ImportError>
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

            let mut scene_transform = Transform::default();
            let scene_transform_index = scene_state.state.borrow_mut().scene.collection.push_named(scene_transform.clone(), scene_state.collection_name());

            let mut aabb_builder = AabbBuilder::new();

            for node in gltf_scene.nodes()
            {
                import_node(&scene_state, &node, scene_transform_index, &Mat4::identity(), &mut aabb_builder)?;
            }

            let aabb = aabb_builder.build();

            scene_transform.stages.push(TransformStage::ShiftAndScale
                {
                    from: crate::desc::edit::geom::Aabb{ min: aabb.min, max: aabb.max },
                    to: crate::desc::edit::geom::Aabb{ min: destination.min, max: destination.max },
                    maintain_aspect: true,
                });
            scene_state.state.borrow_mut().scene.collection.update_value(scene_transform_index, scene_transform);

            Ok(())
        },
    }
}

fn import_node(parent_state: &ScopedState, node: &gltf::Node, parent_transform_index: TransformIndex, parent_transform_matrix: &Mat4, aabb_builder: &mut AabbBuilder) -> Result<(), ImportError>
{
    let node_state = parent_state.sub_state("node", node.name(), node.index());

    if let Some(camera) = node.camera()
    {
        let _camera_state = node_state.sub_state("camera", camera.name(), camera.index());
    }

    let local_matrix = Mat4::from_col_arrays(node.transform().matrix().map(|r| r.map(|e| e as Scalar)));
    let local_transform_index =
    {
        if local_matrix == Mat4::identity()
        {
            parent_transform_index
        }
        else
        {
            let mut state = node_state.state.borrow_mut();

            let (trans, quot, scale) = node.transform().decomposed();


            let mut local_transform = Transform::default();
            local_transform.post = Some(parent_transform_index);

            if scale[0] != 1.0 || scale[1] != 1.0 || scale[2] != 1.0
            {
                local_transform.stages.push(TransformStage::Scale3D(Vec3::new(scale[0] as Scalar, scale[1] as Scalar, scale[2] as Scalar)));
            }

            if quot[0] != 0.0 || quot[1] != 0.0 || quot[2] != 0.0 || quot[3] != 1.0
            {
                local_transform.stages.push(TransformStage::Quaternion(Quaternion{ x: quot[0] as Scalar, y: quot[1] as Scalar, z: quot[2] as Scalar, w: quot[3] as Scalar}));
            }

            if trans[0] != 0.0 || trans[1] != 0.0 || trans[2] != 0.0
            {
                local_transform.stages.push(TransformStage::Translate(Vec3::new(trans[0] as Scalar, trans[1] as Scalar, trans[2] as Scalar)));
            }
            
            state.scene.collection.push_named(local_transform, node_state.collection_name())
        }
    };

    let node_matrix = *parent_transform_matrix * local_matrix;

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
                    let color_coords = primitive_state.decode_accessor_optional_vector_color(primitive.get(&gltf::mesh::Semantic::Colors(0)))?;

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

                            let mut c = None;
                            let mut d = None;
                            let mut e = None;

                            if let Some(color_coords) = &color_coords
                            {
                                c = Some(color_coords[indexes[3 * i + 0]]);
                                d = Some(color_coords[indexes[3 * i + 1]]);
                                e = Some(color_coords[indexes[3 * i + 2]]);
                            }

                            triangles.push(Triangle { vertices: [
                                TriangleVertex{ location: x, texture_coords: u, opt_color: c, },
                                TriangleVertex{ location: y, texture_coords: v, opt_color: d, },
                                TriangleVertex{ location: z, texture_coords: w, opt_color: e, },
                            ]});

                            let x = node_matrix.mul_point(x);                            
                            let y = node_matrix.mul_point(y);
                            let z = node_matrix.mul_point(z);

                            aabb_builder.add_triangle(x, y, z);
                        }

                        let material = import_material(&primitive_state, primitive.material())?;

                        let mut geom_transform = Transform::new();
                        geom_transform.post = Some(local_transform_index);

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
        import_node(&node_state, &child, local_transform_index, &node_matrix, aabb_builder)?;
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
            let mapped_material = map_material(&material_state, material)?;

            let mut state = material_state.state.borrow_mut();
            let added_index = state.scene.collection.push_named(mapped_material, material_state.collection_name());
            state.materials.insert(index, added_index.clone());
            Ok(added_index)
        },
        Some(existing) =>
        {
            Ok(existing)
        }
    }        
}

fn map_material(material_state: &ScopedState, material: gltf::Material) -> Result<Material, ImportError>
{
    if let Some(spec_glossy) = material.pbr_specular_glossiness()
    {
        let diffuse = spec_glossy.diffuse_factor();
        let diffuse = SRGB::new(diffuse[0] as Scalar, diffuse[1] as Scalar, diffuse[2] as Scalar, diffuse[3] as Scalar);

        let texture = import_texture(
            material_state,
            "base_color",
            diffuse.into(),
            spec_glossy.diffuse_texture())?;

        return Ok(Material::Diffuse{ texture });
    }

    let mr = material.pbr_metallic_roughness();

    let base_color_factor = mr.base_color_factor();
    let base_color_factor = SRGB::new(base_color_factor[0] as Scalar, base_color_factor[1] as Scalar, base_color_factor[2] as Scalar, base_color_factor[3] as Scalar);

    let texture = import_texture(
        material_state,
        "base_color",
        base_color_factor.into(),
        mr.base_color_texture())?;

    if mr.metallic_factor() == 0.0
    {
        Ok(Material::Diffuse{ texture })
    }
    else if mr.metallic_factor() == 1.0
    {
        Ok(Material::Metal{ texture, fuzz: mr.roughness_factor().powf(2.0) as f64 })
    }
    else
    {
        Err(material_state.error("Unsupported material - PBR Metallic/Roughness metallic-factor only 0.0 or 1.0 supported"))
    }
}

fn import_texture(parent_state: &ScopedState, part: &'static str, base_color: Color, opt_texture_info: Option<gltf::texture::Info>) -> Result<TextureIndex, ImportError>
{
    let texture = match opt_texture_info
    {
        None =>
        {
            Ok(Texture::Solid(base_color))
        },
        Some(info) =>
        {
            let image = import_image(parent_state, info.texture().source())?;
            Ok(Texture::Image{ base_color, image })
        },
    }?;

    let mut state = parent_state.state.borrow_mut();
    Ok(state.scene.collection.push_named(texture, format!("{} ({})", parent_state.collection_name(), part)))
} 

fn import_image(parent_state: &ScopedState, image: gltf::Image) -> Result<ImageIndex, ImportError>
{
    // Check for existing import
    {
        let state = parent_state.state.borrow_mut();
        if let Some(existing) = state.images.get(&image.index())
        {
            return Ok(*existing);
        }
    }

    // Import for the first time

    let image_state = parent_state.sub_state("image", image.name(), image.index());

    match image.source()
    {
        gltf::image::Source::View { .. } => Err(image_state.error("Loading images from a view not supported")),
        gltf::image::Source::Uri { uri, .. } =>
        {
            let name = image.name().map(|n| n.to_owned()).unwrap_or_else(|| uri.to_owned());

            let mut state = image_state.state.borrow_mut();
            let imported_image = import::image::import_image(uri, &mut state.fs_context)?;
            let image_index = state.scene.collection.push_named(imported_image, name.clone());
            state.images.insert(image.index(), image_index);

            Ok(image_index)
        },
    }
}

struct ImportState<'a>
{
    scene: &'a mut Scene,
    fs_context: FileSystemContext,
    blobs: HashMap<Option<String>, Vec<u8>>,
    materials: HashMap<usize, MaterialIndex>,
    images: HashMap<usize, ImageIndex>,
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
        let images = HashMap::new();
        let state = Rc::new(RefCell::new(ImportState { scene, fs_context, blobs, materials, images }));
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
        if let Some(true) = accessor.as_ref().map(|a| a.data_type() == gltf::accessor::DataType::U16)
        {
            self.decode_accessor_required_vector(
                accessor,
                gltf::accessor::Dimensions::Scalar,
                gltf::accessor::DataType::U16,
                |v| u16::from_ne_bytes(v) as usize)
        }
        else // assume u32
        {
            self.decode_accessor_required_vector(
                accessor,
                gltf::accessor::Dimensions::Scalar,
                gltf::accessor::DataType::U32,
                |v| u32::from_ne_bytes(v) as usize)
        }
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

    fn decode_accessor_optional_vector_color(&self, accessor: Option<gltf::Accessor>) -> Result<Option<Vec<Color>>, ImportError>
    {
        let mut result = None;

        if let Some(dimensions) = accessor.as_ref().map(|a| a.dimensions())
        {
            if dimensions == gltf::accessor::Dimensions::Vec4
            {
                result = self.decode_accessor_optional_vector(accessor, gltf::accessor::Dimensions::Vec4, gltf::accessor::DataType::F32,
                |v: [u8; 16]|
                        {
                            let r = f32::from_ne_bytes([v[0], v[1], v[2], v[3]]) as Scalar;
                            let g = f32::from_ne_bytes([v[4], v[5], v[6], v[7]]) as Scalar;
                            let b = f32::from_ne_bytes([v[8], v[9], v[10], v[11]]) as Scalar;
                            let a = f32::from_ne_bytes([v[12], v[13], v[14], v[15]]) as Scalar;
                            SRGB::new(r, g, b, a).into()
                        })?;
                }
            else // assume RGB with no A
            {
                result = self.decode_accessor_optional_vector(accessor, gltf::accessor::Dimensions::Vec3, gltf::accessor::DataType::F32,
            |v: [u8; 12]|
                    {
                        let r = f32::from_ne_bytes([v[0], v[1], v[2], v[3]]) as Scalar;
                        let g = f32::from_ne_bytes([v[4], v[5], v[6], v[7]]) as Scalar;
                        let b = f32::from_ne_bytes([v[8], v[9], v[10], v[11]]) as Scalar;
                        SRGB::new(r, g, b, 1.0).into()
                    })?;
            }
        }

        Ok(result)
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
                    Err(accessor_state.error(&format!("Expected {:?}/{:?} but got {:?}/{:?}", dimensions, data_type, accessor.dimensions(), accessor.data_type())))
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
