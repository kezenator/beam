use crate::math::Scalar;
use crate::color::SRGB;

#[derive(Clone, Copy, Debug)]
pub struct LinearRGB
{
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
}

impl LinearRGB
{
    pub fn new(r: Scalar, g: Scalar, b: Scalar) -> Self
    {
        LinearRGB { r, g, b }
    }

    pub fn black() -> Self
    {
        LinearRGB{ r: 0.0, g: 0.0, b: 0.0 }
    }

    pub fn grey(albedo: Scalar) -> Self
    {
        LinearRGB{ r: albedo, g: albedo, b: albedo }
    }

    pub fn white() -> Self
    {
        LinearRGB{ r: 1.0, g: 1.0, b: 1.0 }
    }

    pub fn max_color_component(&self) -> Scalar
    {
        self.r.max(self.g.max(self.b))
    }

    pub fn clamped(&self, min: Scalar, max: Scalar) -> Self
    {
        LinearRGB::new(self.r.clamp(min, max), self.g.clamp(min, max), self.b.clamp(min, max))
    }

    pub fn multiplied_by_scalar(&self, mul: Scalar) -> Self
    {
        LinearRGB::new(self.r * mul, self.g * mul, self.b * mul)
    }

    pub fn divided_by_scalar(&self, div: Scalar) -> Self
    {
        LinearRGB::new(self.r / div, self.g / div, self.b / div)
    }

    pub fn combined_with(&self, rhs: &LinearRGB) -> Self
    {
        LinearRGB::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }

    pub fn to_srgb(&self) -> SRGB
    {
        let linear_to_srgb = |c: Scalar| c.sqrt();

        SRGB::new(linear_to_srgb(self.r), linear_to_srgb(self.g), linear_to_srgb(self.b))
    }
}

impl std::ops::Add for LinearRGB
{
    type Output = LinearRGB;

    fn add(self, rhs: LinearRGB) -> Self::Output
    {
        LinearRGB::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl From<SRGB> for LinearRGB
{
    fn from(val: SRGB) -> Self
    {
        let srgb_to_linear = |c: Scalar| c * c;

        LinearRGB::new(srgb_to_linear(val.r), srgb_to_linear(val.g), srgb_to_linear(val.b))
    }
}