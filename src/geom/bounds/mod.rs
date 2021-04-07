use crate::geom::{BoundingSurface, Surface, SurfaceIntersectionCollector};
use crate::ray::Ray;

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
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collect: &'c mut SurfaceIntersectionCollector<'r, 'c>)
    {
        if self.bounds.enters_bounds(ray)
        {
            self.surface.get_intersections(ray, collect);
        }
    }
}