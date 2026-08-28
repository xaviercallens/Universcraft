//! HoloEngine Atmospheric Coupling Plugin
//! Leverages open-source paradigms from:
//! - `Dimeylead/bevy_atmosphere` : Physical Rayleigh & Mie scattering coupled to dynamic humidity waves.
//! - `rust-adventure/bevy_volumetric_clouds` : Volumetric Cumulus Congestus cloud rendering with Henyey-Greenstein phase.

use bevy::prelude::*;

/// Dynamic Atmosphere Resource coupled to Whittaker Climate & Thermodynamics
#[derive(Resource, Debug, Clone)]
pub struct WhittakerAtmosphere {
    /// Solar vector (direction towards the sun)
    pub sun_direction: Vec3,
    /// Solar irradiance intensity (W/m²)
    pub sun_intensity: f32,
    /// Rayleigh scattering coefficients (R, G, B) in m⁻¹
    pub rayleigh_coeffs: Vec3,
    /// Mie scattering coefficient in m⁻¹
    pub mie_coeff: f32,
    /// Mie asymmetry parameter g (0.75 - 0.85)
    pub mie_asymmetry: f32,
    /// Local humidity wave parameter H in [0.0, 1.0]
    pub humidity_wave: f32,
    /// Local ambient temperature T in Kelvin (240K - 320K)
    pub temperature_k: f32,
    /// Cloud base altitude (meters)
    pub cloud_base_m: f32,
    /// Cloud ceiling altitude (meters)
    pub cloud_top_m: f32,
    /// Volumetric cloud coverage factor in [0.0, 1.0]
    pub cloud_coverage: f32,
}

impl Default for WhittakerAtmosphere {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(0.65, 0.52, 0.55).normalize(),
            sun_intensity: 22.0,
            rayleigh_coeffs: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
            mie_coeff: 2.1e-6,
            mie_asymmetry: 0.78,
            humidity_wave: 0.65,
            temperature_k: 293.15, // 20°C
            cloud_base_m: 500.0,
            cloud_top_m: 3500.0,
            cloud_coverage: 0.55,
        }
    }
}

impl WhittakerAtmosphere {
    /// Dynamically recalculates scattering coefficients based on Whittaker humidity and temperature
    pub fn update_from_whittaker(&mut self, humidity: f32, temperature_c: f32) {
        self.humidity_wave = humidity.clamp(0.0, 1.0);
        self.temperature_k = (temperature_c + 273.15).clamp(200.0, 350.0);

        // Relative humidity scales Mie aerosol turbidity
        let turbidity = 1.0 + 3.8 * self.humidity_wave.powi(2);
        self.mie_coeff = 2.1e-6 * turbidity;

        // Dew point & Clausius-Clapeyron determines cloud base lifting condensation level (LCL)
        // LCL ~ 125 * (T - T_dew) meters
        let spread = (temperature_c * (1.0 - self.humidity_wave)).max(0.5);
        self.cloud_base_m = 125.0 * spread + 400.0;
        self.cloud_top_m = self.cloud_base_m + 1500.0 * (1.0 + self.humidity_wave);
        self.cloud_coverage = (self.humidity_wave * 1.2 - 0.2).clamp(0.0, 0.95);
    }

    /// Evaluates analytic Henyey-Greenstein dual-phase function for cloud in-scattering
    pub fn evaluate_phase(&self, cos_theta: f32) -> f32 {
        let g1 = self.mie_asymmetry;
        let g2 = -0.22;
        let hg1 = (1.0 - g1 * g1) / (4.0 * std::f32::consts::PI * (1.0 + g1 * g1 - 2.0 * g1 * cos_theta).max(0.001).powf(1.5));
        let hg2 = (1.0 - g2 * g2) / (4.0 * std::f32::consts::PI * (1.0 + g2 * g2 - 2.0 * g2 * cos_theta).max(0.001).powf(1.5));
        0.8 * hg1 + 0.2 * hg2
    }

    /// Evaluates Rayleigh sky dome color for a given view direction
    pub fn evaluate_sky_color(&self, view_dir: Vec3) -> Vec3 {
        let cos_theta = view_dir.dot(self.sun_direction);
        let rayleigh_phase = 0.0597 * (1.0 + cos_theta * cos_theta);
        let mie_phase = self.evaluate_phase(cos_theta);

        let sun_elevation = self.sun_direction.y.max(-0.2);
        let day_factor = (sun_elevation + 0.1).clamp(0.0, 1.0);

        let scatter_r = (self.rayleigh_coeffs.x * rayleigh_phase * 5.0e4 + self.mie_coeff * mie_phase * 1.0e5) * day_factor;
        let scatter_g = (self.rayleigh_coeffs.y * rayleigh_phase * 5.0e4 + self.mie_coeff * mie_phase * 1.0e5) * day_factor;
        let scatter_b = (self.rayleigh_coeffs.z * rayleigh_phase * 5.0e4 + self.mie_coeff * mie_phase * 1.0e5) * day_factor;

        Vec3::new(scatter_r.clamp(0.02, 1.0), scatter_g.clamp(0.04, 1.0), scatter_b.clamp(0.08, 1.0))
    }
}

pub struct AtmosphereCouplingPlugin;

impl Plugin for AtmosphereCouplingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WhittakerAtmosphere>()
           .add_systems(Update, update_atmospheric_thermodynamics);
    }
}

fn update_atmospheric_thermodynamics(
    time: Res<Time>,
    mut atmosphere: ResMut<WhittakerAtmosphere>,
) {
    let t = time.elapsed_seconds();
    // Simulate diurnal humidity fluctuation
    let humidity = 0.5 + 0.25 * (t * 0.05).sin();
    let temperature = 22.0 + 8.0 * (t * 0.05 + 1.2).cos();
    atmosphere.update_from_whittaker(humidity, temperature);
}
