use crate::color::RGBA;
use crate::intersection::SurfaceIntersection;
use crate::math::{EPSILON, Scalar};
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::texture::Texture;

pub enum Material
{
    Diffuse(Texture),
    Metal(Texture, Scalar),
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

                let reflect_dir =
                    intersection.ray.dir
                    - (2.0 * intersection.ray.dir.dot(intersection.normal) * intersection.normal);

                // Add in some fuzzyness to the reflected ray

                let reflect_dir =
                    reflect_dir.normalized()
                    + (*fuzz * sampler.uniform_point_in_unit_sphere());

                // With this fuzzyness, glancing rays or large
                // fuzzyness can cause the reflected ray to point
                // inside the object. Ignore there

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
        }
    }
}