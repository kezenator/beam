use crate::vec::{Dir3, Point3};
use crate::geom::{Surface, BoundingSurface, Volume};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

pub struct AABB
{
    min: Point3,
    max: Point3,
}

impl AABB
{
    pub fn new(min: Point3, max: Point3) -> Self
    {
        AABB { min, max }
    }

    pub fn union(&self, other: &AABB) -> Self
    {
        AABB
        {
            min: Point3::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y), self.min.y.min(other.min.y)),
            max: Point3::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y), self.max.y.max(other.max.y)),
        }
    }
}

impl Surface for AABB
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        // First, find the possible intersections in range, using algorithm from:
        // From https://tavianator.com/2011/ray_box.html accessed 2021-04-14

        let mut tmin = range.min();
        let mut tmax = range.max();

        let r_d_inv_x = 1.0 / ray.dir.x;
        let tx1 = (self.min.x - ray.source.x)*r_d_inv_x;
        let tx2 = (self.max.x - ray.source.x)*r_d_inv_x;

        tmin = tmin.max(tx1.min(tx2));
        tmax = tmax.min(tx1.max(tx2));

        let r_d_inv_y = 1.0 / ray.dir.y;
        let ty1 = (self.min.y - ray.source.y)*r_d_inv_y;
        let ty2 = (self.max.y - ray.source.y)*r_d_inv_y;

        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));

        let r_d_inv_z = 1.0 / ray.dir.z;
        let tz1 = (self.min.z - ray.source.z)*r_d_inv_z;
        let tz2 = (self.max.z - ray.source.z)*r_d_inv_z;

        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));

        if tmin < tmax
        {
            // There is a valid intersection at tmin - lets see which one it is

            if tmin == tx1 { return Some(ray.new_intersection(tx1, Dir3::new(-1.0, 0.0, 0.0))); }
            if tmin == tx2 { return Some(ray.new_intersection(tx2, Dir3::new(1.0, 0.0, 0.0))); }
            if tmin == ty1 { return Some(ray.new_intersection(ty1, Dir3::new(0.0, -1.0, 0.0))); }
            if tmin == ty2 { return Some(ray.new_intersection(ty2, Dir3::new(0.0, 1.0, 0.0))); }
            if tmin == tz1 { return Some(ray.new_intersection(tz1, Dir3::new(0.0, 0.0, -1.0))); }
            if tmin == tz2 { return Some(ray.new_intersection(tz2, Dir3::new(0.0, 0.0, 1.0))); }
        }

        None
    }
}

impl BoundingSurface for AABB
{
    fn may_intersect_in_range(&self, ray: &Ray, range: &RayRange) -> bool
    {
        // From https://tavianator.com/2011/ray_box.html accessed 2021-04-14

        let mut tmin = range.min();
        let mut tmax = range.max();

        {
            let r_d_inv_x = 1.0 / ray.dir.x;
            let tx1 = (self.min.x - ray.source.x)*r_d_inv_x;
            let tx2 = (self.max.x - ray.source.x)*r_d_inv_x;

            tmin = tmin.max(tx1.min(tx2));
            tmax = tmax.min(tx1.max(tx2));
        }

        {
            let r_d_inv_y = 1.0 / ray.dir.y;
            let ty1 = (self.min.y - ray.source.y)*r_d_inv_y;
            let ty2 = (self.max.y - ray.source.y)*r_d_inv_y;

            tmin = tmin.max(ty1.min(ty2));
            tmax = tmax.min(ty1.max(ty2));
        }

        {
            let r_d_inv_z = 1.0 / ray.dir.z;
            let tz1 = (self.min.z - ray.source.z)*r_d_inv_z;
            let tz2 = (self.max.z - ray.source.z)*r_d_inv_z;

            tmin = tmin.max(tz1.min(tz2));
            tmax = tmax.min(tz1.max(tz2));
        }

        return tmax >= tmin;
    }
}

impl Volume for AABB
{
    fn is_point_inside(&self, point: Point3) -> bool
    {
        self.min.x < point.x && point.x < self.max.x
            && self.min.y < point.y && point.y < self.max.y
            && self.min.z < point.z && point.z < self.max.z
    }
}