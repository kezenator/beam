use crate::ray::Ray;
use crate::vec::{Point3, Dir3};

pub struct Camera
{
    location: Point3,
    lower_left_corner: Point3,
    horizontal: Dir3,
    vertical: Dir3,
}

impl Camera
{
    pub fn new(width: u32, height: u32) -> Self
    {
        let location = Point3::new(-3.0, 12.0, 12.0);
        let look_at = Point3::new(0.0, -1.0, 0.0);
        let up = Point3::new(0.0, 1.0, 0.0);
        let fov = 40.0;
        let aspect_ratio = (width as f64) / (height as f64);

        let theta = crate::math::degrees_to_radians(fov);
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