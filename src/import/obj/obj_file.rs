use crate::math::Scalar;
use crate::import::ImportError;
use crate::import::obj::parser::Parser;

pub type Vector = (Scalar, Scalar, Scalar);

pub type VectorIndex = usize;

#[derive(Debug, Clone)]
pub struct Vertex
{
    pub vertex_index: VectorIndex,
    pub normal_index: Option<VectorIndex>,
    pub texture_index: Option<VectorIndex>,
}
pub type Triangle = [Vertex; 3];

#[derive(Debug, Clone)]
pub struct Geometry
{
    pub material_name: Option<String>,
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, Clone)]
pub struct Object
{
    pub name: String,
    pub geometry: Vec<Geometry>
}

#[derive(Debug, Clone)]
pub struct ObjFile
{
    pub material_library: Option<String>,
    pub vertices: Vec<Vector>,
    pub texture_coords: Vec<Vector>,
    pub normals: Vec<Vector>,
    pub objects: Vec<Object>,
}

pub fn parse<'a>(contents: &'a str, filename: &'a str) -> Result<ObjFile, ImportError>
{
    let mut parser = Parser::new(contents, filename);

    let mut material_library = None;
    let mut vertices = Vec::new();
    let mut texture_coords = Vec::new();
    let mut normals = Vec::new();
    let mut objects = Vec::new();

    while !parser.is_empty()
    {
        match parser.first_part()
        {
            "mtllib" =>
            {
                if material_library.is_some()
                {
                    return Err(parser.create_error("Duplicate material library"));
                }
                material_library = Some(parser.parse_line_1_string()?.to_owned());
            },
            "o" =>
            {
                objects.push(Object{
                    name: parser.parse_line_1_string()?.to_owned(),
                    geometry: Vec::new(),
                });
            },
            "v" =>
            {
                vertices.push(parser.parse_line_vector()?);
            },
            "vn" =>
            {
                normals.push(parser.parse_line_vector()?);
            },
            "vt" =>
            {
                texture_coords.push(parser.parse_line_vector()?);
            },
            "usemtl" =>
            {
                if objects.is_empty()
                {
                    return Err(parser.create_error("\"o\" line required before \"usemtl\""));
                }

                let last_obj_index = objects.len() - 1;
                let obj = &mut objects[last_obj_index];

                obj.geometry.push(Geometry
                {
                    material_name: Some(parser.parse_line_1_string()?.to_owned()),
                    triangles: Vec::new(),
                });
            },
            "f" =>
            {
                if objects.is_empty()
                {
                    return Err(parser.create_error("\"o\" line required before \"f\""));
                }

                let last_obj_index = objects.len() - 1;
                let obj = &mut objects[last_obj_index];

                if obj.geometry.is_empty()
                {
                    obj.geometry.push(Geometry
                    {
                        material_name: None,
                        triangles: Vec::new()
                    });
                }

                let last_geom_index = obj.geometry.len() - 1;
                let obj = &mut obj.geometry[last_geom_index];

                let mut triangles = parser.parse_line_triangles(vertices.len(), texture_coords.len(), normals.len())?;

                obj.triangles.append(&mut triangles);
            },
            "l" => { parser.ignore_line(); },
            "g" => { parser.ignore_line(); },
            "s" => { parser.ignore_line(); },
            _ =>
            {
                return Err(parser.create_error("Unsupported line"));
            }
        }
    }

    Ok(ObjFile{ material_library, vertices, texture_coords, normals, objects })
}
