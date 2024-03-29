use crate::color::LinearRGB;
use crate::math::EPSILON;
use crate::vec::{Point3, Mat4};
use crate::geom::{Aabb, AabbBoundedSurface, Surface};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct Triangle
{
    pub p0: Point3,
    pub p1: Point3,
    pub p2: Point3,
    pub t0: Point3,
    pub t1: Point3,
    pub t2: Point3,
    pub opt_colors: Option<[LinearRGB;3]>,
}

impl Triangle
{
    pub fn new(p0: Point3, p1: Point3, p2: Point3, t0: Point3, t1: Point3, t2: Point3, opt_colors: Option<[LinearRGB;3]>) -> Self
    {
        Triangle { p0, p1, p2, t0, t1, t2, opt_colors }
    }

    pub fn transformed(&self, matrix: &Mat4) -> Self
    {
        Triangle
        {
            p0: matrix.mul_point(self.p0),
            p1: matrix.mul_point(self.p1),
            p2: matrix.mul_point(self.p2),
            t0: self.t0,
            t1: self.t1,
            t2: self.t2,
            opt_colors: self.opt_colors,
        }
    }
}

impl AabbBoundedSurface for Triangle
{
    fn get_bounding_aabb(&self) -> Aabb
    {
        Aabb::new(
            Point3::new(self.p0.x.min(self.p1.x.min(self.p2.x)), self.p0.y.min(self.p1.y.min(self.p2.y)), self.p0.z.min(self.p1.z.min(self.p2.z))),
            Point3::new(self.p0.x.max(self.p1.x.max(self.p2.x)), self.p0.y.max(self.p1.y.max(self.p2.y)), self.p0.z.max(self.p1.z.max(self.p2.z))))
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
            let w = 1.0 - u - v;

            let texture_coords =
                self.t0 * w
                + self.t1 * u
                + self.t2 * v;

            let opt_color = self.opt_colors.map(|vertex_colors|
            {
                vertex_colors[0].multiplied_by_scalar_inc_alpha(w)
                    + vertex_colors[1].multiplied_by_scalar_inc_alpha(u)
                    + vertex_colors[2].multiplied_by_scalar_inc_alpha(v)
            });

            return Some(ray.new_intersection_with_texture_coords(
                t,
                edge1.cross(edge2).normalized(),
                texture_coords,
                opt_color
            ));
        }

        return None;
    }
}
