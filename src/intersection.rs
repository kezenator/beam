use crate::math::Float;
use crate::vec::{Dir3, Point3};
use crate::color::RGBA;

pub struct Intersection
{
    pub distance: Float,
    pub location: Point3,
    pub normal: Dir3,
    pub color: RGBA,
}

impl Intersection
{
    pub fn new(distance: Float, location: Point3, normal: Dir3, color: RGBA) -> Self
    {
        Intersection { distance, location, normal, color }
    }
}