use crate::geom::{SampleableSurface, Volume};
use crate::vec::Point3;

#[derive(Clone)]
pub struct LightingRegion
{
    pub covered_volume: Box<dyn Volume>,
    pub global_surfaces: Vec<Box<dyn SampleableSurface>>,
    pub local_points: Vec<Point3>,
}

impl LightingRegion
{
    pub fn new<V: Volume + 'static>(covered_volume: V) -> Self
    {
        LightingRegion
        {
            covered_volume: Box::new(covered_volume),
            global_surfaces: Vec::new(),
            local_points: Vec::new(),
        }
    }

    pub fn new_1<V, S1>(covered_volume: V, global_surface_1: S1, local_points: Vec<Point3>) -> Self
        where V: Volume + 'static,
            S1: SampleableSurface + 'static,
    {
        LightingRegion
        {
            covered_volume: Box::new(covered_volume),
            global_surfaces: vec![Box::new(global_surface_1)],
            local_points: local_points,
        }
    }

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
