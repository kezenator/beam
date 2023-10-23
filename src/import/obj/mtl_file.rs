use crate::color::SRGB;
use crate::import::ImportError;
use crate::import::obj::parser::Parser;

#[derive(Debug, Clone)]
pub struct Material
{
    pub name: String,
    pub diffuse: SRGB,
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

                materials.push(Material { name, diffuse: SRGB::new(0.0, 0.0, 0.0) });
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
            // TODO - support these lines
            "Ka" => { parser.ignore_line(); },
            "Ks" => { parser.ignore_line(); },
            "Ke" => { parser.ignore_line(); },
            "Ns" => { parser.ignore_line(); },
            "Ni" => { parser.ignore_line(); },
            "map_Kd" => { parser.ignore_line(); },
            "map_Ks" => { parser.ignore_line(); },
            "map_Ns" => { parser.ignore_line(); },
            "map_Bump" => { parser.ignore_line(); },
            "refl" => { parser.ignore_line(); },
            "illum" => { parser.ignore_line(); },
            "d" => { parser.ignore_line(); },
            _ =>
            {
                return Err(parser.create_error("Unsupported line"));
            }
        }
    }

    Ok(MaterialFile{ materials })
}
