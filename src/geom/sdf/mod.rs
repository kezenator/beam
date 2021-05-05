use crate::math::Scalar;
use crate::vec::{Dir3, Point3};

pub trait Sdf: CloneableSdf
{
    fn distance_and_normal(&self, pos: Point3) -> (Scalar, Dir3);
}

pub trait CloneableSdf
{
    fn clone_boxed_sdf(&self) -> Box<dyn Sdf>;
}

impl Clone for Box<dyn Sdf>
{
    fn clone(&self) -> Box<dyn Sdf>
    {
        self.clone_boxed_sdf()
    }
}

impl<T> CloneableSdf for T
    where T: Sdf + Clone + Sized + 'static
{
    fn clone_boxed_sdf(&self) -> Box<dyn Sdf>
    {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub enum ConcreteSdf
{
    Sphere{ center: Point3, radius: Scalar },
    Capsule{ a: Point3, b: Point3, radius: Scalar },
    Union{ members: Vec<ConcreteSdf> },
    Annular{ sdf: Box<ConcreteSdf>, radius: Scalar },

}

impl Sdf for ConcreteSdf
{
    fn distance_and_normal(&self, pos: Point3) -> (Scalar, Dir3)
    {
        match self
        {
            ConcreteSdf::Sphere{center, radius} =>
            {
                let o = pos - center;
                let d = o.magnitude();

                (d - radius, o / d)
            },
            ConcreteSdf::Capsule{a, b, radius} =>
            {
                // https://iquilezles.org/www/articles/distgradfunctions2d/distgradfunctions2d.htm

                let ba = b - a;
                let pa = pos - a;
                let h =(pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
                let q = pa - h * ba;
                let d = q.magnitude();

                (d - radius, q / d)
            },
            ConcreteSdf::Union{ members } =>
            {
                let mut result = members[0].distance_and_normal(pos);

                for member in members.iter().skip(1)
                {
                    let trial = member.distance_and_normal(pos);

                    if trial.0 < result.0
                    {
                        result = trial;
                    }
                }

                result
            },
            ConcreteSdf::Annular{ sdf, radius } =>
            {
                let (distance, normal) = sdf.distance_and_normal(pos);

                (distance.abs() - radius, distance.signum() * normal)
            },
        }
    }
}
