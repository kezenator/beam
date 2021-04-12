use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};
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
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>;
}

pub trait BoundingSurface: Surface
{
    fn enters_bounds(&self, ray: &Ray) -> bool;
}

pub trait Volume: Surface
{
    fn is_point_inside(&self, point: Point3) -> bool;
}
