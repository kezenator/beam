use crate::math::Scalar;

pub type Mat4 = vek::mat::Mat4<Scalar>;
pub type Vec3 = vek::vec::Vec3<Scalar>;
pub type Vec4 = vek::vec::Vec4<Scalar>;

pub type Dir3 = Vec3;
pub type Point3 = Vec3;

pub fn bsdf_reflect(incoming: Dir3, normal: Dir3) -> Dir3
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#metal
    // Except "inverted" because incoming and normal are in the same direction.
    // Both must be normalized.
    
    ((2.0 * incoming.dot(normal)) * normal) - incoming
}

pub enum RefractResult
{
    ReflectOrRefract{ reflect_dir: Dir3, refract_dir: Dir3, reflect_probability: Scalar },
    TotalInternalReflection{ reflect_dir: Dir3 }
}

pub fn bsdf_refract_or_reflect(incoming: Dir3, normal: Dir3, refraction_ratio: Scalar) -> RefractResult
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
    // Except "inverted" because incoming and normal are in the same direction

    let cos_theta = normal.dot(incoming).min(1.0);

    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    if cannot_refract
    {
        RefractResult::TotalInternalReflection
        {
            reflect_dir: bsdf_reflect(incoming, normal),
        }
    }
    else
    {
        let r_out_perp =  refraction_ratio * (cos_theta*normal - incoming);
        let r_out_parallel = -(1.0 - r_out_perp.magnitude_squared()).abs().sqrt() * normal;

        let refract_dir = r_out_perp + r_out_parallel;
        let reflect_dir = bsdf_reflect(incoming, normal);
        let reflect_probability = schlicks_reflectance(cos_theta, refraction_ratio);

        RefractResult::ReflectOrRefract
        {
            reflect_dir,
            refract_dir,
            reflect_probability
        }
    }
}

fn schlicks_reflectance(cosine: Scalar, ref_idx: Scalar) -> Scalar
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
    // Use Schlick's approximation for reflectance.
    
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
