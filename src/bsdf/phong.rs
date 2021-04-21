use crate::bsdf::{Bsdf, random_sample_dir_from_onb_phi_theta, random_sample_dir_from_onb_xyz};
use crate::color::LinearRGB;
use crate::intersection::ShadingIntersection;
use crate::material::MaterialInteraction;
use crate::math::{Scalar, ScalarConsts};
use crate::ray::Ray;
use crate::sample::Sampler;
use crate::scene::{Scene, SceneSampleStats};
use crate::vec::{Dir3, bsdf_reflect};

/// Implements the Phong BSDF for diffuse/specular surfaces.
///
/// Equations are taken from "Importance Sampling of the Phong Reflectance Model"
/// by Jason Lawrence
pub struct Phong
{
    specular_dir: Dir3,
    normal: Dir3,
    kd: Scalar,
    ks: Scalar,
    n: Scalar,
}

impl Phong
{
    pub fn new(intersection: &ShadingIntersection, kd: Scalar, ks: Scalar, n: Scalar) -> Self
    {
        let specular_dir = bsdf_reflect(intersection.incoming, intersection.normal);
        let normal = intersection.normal;

        Phong { specular_dir, normal, kd, ks, n }
    }

    pub fn local_shading(scene: &Scene, intersection: &ShadingIntersection, diffuse_color: LinearRGB, ka: Scalar, kd: Scalar, specular_color: LinearRGB, ks: Scalar, n: Scalar, stats: &mut SceneSampleStats) -> LinearRGB
    {
        let mut result = diffuse_color.multiplied_by_scalar(ka);
    
        // Shadow rays, for lights in the first lighting region that
        // covers the intersection location
    
        if let Some(lighting_region) = scene.get_lighting_region_at(intersection.location)
        {
            // Scale effects by the number of lights
    
            let lights_factor = (lighting_region.local_points.len() as f64).recip();
            let kd = kd * lights_factor;
            let ks = ks * lights_factor;
    
            for local_light_loc in lighting_region.local_points.iter()
            {
                let light_dir = local_light_loc - intersection.location;
    
                if intersection.normal.dot(light_dir) > 0.0
                {
                    // The light is in the same direction as the normal - this light
                    // can contribute
    
                    let light_dir = light_dir.normalized();
    
                    stats.num_rays += 1;
    
                    if let Some(shadow_int) = scene.trace_intersection(&Ray::new(intersection.location, light_dir))
                    {
                        if let MaterialInteraction::Emit{ emitted_color } = shadow_int.material.get_surface_interaction(&shadow_int.surface.into())
                        {
                            // Our shadow ray has hit an emitting surface:
                            // 1) Clamp the emitted color - global illumination can need lights "brighter" than 1.0
                            // 2) Add diffuse and specular components as required
    
                            let emitted_color = emitted_color.clamped(0.0, 1.0);
    
                            if kd > 0.0
                            {
                                result = result + diffuse_color.combined_with(&emitted_color).multiplied_by_scalar(kd * light_dir.dot(intersection.normal));
                            }
    
                            if ks > 0.0
                            {
                                let reflected = bsdf_reflect(light_dir, intersection.normal);
    
                                let r_dot_v = reflected.dot(intersection.incoming);
    
                                if r_dot_v > 0.0
                                {
                                    result = result + specular_color.combined_with(&emitted_color).multiplied_by_scalar(ks * r_dot_v.powf(n));
                                }
                            }
                        }
                    }
                }
            }
        }
    
        result
    }
}

impl Bsdf for Phong
{
    fn generate_random_sample_dir_and_calc_pdf(&self, sampler: &mut Sampler) -> (Dir3, Scalar)
    {
        let dir = if ((self.kd + self.ks) * sampler.uniform_scalar_unit()) < self.kd
        {
            // Sample using Diffuse Lambertian cosine distribution

            let r1 = sampler.uniform_scalar_unit();
            let r2 = sampler.uniform_scalar_unit();
    
            let z = r1.sqrt();
            let sin_theta = (1.0 - r1).sqrt();
    
            let phi = 2.0 * ScalarConsts::PI * r2;
    
            let x = phi.cos() * sin_theta;
            let y = phi.sin() * sin_theta;
    
            random_sample_dir_from_onb_xyz(self.normal, x, y, z)
        }
        else
        {
            // Sample using Phong Specular distribution

            // omega = (alpha, phi)
            // alpha = arccos(r1 ^ (1 / (n + 1)))
            // phi = 2 * pi * r2

            let alpha = sampler.uniform_scalar_unit().powf((self.n + 1.0).recip()).acos();
            let phi = 2.0 * ScalarConsts::PI * sampler.uniform_scalar_unit();

            random_sample_dir_from_onb_phi_theta(self.specular_dir, phi, alpha)
        };

        (dir, self.calculate_pdf_for_dir(dir))
    }

    fn calculate_pdf_for_dir(&self, output_dir: Dir3) -> Scalar
    {
        let cos_theta = self.normal.dot(output_dir.normalized());

        if cos_theta >= 0.0
        {
            // Diffuse - use lambertian PDF

            let pdf_d = cos_theta * ScalarConsts::FRAC_1_PI;

            // Specular - (n + 1) / (2*pi) * cos^n(alpha)

            let cos_alpha = output_dir.dot(self.specular_dir);

            let pdf_s = if cos_alpha > 0.0
            {
                (self.n + 1.0) * (0.5 * ScalarConsts::FRAC_1_PI) * cos_alpha.powf(self.n)
            }
            else
            {
                0.0
            };

            // Combine

            (self.kd * pdf_d) + (self.ks * pdf_s)
        }
        else
        {
            0.0
        }
    }

    fn reflectance(&self, output_dir: Dir3) -> Scalar
    {
        let cos_theta = self.normal.dot(output_dir.normalized());

        if cos_theta >= 0.0
        {
            // Diffuse = kd / PI

            let mut result = self.kd * cos_theta * ScalarConsts::FRAC_1_PI;

            // Specular = ks * ((n + 2) / (2 * pi)) * cos^n(alpha)
            // (where n = shininess, alpha = cosone of angle between the
            //    perfectly reflective direction and the sample direction)

            let cos_alpha = output_dir.dot(self.specular_dir);

            if cos_alpha > 0.0
            {
                result = result + (self.ks * ((self.n + 2.0) * 0.5 * ScalarConsts::FRAC_1_PI) * cos_alpha.powf(self.n));
            }

            result
        }
        else
        {
            0.0
        }
    }
}
