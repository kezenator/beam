use crate::ray::Ray;
use crate::intersection::SurfaceIntersectionCollector;

pub mod plane;
pub mod sphere;

pub use plane::Plane;
pub use sphere::Sphere;

pub trait Surface
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>);
}
