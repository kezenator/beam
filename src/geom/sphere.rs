use crate::math::Scalar;
use crate::vec::Point3;
use crate::geom::{Surface, Volume};
use crate::intersection::SurfaceIntersectionCollector;
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
        if discriminant <= 0.0
        {
            return;
        }

        let sqrtd = discriminant.sqrt();

        let mut add_intersection = move |distance: Scalar|
        {
            let location = ray.source + (distance * ray.dir);
            let normal = (location - self.centre) / self.radius;
    
            collect((*ray).new_intersection(distance, normal));
        };

        add_intersection((-half_b - sqrtd) / a);
        add_intersection((-half_b + sqrtd) / a);
    }
}

impl Volume for Sphere
{
    fn is_point_inside(&self, point: Point3) -> bool
    {
        let dist_squared = (point - self.centre).magnitude_squared();
        let radius_squared = self.radius * self.radius;

        if self.radius >= 0.0
        {
            dist_squared <= radius_squared
        }
        else
        {
            dist_squared > radius_squared
        }
    }
}