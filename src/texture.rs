use crate::color::LinearRGB;
use crate::vec::Point3;

pub enum Texture
{
    Solid(LinearRGB),
    Checkerboard(LinearRGB, LinearRGB),
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
        }
    }
}