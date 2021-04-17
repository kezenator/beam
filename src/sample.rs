use crate::math::{EPSILON, Scalar};
use crate::vec::{Dir3, Point3};

use rand::{thread_rng, Rng, RngCore, SeedableRng};

pub struct Sampler
{
    rng: rand::rngs::SmallRng,
    dist_uniform_scalar_unit: rand::distributions::Uniform<Scalar>,
}

impl Sampler
{
    pub fn new() -> Self
    {
        Sampler
        {
            rng: rand::rngs::SmallRng::seed_from_u64(thread_rng().next_u64()),
            dist_uniform_scalar_unit: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }

    pub fn new_reproducable(seed: u64) -> Self
    {
        Sampler
        {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
            dist_uniform_scalar_unit: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }

    pub fn uniform_index(&mut self, len: usize) -> usize
    {
        (self.rng.next_u64() % (len as u64)) as usize
    }

    pub fn uniform_scalar_unit(&mut self) -> Scalar
    {
        self.rng.sample(self.dist_uniform_scalar_unit)
    }

    pub fn uniform_point_in_unit_sphere(&mut self) -> Point3
    {
        loop
        {
            let x = -1.0 + 2.0 * self.uniform_scalar_unit();
            let y = -1.0 + 2.0 * self.uniform_scalar_unit();
            let z = -1.0 + 2.0 * self.uniform_scalar_unit();

            let dir = Dir3::new(x, y, z);

            let mag_squared = dir.magnitude_squared();

            if mag_squared >= EPSILON && mag_squared <= 1.0
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
