use crate::geom::{SampleableSurface, Volume};
use crate::vec::Point3;

pub struct LightingRegion
{
    pub covered_volume: Box<dyn Volume>,
    pub global_surfaces: Vec<Box<dyn SampleableSurface>>,
    pub local_points: Vec<Point3>,
}

impl LightingRegion
{
    pub fn new_2<V, S1, S2>(covered_volume: V, global_surface_1: S1, global_surface_2: S2, local_points: Vec<Point3>) -> Self
        where V: Volume + 'static,
            S1: SampleableSurface + 'static,
            S2: SampleableSurface + 'static,
    {
        LightingRegion
        {
            covered_volume: Box::new(covered_volume),
            global_surfaces: vec![Box::new(global_surface_1), Box::new(global_surface_2)],
            local_points: local_points,
        }
    }
}
