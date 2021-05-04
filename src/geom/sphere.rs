use crate::geom::{BoundingSurface, Disc, SampleableSurface, Surface, Volume};
use crate::intersection::SurfaceIntersection;
use crate::math::{Scalar, ScalarConsts};
use crate::ray::{Ray, RayRange};
use crate::sample::Sampler;
use crate::vec::{Dir3, Point3};

#[derive(Clone)]
pub struct Sphere
{
    center: Point3,
    radius: Scalar,
}

impl Sphere
{
    pub fn new(center: Point3, radius: Scalar) -> Self
    {
        Sphere { center, radius }
    }

    pub fn get_projected_disk_as_seen_from(&self, location: Point3) -> Option<Disc>
    {
        // Work out the direction from the location to the center

        let loc_to_center = self.center - location;

        // Construct a right-triangle that is tangent to the
        // sphere.
        // Hypotenuse = distance(location to center)
        // Opposite = radius
        // => Adjacent = sqrt(dist^2 - radius^2)

        let a_squared = loc_to_center.magnitude_squared() - (self.radius * self.radius);

        if a_squared <= 0.0
        {
            // The location must be inside the sphere -
            // there is no protected disc

            None
        }
        else
        {
            let a = a_squared.sqrt();

            // Now, construct a line segment that's perpenticular
            // to the location->center line, and touches the tangent
            // point. This will be the radius of the disc.
            //
            // Theta = adjacent angle
            //       = atan r / a
            // disc_radius = r cos theta

            let theta = (self.radius / a).atan();
            let cos_theta = theta.cos();

            if cos_theta <= 0.0
            {
                // Numerical error - perhaps the point is too close
                // to the disc - we'll just say there's none

                None
            }
            else
            {
                let disc_radius = self.radius * cos_theta;
                
                // Now finally - where is the disc.
                // Construct another right triangle where
                // Hypotenuse = a
                // Opposite = b
                //
                // This means Adjacent = a cos theta
                // which is the distance from the location to the
                // center of the disc

                let dist_to_center = a * cos_theta;

                let loc_to_center_normalized = loc_to_center.normalized();

                let disc_center = location + (dist_to_center * loc_to_center_normalized);
                let normal = -loc_to_center_normalized;

                Some(Disc::new(disc_center, normal, disc_radius))
            }
        }
    }
}

impl Surface for Sphere
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        let oc = ray.source - self.center;
        let a = ray.dir.magnitude_squared();
        let half_b = oc.dot(ray.dir);
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
            let normal = (location - self.center) / self.radius;
    
            return Some(ray.new_intersection(distance, normal));
        }

        let distance = (-half_b + sqrtd) / a;

        if range.contains(distance)
        {
            let location = ray.source + (distance * ray.dir);
            let normal = (location - self.center) / self.radius;
    
            return Some(ray.new_intersection(distance, normal));
        }

        None
    }
}

impl BoundingSurface for Sphere
{
    fn may_intersect_in_range(&self, ray: &Ray, range: &RayRange) -> bool
    {
        let oc = ray.source - self.center;
        let a = ray.dir.magnitude_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.magnitude_squared() - (self.radius * self.radius);

        let discriminant = half_b*half_b - a*c;

        if discriminant > 0.0
        {
            let sqrtd = discriminant.sqrt();

            let d1 = (-half_b - sqrtd) / a;
            let d2 = (-half_b + sqrtd) / a;

            if range.intersection(&RayRange::new(d1, d2)).is_some()
            {
                return true;
            }
        }

        false
    }
}

impl Volume for Sphere
{
    fn is_point_inside(&self, point: Point3) -> bool
    {
        let dist_squared = (point - self.center).magnitude_squared();
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

impl SampleableSurface for Sphere
{
    fn generate_random_sample_direction_from_and_calc_pdf(&self, location: Point3, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        match self.get_projected_disk_as_seen_from(location)
        {
            Some(disc) =>
            {
                disc.generate_random_sample_direction_from_and_calc_pdf(location, sampler)
            },
            None =>
            {
                // The location is inside the sphere - we should
                // sample in any random direction, which all have
                // proability 1/4PI

                (sampler.uniform_dir_on_unit_sphere(), 0.25 * ScalarConsts::FRAC_1_PI)
            }
        }
    }

    fn calculate_pdf_for_ray(&self, ray: &Ray) -> Scalar
    {
        match self.get_projected_disk_as_seen_from(ray.source)
        {
            Some(disc) =>
            {
                disc.calculate_pdf_for_ray(ray)
            },
            None =>
            {
                // The location is inside the sphere - all directions
                // are equally likely with probability 1/4PI

                0.25 * ScalarConsts::FRAC_1_PI
            }
        }
    }
}
