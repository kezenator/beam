use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};
use crate::geom::Surface;
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

pub struct Rectangle
{
    point: Point3,
    normal: Dir3,
    dir_u: Dir3,
    dir_v: Dir3,
    len_u: Scalar,
    len_v: Scalar,
}

impl Rectangle
{
    pub fn new(point: Point3, u: Dir3, v: Dir3) -> Self
    {
        let len_u = u.magnitude();
        let len_v = v.magnitude();
        let normal = u.cross(v).normalized();
        let dir_v = normal.cross(u).normalized();
        let dir_u = dir_v.cross(normal).normalized();

        Rectangle { point, normal, dir_u, dir_v, len_u, len_v }
    }
}

impl Surface for Rectangle
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
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

            // Work out if an intersection at this distance is wanted

            if range.contains(distance)
            {
                // Now, work out if this is in the rectangle
                
                let int_offset = ray.point_at(distance) - self.point;
                let int_u = int_offset.dot(self.dir_u);
                let int_v = int_offset.dot(self.dir_v);

                if int_u >= 0.0 && int_u <= self.len_u && int_v >= 0.0 && int_v <= self.len_v
                {
                    let normal = self.normal.clone();

                    return Some(ray.new_intersection(distance, normal))
                }
            }
        }

        None
    }
}
