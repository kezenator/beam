use crate::color::RGBA;
use crate::vec::Point3;

pub enum Texture
{
    Solid(RGBA),
    Checkerboard(RGBA, RGBA),
}

impl Texture
{
    pub fn solid(color: RGBA) -> Texture
    {
        Texture::Solid(color)
    }

    pub fn checkerboard(c1: RGBA, c2: RGBA) -> Texture
    {
        Texture::Checkerboard(c1, c2)
    }

    pub fn get_color_at(&self, point: Point3) -> RGBA
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