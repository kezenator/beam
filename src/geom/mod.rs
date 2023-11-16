use crate::math::Scalar;
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};
use crate::vec::{Dir3, Point3};
use crate::sample::Sampler;

pub mod aabb;
pub mod blob;
pub mod bounds;
pub mod csg;
pub mod disc;
pub mod mesh;
pub mod octree;
pub mod plane;
pub mod rectangle;
pub mod sdf;
pub mod sphere;
pub mod triangle;

pub use aabb::{Aabb, AabbBuilder};
pub use blob::{Blob, BlobPart};
pub use bounds::BoundedSurface;
pub use disc::Disc;
pub use mesh::Mesh;
pub use octree::Octree;
pub use plane::Plane;
pub use rectangle::{OneWayRectangle, Rectangle};
pub use sdf::Sdf;
pub use sphere::Sphere;
pub use triangle::Triangle;

pub trait Surface: CloneableSurface + Send
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>;
}

pub trait BoundingSurface: Surface
{
    fn may_intersect_in_range(&self, ray: &Ray, range: &RayRange) -> bool;
}

pub trait AabbBoundedSurface: Surface
{
    fn get_bounding_aabb(&self) -> Aabb;
}

pub trait Volume: Surface + CloneableVolume
{
    fn is_point_inside(&self, point: Point3) -> bool;
}

pub trait SampleableSurface: Surface + CloneableSampleableSurface
{
    fn generate_random_sample_direction_from_and_calc_pdf(&self, location: Point3, sampler: &mut Sampler) -> (Dir3, Scalar);
    fn calculate_pdf_for_ray(&self, ray: &Ray) -> Scalar;
}

pub trait CloneableSurface
{
    fn clone_boxed_surface(&self) -> Box<dyn Surface>;
}

pub trait CloneableSampleableSurface
{
    fn clone_boxed_sampleable_surface(&self) -> Box<dyn SampleableSurface>;
}

pub trait CloneableVolume
{
    fn clone_boxed_volume(&self) -> Box<dyn Volume>;
}

impl Clone for Box<dyn Surface>
{
    fn clone(&self) -> Box<dyn Surface>
    {
        self.clone_boxed_surface()
    }
}

impl Clone for Box<dyn SampleableSurface>
{
    fn clone(&self) -> Box<dyn SampleableSurface>
    {
        self.clone_boxed_sampleable_surface()
    }
}

impl Clone for Box<dyn Volume>
{
    fn clone(&self) -> Box<dyn Volume>
    {
        self.clone_boxed_volume()
    }
}

impl<T> CloneableSurface for T
    where T: Surface + Clone + Sized + 'static
{
    fn clone_boxed_surface(&self) -> Box<dyn Surface>
    {
        Box::new(self.clone())
    }
}

impl<T> CloneableSampleableSurface for T
    where T: SampleableSurface + Clone + Sized + 'static
{
    fn clone_boxed_sampleable_surface(&self) -> Box<dyn SampleableSurface>
    {
        Box::new(self.clone())
    }
}

impl<T> CloneableVolume for T
    where T: Volume + Clone + Sized + 'static
{
    fn clone_boxed_volume(&self) -> Box<dyn Volume>
    {
        Box::new(self.clone())
    }
}
