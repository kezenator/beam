use crate::math::{Scalar, ScalarConsts};
use crate::sample::Sampler;
use crate::vec::Dir3;

pub trait Bsdf
{
    fn generate_random_sample_direction_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar);
    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar;
    fn reflectance(&self, input_dir: Dir3, output_dir: Dir3) -> Scalar;
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

        // Generate two random variables

        let r1 = sampler.uniform_scalar_unit();
        let r2 = sampler.uniform_scalar_unit();

        // Convert these to x/y/z parameters
        // TODO - for now we're just uniform sampling...

        let x = (1.0 - r1 * r1).sqrt() * (2.0 * ScalarConsts::PI * r2).cos();
        let y = (1.0 - r1 * r1).sqrt() * (2.0 * ScalarConsts::PI * r2).sin();
        let z = r1;

        // // Calculate the direction, relative to the ONB

        let dir = (x * u) + (y * v) + (z * self.normal);
        let dir = dir.normalized();

        // // Calculate the PDF

        let pdf = 0.5 * ScalarConsts::PI;

        (dir, pdf)
    }

    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar
    {
        let cosine = self.normal.dot(dir.normalized());

        if cosine >= 0.0
        {
            0.5 * ScalarConsts::PI
        }
        else
        {
            0.0
        }
    }

    fn reflectance(&self, _input_dir: Dir3, output_dir: Dir3) -> Scalar
    {
        let cosine = self.normal.dot(output_dir.normalized());

        if cosine >= 0.0
        {
            cosine
        }
        else
        {
            0.0
        }
    }
}
