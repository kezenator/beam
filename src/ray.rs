use crate::math::{EPSILON, Scalar};
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

    pub fn point_at(&self, distance: Scalar) -> Point3
    {
        self.source + distance * self.dir
    }
}

#[derive(Clone)]
pub struct RayRange
{
    min: Scalar,
    max: Scalar,
}

impl RayRange
{
    pub fn new(min: Scalar, max: Scalar) -> Self
    {
        RayRange
        {
            min,
            max,
        }
    }

    pub fn intersection(&self, other: &RayRange) -> Option<Self>
    {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);

        if min < max
        {
            Some(RayRange{ min, max })
        }
        else
        {
            None
        }
    }

    pub fn contains(&self, val: Scalar) -> bool
    {
        val > self.min && val < self.max
    }

    pub fn update_max(&mut self, max: Scalar)
    {
        self.max = max;
    }
}