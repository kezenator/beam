use crate::color::SRGB;
use crate::desc::CameraDescription;
use crate::exec::{Context, Function, Value};
use crate::geom::{Plane, Sphere};
use crate::geom::sdf::ConcreteSdf;
use crate::material::Material;
use crate::object::Object;
use crate::texture::Texture;

pub fn get_inbuilt_functions() -> Vec<Function>
{
    let mut result = Vec::new();

    result.push(Function::new_inbuilt(
        "neg".to_string(),
        vec!["val".to_owned()],
        |context: &mut Context|
        {
            let val = context.get_param_positional(0)?.into_scalar()?;

            Ok(Value::new_scalar(context.get_call_site(), -val))
        }
    ));

    for name in ["+", "add"].iter()
    {
        result.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs + rhs))
            }
        ));
    }

    for name in ["*", "mul"].iter()
    {
        result.push(Function::new_inbuilt(
            name.to_string(),
            vec!["lhs".to_owned(), "rhs".to_owned()],
            |context: &mut Context|
            {
                let lhs = context.get_param_positional(0)?.into_scalar()?;
                let rhs = context.get_param_positional(1)?.into_scalar()?;

                Ok(Value::new_scalar(context.get_call_site(), lhs * rhs))
            }
        ));
    }

    result.push(Function::new_inbuilt(
        "rgb".to_owned(),
        vec!["r".to_owned(), "g".to_owned(), "b".to_owned()],
        |context: &mut Context|
        {
            let r = context.get_param_named("r")?.into_scalar()?;
            let g = context.get_param_named("g")?.into_scalar()?;
            let b = context.get_param_named("b")?.into_scalar()?;

            Ok(Value::new_color(context.get_call_site(), SRGB::new(r, g, b).into()))
        }
    ));

    result.push(Function::new_inbuilt(
        "camera".to_owned(),
        vec!["location".to_owned(), "look_at".to_owned(), "up".to_owned(), "fov".to_owned()],
        |context: &mut Context|
        {
            let location = context.get_param_named("location")?.into_vec3()?;
            let look_at = context.get_param_named("look_at")?.into_vec3()?;
            let up = context.get_param_named("up")?.into_vec3()?;
            let fov = context.get_param_named("fov")?.into_scalar()?;

            Ok(Value::new_camera(context.get_call_site(), CameraDescription { location, look_at, up, fov }))
        }
    ));

    result.push(Function::new_inbuilt(
        "sphere".to_owned(),
        vec!["center".to_owned(), "radius".to_owned()],
        |context: &mut Context|
        {
            let center = context.get_param_named("center")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_surface(context.get_call_site(), Box::new(Sphere::new(center, radius))))
        }
    ));

    result.push(Function::new_inbuilt(
        "sdf_sphere".to_owned(),
        vec!["center".to_owned(), "radius".to_owned()],
        |context: &mut Context|
        {
            let center = context.get_param_named("center")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), ConcreteSdf::Sphere{ center, radius }))
        }
    ));

    result.push(Function::new_inbuilt(
        "sdf_capsule".to_owned(),
        vec!["a".to_owned(), "b".to_owned(), "radius".to_owned()],
        |context: &mut Context|
        {
            let a = context.get_param_named("a")?.into_vec3()?;
            let b = context.get_param_named("b")?.into_vec3()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), ConcreteSdf::Capsule{ a, b, radius }))
        }
    ));

    result.push(Function::new_inbuilt(
        "sdf_union".to_owned(),
        vec![],
        |context: &mut Context|
        {
            let members = context.get_param_all_positional();

            let members = members
                .into_iter()
                .map(|i| i.into_sdf())
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Value::new_sdf(context.get_call_site(), ConcreteSdf::Union{ members }))
        }
    ));

    result.push(Function::new_inbuilt(
        "sdf_annular".to_owned(),
        vec!["sdf".to_owned(), "radius".to_owned()],
        |context: &mut Context|
        {
            let sdf = context.get_param_named("sdf")?.into_sdf()?;
            let radius = context.get_param_named("radius")?.into_scalar()?;

            Ok(Value::new_sdf(context.get_call_site(), ConcreteSdf::Annular{ sdf: Box::new(sdf), radius }))
        }
    ));

    result.push(Function::new_inbuilt(
        "plane".to_owned(),
        vec!["point".to_owned(), "normal".to_owned()],
        |context: &mut Context|
        {
            let point = context.get_param_named("point")?.into_vec3()?;
            let normal = context.get_param_named("normal")?.into_vec3()?;

            Ok(Value::new_surface(context.get_call_site(), Box::new(Plane::new(point, normal))))
        }
    ));

    result.push(Function::new_inbuilt(
        "texture_checkerboard".to_owned(),
        vec!["a".to_owned(), "b".to_owned()],
        |context: &mut Context|
        {
            let a = context.get_param_named("a")?.into_color()?;
            let b = context.get_param_named("b")?.into_color()?;

            Ok(Value::new_texture(context.get_call_site(), Texture::checkerboard(a, b)))
        }
    ));

    result.push(Function::new_inbuilt(
        "texture_sdf".to_owned(),
        vec!["sdf".to_owned()],
        |context: &mut Context|
        {
            let sdf = context.get_param_named("sdf")?.into_sdf()?;

            Ok(Value::new_texture(context.get_call_site(), Texture::sdf(sdf)))
        }
    ));

    result.push(Function::new_inbuilt(
        "emit".to_owned(),
        vec!["texture".to_owned()],
        |context: &mut Context|
        {
            let texture = context.get_param_named("texture")?.into_texture()?;

            Ok(Value::new_material(context.get_call_site(), Material::emit(texture)))
        }
    ));

    result.push(Function::new_inbuilt(
        "diffuse".to_owned(),
        vec!["texture".to_owned()],
        |context: &mut Context|
        {
            let texture = context.get_param_named("texture")?.into_texture()?;

            Ok(Value::new_material(context.get_call_site(), Material::diffuse(texture)))
        }
    ));

    result.push(Function::new_inbuilt(
        "object".to_owned(),
        vec!["surface".to_owned(), "material".to_owned()],
        |context: &mut Context|
        {
            let surface = context.get_param_named("surface")?.into_surface()?;
            let material = context.get_param_named("material")?.into_material()?;

            Ok(Value::new_object(context.get_call_site(), Object::new_boxed(surface, material)))
        }
    ));

    result
}
