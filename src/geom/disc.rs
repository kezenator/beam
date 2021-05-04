use crate::geom::{SampleableSurface, Surface};
use crate::intersection::SurfaceIntersection;
use crate::math::{EPSILON, Scalar, ScalarConsts};
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::vec::{Dir3, Point3};

#[derive(Clone)]
pub struct Disc
{
    point: Point3,
    normal: Dir3,
    radius: Scalar,
}

impl Disc
{
    pub fn new(point: Point3, normal: Dir3, radius: Scalar) -> Self
    {
        Disc { point, normal, radius }
    }
}

impl Surface for Disc
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        // First - intersect with the plane the disc is in.
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
                // Now, work out if this is in the disc

                let int_offset = ray.point_at(distance) - self.point;

                let dist_squared = int_offset.magnitude_squared();

                if dist_squared < (self.radius * self.radius)
                {
                    let normal = self.normal;

                    return Some(ray.new_intersection(distance, normal))
                }
            }
        }

        None
    }
}

impl SampleableSurface for Disc
{
    fn generate_random_sample_direction_from_and_calc_pdf(&self, location: Point3, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        // First, calcualte a random point on the disc.
        // For this, we need a set of ortho-normal bases

        let u = if self.normal.x.abs() > 0.9 { Dir3::new(0.0, 1.0, 0.0) } else { Dir3::new(1.0, 0.0, 0.0) };
        let v = self.normal.cross(u).normalized();
        let u = self.normal.cross(v);

        // Now we can use the orthonormal base to calculate
        // a point on the disc

        let rad = sampler.uniform_scalar_unit().sqrt() * self.radius;
        let ang = sampler.uniform_scalar_unit() * 2.0 * ScalarConsts::PI;

        let cu = rad * ang.sin();
        let cv = rad * ang.cos();

        let rand_point_on_surface = self.point
            + (cu * u)
            + (cv * v);

        let dir_with_len = rand_point_on_surface - location;

        let dir_normalized = dir_with_len.normalized();

        // Now, repeat the PDF calculcation from below

        let area = ScalarConsts::PI * self.radius * self.radius;
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
                let area = ScalarConsts::PI * self.radius * self.radius;
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
