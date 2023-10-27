use crate::math::Scalar;

#[derive(Clone, Copy, Debug, Default)]
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

impl From<SRGB> for [f32; 3]
{
    fn from(value: SRGB) -> Self
    {
        [value.r as f32, value.g as f32, value.b as f32]
    }
}

impl From<[f32; 3]> for SRGB
{
    fn from(value: [f32; 3]) -> Self
    {
        SRGB{ r: value[0] as Scalar, g: value[1] as Scalar, b: value[2] as Scalar }
    }
}
