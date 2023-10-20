use wavefront_obj::obj;

use crate::{geom::Surface, geom::{Triangle, Mesh}, vec::Point3};

pub fn import_obj_file_as_triangle_mesh(path: &str) -> Box<dyn Surface>
{
    let obj = obj::parse(std::fs::read_to_string(path).expect("Can't read .obj file")).expect("Can't parse .obj file");

    convert_objset_to_mesh(&obj.objects[0])
}

fn convert_objset_to_mesh(obj_set: &obj::Object) -> Box<dyn Surface>
{
    let mut triangles = Vec::new();

    for geom in obj_set.geometry.iter()
    {
        for shape in geom.shapes.iter()
        {
            if let obj::Primitive::Triangle(v0, v1, v2) = shape.primitive
            {
                triangles.push(Triangle::new(
                    vertex_to_vec(&obj_set.vertices[v0.0]),
                    vertex_to_vec(&obj_set.vertices[v1.0]),
                    vertex_to_vec(&obj_set.vertices[v2.0])));
            }
        }
    }

    return Box::new(Mesh::new(triangles));
}

fn vertex_to_vec(src: &obj::Vertex) -> Point3
{
    Point3::new(src.x, src.y, src.z)
}
