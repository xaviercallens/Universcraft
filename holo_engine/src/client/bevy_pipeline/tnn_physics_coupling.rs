/// TNN & TDA Physics Coupling Module (Universcraft Engine)
/// Connects machine-verified scientific invariants and certified JSON datasets
/// from SocrateAI-Scientific-TNN-UniversModel into Universcraft's real-time physics & rendering.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanJhtdbParams {
    pub gravity_m_s2: f32,
    pub kinematic_viscosity_m2_s: f32,
    pub surface_tension_N_m: f32,
    pub enstrophy_cutoff_s2: f32,
    pub target_b1_vortices: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunesExnerParams {
    pub repose_angle_deg: f32,
    pub max_slope_tan: f32,
    pub sand_density_kg_m3: f32,
    pub target_b1_loops: u32,
    pub target_b2_voids: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryosphereEra5Params {
    pub glen_exponent_n: f32,
    pub ice_density_kg_m3: f32,
    pub basal_sliding_coeff: f32,
    pub surface_temperature_c: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstrophysicsDesiParams {
    pub dark_matter_core_rc_kpc: f32,
    pub stars_count: usize,
    pub target_b1_filaments: u32,
    pub z_score_vs_poisson: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackHoleEhtParams {
    pub mass_solar_masses: f32,
    pub dimensionless_spin_a: f32,
    pub isco_radius_rg: f32,
    pub ergosphere_radius_rg: f32,
    pub t_dual_effective_metric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallographyMagmaParams {
    pub space_group: String,
    pub lattice_constant_a_angstrom: f32,
    pub magma_temperature_k: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologicalFloraParams {
    pub murray_exponent: f32,
    pub canopy_height_max_m: f32,
    pub turing_wave_number_k: f32,
    pub target_b0_clusters: u32,
}

/// Global Master TNN & TDA Physics Registry
#[derive(Debug, Clone, Default)]
pub struct TnnPhysicsRegistry {
    pub ocean: Option<OceanJhtdbParams>,
    pub dunes: Option<DunesExnerParams>,
    pub cryosphere: Option<CryosphereEra5Params>,
    pub astrophysics: Option<AstrophysicsDesiParams>,
    pub black_hole: Option<BlackHoleEhtParams>,
    pub crystallography: Option<CrystallographyMagmaParams>,
    pub flora: Option<EcologicalFloraParams>,
}

impl TnnPhysicsRegistry {
    /// Loads all certified JSON datasets from assets/physics
    pub fn load_from_assets<P: AsRef<Path>>(assets_dir: P) -> Self {
        let base = assets_dir.as_ref();
        let mut reg = Self::default();

        // 1. Ocean JHTDB
        if let Ok(content) = fs::read_to_string(base.join("ocean_jhtdb_fno3d.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.ocean = Some(OceanJhtdbParams {
                        gravity_m_s2: p.get("gravity_m_s2").and_then(|x| x.as_f64()).unwrap_or(9.81) as f32,
                        kinematic_viscosity_m2_s: p.get("kinematic_viscosity_m2_s").and_then(|x| x.as_f64()).unwrap_or(1.05e-6) as f32,
                        surface_tension_N_m: p.get("surface_tension_N_m").and_then(|x| x.as_f64()).unwrap_or(0.0728) as f32,
                        enstrophy_cutoff_s2: p.get("enstrophy_cutoff_s2").and_then(|x| x.as_f64()).unwrap_or(50.0) as f32,
                        target_b1_vortices: 471,
                    });
                }
            }
        }

        // 2. Dunes Exner
        if let Ok(content) = fs::read_to_string(base.join("dunes_geomorphology_exner.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.dunes = Some(DunesExnerParams {
                        repose_angle_deg: p.get("repose_angle_deg").and_then(|x| x.as_f64()).unwrap_or(34.0) as f32,
                        max_slope_tan: p.get("max_slope_tan").and_then(|x| x.as_f64()).unwrap_or(0.6745) as f32,
                        sand_density_kg_m3: p.get("sand_density_kg_m3").and_then(|x| x.as_f64()).unwrap_or(1600.0) as f32,
                        target_b1_loops: 235,
                        target_b2_voids: 482,
                    });
                }
            }
        }

        // 3. Cryosphere ERA5
        if let Ok(content) = fs::read_to_string(base.join("cryosphere_era5_glacier.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.cryosphere = Some(CryosphereEra5Params {
                        glen_exponent_n: p.get("glen_flow_exponent_n").and_then(|x| x.as_f64()).unwrap_or(3.0) as f32,
                        ice_density_kg_m3: p.get("ice_density_kg_m3").and_then(|x| x.as_f64()).unwrap_or(917.0) as f32,
                        basal_sliding_coeff: p.get("basal_sliding_coeff_C").and_then(|x| x.as_f64()).unwrap_or(1e-10) as f32,
                        surface_temperature_c: p.get("surface_temperature_celsius").and_then(|x| x.as_f64()).unwrap_or(-15.0) as f32,
                    });
                }
            }
        }

        // 4. Astrophysics DESI
        if let Ok(content) = fs::read_to_string(base.join("astrophysics_desi_gaia_sympnet.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.astrophysics = Some(AstrophysicsDesiParams {
                        dark_matter_core_rc_kpc: p.get("dark_matter_core_rc_kpc").and_then(|x| x.as_f64()).unwrap_or(5.66) as f32,
                        stars_count: p.get("stars_count").and_then(|x| x.as_u64()).unwrap_or(300) as usize,
                        target_b1_filaments: 1186,
                        z_score_vs_poisson: 4.56,
                    });
                }
            }
        }

        // 5. Black Hole EHT
        if let Ok(content) = fs::read_to_string(base.join("blackhole_eht_tdual_grmhd.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.black_hole = Some(BlackHoleEhtParams {
                        mass_solar_masses: p.get("mass_solar_masses").and_then(|x| x.as_f64()).unwrap_or(6.5e9) as f32,
                        dimensionless_spin_a: p.get("dimensionless_spin_a").and_then(|x| x.as_f64()).unwrap_or(0.94) as f32,
                        isco_radius_rg: p.get("isco_radius_rg").and_then(|x| x.as_f64()).unwrap_or(2.04) as f32,
                        ergosphere_radius_rg: p.get("ergosphere_radius_rg").and_then(|x| x.as_f64()).unwrap_or(2.0) as f32,
                        t_dual_effective_metric: true,
                    });
                }
            }
        }

        // 6. Crystallography Magma
        if let Ok(content) = fs::read_to_string(base.join("crystallography_magma_egnn.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.crystallography = Some(CrystallographyMagmaParams {
                        space_group: p.get("space_group").and_then(|x| x.as_str()).unwrap_or("Fd-3m").to_string(),
                        lattice_constant_a_angstrom: p.get("lattice_constant_a_angstrom").and_then(|x| x.as_f64()).unwrap_or(3.567) as f32,
                        magma_temperature_k: p.get("magma_temperature_k").and_then(|x| x.as_f64()).unwrap_or(1473.15) as f32,
                    });
                }
            }
        }

        // 7. Ecological Flora GEDI
        if let Ok(content) = fs::read_to_string(base.join("ecological_flora_gedi_nca.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = v.get("physical_parameters") {
                    reg.flora = Some(EcologicalFloraParams {
                        murray_exponent: p.get("murray_exponent").and_then(|x| x.as_f64()).unwrap_or(3.0) as f32,
                        canopy_height_max_m: p.get("canopy_height_max_m").and_then(|x| x.as_f64()).unwrap_or(45.0) as f32,
                        turing_wave_number_k: p.get("turing_wave_number_k").and_then(|x| x.as_f64()).unwrap_or(0.125) as f32,
                        target_b0_clusters: 298,
                    });
                }
            }
        }

        reg
    }

    /// Hardcoded fallbacks if assets are not present
    pub fn fallback() -> Self {
        Self {
            ocean: Some(OceanJhtdbParams {
                gravity_m_s2: 9.81,
                kinematic_viscosity_m2_s: 1.05e-6,
                surface_tension_N_m: 0.0728,
                enstrophy_cutoff_s2: 50.0,
                target_b1_vortices: 471,
            }),
            dunes: Some(DunesExnerParams {
                repose_angle_deg: 34.0,
                max_slope_tan: 0.6745,
                sand_density_kg_m3: 1600.0,
                target_b1_loops: 235,
                target_b2_voids: 482,
            }),
            cryosphere: Some(CryosphereEra5Params {
                glen_exponent_n: 3.0,
                ice_density_kg_m3: 917.0,
                basal_sliding_coeff: 1e-10,
                surface_temperature_c: -15.0,
            }),
            astrophysics: Some(AstrophysicsDesiParams {
                dark_matter_core_rc_kpc: 5.66,
                stars_count: 300,
                target_b1_filaments: 1186,
                z_score_vs_poisson: 4.56,
            }),
            black_hole: Some(BlackHoleEhtParams {
                mass_solar_masses: 6.5e9,
                dimensionless_spin_a: 0.94,
                isco_radius_rg: 2.04,
                ergosphere_radius_rg: 2.0,
                t_dual_effective_metric: true,
            }),
            crystallography: Some(CrystallographyMagmaParams {
                space_group: "Fd-3m".to_string(),
                lattice_constant_a_angstrom: 3.567,
                magma_temperature_k: 1473.15,
            }),
            flora: Some(EcologicalFloraParams {
                murray_exponent: 3.0,
                canopy_height_max_m: 45.0,
                turing_wave_number_k: 0.125,
                target_b0_clusters: 298,
            }),
        }
    }
}
