use crate::geom::{Surface, SurfaceIntersectionCollector};
use crate::ray::Ray;

pub struct Merge
{
    surfaces: Vec<Box<dyn Surface>>,
}

impl Merge
{
    pub fn new() -> Self
    {
        Merge { surfaces: Vec::new() }
    }

    pub fn push<S: Surface + 'static>(&mut self, s: S)
    {
        self.surfaces.push(Box::new(s));
    }
}

impl Surface for Merge
{
    fn get_intersections<'r, 'c>(&self, ray: &'r Ray, collector: &'c mut SurfaceIntersectionCollector<'r, 'c>)
    {
        for s in self.surfaces.iter()
        {
            s.get_intersections(ray, collector);
        }
    }
}
