use crate::math::Scalar;
use crate::ray::Ray;
use crate::vec::{Point3, Dir3};

#[derive(Clone)]
pub struct Camera
{
    location: Point3,
    lower_left_corner: Point3,
    horizontal: Dir3,
    vertical: Dir3,
}

impl Camera
{
    pub fn new(location: Point3, look_at: Point3, up: Point3, fov: Scalar, aspect_ratio: Scalar) -> Self
    {
        let theta = fov.to_radians();
        let w = (theta / 2.0).tan();
        let viewport_width = 2.0 * w;
        let viewport_height = viewport_width / aspect_ratio;

        let w = (location - look_at).normalized();
        let u = up.cross(w).normalized();
        let v = w.cross(u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * -v;
        let lower_left_corner = location - (horizontal / 2.0) - (vertical / 2.0) - w;

        Camera { location, lower_left_corner, horizontal, vertical }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray
    {
        Ray::new(
            self.location,
            (self.lower_left_corner + (self.horizontal * u) + (self.vertical * v)) - self.location)
    }
}