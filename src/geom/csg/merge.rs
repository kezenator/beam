use crate::geom::{Surface, SurfaceIntersection};
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
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
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        let mut range = range.clone();
        let mut closest = None;

        for s in self.surfaces.iter()
        {
            if let Some(intersection) = s.closest_intersection_in_range(ray, &range)
            {
                range.set_max(intersection.distance);
                closest = Some(intersection);
            }
        }

        closest
    }
}
