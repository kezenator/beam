use crate::math::Scalar;
use crate::vec::{Dir3, Point3};

use rand::{thread_rng, Rng};

pub struct Sampler
{
    rng: rand::prelude::ThreadRng,
    dist_uniform_scalar_unit: rand::distributions::Uniform<Scalar>,
}

impl Sampler
{
    pub fn new() -> Self
    {
        Sampler
        {
            rng: thread_rng(),
            dist_uniform_scalar_unit: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }

    pub fn uniform_scalar_unit(&mut self) -> Scalar
    {
        self.rng.sample(self.dist_uniform_scalar_unit)
    }

    pub fn uniform_point_in_unit_sphere(&mut self) -> Point3
    {
        loop
        {
            let x = self.uniform_scalar_unit();
            let y = self.uniform_scalar_unit();
            let z = self.uniform_scalar_unit();

            let dir = Dir3::new(x, y, z);

            if dir.magnitude_squared() < 1.0
            {
                return dir;
            }
        }
    }

    pub fn uniform_dir_on_unit_sphere(&mut self) -> Dir3
    {
        self.uniform_point_in_unit_sphere().normalized()
    }
}
