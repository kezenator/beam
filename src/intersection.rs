use crate::material::Material;
use crate::math::Scalar;
use crate::ray::Ray;
use crate::vec::{Dir3, Point3};

pub struct SurfaceIntersection<'r>
{
    pub ray: &'r Ray,
    pub distance: Scalar,
    pub normal: Dir3,
}

impl<'r> SurfaceIntersection<'r>
{
    pub fn location(&self) -> Point3
    {
        self.ray.source + self.distance * self.ray.dir
    }
}

pub type SurfaceIntersectionCollector<'r, 'c> = dyn FnMut(SurfaceIntersection<'r>) + 'c;

pub struct ObjectIntersection<'r, 'm>
{
    pub surface: SurfaceIntersection<'r>,
    pub material: &'m Material,
}
