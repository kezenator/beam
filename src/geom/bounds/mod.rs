use crate::geom::{BoundingSurface, Surface, SurfaceIntersection};
use crate::ray::{Ray, RayRange};

pub struct BoundedSurface<B: BoundingSurface, S: Surface>
{
    bounds: B,
    surface: S,
}

impl<B: BoundingSurface, S: Surface> BoundedSurface<B, S>
{
    pub fn new(bounds: B, surface: S) -> Self
    {
        BoundedSurface{ bounds, surface }
    }
}

impl<B: BoundingSurface, S: Surface> Surface for BoundedSurface<B, S>
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        if self.bounds.enters_bounds(ray)
        {
            self.surface.closest_intersection_in_range(ray, range)
        }
        else
        {
            None
        }
    }
}