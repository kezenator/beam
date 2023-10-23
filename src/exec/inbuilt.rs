use crate::color::SRGB;
use crate::desc::edit::{Camera, Geom, Material, Object, Scene, Texture, Triangle, TriangleVertex};
use crate::exec::{Context, Function, Value};
use crate::import;
use crate::geom::Sdf;

use super::ExecError;

pub fn add_inbuilt_functions(root_context: &mut Context)
{
    let mut funcs = Vec::new();

    for name in ["==", "eq"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_bool(context.get_call_site(), lhs == rhs))
            }
        ));
    }

    for name in ["!=", "neq"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_bool(context.get_call_site(), lhs != rhs))
            }
        ));
    }

    funcs.push(Function::new_inbuilt(
        "neg".to_string(),
        vec!["val".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let val = context.get_param_positional(0)?.into_scalar()?;

            Ok(Value::new_scalar(context.get_call_site(), -val))
        }
    ));

    for name in ["+", "add"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs + rhs))
            }
        ));
    }

    for name in ["-", "sub"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs - rhs))
            }
        ));
    }

    for name in ["*", "mul"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs * rhs))
            }
        ));
    }

    for name in ["/", "div"].iter()
    {
        funcs.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            root_context,
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs / rhs))
            }
        ));
    }

    funcs.push(Function::new_inbuilt(
        "rgb".to_owned(),
        vec!["r".to_owned(), "g".to_owned(), "b".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let r = context.get_param_named("r")?.into_scalar()?;
            let g = context.get_param_named("g")?.into_scalar()?;
            let b = context.get_param_named("b")?.into_scalar()?;

            Ok(Value::new_color(context.get_call_site(), SRGB::new(r, g, b).into()))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "camera".to_owned(),
        vec!["location".to_owned(), "look_at".to_owned(), "up".to_owned(), "fov".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let location = context.get_param_named("location")?.into_vec3()?;
            let look_at = context.get_param_named("look_at")?.into_vec3()?;
            let up = context.get_param_named("up")?.into_vec3()?;
            let fov = context.get_param_named("fov")?.into_scalar()?;

            let camera = Camera { location, look_at, up, fov };

            context.with_app_state::<Scene, _, _>(|scene| { scene.camera = camera.clone(); Ok(()) })?;

            Ok(Value::new_camera(context.get_call_site(), camera))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "sphere".to_owned(),
        vec!["center".to_owned(), "radius".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let center = context.get_param_named("center")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            let geom = Geom::Sphere{ center, radius };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.geom.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "sdf_sphere".to_owned(),
        vec!["center".to_owned(), "radius".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let center = context.get_param_named("center")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), Sdf::Sphere{ center, radius }))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "sdf_capsule".to_owned(),
        vec!["a".to_owned(), "b".to_owned(), "radius".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let a = context.get_param_named("a")?.into_vec3()?;
            let b = context.get_param_named("b")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), Sdf::Capsule{ a, b, radius }))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "sdf_union".to_owned(),
        vec![],
        root_context,
        |context: &mut Context|
        {
            let members = context.get_param_all_positional();

            let members = members
                .into_iter()
                .map(|i| i.into_sdf())
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Value::new_sdf(context.get_call_site(), Sdf::Union{ members }))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "sdf_annular".to_owned(),
        vec!["sdf".to_owned(), "radius".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let sdf = context.get_param_named("sdf")?.into_sdf()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), Sdf::Annular{ sdf: Box::new(sdf), radius }))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "plane".to_owned(),
        vec!["point".to_owned(), "normal".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let point = context.get_param_named("point")?.into_vec3()?;
            let normal = context.get_param_named("normal")?.into_vec3()?;
            let geom = Geom::Plane{ point, normal };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.geom.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "triangle".to_owned(),
        vec!["v1".to_owned(), "v2".to_owned(), "v3".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let v1 = TriangleVertex{ location: context.get_param_named("v1")?.into_vec3()? };
            let v2 = TriangleVertex{ location: context.get_param_named("v2")?.into_vec3()? };
            let v3 = TriangleVertex{ location: context.get_param_named("v3")?.into_vec3()? };
            let geom = Geom::Triangle{triangle: Triangle { vertices: [v1, v2, v3]}};
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.geom.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "load_obj".to_owned(),
        vec!["path".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let path = context.get_param_named("path")?;
            let source_location = path.source_location();
            let path = path.into_string()?;

            context.with_app_state::<Scene, _, _>(|scene|
                {
                    import::obj::import_obj_file(&path, scene)
                        .map_err(|i| ExecError::new(source_location, i.0))?;

                    Ok(())
                })?;

            Ok(Value::new_void())
        }
    ));

    funcs.push(Function::new_inbuilt(
        "load_obj_as_mesh".to_owned(),
        vec!["path".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let path = context.get_param_named("path")?;
            let source_location = path.source_location();
            let path = path.into_string()?;

            let geom = import::obj::import_obj_file_as_triangle_mesh(&path).map_err(|i| ExecError::new(source_location, i.0))?;
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.geom.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "texture_checkerboard".to_owned(),
        vec!["a".to_owned(), "b".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let a = context.get_param_named("a")?.into_color()?;
            let b = context.get_param_named("b")?.into_color()?;

            let texture = Texture::Checkerboard(a, b);
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.textures.push(texture)))?;

            Ok(Value::new_texture(context.get_call_site(), index))
        }
    ));

    // funcs.push(Function::new_inbuilt(
    //     "texture_sdf".to_owned(),
    //     vec!["sdf".to_owned()],
    //     root_context,
    //     |context: &mut Context|
    //     {
    //         let sdf = context.get_param_named("sdf")?.into_sdf()?;

    //         Ok(Value::new_texture(context.get_call_site(), Texture::sdf(sdf)))
    //     }
    // ));

    funcs.push(Function::new_inbuilt(
        "dielectric".to_owned(),
        vec!["ior".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let ior = context.get_param_named("ior")?.into_scalar()?;
            let material = Material::Dielectric{ ior };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.materials.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "diffuse".to_owned(),
        vec!["texture".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let texture = context.get_param_named("texture")?.into_texture(context)?;
            let material = Material::Diffuse{ texture };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.materials.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "emit".to_owned(),
        vec!["texture".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let texture = context.get_param_named("texture")?.into_texture(context)?;
            let material = Material::Emit{ texture };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.materials.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "metal".to_owned(),
        vec!["texture".to_owned(), "fuzz".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let texture = context.get_param_named("texture")?.into_texture(context)?;
            let fuzz = context.get_param_named("fuzz")?.into_scalar()?;
            let material = Material::Metal{ texture, fuzz };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.materials.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    ));

    funcs.push(Function::new_inbuilt(
        "object".to_owned(),
        vec!["geometry".to_owned(), "material".to_owned()],
        root_context,
        |context: &mut Context|
        {
            let geom = context.get_param_named("geometry")?.into_geom()?;
            let material = context.get_param_named("material")?.into_material()?;
            let object = Object{ geom, material };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.objects.push(object)))?;

            Ok(Value::new_object(context.get_call_site(), index))
        }
    ));

    for func in funcs
    {
        let name = func.get_name().to_owned();
        root_context.set_var_named(&name, Value::new_function(func));
    }
}
