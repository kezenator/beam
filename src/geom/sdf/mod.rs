use crate::intersection::SurfaceIntersection;
use crate::math::Scalar;
use crate::geom::{Surface, Volume};
use crate::ray::{Ray, RayRange};
use crate::vec::{Dir3, Point3};

#[derive(Clone, Debug)]
pub enum Sdf
{
    Sphere{ center: Point3, radius: Scalar },
    Capsule{ a: Point3, b: Point3, radius: Scalar },
    Union{ members: Vec<Sdf> },
    Annular{ sdf: Box<Sdf>, radius: Scalar },
}

impl Sdf
{
    pub fn distance(&self, pos: Point3) -> Scalar
    {
        match self
        {
            Sdf::Sphere{center, radius} =>
            {
                let o = pos - center;
                let d = o.magnitude();

                d - radius
            },
            Sdf::Capsule{a, b, radius} =>
            {
                // https://iquilezles.org/www/articles/distgradfunctions2d/distgradfunctions2d.htm

                let ba = b - a;
                let pa = pos - a;
                let h =(pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
                let q = pa - h * ba;
                let d = q.magnitude();

                d - radius
            },
            Sdf::Union{ members } =>
            {
                members.iter()
                    .map(|m| m.distance(pos))
                    .fold(Scalar::INFINITY, Scalar::min)
            },
            Sdf::Annular{ sdf, radius } =>
            {
                sdf.distance(pos).abs() - radius
            },
        }
    }

    pub fn normal(&self, pos: Point3) -> Dir3
    {
        match self
        {
            Sdf::Sphere{center, ..} =>
            {
                let o = pos - center;
                let d = o.magnitude();

                o / d
            },
            Sdf::Capsule{a, b, ..} =>
            {
                // https://iquilezles.org/www/articles/distgradfunctions2d/distgradfunctions2d.htm

                let ba = b - a;
                let pa = pos - a;
                let h =(pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
                let q = pa - h * ba;
                let d = q.magnitude();

                q / d
            },
            Sdf::Union{ members } =>
            {
                let mut distance = Scalar::MAX;
                let mut normal = Dir3::new(1.0, 0.0, 0.0);

                for m in members.iter()
                {
                    let m_dist = m.distance(pos);

                    if m_dist < distance
                    {
                        distance = m_dist;
                        normal = m.normal(pos);
                    }
                }

                normal
            },
            Sdf::Annular{ sdf, .. } =>
            {
                let distance = sdf.distance(pos);
                let normal = sdf.normal(pos);

                distance.signum() * normal
            },
        }
    }
}

impl Surface for Sdf
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        // First, calculcate the range into real distance

        let ray_dir_mag = ray.dir.magnitude();
        let ray_dir_mag_recip = ray_dir_mag.recip();
        let euc_range = RayRange::new(range.min() * ray_dir_mag, range.max() * ray_dir_mag);
        let euc_ray = Ray::new(ray.source, ray.dir * ray_dir_mag_recip);

        // March until we find an intersection

        let mut cur_euc_ray_time = euc_range.min();
        let mut cur_pos = euc_ray.point_at(cur_euc_ray_time);
        let mut cur_dist = self.distance(cur_pos);
        let orig_sign = cur_dist.signum();
        let mut factor = 0.99;

        for _ in 0..500
        {
            let next_euc_ray_time = cur_euc_ray_time + (factor * orig_sign * cur_dist);
            let next_pos = euc_ray.point_at(next_euc_ray_time);
            let next_dist = self.distance(next_pos);

            if next_dist.signum() != orig_sign
            {
                factor *= 0.825;
                continue;
            }

            if next_dist.abs() < 1e-12
            {
                return Some(ray.new_intersection_at(next_euc_ray_time * ray_dir_mag, next_pos, self.normal(next_pos)));
            }

            cur_euc_ray_time = next_euc_ray_time;
            cur_pos = next_pos;
            cur_dist = next_dist;
            factor = (factor * 1.99).clamp(0.01, 0.99);
        }

        None
    }
}

impl Volume for Sdf
{
    fn is_point_inside(&self, point: Point3) -> bool
    {
        self.distance(point) < 0.0
    }
}
