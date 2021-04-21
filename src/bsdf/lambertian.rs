use crate::bsdf::{Bsdf, random_sample_dir_from_onb_xyz};
use crate::math::{Scalar, ScalarConsts};
use crate::sample::Sampler;
use crate::vec::Dir3;

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
    fn generate_random_sample_dir_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        // Generate two random variables

        let r1 = sampler.uniform_scalar_unit();
        let r2 = sampler.uniform_scalar_unit();

        // Convert these to x/y/z parameters

        let z = r1.sqrt();
        let sin_theta = (1.0 - r1).sqrt();

        let phi = 2.0 * ScalarConsts::PI * r2;

        let x = phi.cos() * sin_theta;
        let y = phi.sin() * sin_theta;

        // Convert to a direction

        let dir = random_sample_dir_from_onb_xyz(self.normal, x, y, z);

        // Calculate the PDF

        let pdf = z * ScalarConsts::FRAC_1_PI;

        (dir, pdf)
    }

    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar
    {
        let cos_theta = self.normal.dot(dir.normalized());

        if cos_theta >= 0.0
        {
            cos_theta * ScalarConsts::FRAC_1_PI
        }
        else
        {
            0.0
        }
    }

    fn reflectance(&self, dir: Dir3) -> Scalar
    {
        let cos_theta = self.normal.dot(dir.normalized());

        if cos_theta >= 0.0
        {
            cos_theta * ScalarConsts::FRAC_1_PI
        }
        else
        {
            0.0
        }
    }
}
