use crate::math::Scalar;

#[derive(Clone, Copy, Debug)]
pub struct SRGB
{
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
}

impl SRGB
{
    pub fn new(r: Scalar, g: Scalar, b: Scalar) -> Self
    {
        SRGB { r, g, b }
    }

    pub fn to_u8_rgba_tuple(&self) -> (u8, u8, u8, u8)
    {
        (
            to_u8_saturate(self.r),
            to_u8_saturate(self.g),
            to_u8_saturate(self.b),
            255,
        )
    }
}

fn to_u8_saturate(f: Scalar) -> u8
{
    let f = f * 255.0;

    if f >= 255.0
    {
        255u8
    }
    else if f >= 0.0
    {
        f as u8
    }
    else
    {
        0u8
    }    
}
