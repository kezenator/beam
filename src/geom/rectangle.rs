use crate::geom::{SampleableSurface, Surface};
use crate::intersection::{Face, SurfaceIntersection};
use crate::math::{EPSILON, Scalar};
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::vec::{Dir3, Point3};

#[derive(Clone)]
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
                    let normal = self.normal;

                    return Some(ray.new_intersection(distance, normal))
                }
            }
        }

        None
    }
}

impl SampleableSurface for Rectangle
{
    fn generate_random_sample_direction_from_and_calc_pdf(&self, location: Point3, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        // First, calcualte a random point on the rectangle

        let r1 = sampler.uniform_scalar_unit();
        let r2 = sampler.uniform_scalar_unit();

        let rand_point_on_surface = self.point + ((r1 * self.len_u) * self.dir_u) + ((r2 * self.len_v) * self.dir_v);

        let dir_with_len = rand_point_on_surface - location;

        let dir_normalized = dir_with_len.normalized();

        // Now, repeat the PDF calculation (from below)

        let area = self.len_u * self.len_v;
        let distance_squared = dir_with_len.magnitude_squared();

        let cosine = dir_normalized.dot(self.normal).abs();

        let pdf = distance_squared / (cosine * area);

        (dir_normalized, pdf)
    }

    fn calculate_pdf_for_ray(&self, ray: &Ray) -> Scalar
    {
        match self.closest_intersection_in_range(ray, &RayRange::new(EPSILON, Scalar::MAX))
        {
            Some(intersection) =>
            {
                let area = self.len_u * self.len_v;
                let distance_squared = ray.dir.magnitude_squared() * intersection.distance * intersection.distance;
        
                let cosine = ray.dir.normalized().dot(self.normal).abs();
        
                distance_squared / (cosine * area)
            },
            None =>
            {
                // This ray doesn't hit the surface - there is
                // no chance that we will generate it

                0.0
            },
        }
    }
}

#[derive(Clone)]
pub struct OneWayRectangle
{
    rect: Rectangle,
}

impl OneWayRectangle
{
    pub fn new(point: Point3, u: Dir3, v: Dir3) -> Self
    {
        OneWayRectangle { rect: Rectangle::new(point, u, v) }
    }
}

impl Surface for OneWayRectangle
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        self.rect.closest_intersection_in_range(ray, range)
            .filter(|i| i.face == Face::Front)
    }
}