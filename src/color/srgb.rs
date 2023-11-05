use crate::math::Scalar;

#[derive(Clone, Copy, Debug, Default)]
pub struct SRGB
{
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
    pub a: Scalar,
}

impl SRGB
{
    pub fn new(r: Scalar, g: Scalar, b: Scalar, a: Scalar) -> Self
    {
        SRGB { r, g, b, a }
    }

    pub fn to_u8_rgba_tuple(&self) -> (u8, u8, u8, u8)
    {
        (
            to_u8_saturate(self.r),
            to_u8_saturate(self.g),
            to_u8_saturate(self.b),
            to_u8_saturate(self.a),
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

impl From<SRGB> for [f32; 4]
{
    fn from(value: SRGB) -> Self
    {
        [value.r as f32, value.g as f32, value.b as f32, value.a as f32]
    }
}

impl From<[f32; 4]> for SRGB
{
    fn from(value: [f32; 4]) -> Self
    {
        SRGB{ r: value[0] as Scalar, g: value[1] as Scalar, b: value[2] as Scalar, a: value[3] as Scalar }
    }
}
