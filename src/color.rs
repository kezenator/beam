use crate::math::Float;

#[derive(Clone)]
pub struct RGBA
{
    r: Float,
    g: Float,
    b: Float,
    a: Float,
}

impl RGBA
{
    pub fn new(r: Float, g: Float, b: Float, a: Float) -> Self
    {
        RGBA { r, g, b, a}
    }

    pub fn to_u8_tuple(&self) -> (u8, u8, u8, u8)
    {
        (
            to_u8_saturate(self.r),
            to_u8_saturate(self.g),
            to_u8_saturate(self.b),
            to_u8_saturate(self.a),
        )
    }
}

fn to_u8_saturate(f: Float) -> u8
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
