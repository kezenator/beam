use crate::color::SRGB;
use crate::desc::edit::{Camera, Geom, Material, Object, Scene, Texture, Triangle, TriangleVertex};
use crate::exec::{Context, Value};
use crate::math::Scalar;
use crate::import;
use crate::geom::{Sdf, Aabb};
use crate::vec::{Dir3, Point3};

use super::{ExecError, NativeFunctionBuilder};

pub fn add_inbuilt_functions(root_context: &mut Context)
{
    let mut builder = NativeFunctionBuilder::new(root_context);

    builder.add_2(
        ["==", "eq"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_bool(context.get_call_site(), lhs == rhs))
        }
    );

    builder.add_2(
        ["!=", "ne"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_bool(context.get_call_site(), lhs != rhs))
        }
    );

    builder.add_1(
        "neg",
        ["val"],
        |context, val: Scalar|
        {
            Ok(Value::new_scalar(context.get_call_site(), -val))
        }
    );

    builder.add_2(
        ["+", "add"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_scalar(context.get_call_site(), lhs + rhs))
        }
    );

    builder.add_2(
        ["-", "sub"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_scalar(context.get_call_site(), lhs - rhs))
        }
    );

    builder.add_2(
        ["*", "mul"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_scalar(context.get_call_site(), lhs * rhs))
        }
    );

    builder.add_2(
        ["/", "div"],
        ["lhs", "rhs"],
        |context, lhs: Scalar, rhs: Scalar|
        {
            Ok(Value::new_scalar(context.get_call_site(), lhs / rhs))
        }
    );

    builder.add_3(
        "rgb",
        ["r", "g", "b"],
        |context, r: Scalar, g: Scalar, b: Scalar|
        {
            Ok(Value::new_color(context.get_call_site(), SRGB::new(r, g, b, 1.0).into()))
        }
    );

    builder.add_4(
        "rgba",
        ["r", "g", "b", "a"],
        |context, r: Scalar, g: Scalar, b: Scalar, a: Scalar|
        {
            Ok(Value::new_color(context.get_call_site(), SRGB::new(r, g, b, a).into()))
        }
    );

    builder.add_4(
        "camera",
        ["location", "look_at", "up", "fov"],
        |context, location: Point3, look_at: Point3, up: Dir3, fov: Scalar|
        {
            let camera = Camera { location, look_at, up, fov };

            context.with_app_state::<Scene, _, _>(|scene| { scene.camera = camera.clone(); Ok(()) })?;

            Ok(Value::new_camera(context.get_call_site(), camera))
        }
    );

    builder.add_2(
        "aabb",
        ["min", "max"],
        |context, min: Point3, max: Point3|
        {
            let aabb = Aabb::new(min, max);

            Ok(Value::new_aabb(context.get_call_site(), aabb))
        }
    );

    builder.add_3(
        "sphere",
        ["center", "radius", "name"],
        |context, center: Point3, radius: Scalar, name: Option<String>|
        {
            let geom = Geom::Sphere{ center, radius };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push_opt_name(geom, name)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    );

    builder.add_2(
        "sdf_sphere",
        ["center", "radius"],
        |context, center: Point3, radius: Scalar|
        {
            Ok(Value::new_sdf(context.get_call_site(), Sdf::Sphere{ center, radius }))
        }
    );

    builder.add_3(
        "sdf_capsule",
        ["a", "b", "radius"],
        |context, a: Point3, b: Point3, radius: Scalar|
        {
            Ok(Value::new_sdf(context.get_call_site(), Sdf::Capsule{ a, b, radius }))
        }
    );

    builder.add_vec(
        "sdf_union",
        "items",
        |context, members|
        {
            Ok(Value::new_sdf(context.get_call_site(), Sdf::Union { members }))
        }
    );

    builder.add_2(
        "sdf_annular",
        ["sdf", "radius"],
        |context, sdf: Sdf, radius: Scalar|
        {
            Ok(Value::new_sdf(context.get_call_site(), Sdf::Annular{ sdf: Box::new(sdf), radius }))
        }
    );

    builder.add_2(
        "plane",
        ["point", "normal"],
        |context, point, normal|
        {
            let geom = Geom::Plane{ point, normal };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    );

    builder.add_3(
        "triangle",
        ["v1", "v2", "v3"],
        |context, v1, v2, v3|
        {
            let v1 = TriangleVertex{ location: v1, texture_coords: Point3::new(0.0, 0.0, 0.0), opt_color: None, };
            let v2 = TriangleVertex{ location: v2, texture_coords: Point3::new(0.0, 0.0, 0.0), opt_color: None, };
            let v3 = TriangleVertex{ location: v3, texture_coords: Point3::new(0.0, 0.0, 0.0), opt_color: None, };
            let geom = Geom::Triangle{triangle: Triangle { vertices: [v1, v2, v3]}};
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    );

    builder.add_2(
        "load_obj",
        ["path", "destination"],
        |context, path: Value, destination|
        {
            let source_location = path.source_location();
            let path = path.into_string()?;

            context.with_app_state::<Scene, _, _>(|scene|
                {
                    import::obj::import_obj_file(&path, &destination, scene)
                        .map_err(|i| ExecError::new(source_location, i.0))?;

                    Ok(())
                })?;

            Ok(Value::new_void())
        }
    );

    builder.add_1(
        "load_obj_as_mesh",
        ["path"],
        |context, path: Value|
        {
            let source_location = path.source_location();
            let path = path.into_string()?;

            let geom = import::obj::import_obj_file_as_triangle_mesh(&path).map_err(|i| ExecError::new(source_location, i.0))?;
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(geom)))?;

            Ok(Value::new_geom(context.get_call_site(), index))
        }
    );

    builder.add_2(
        "load_gltf",
        ["path", "destination"],
        |context, path: Value, destination|
        {
            let source_location = path.source_location();
            let path = path.into_string()?;

            context.with_app_state::<Scene, _, _>(|scene|
                {
                    import::gltf::import_gltf_file(&path, &destination, scene)
                        .map_err(|i| ExecError::new(source_location, i.0))?;

                    Ok(())
                })?;

            Ok(Value::new_void())
        }
    );

    builder.add_2(
        "texture_checkerboard",
        ["a", "b"],
        |context, a, b|
        {
            let texture = Texture::Checkerboard(a, b);
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(texture)))?;

            Ok(Value::new_texture(context.get_call_site(), index))
        }
    );

    builder.add_1(
        "dielectric",
        ["ior"],
        |context, ior|
        {
            let material = Material::Dielectric{ ior };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    );

    builder.add_1(
        "diffuse",
        ["texture"],
        |context, texture|
        {
            let material = Material::Diffuse{ texture };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    );

    builder.add_1(
        "emit",
        ["texture"],
        |context, texture|
        {
            let material = Material::Emit{ texture };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    );

    builder.add_2(
        "metal",
        ["texture", "fuzz"],
        |context, texture, fuzz|
        {
            let material = Material::Metal{ texture, fuzz };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(material)))?;

            Ok(Value::new_material(context.get_call_site(), index))
        }
    );

    builder.add_2(
        "object",
        ["geometry", "material"],
        |context, geom, material|
        {
            let object = Object{ geom, material };
            let index = context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(object)))?;

            Ok(Value::new_object(context.get_call_site(), index))
        }
    );

    for func in builder.build()
    {
        let name = func.get_name().to_owned();
        root_context.set_var_named(&name, Value::new_function(func));
    }
}
