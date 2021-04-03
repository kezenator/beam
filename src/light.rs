use crate::color::RGBA;
use crate::ray::Ray;
use crate::vec::Dir3;

pub enum Light
{
    Ambient{ color: RGBA },
    Directional{ color: RGBA, direction: Dir3 },
}

impl Light
{
    pub fn ambient(color: RGBA) -> Self
    {
        Light::Ambient{ color }
    }

    pub fn directional(color: RGBA, direction: Dir3) -> Self
    {
        Light::Directional{ color, direction }
    }

    pub fn get_light(&self, ray: &Ray) -> RGBA
    {
        match self
        {
            Light::Ambient{ color } =>
            {
                color.clone()
            },
            Light::Directional{ color, direction } =>
            {
                // Get factor that the ray is looking into the light

                let mut factor = - ray.dir.normalized().dot(direction.normalized());
                
                // If looking away from the light then no light is added

                if factor < 0.0
                {
                    factor = 0.0;
                }

                // Make it a bit tigher

                factor = factor * factor * factor * factor * factor;

                color.multiplied_by_scalar(factor)
            },
        }
    }
}
