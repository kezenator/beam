use crate::math::Scalar;
use crate::intersection::{Face, SurfaceIntersection};
use crate::vec::{Dir3, Point3};

pub struct Ray
{
    pub source: Point3,
    pub dir: Dir3,
}

impl Ray
{
    pub fn new(source: Point3, dir: Dir3) -> Self
    {
        Ray { source, dir }
    }

    pub fn new_intersection(&self, distance: Scalar, normal: Dir3) -> SurfaceIntersection<'_>
    {
        if self.dir.dot(normal) <= 0.0
        {
            SurfaceIntersection
            {
                ray: self,
                face: Face::FrontFace,
                distance: distance,
                normal: normal,
            }
        }
        else
        {
            SurfaceIntersection
            {
                ray: self,
                face: Face::BackFace,
                distance: distance,
                normal: -normal,
            }
        }
    }
}