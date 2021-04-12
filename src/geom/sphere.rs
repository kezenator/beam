use crate::math::Scalar;
use crate::vec::Point3;
use crate::geom::{Surface, BoundingSurface, Volume};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

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
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        let oc = ray.source - self.centre;
        let a = ray.dir.magnitude_squared();
        let half_b = oc.dot(ray.dir.clone());
        let c = oc.magnitude_squared() - (self.radius * self.radius);

        let discriminant = half_b*half_b - a*c;
        if discriminant <= 0.0
        {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let distance = (-half_b - sqrtd) / a;

        if range.contains(distance)
        {
            let location = ray.source + (distance * ray.dir);
            let normal = (location - self.centre) / self.radius;
    
            return Some(ray.new_intersection(distance, normal));
        }

        let distance = (-half_b + sqrtd) / a;

        if range.contains(distance)
        {
            let location = ray.source + (distance * ray.dir);
            let normal = (location - self.centre) / self.radius;
    
            return Some(ray.new_intersection(distance, normal));
        }

        None
    }
}

impl BoundingSurface for Sphere
{
    fn enters_bounds(&self, ray: &Ray) -> bool
    {
        let oc = ray.source - self.centre;
        let a = ray.dir.magnitude_squared();
        let half_b = oc.dot(ray.dir.clone());
        let c = oc.magnitude_squared() - (self.radius * self.radius);

        let discriminant = half_b*half_b - a*c;

        discriminant > 0.0
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