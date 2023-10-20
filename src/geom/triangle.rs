use crate::math::EPSILON;
use crate::vec::{Dir3, Point3};
use crate::geom::{Surface, Volume};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct Triangle
{
    pub p0: Point3,
    pub p1: Point3,
    pub p2: Point3,
}

impl Triangle
{
    pub fn new(p0: Point3, p1: Point3, p2: Point3) -> Self
    {
        Triangle { p0, p1, p2 }
    }
}

impl Surface for Triangle
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        // Möller–Trumbore intersection algorithm
        // Adapted from Wikipedia https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
        // on 2023-10-20

        let edge1 = self.p1 - self.p0;
        let edge2 = self.p2 - self.p0;

        let h = ray.dir.cross(edge2);
        let a = edge1.dot(h);

        if (a > -EPSILON) && (a < EPSILON)
        {
            // This ray is parallel to this triangle
            return None;
        }

        let f = 1.0 / a;
        let s = ray.source - self.p0;
        let u = f * s.dot(h);

        if (u < 0.0) || (u > 1.0)
        {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.dir.dot(q);

        if (v < 0.0) || ((u + v) > 1.0)
        {
            return None;
        }

        let t = f * edge2.dot(q);

        if range.contains(t)
        {
            return Some(ray.new_intersection(t, edge1.cross(edge2).normalized()));
        }

        return None;
    }
}