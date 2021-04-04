use crate::math::EPSILON;
use crate::vec::{Dir3, Point3};
use crate::geom::Surface;
use crate::intersection::{SurfaceIntersection, SurfaceIntersectionCollector};
use crate::ray::Ray;

pub struct Plane
{
    point: Point3,
    normal: Dir3,
}

impl Plane
{
    pub fn new(point: Point3, normal: Dir3) -> Self
    {
        Plane { point, normal }
    }
}

impl Surface for Plane
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>)
    {
        // When the ray intersection is on the plane, the dot
        // product of the location will be zero.
        // i.e. (ray.source + t * ray.dir - self.point) dot self.normal == 0
        // Solving for t gives:
        // t = ((self.point - ray.source) dot self.normal) / (ray.dir dot self.normal)

        let denom = ray.dir.dot(self.normal);

        if denom.abs() > EPSILON
        {
            let distance = (self.point - ray.source).dot(self.normal) / denom;
            let normal = self.normal.clone();

            collect(SurfaceIntersection{ ray, distance, normal });
        }
    }
}