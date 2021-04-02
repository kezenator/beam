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
}