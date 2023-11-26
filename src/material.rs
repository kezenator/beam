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
    FrontBack(Box<Material>, Box<Material>),
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

    pub fn front_back(front: Material, back: Material) -> Material
    {
        Material::FrontBack(Box::new(front), Box::new(back))
    }

    pub fn front_only(front: Material) -> Material
    {
        Self::front_back(
            front,
            Material::Emit(Texture::solid(LinearRGB::black())))
    }

    pub fn get_surface_interaction(&self, intersection: &ShadingIntersection) -> MaterialInteraction
    {
        match self
        {
            Material::Diffuse(texture) =>
            {
                let mut diffuse_color = texture.get_color_at(intersection.texture_coords);

                if let Some(color_coords) = intersection.opt_color
                {
                    diffuse_color = diffuse_color.combined_with(&color_coords);
                }

                MaterialInteraction::Diffuse { diffuse_color }
            },
            Material::Metal(texture, fuzz) =>
            {
                let mut attenuate_color = texture.get_color_at(intersection.texture_coords);

                if let Some(color_coords) = intersection.opt_color
                {
                    attenuate_color = attenuate_color.combined_with(&color_coords);
                }

                MaterialInteraction::Reflection
                {
                    attenuate_color,
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
                let mut emitted_color = texture.get_color_at(intersection.texture_coords);

                if let Some(color_coords) = intersection.opt_color
                {
                    emitted_color = emitted_color.combined_with(&color_coords);
                }

                MaterialInteraction::Emit { emitted_color }
            },
            Material::FrontBack(front, back) =>
            {
                match intersection.face
                {
                    Face::Front => front.get_surface_interaction(intersection),
                    Face::Back => back.get_surface_interaction(intersection),
                }
            },
        }
    }
}
