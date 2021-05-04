use crate::color::LinearRGB;
use crate::intersection::{Face, ShadingIntersection};
use crate::math::Scalar;
use crate::texture::Texture;

pub enum MaterialInteraction
{
    Diffuse{ diffuse_color: LinearRGB},
    Reflection{ attenuate_color: LinearRGB, fuzz: Scalar },
    Refraction{ ior: Scalar },
    Emit{ emitted_color: LinearRGB},
}

#[derive(Clone)]
pub enum Material
{
    Diffuse(Texture),
    Metal(Texture, Scalar),
    Dielectric(Scalar),
    Emit(Texture),
    EmitFrontFaceOnly(Texture),
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

    pub fn emit_front_face_only(texture: Texture) -> Material
    {
        Material::EmitFrontFaceOnly(texture)
    }

    pub fn get_surface_interaction(&self, intersection: &ShadingIntersection) -> MaterialInteraction
    {
        match self
        {
            Material::Diffuse(texture) =>
            {
                MaterialInteraction::Diffuse
                {
                    diffuse_color: texture.get_color_at(intersection.location),
                }
            },
            Material::Metal(texture, fuzz) =>
            {
                MaterialInteraction::Reflection
                {
                    attenuate_color: texture.get_color_at(intersection.location),
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
                    emitted_color: texture.get_color_at(intersection.location),
                }
            },
            Material::EmitFrontFaceOnly(texture) =>
            {
                match intersection.face
                {
                    Face::Front =>
                        MaterialInteraction::Emit
                        {
                            emitted_color: texture.get_color_at(intersection.location),
                        },
                    Face::Back =>
                        MaterialInteraction::Diffuse
                        {
                            diffuse_color: LinearRGB::new(0.0, 0.0, 0.0),
                        },
                }
            },
        }
    }
}
