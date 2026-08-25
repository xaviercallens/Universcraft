//! HoloEngine Atmospheric Scattering & Astrophysical Rendering Module
//! Implements Rayleigh & Mie Volumetric Scattering, Dynamic Day/Night Cycle, and Volumetric Altitude Fog.

#[derive(Debug, Clone, Copy)]
pub struct SunPosition {
    pub azimuth: f32,
    pub elevation: f32,
    pub direction: (f32, f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub struct AtmosphericScatteringParams {
    pub rayleigh_coeff: (f32, f32, f32), // (R, G, B) scattering coefficients
    pub mie_coeff: f32,
    pub sun_intensity: f32,
    pub fog_density: f32,
}

impl Default for AtmosphericScatteringParams {
    fn default() -> Self {
        Self {
            rayleigh_coeff: (5.8e-6, 1.35e-5, 3.31e-5),
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
        let direction = (azimuth * 0.8, elevation, 0.3).normalize_vec();
        SunPosition { azimuth, elevation, direction }
    }

    /// Computes physical sky RGB color using Rayleigh & Mie scattering model
    pub fn evaluate_sky_color(&self, view_dir: (f32, f32, f32)) -> (f32, f32, f32) {
        let sun = self.compute_sun_position();
        let sun_dir = sun.direction;
        
        let cos_theta = view_dir.0 * sun_dir.0 + view_dir.1 * sun_dir.1 + view_dir.2 * sun_dir.2;
        
        // Rayleigh Phase Function: P_R(theta) = 3/16pi * (1 + cos^2(theta))
        let phase_rayleigh = 0.0597 * (1.0 + cos_theta * cos_theta);
        
        // Mie Phase Function (Henyey-Greenstein g = 0.76)
        let g = 0.76;
        let phase_mie = (1.0 - g * g) / (4.0 * std::f32::consts::PI * (1.0 + g * g - 2.0 * g * cos_theta).powf(1.5));
        
        // Sun elevation factor
        let sun_height = sun.elevation.max(-0.2);
        let day_factor = (sun_height + 0.1).clamp(0.0, 1.0);
        
        let (r_r, r_g, r_b) = self.params.rayleigh_coeff;
        let mie = self.params.mie_coeff * phase_mie;

        if sun_height > 0.0 {
            // Daytime Sky (Rayleigh Blue + Mie Solar Glow)
            let r = (r_r * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            let g_c = (r_g * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            let b = (r_b * phase_rayleigh * 5.0e4 + mie * 1.0e5) * day_factor;
            (r.clamp(0.01, 1.0), g_c.clamp(0.02, 1.0), b.clamp(0.05, 1.0))
        } else {
            // Sunset / Night Sky (Red Rayleigh Shift + Dark Starfield)
            let sunset_factor = (-sun_height * 5.0).clamp(0.0, 1.0);
            let r = 0.05 + 0.3 * (1.0 - sunset_factor);
            let g_c = 0.02 + 0.1 * (1.0 - sunset_factor);
            let b = 0.08 + 0.2 * (1.0 - sunset_factor);
            (r, g_c, b)
        }
    }

    /// Evaluates exponential volumetric altitude fog factor
    pub fn compute_volumetric_fog(&self, distance: f32, height: f32) -> f32 {
        let height_dampening = (-height * 0.1).exp();
        let fog_amount = 1.0 - (-distance * self.params.fog_density * height_dampening).exp();
        fog_amount.clamp(0.0, 1.0)
    }
}

trait Vec3Normalize {
    fn normalize_vec(self) -> (f32, f32, f32);
}

impl Vec3Normalize for (f32, f32, f32) {
    fn normalize_vec(self) -> (f32, f32, f32) {
        let len = (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt().max(0.0001);
        (self.0 / len, self.1 / len, self.2 / len)
    }
}
