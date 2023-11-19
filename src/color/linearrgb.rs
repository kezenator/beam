use crate::math::Scalar;
use crate::color::SRGB;

#[derive(Clone, Copy, Debug)]
pub struct LinearRGB
{
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
    pub a: Scalar,
}

impl LinearRGB
{
    pub fn new(r: Scalar, g: Scalar, b: Scalar, a: Scalar) -> Self
    {
        LinearRGB { r, g, b, a }
    }

    pub fn black() -> Self
    {
        LinearRGB{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn grey(albedo: Scalar) -> Self
    {
        LinearRGB{ r: albedo, g: albedo, b: albedo, a: 1.0 }
    }

    pub fn white() -> Self
    {
        LinearRGB{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    pub fn max_color_component(&self) -> Scalar
    {
        self.r.max(self.g.max(self.b))
    }

    pub fn clamped(&self, min: Scalar, max: Scalar) -> Self
    {
        LinearRGB::new(self.r.clamp(min, max), self.g.clamp(min, max), self.b.clamp(min, max), self.a)
    }

    pub fn multiplied_by_scalar(&self, mul: Scalar) -> Self
    {
        LinearRGB::new(self.r * mul, self.g * mul, self.b * mul, self.a)
    }

    pub fn multiplied_by_scalar_inc_alpha(&self, mul: Scalar) -> Self
    {
        LinearRGB::new(self.r * mul, self.g * mul, self.b * mul, self.a * mul)
    }

    pub fn divided_by_scalar(&self, div: Scalar) -> Self
    {
        LinearRGB::new(self.r / div, self.g / div, self.b / div, self.a)
    }

    pub fn combined_with(&self, rhs: &LinearRGB) -> Self
    {
        LinearRGB::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b, self.a * rhs.a)
    }

    pub fn to_srgb(&self) -> SRGB
    {
        let linear_to_srgb = |c: Scalar|
        {
            if c <= 0.0031308
            {
                12.92 * c
            }
            else
            {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        };

        SRGB::new(linear_to_srgb(self.r), linear_to_srgb(self.g), linear_to_srgb(self.b), self.a)
    }
}

impl std::ops::Add for LinearRGB
{
    type Output = LinearRGB;

    fn add(self, rhs: LinearRGB) -> Self::Output
    {
        LinearRGB::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a + rhs.a)
    }
}

impl From<SRGB> for LinearRGB
{
    fn from(val: SRGB) -> Self
    {
        let srgb_to_linear = |c: Scalar|
        {
            if c <= 0.04045
            {
                c / 12.92
            }
            else
            {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        LinearRGB::new(srgb_to_linear(val.r), srgb_to_linear(val.g), srgb_to_linear(val.b), val.a)
    }
}