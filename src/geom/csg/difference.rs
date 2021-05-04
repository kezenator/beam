use crate::geom::{Volume, Surface, SurfaceIntersection};
use crate::math::EPSILON;
use crate::ray::{Ray, RayRange};
use crate::vec::Point3;

#[derive(Clone)]
pub struct Difference<A: Clone, B: Clone>
{
    a: A,
    b: B,
}

impl<A: Volume + Clone + 'static, B: Volume + Clone + 'static> Difference<A, B>
{
    pub fn new(a: A, b: B) -> Self
    {
        Difference { a, b }
    }
}

impl<A: Volume + Clone + 'static, B: Volume + Clone + 'static> Surface for Difference<A, B>
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        let mut range = range.clone();

        while !range.is_empty()
        {
            let int_a = self.a.closest_intersection_in_range(ray, &range);
            let int_b = self.b.closest_intersection_in_range(ray, &range);

            match (int_a, int_b)
            {
                (Some(a), Some(b)) =>
                {
                    // Select intersections that meet the difference op:
                    // A while not in B, B while in A
                    // Use range.max() as not valid

                    let da = if !self.b.is_point_inside(ray.point_at(a.distance)) { a.distance } else { range.max() };
                    let db = if self.a.is_point_inside(ray.point_at(b.distance)) { b.distance } else { range.max() };

                    let min = da.min(db);

                    if min == a.distance
                    {
                        return Some(a);
                    }
                    else if min == b.distance
                    {
                        return Some(b);
                    }
                    else
                    {
                        range.set_min(a.distance.max(b.distance) + EPSILON);
                    }
                },
                (Some(a), None) =>
                {
                    // Intersect with only A - return it if
                    // it's NOT inside B, else search again after
                    // this distance

                    if !self.b.is_point_inside(ray.point_at(a.distance))
                    {
                        return Some(a);
                    }

                    range.set_min(a.distance + EPSILON);
                },
                (None, Some(b)) =>
                {
                    // Intersect with only B - return it if
                    // it's inside A, else search
                    // again after this distance

                    if self.a.is_point_inside(ray.point_at(b.distance))
                    {
                        return Some(b);
                    }

                    range.set_min(b.distance + EPSILON);
                },
                (None, None) =>
                {
                    // No more intersections with either object
                    return None;
                }
            }
        }

        // We may have more intersections, but they
        // are beyon the range we have been asked for

        None
    }
}

impl<A: Volume + Clone + 'static, B: Volume + Clone + 'static> Volume for Difference<A, B>
{
    fn is_point_inside(&self, point: Point3) -> bool
    {
        self.a.is_point_inside(point) && !self.b.is_point_inside(point)
    }
}