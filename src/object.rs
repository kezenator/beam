use crate::math::{Float, EPSILON};
use crate::vec::Point3;
use crate::color::RGBA;
use crate::intersection::Intersection;
use crate::ray::Ray;

pub struct Sphere
{
    centre: Point3,
    radius: Float,
    color: RGBA,
}

impl Sphere
{
    pub fn new(centre: Point3, radius: Float, color: RGBA) -> Self
    {
        Sphere { centre, radius, color }
    }

    pub fn get_intersections(&self, ray: &Ray, intersections: &mut Vec<Intersection>)
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

        intersections.push(Intersection::new(distance, location, normal, self.color.clone()));
    }
}
