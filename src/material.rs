use crate::color::RGBA;
use crate::intersection::SurfaceIntersection;
use crate::math::Scalar;
use crate::texture::Texture;

pub enum MaterialInteraction
{
    Diffuse{ diffuse_color: RGBA},
    Reflection{ attenuate_color: RGBA, fuzz: Scalar },
    Refraction{ ior: Scalar },
    Emit{ emitted_color: RGBA},
}

pub enum Material
{
    Diffuse(Texture),
    Metal(Texture, Scalar),
    Dielectric(Scalar),
    Emit(Texture),
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

    pub fn emit(texture: Texture) -> Material
    {
        Material::Emit(texture)
    }

    pub fn get_surface_interaction<'r>(&self, intersection: &SurfaceIntersection<'r>) -> MaterialInteraction
    {
        match self
        {
            Material::Diffuse(texture) =>
            {
                MaterialInteraction::Diffuse
                {
                    diffuse_color: texture.get_color_at(intersection.location()),
                }
            },
            Material::Metal(texture, fuzz) =>
            {
                MaterialInteraction::Reflection
                {
                    attenuate_color: texture.get_color_at(intersection.location()),
                    fuzz: *fuzz,
                }
            },
            Material::Dielectric(ior) =>
            {
                MaterialInteraction::Refraction
                {
                    ior: *ior,
                }
            },
            Material::Emit(texture) =>
            {
                MaterialInteraction::Emit
                {
                    emitted_color: texture.get_color_at(intersection.location()),
                }
            },
        }
    }
}
