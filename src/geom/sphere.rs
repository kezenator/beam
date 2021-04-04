use crate::math::{Scalar, EPSILON};
use crate::vec::Point3;
use crate::geom::Surface;
use crate::intersection::{SurfaceIntersection, SurfaceIntersectionCollector};
use crate::ray::Ray;

pub struct Sphere
{
    centre: Point3,
    radius: Scalar,
}

impl Sphere
{
    pub fn new(centre: Point3, radius: Scalar) -> Self
    {
        Sphere { centre, radius }
    }
}

impl Surface for Sphere
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>)
    {
        let oc = ray.source - self.centre;
        let a = ray.dir.magnitude_squared();
        let half_b = oc.dot(ray.dir.clone());
        let c = oc.magnitude_squared() - (self.radius * self.radius);

        let discriminant = half_b*half_b - a*c;
        if discriminant < 0.0
        {
            return;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < EPSILON
        {
            root = (-half_b + sqrtd) / a;
            if root < EPSILON
            {
                return;
            }
        }

        let distance = root;
        let location = ray.source + (root * ray.dir);
        let normal = (location - self.centre) / self.radius;

        collect(SurfaceIntersection { ray, distance, normal });
    }
}
