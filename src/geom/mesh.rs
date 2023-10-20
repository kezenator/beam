use crate::vec::Point3;
use crate::geom::{Aabb, BoundingSurface, Surface, Triangle};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct Mesh
{
    bounds: Option<Aabb>,
    triangles: Vec<Triangle>,
}

impl Mesh
{
    pub fn new(triangles: Vec<Triangle>) -> Self
    {
        let bounds = if triangles.is_empty()
        {
            None
        }
        else
        {
            let mut min = triangles[0].p0;
            let mut max = triangles[0].p0;

            for tri in triangles.iter()
            {
                for p in [tri.p0, tri.p1, tri.p2]
                {
                    min = Point3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z));
                    max = Point3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z));
                }
            }

            Some(Aabb::new(min, max))
        };

        Mesh { bounds, triangles }
    }
}

impl Surface for Mesh
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        if let Some(bounds) = &self.bounds
        {
            if bounds.may_intersect_in_range(ray, range)
            {
                let mut range = range.clone();
                let mut closest = None;
        
                for tri in self.triangles.iter()
                {
                    if let Some(intersection) = tri.closest_intersection_in_range(ray, &range)
                    {
                        range.set_max(intersection.distance);
                        closest = Some(intersection);
                    }
                }
                return closest;
            }
        }
        
        return None;
    }
}
