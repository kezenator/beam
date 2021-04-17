use crate::math::Scalar;

#[derive(Clone)]
pub struct RGBA
{
    r: Scalar,
    g: Scalar,
    b: Scalar,
    a: Scalar,
}

impl RGBA
{
    pub fn new(r: Scalar, g: Scalar, b: Scalar, a: Scalar) -> Self
    {
        RGBA { r, g, b, a}
    }

    pub fn clamped(&self) -> Self
    {
        RGBA::new(self.r.clamp(0.0, 1.0), self.g.clamp(0.0, 1.0), self.b.clamp(0.0, 1.0), self.a.clamp(0.0, 1.0))
    }

    pub fn multiplied_by_scalar(&self, mul: Scalar) -> Self
    {
        RGBA::new(self.r * mul, self.g * mul, self.b * mul, self.a)
    }

    pub fn divided_by_scalar(&self, div: Scalar) -> Self
    {
        RGBA::new(self.r / div, self.g / div, self.b / div, self.a)
    }

    pub fn combined_with(&self, rhs: &RGBA) -> Self
    {
        RGBA::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b, self.a * rhs.a)
    }

    pub fn gamma_corrected_2(&self) -> Self
    {
        RGBA::new(self.r.sqrt(), self.g.sqrt(), self.b.sqrt(), self.a)
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

impl std::ops::Add for RGBA
{
    type Output = RGBA;

    fn add(self, rhs: RGBA) -> Self::Output
    {
        RGBA::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a * rhs.a)
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
