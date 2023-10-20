use wavefront_obj::obj;

use crate::desc::edit::{Geom, Triangle, TriangleVertex};
use crate::vec::Point3;

pub fn import_obj_file_as_triangle_mesh(path: &str) -> Geom
{
    let obj = obj::parse(std::fs::read_to_string(path).expect("Can't read .obj file")).expect("Can't parse .obj file");

    convert_objset_to_mesh(&obj.objects[0])
}

fn convert_objset_to_mesh(obj_set: &obj::Object) -> Geom
{
    let mut triangles = Vec::new();

    for geom in obj_set.geometry.iter()
    {
        for shape in geom.shapes.iter()
        {
            if let obj::Primitive::Triangle(v0, v1, v2) = shape.primitive
            {
                triangles.push(Triangle{ vertices: [
                    convert_vertex(&obj_set.vertices[v0.0]),
                    convert_vertex(&obj_set.vertices[v1.0]),
                    convert_vertex(&obj_set.vertices[v2.0]),
                ]});
            }
        }
    }

    Geom::Mesh{ triangles }
}

fn convert_vertex(src: &obj::Vertex) -> TriangleVertex
{
    TriangleVertex { location: Point3::new(src.x, src.y, src.z) }
}
