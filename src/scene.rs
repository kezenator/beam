use crate::math::{EPSILON, Float};
use crate::vec::Point3;
use crate::color::RGBA;
use crate::camera::Camera;
use crate::object::Sphere;

pub struct Scene
{
    camera: Camera,
    objects: Vec<Sphere>,
}

impl Scene
{
    pub fn new(camera: Camera, objects: Vec<Sphere>) -> Self
    {
        Scene { camera, objects }
    }

    pub fn new_default() -> Self
    {
        Self::new(
            Camera::new(),
            vec![
                Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0, RGBA::new(1.0, 1.0, 1.0, 1.0)),
                Sphere::new(Point3::new(2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 1.0, 0.0, 1.0)),
                Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0, RGBA::new(0.0, 0.0, 1.0, 1.0)),
                Sphere::new(Point3::new(0.0, 2.0, 0.0), 1.0, RGBA::new(1.0, 0.0, 0.0, 1.0)),
                Sphere::new(Point3::new(0.0, -2.0, 0.0), 1.0, RGBA::new(0.0, 1.0, 1.0, 1.0)),
                Sphere::new(Point3::new(0.0, 0.0, -10.0), 5.0, RGBA::new(1.0, 1.0, 0.0, 1.0)),
            ])
    }

    pub fn sample_pixel(&self, u: Float, v: Float) -> RGBA
    {
        let ray = self.camera.get_ray(u, v);

        let mut intersections = Vec::new();

        for obj in self.objects.iter()
        {
            obj.get_intersections(&ray, &mut intersections);
        }

        let mut intersections = intersections
            .drain(..)
            .filter(|i| i.distance >= EPSILON)
            .collect::<Vec<_>>();

        intersections.sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        match intersections.iter().nth(0)
        {
            Some(intersection) => intersection.color.clone(),
            None => RGBA::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
