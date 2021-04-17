use crate::math::{Scalar, ScalarConsts};
use crate::sample::Sampler;
use crate::vec::Dir3;

pub trait Bsdf
{
    fn generate_random_sample_direction_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar);
    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar;
}

pub struct Lambertian
{
    normal: Dir3,
}

impl Lambertian
{
    pub fn new(normal: Dir3) -> Self
    {
        Lambertian { normal }
    }
}

impl Bsdf for Lambertian
{
    fn generate_random_sample_direction_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        // First, generate an ONB from the normal

        let u = if self.normal.x.abs() > 0.9 { Dir3::new(0.0, 1.0, 0.0) } else { Dir3::new(1.0, 0.0, 0.0) };
        let v = self.normal.cross(u).normalized();
        let u = self.normal.cross(v);

        // Generate an angle from two random variables

        let r1 = sampler.uniform_scalar_unit();
        let r2 = sampler.uniform_scalar_unit();

        let z = (1.0 - r2).sqrt();

        let phi = 2.0 * ScalarConsts::PI * r1;
        let xy_factor = r2.sqrt();

        let x = phi.cos() * xy_factor;
        let y = phi.sin() * xy_factor;

        // Calculate the direction, relative to the ONB

        let dir = (x * u) + (y * v) + (z * self.normal);

        // Calculate the PDF

        let pdf = self.normal.dot(dir);

        (dir, pdf)
    }

    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar
    {
        let cosine = self.normal.dot(dir.normalized());

        if cosine >= 0.0
        {
            cosine / ScalarConsts::PI
        }
        else
        {
            0.0
        }
    }
}
