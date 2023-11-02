use crate::material::Material;
use crate::math::Scalar;
use crate::ray::Ray;
use crate::vec::{Dir3, Point3};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Face
{
    Front,
    Back,
}

pub struct SurfaceIntersection<'r>
{
    pub ray: &'r Ray,
    pub distance: Scalar,
    pub location: Option<Point3>,
    pub face: Face,
    pub normal: Dir3,
    pub texture_coords: Option<Point3>,
}

impl<'r> SurfaceIntersection<'r>
{
    pub fn location(&self) -> Point3
    {
        match self.location
        {
            Some(location) => location,
            None => self.ray.point_at(self.distance),
        }
    }

    pub fn texture_coords(&self) -> Point3
    {
        self.texture_coords.unwrap_or(self.location())
    }
}

pub struct ObjectIntersection<'r, 'm>
{
    pub surface: SurfaceIntersection<'r>,
    pub material: &'m Material,
}

pub struct ShadingIntersection
{
    pub location: Point3,
    pub normal: Point3,
    pub incoming: Point3,
    pub texture_coords: Point3,
    pub face: Face,
}

impl<'r> From<SurfaceIntersection<'r>> for ShadingIntersection
{
    fn from(val: SurfaceIntersection<'r>) -> Self
    {
        ShadingIntersection
        {
            location: val.location(),
            normal: val.normal,
            incoming: -val.ray.dir.normalized(),
            texture_coords: val.texture_coords(),
            face: val.face,
        }
    }
}
