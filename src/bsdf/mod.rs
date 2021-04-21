use crate::math::Scalar;
use crate::sample::Sampler;
use crate::vec::Dir3;

pub mod lambertian;
pub mod phong;

pub use lambertian::*;
pub use phong::*;

pub trait Bsdf
{
    fn generate_random_sample_dir_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar);
    fn calculate_pdf_for_dir(&self, dir: Dir3) -> Scalar;
    fn reflectance(&self, output_dir: Dir3) -> Scalar;
}

fn random_sample_dir_from_onb_phi_theta(onb: Dir3, phi: Scalar, theta: Scalar) -> Dir3
{
    let z = theta.cos();
    let sin_theta = theta.sin();

    let x = phi.cos() * sin_theta;
    let y = phi.sin() * sin_theta;

    random_sample_dir_from_onb_xyz(onb, x, y, z)
}

fn random_sample_dir_from_onb_xyz(onb: Dir3, x: Scalar, y: Scalar, z: Scalar) -> Dir3
{
    let u = if onb.x.abs() > 0.9 { Dir3::new(0.0, 1.0, 0.0) } else { Dir3::new(1.0, 0.0, 0.0) };
    let v = onb.cross(u).normalized();
    let u = onb.cross(v);

    (x * u) + (y * v) + (z * onb)
}
