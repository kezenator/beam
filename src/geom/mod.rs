use crate::intersection::SurfaceIntersectionCollector;
use crate::ray::Ray;
use crate::vec::Point3;

pub mod blob;
pub mod bounds;
pub mod csg;
pub mod plane;
pub mod rectangle;
pub mod sphere;

pub use blob::{Blob, BlobPart};
pub use bounds::BoundedSurface;
pub use plane::Plane;
pub use rectangle::Rectangle;
pub use sphere::Sphere;

pub trait Surface
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>);
}

pub trait BoundingSurface: Surface
{
    fn enters_bounds(&self, ray: &Ray) -> bool;
}

pub trait Volume: Surface
{
    fn is_point_inside(&self, point: Point3) -> bool;
}
