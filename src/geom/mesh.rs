use crate::geom::{Octree, Surface, Triangle};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct Mesh
{
    octree: Octree<Triangle>,
}

impl Mesh
{
    pub fn new(triangles: Vec<Triangle>) -> Self
    {
        Mesh { octree: Octree::new(triangles, 10) }
    }
}

impl Surface for Mesh
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        self.octree.closest_intersection_in_range(ray, range)
    }
}
