use float_ord::FloatOrd;

use crate::geom::{AABB, BoundedSurface, Surface, SurfaceIntersection};
use crate::math::{EPSILON, Scalar};
use crate::ray::{Ray, RayRange};
use crate::vec::{Dir3, Point3};

pub struct Blob
{
    parts: Vec<BlobPart>,
    threshold: Scalar,
}

pub struct BlobPart
{
    pub center: Point3,
    pub radius: Scalar,
}

impl Blob
{
    pub fn new(parts: Vec<BlobPart>, threshold: Scalar) -> BoundedSurface<AABB, Blob>
    {
        assert!(!parts.is_empty());

        let aabb = parts[0].to_aabb();

        let aabb = parts.iter().skip(1).fold(aabb, |aabb, p| aabb.union(&p.to_aabb()));

        BoundedSurface::new(
            aabb,
            Blob { parts, threshold })
    }

    fn create_intersections(&self, ray: &Ray) -> Vec<BlobPartIntersection>
    {
        let mut result = Vec::new();

        for (index, part) in self.parts.iter().enumerate()
        {
            // Standard sphere intersection code

            let oc = ray.source - part.center;
            let a = ray.dir.magnitude_squared();
            let half_b = oc.dot(ray.dir.clone());
            let c = oc.magnitude_squared() - (part.radius * part.radius);
    
            let discriminant = half_b*half_b - a*c;

            if discriminant > 0.0
            {
                let sqrtd = discriminant.sqrt();

                result.push(BlobPartIntersection{
                    index: index,
                    distance: (-half_b - sqrtd) / a,
                    event: BlobPartIntersectionEvent::Enter,
                });
        
                result.push(BlobPartIntersection{
                    index: index,
                    distance: -half_b / a,
                    event: BlobPartIntersectionEvent::DerivativeChanged,
                });
        
                result.push(BlobPartIntersection{
                    index: index,
                    distance: (-half_b + sqrtd) / a,
                    event: BlobPartIntersectionEvent::Exit,
                });
            }
        }

        result.sort_by(|a, b| FloatOrd(a.distance).cmp(&FloatOrd(b.distance)));

        result
    }

    fn value_at(&self, point: Point3, indexes: &Vec<usize>) -> Scalar
    {
        let mut result = 0.0;

        for index in indexes.iter()
        {
            result += self.parts[*index].weight_at(point);
        }

        result - self.threshold
    }

    fn normal_at(&self, point: Point3, indexes: &Vec<usize>) -> Dir3
    {
        let mut sum = Point3::new(0.0, 0.0, 0.0);

        for index in indexes.iter()
        {
            let part = &self.parts[*index];

            let normal = (point - part.center).normalized();

            sum = sum + part.weight_at(point) * normal;
        }

        sum.normalized()
    }
}

impl Surface for Blob
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        // First, get the interesting points along the ray

        let points = self.create_intersections(ray);

        if !points.is_empty()
        {
            let mut indexes = Vec::new();

            let mut cur_dist = points[0].distance;
            let mut cur_val = self.value_at(ray.point_at(cur_dist), &indexes);

            if points[0].event == BlobPartIntersectionEvent::Enter
            {
                indexes.push(points[0].index);
            }

            for point in points.iter().skip(1)
            {
                if point.event == BlobPartIntersectionEvent::Enter
                {
                    indexes.push(point.index);
                }

                let next_dist = point.distance;
                let next_val = self.value_at(ray.point_at(next_dist), &indexes);

                if cur_val.signum() != next_val.signum()
                    && range.intersection(&RayRange::new(cur_dist, next_dist)).is_some()
                {
                    // We've passed through zero between these points and
                    // this range of distances intersects requested ray range -
                    // Do a binary search for the location, and return if
                    // an intersection is found.

                    let mut a = cur_dist;
                    let mut b = next_dist;

                    while (b - a) > (EPSILON / 100.0)
                    {
                        let c = (a + b) / 2.0;
                        let cp = ray.point_at(c);
                        let cv = self.value_at(cp, &indexes);

                        if cur_val.signum() != cv.signum()
                        {
                            b = c;
                        }
                        else
                        {
                            a = c;
                        }
                    }

                    let distance = (a + b) / 2.0;

                    if range.contains(distance)
                    {
                        let normal = self.normal_at(ray.point_at(distance), &indexes);

                        return Some(ray.new_intersection(distance, normal));
                    }
                }

                if point.event == BlobPartIntersectionEvent::Exit
                {
                    indexes = indexes.drain(..).filter(|&i| i != point.index).collect();
                }

                cur_dist = next_dist;
                cur_val = next_val;
            }
        }

        None
    }
}

impl BlobPart
{
    fn to_aabb(&self) -> AABB
    {
        let rad = self.radius.abs();

        AABB::new(
            Point3::new(self.center.x - rad, self.center.y - rad, self.center.z - rad),
            Point3::new(self.center.x + rad, self.center.y + rad, self.center.z + rad))
    }

    fn weight_at(&self, point: Point3) -> Scalar
    {
        let distance = (point - self.center).magnitude() / self.radius;

        if distance > 1.0
        {
            0.0
        }
        else
        {
            let x = distance;
            let func = x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
            1.0 - func
        }
    }
}

#[derive(PartialEq, Eq)]
enum BlobPartIntersectionEvent
{
    Enter,
    DerivativeChanged,
    Exit,
}

struct BlobPartIntersection
{
    index: usize,
    distance: Scalar,
    event: BlobPartIntersectionEvent,
}