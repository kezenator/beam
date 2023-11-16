use crate::color::LinearRGB;
use crate::geom::Sdf;
use crate::import::image::Image;
use crate::vec::Point3;

#[derive(Clone)]
pub enum Texture
{
    Solid(LinearRGB),
    Checkerboard(LinearRGB, LinearRGB),
    Image{ base_color: LinearRGB, image: Image},
    Sdf(Sdf),
}

impl Texture
{
    pub fn solid<C: Into<LinearRGB>>(color: C) -> Texture
    {
        Texture::Solid(color.into())
    }

    pub fn checkerboard<C1: Into<LinearRGB>, C2: Into<LinearRGB>>(c1: C1, c2: C2) -> Texture
    {
        Texture::Checkerboard(c1.into(), c2.into())
    }

    pub fn image<C: Into<LinearRGB>>(base_color: C, image: Image) -> Texture
    {
        Texture::Image{ base_color: base_color.into(), image }
    }

    pub fn sdf(sdf: Sdf) -> Texture
    {
        Texture::Sdf(sdf)
    }

    pub fn get_color_at(&self, point: Point3) -> LinearRGB
    {
        match self
        {
            Texture::Solid(c1) =>
            {
                *c1
            },
            Texture::Checkerboard(c1, c2) =>
            {
                let scalar = point[0].round() + point[1].round() + point[2].round();
                
                if ((scalar as i64) & 1) != 0
                {
                    *c1
                }
                else
                {
                    *c2
                }
            }
            Texture::Image{ base_color, image } =>
            {
                let u = point[0].fract();
                let v = point[1].fract();

                base_color.combined_with(&image.sample_at_uv(u, v).into())
            },
            Texture::Sdf(sdf) =>
            {
                let val = sdf.distance(point);

                if val.abs() < 0.5
                {
                    LinearRGB::white()
                }
                else
                {
                    let prod = if ((val.abs().round() as u64) & 1) == 0 { 0.5 } else { 1.0 };

                    if val.is_sign_positive()
                    {
                        LinearRGB::new(1.0, 0.5, 0.2, 1.0).multiplied_by_scalar(prod)
                    }
                    else
                    {
                        LinearRGB::new(0.1, 0.6, 0.8, 1.0).multiplied_by_scalar(prod)
                    }
                }
            }
        }
    }
}