//! HoloEngine Atmospheric Scattering & Astrophysical Rendering Module
//! Implements Rayleigh & Mie Volumetric Scattering, Dynamic Day/Night Cycle, and Volumetric Altitude Fog.

use crate::math_types::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct SunPosition {
    pub azimuth: f32,
    pub elevation: f32,
    pub direction: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct AtmosphericScatteringParams {
    pub rayleigh_coeff: Vec3,  // (R, G, B) scattering coefficients
    pub mie_coeff: f32,
    pub sun_intensity: f32,
    pub fog_density: f32,
}

impl Default for AtmosphericScatteringParams {
    fn default() -> Self {
        Self {
            rayleigh_coeff: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
            mie_coeff: 2.1e-6,
            sun_intensity: 20.0,
            fog_density: 0.015,
        }
    }
}

pub struct AtmosphericEngine {
    pub params: AtmosphericScatteringParams,
    pub time_of_day: f32, // 0.0 to 24.0 hours
}

impl AtmosphericEngine {
    pub fn new(time_of_day: f32) -> Self {
        Self {
            params: AtmosphericScatteringParams::default(),
            time_of_day,
        }
    }

    /// Computes Sun position direction vector based on time of day (0 to 24 hours)
    pub fn compute_sun_position(&self) -> SunPosition {
        let angle = (self.time_of_day / 24.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let elevation = angle.sin();
        let azimuth = angle.cos();
        let direction = Vec3::new(azimuth * 0.8, elevation, 0.3).normalize();
        SunPosition { azimuth, elevation, direction }
    }

    /// Computes physical sky RGB color using Rayleigh & Mie scattering model
    pub fn evaluate_sky_color(&self, view_dir: Vec3) -> Vec3 {
        let sun = self.compute_sun_position();
        let sun_dir = sun.direction;
        
        let cos_theta = view_dir.dot(sun_dir);
        
        // Rayleigh Phase Function: P_R(theta) = 3/16pi * (1 + cos^2(theta))
        let phase_rayleigh = 0.0597 * (1.0 + cos_theta * cos_theta);
        
        // Mie Phase Function (Henyey-Greenstein g = 0.76)
        let g = 0.76;
        let phase_mie = (1.0 - g * g) / (4.0 * std::f32::consts::PI * (1.0 + g * g - 2.0 * g * cos_theta).powf(1.5));
        
        // Sun elevation factor
        let sun_height = sun.elevation.max(-0.2);
        let day_factor = (sun_height + 0.1).clamp(0.0, 1.0);
        
        let r_coeff = self.params.rayleigh_coeff;
        let mie = self.params.mie_coeff * phase_mie;

        if sun_height > 0.0 {
            // Daytime Sky (Rayleigh Blue + Mie Solar Glow)
            let r = (r_coeff.x * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            let g_c = (r_coeff.y * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            let b = (r_coeff.z * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            Vec3::new(r.clamp(0.01, 1.0), g_c.clamp(0.02, 1.0), b.clamp(0.05, 1.0))
        } else {
            // Sunset / Night Sky (Red Rayleigh Shift + Dark Starfield)
            let sunset_factor = (-sun_height * 5.0).clamp(0.0, 1.0);
            let r = 0.05 + 0.3 * (1.0 - sunset_factor);
            let g_c = 0.02 + 0.1 * (1.0 - sunset_factor);
            let b = 0.08 + 0.2 * (1.0 - sunset_factor);
            Vec3::new(r, g_c, b)
        }
    }

    /// Evaluates exponential volumetric altitude fog factor
    pub fn compute_volumetric_fog(&self, distance: f32, height: f32) -> f32 {
        let height_dampening = (-height * 0.1).exp();
        let fog_amount = 1.0 - (-distance * self.params.fog_density * height_dampening).exp();
        fog_amount.clamp(0.0, 1.0)
    }
}
