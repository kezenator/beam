use crate::ray::Ray;
use crate::vec::{Point3, Dir3};

pub struct Camera
{
}

impl Camera
{
    pub fn new() -> Self
    {
        Camera{}
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray
    {
        let x = 10.0 * u - 5.0;
        let y = 10.0 * v - 5.0;

        let source = Point3::new(x, y, 5.0);
        let dir = Dir3::new(0.0, 0.0, -1.0);

        Ray::new(source, dir)
    }
}