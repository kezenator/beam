use crate::geom::Surface;
use crate::intersection::ObjectIntersection;
use crate::material::Material;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct Object
{
    surface: Box<dyn Surface>,
    material: Material,
}

impl Object
{
    pub fn new_boxed(surface: Box<dyn Surface>, material: Material) -> Self
    {
        Object
        {
            surface,
            material,
        }
    }

    pub fn new<S>(surface: S, material: Material) -> Self
        where S: Surface + 'static
    {
        Object
        {
            surface: Box::new(surface),
            material,
        }
    }

    pub fn closest_intersection_in_range<'r, 'm>(&'m self, ray: &'r Ray, range: &RayRange) -> Option<ObjectIntersection<'r, 'm>>
    {
        match self.surface.closest_intersection_in_range(ray, range)
        {
            Some(si) =>
            {
                Some(ObjectIntersection
                {
                    surface: si,
                    material: &self.material,
                })
            },
            None =>
            {
                None
            },
        }
    }
}
