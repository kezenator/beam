use crate::color::RGBA;
use crate::intersection::{Face, SurfaceIntersection};
use crate::math::{EPSILON, Scalar};
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::texture::Texture;
use crate::vec::Dir3;

pub enum Material
{
    Diffuse(Texture),
    Metal(Texture, Scalar),
    Dielectric(Scalar),
}

impl Material
{
    pub fn diffuse(texture: Texture) -> Material
    {
        Material::Diffuse(texture)
    }

    pub fn metal(texture: Texture, fuzz: Scalar) -> Material
    {
        Material::Metal(texture, fuzz)
    }

    pub fn dielectric(ior: Scalar) -> Material
    {
        Material::Dielectric(ior)
    }

    pub fn scatter<'r>(&self, sampler: &mut Sampler, intersection: &SurfaceIntersection<'r>) -> Option<(Ray, RGBA)>
    {
        match self
        {
            Material::Diffuse(texture) =>
            {
                let location = intersection.location();

                let scatter_dir = intersection.normal + sampler.uniform_dir_on_unit_sphere();

                let scatter_ray = Ray::new(location, scatter_dir);

                let attenuation_color = texture.get_color_at(location);

                Some((scatter_ray, attenuation_color))
            },
            Material::Metal(texture, fuzz) =>
            {
                let location = intersection.location();

                // Reflect incoming ray about the normal

                let reflect_dir = reflect(intersection.ray.dir, intersection.normal);

                // Add in some fuzzyness to the reflected ray

                let reflect_dir =
                    reflect_dir.normalized()
                    + (*fuzz * sampler.uniform_point_in_unit_sphere());

                // With this fuzzyness, glancing rays or large
                // fuzzyness can cause the reflected ray to point
                // inside the object. Think about

                if reflect_dir.dot(intersection.normal) > EPSILON
                {
                    let scatter_ray = Ray::new(location, reflect_dir);

                    let attenuation_color = texture.get_color_at(location);

                    Some((scatter_ray, attenuation_color))
                }
                else
                {
                    None
                }
            },
            Material::Dielectric(ior) =>
            {
                let refraction_ratio = if intersection.face == Face::FrontFace
                {
                    1.0 / ior
                }
                else
                {
                    *ior
                };

                let unit_direction = intersection.ray.dir.normalized();

                let new_dir = refract_or_reflect(sampler, unit_direction, intersection.normal, refraction_ratio);

                let new_ray = Ray::new(intersection.location(), new_dir);                

                Some((new_ray, RGBA::new(1.0, 1.0, 1.0, 1.0)))
            },
        }
    }
}

fn reflect(incoming: Dir3, normal: Dir3) -> Dir3
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#metal
    //
    // incoming is the ray coming in.
    // normal is the surface normal.
    //
    // Both must be normalized, and must
    // be in opposite directions (i.e. dot product is negative).
    //
    // incoming.dot(normal) * normal is the component that brings the
    // incoming ray back to perpendicular with the normal.
    // Adding twice this will give the reflection
    
    incoming - ((2.0 * incoming.dot(normal)) * normal)
}

fn refract_or_reflect(sampler: &mut Sampler, incoming: Dir3, normal: Dir3, refraction_ratio: Scalar) -> Dir3
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics

    let cos_theta = normal.dot(-incoming).min(1.0);

    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    if cannot_refract
        || reflectance(cos_theta, refraction_ratio) > sampler.uniform_scalar_unit()
    {
        reflect(incoming, normal)
    }
    else
    {
        let r_out_perp =  refraction_ratio * (incoming + cos_theta*normal);
        let r_out_parallel = -(1.0 - r_out_perp.magnitude_squared()).abs().sqrt() * normal;

        r_out_perp + r_out_parallel
    }
}


fn reflectance(cosine: Scalar, ref_idx: Scalar) -> Scalar
{
    // From https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
    // Use Schlick's approximation for reflectance.
    
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}