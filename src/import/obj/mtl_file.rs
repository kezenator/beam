use crate::color::SRGB;
use crate::import::ImportError;
use crate::import::obj::parser::Parser;
use crate::math::Scalar;

#[derive(Debug, Clone, Default)]
pub struct Material
{
    pub name: String,
    pub diffuse: SRGB,
    pub disolve: Option<Scalar>,
    pub ior: Option<Scalar>,
    pub diffuse_map: Option<String>,
}

impl Material
{
    pub fn new(name: String) -> Self
    {
        Material
        {
            name,
            diffuse: SRGB::new(1.0, 1.0, 1.0),
            disolve: None,
            ior: None,
            diffuse_map: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialFile
{
    pub materials: Vec<Material>
}

pub fn parse<'a>(contents: &'a str, filename: &'a str) -> Result<MaterialFile, ImportError>
{
    let mut parser = Parser::new(contents, filename);

    let mut materials = Vec::new();

    while !parser.is_empty()
    {
        match parser.first_part()
        {
            "newmtl" =>
            {
                let name = parser.parse_line_1_string()?.to_owned();

                materials.push(Material::new(name));
            },
            "Kd" =>
            {
                if materials.is_empty()
                {
                    return Err(parser.create_error("Expect \"newmtl\" line first"));
                }

                let diffuse = parser.parse_line_vector()?;

                let last_material_index = materials.len() - 1;
                materials[last_material_index].diffuse = SRGB::new(diffuse.0, diffuse.1, diffuse.2);
            },
            "Ni" =>
            {
                if materials.is_empty()
                {
                    return Err(parser.create_error("Expect \"newmtl\" line first"));
                }

                let ior = parser.parse_line_1_float()?;

                let last_material_index = materials.len() - 1;
                materials[last_material_index].ior = Some(ior);
            },
            "d" =>
            {
                if materials.is_empty()
                {
                    return Err(parser.create_error("Expect \"newmtl\" line first"));
                }

                let disolve = parser.parse_line_1_float()?;

                let last_material_index = materials.len() - 1;
                materials[last_material_index].disolve = Some(disolve);
            },
            "map_Kd" =>
            {
                if materials.is_empty()
                {
                    return Err(parser.create_error("Expect \"newmtl\" line first"));
                }

                let filename = parser.parse_line_1_string()?.to_owned();

                let last_material_index = materials.len() - 1;
                materials[last_material_index].diffuse_map = Some(filename);
            },
            // TODO - support these lines
            "Ka" => { parser.ignore_line(); },
            "Ks" => { parser.ignore_line(); },
            "Ke" => { parser.ignore_line(); },
            "Ns" => { parser.ignore_line(); },
            "map_Ks" => { parser.ignore_line(); },
            "map_Ns" => { parser.ignore_line(); },
            "map_Bump" => { parser.ignore_line(); },
            "refl" => { parser.ignore_line(); },
            "illum" => { parser.ignore_line(); },
            _ =>
            {
                return Err(parser.create_error("Unsupported line"));
            }
        }
    }

    Ok(MaterialFile{ materials })
}
