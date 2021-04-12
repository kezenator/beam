use crate::material::Material;
use crate::math::Scalar;
use crate::ray::Ray;
use crate::vec::{Dir3, Point3};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Face
{
    FrontFace,
    BackFace,
}

pub struct SurfaceIntersection<'r>
{
    pub ray: &'r Ray,
    pub distance: Scalar,
    pub face: Face,
    pub normal: Dir3,
}

impl<'r> SurfaceIntersection<'r>
{
    pub fn location(&self) -> Point3
    {
        self.ray.point_at(self.distance)
    }
}

pub struct ObjectIntersection<'r, 'm>
{
    pub surface: SurfaceIntersection<'r>,
    pub material: &'m Material,
}
