use crate::intersection::SurfaceIntersectionCollector;
use crate::ray::Ray;
use crate::vec::Point3;

pub mod plane;
pub mod sphere;

pub use plane::Plane;
pub use sphere::Sphere;

pub trait Surface
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>);
}

pub trait Volume: Surface
{
    fn is_point_inside(&self, point: Point3) -> bool;
}
