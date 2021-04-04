use crate::geom::Surface;
use crate::intersection::{ObjectIntersection, SurfaceIntersection};
use crate::material::Material;
use crate::ray::Ray;

pub struct Object
{
    surface: Box<dyn Surface>,
    material: Material,
}

impl Object
{
    pub fn new<S>(surface: S, material: Material) -> Self
        where S: Surface + 'static
    {
        Object
        {
            surface: Box::new(surface),
            material,
        }
    }

    pub fn get_intersections<'r, 'm>(&'m self, ray: &'r Ray, intersections: &mut Vec<ObjectIntersection<'r, 'm>>)
    {
        let mut collector = move |si: SurfaceIntersection<'r>|
        {
            intersections.push(ObjectIntersection
            {
                surface: si,
                material: &self.material,
            });
        };

        self.surface.get_intersections(ray, &mut collector);
    }
}
