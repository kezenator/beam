pub type Scalar = f64;

pub const EPSILON: Scalar = 1e-9;

pub fn degrees_to_radians(degrees: Scalar) -> Scalar
{
    return degrees * std::f64::consts::PI / 180.0;
}
