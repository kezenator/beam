use crate::geom::{BoundingSurface, Surface, SurfaceIntersection};
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
pub struct BoundedSurface<B: BoundingSurface + Clone + 'static, S: Surface + Clone + 'static>
{
    bounds: B,
    surface: S,
}

impl<B: BoundingSurface + Clone + 'static, S: Surface + Clone + 'static> BoundedSurface<B, S>
{
    pub fn new(bounds: B, surface: S) -> Self
    {
        BoundedSurface{ bounds, surface }
    }
}

impl<B: BoundingSurface + Clone + 'static, S: Surface + Clone + 'static> Surface for BoundedSurface<B, S>
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        if self.bounds.may_intersect_in_range(ray, range)
        {
            self.surface.closest_intersection_in_range(ray, range)
        }
        else
        {
            None
        }
    }
}
