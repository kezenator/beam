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
                face: Face::Front,
                distance: distance,
                location: None,
                normal: normal,
            }
        }
        else
        {
            SurfaceIntersection
            {
                ray: self,
                face: Face::Back,
                distance: distance,
                location: None,
                normal: -normal,
            }
        }
    }

    pub fn new_intersection_at(&self, distance: Scalar, location: Point3, normal: Dir3) -> SurfaceIntersection<'_>
    {
        if self.dir.dot(normal) <= 0.0
        {
            SurfaceIntersection
            {
                ray: self,
                face: Face::Front,
                distance: distance,
                location: Some(location),
                normal: normal,
            }
        }
        else
        {
            SurfaceIntersection
            {
                ray: self,
                face: Face::Back,
                distance: distance,
                location: Some(location),
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

    pub fn min(&self) -> Scalar
    {
        self.min
    }

    pub fn max(&self) -> Scalar
    {
        self.max
    }

    pub fn is_empty(&self) -> bool
    {
        !(self.min < self.max)
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

    pub fn set_min(&mut self, min: Scalar)
    {
        self.min = min;
    }

    pub fn set_max(&mut self, max: Scalar)
    {
        self.max = max;
    }
}