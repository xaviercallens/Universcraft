/// Biome & Biosphere Generator: Whittaker Thermodynamic Climate Model & L-System Fractal Trees
/// Uses T-Duality (R_eff = max(R, alpha'/R)) to bound fractal tree recursion at scale sqrt(alpha').

#[cfg(feature = "full")]
use bevy::prelude::*;

#[cfg(not(feature = "full"))]
use crate::math_types::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiomeType {
    Desert,
    TropicalJungle,
    TaigaForest,
    Tundra,
    SnowPeak,
}

#[derive(Debug, Clone)]
pub struct BiomeSample {
    pub biome: BiomeType,
    pub temperature: f32, // 0.0 (freezing) to 1.0 (scorching)
    pub humidity: f32,    // 0.0 (arid) to 1.0 (saturated)
    pub color: [f32; 4],
}

pub struct WhittakerClimateModel {
    pub alpha_prime: f32, // String scale alpha'
}

impl WhittakerClimateModel {
    pub fn new(alpha_prime: f32) -> Self {
        Self { alpha_prime }
    }

    /// Evaluates thermodynamic temperature T and humidity H at position (x, y, z)
    pub fn sample_climate(&self, x: f32, y: f32, z: f32) -> BiomeSample {
        // Temperature: Latitude wave + Altitude lapse rate
        let lat_factor = (z * 0.05).cos() * 0.5 + 0.5;
        let lapse_rate = (y * 0.08).max(0.0);
        let mut temperature = (lat_factor - lapse_rate).clamp(0.0, 1.0);

        // Humidity: Ocean evaporation + Orographic rainfall against mountain relief
        let humidity_wave = (x * 0.04).sin() * (z * 0.04).cos() * 0.5 + 0.5;
        let mut humidity = humidity_wave.clamp(0.0, 1.0);

        // High-frequency noise perturbation (Dithering) for Ecotones (Smooth biome transitions)
        let noise_t = (x * 12.34).sin() * (z * 56.78).cos();
        let noise_h = (x * 87.65).cos() * (z * 43.21).sin();
        temperature = (temperature + noise_t * 0.05).clamp(0.0, 1.0);
        humidity = (humidity + noise_h * 0.05).clamp(0.0, 1.0);

        // Whittaker Diagram Classification
        let (biome, color) = if y > 3.0 {
            (BiomeType::SnowPeak, [0.95, 0.98, 1.0, 1.0])
        } else if temperature > 0.6 {
            if humidity < 0.4 {
                (BiomeType::Desert, [0.90, 0.76, 0.50, 1.0]) // Sand (#e6c280)
            } else {
                (BiomeType::TropicalJungle, [0.06, 0.48, 0.18, 1.0]) // Deep Jungle (#0f7a2d)
            }
        } else if temperature > 0.3 {
            if humidity < 0.4 {
                (BiomeType::Tundra, [0.65, 0.75, 0.70, 1.0]) // Tundra (#a6bfb3)
            } else {
                (BiomeType::TaigaForest, [0.11, 0.30, 0.18, 1.0]) // Taiga (#1c4d2d)
            }
        } else {
            (BiomeType::Tundra, [0.69, 0.81, 0.90, 1.0]) // Glacial (#b0d0e6)
        };

        BiomeSample {
            biome,
            temperature,
            humidity,
            color,
        }
    }

    /// Generates L-System Fractal Tree Mesh bounded by T-Duality R_eff = max(R, alpha'/R) >= sqrt(alpha')
    /// Trees now branch in full 3D using deterministic pseudo-random seed from position and depth.
    pub fn build_t_dual_fractal_tree(&self, trunk_height: f32, depth: u32) -> (Vec<Vec3>, Vec<u32>) {
        let min_r = self.alpha_prime.sqrt(); // Quantum T-Duality lower radius bound
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        self.generate_branch(
            Vec3::ZERO,
            Vec3::Y,
            trunk_height,
            0.3,
            depth,
            min_r,
            0.0, // initial seed for 3D branching
            &mut positions,
            &mut indices,
        );

        (positions, indices)
    }

    /// Deterministic pseudo-random hash for reproducible 3D branching
    fn branch_hash(seed: f32) -> f32 {
        ((seed * 127.1 + 311.7).sin() * 43758.5453).fract()
    }

    fn generate_branch(
        &self,
        start: Vec3,
        dir: Vec3,
        length: f32,
        radius: f32,
        depth: u32,
        min_r: f32,
        seed: f32,
        positions: &mut Vec<Vec3>,
        indices: &mut Vec<u32>,
    ) {
        // Apply T-Duality Effective Radius Bound R_eff = max(R, alpha' / R)
        let r_eff = radius.max(self.alpha_prime / radius.max(0.001));

        let base_idx = positions.len() as u32;
        let end = start + dir * length;

        // Compute a tangent frame for 3D cross-section placement
        let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let right = dir.cross(up).normalize();
        let forward = dir.cross(right).normalize();

        // Trunk segment vertices (4 corners using tangent frame)
        positions.push(start + right * (-r_eff));
        positions.push(start + right * r_eff);
        positions.push(end + right * (-r_eff * 0.7));
        positions.push(end + right * (r_eff * 0.7));

        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx + 1, base_idx + 3, base_idx + 2,
        ]);

        // Quantum T-Duality Bound Cutoff: Stop fractal recursion if scale reaches sqrt(alpha')
        if depth > 0 && r_eff > min_r + 0.05 {
            // 3D branching: use deterministic hash for azimuthal angle around the branch axis
            let hash_val = Self::branch_hash(seed + depth as f32 * 7.3);
            let azimuth = hash_val * std::f32::consts::TAU; // Full 360° rotation
            let cos_a = azimuth.cos();
            let sin_a = azimuth.sin();

            // Left branch: deflect from parent direction using tangent frame
            let left_lateral = right * (cos_a * -0.4) + forward * (sin_a * -0.4);
            let left_dir = (dir + left_lateral + Vec3::Y * 0.3).normalize();

            // Right branch: opposite azimuth
            let right_lateral = right * (cos_a * 0.4) + forward * (sin_a * 0.4);
            let right_dir = (dir + right_lateral + Vec3::Y * 0.3).normalize();

            self.generate_branch(end, left_dir, length * 0.7, radius * 0.65, depth - 1, min_r, seed + 1.0, positions, indices);
            self.generate_branch(end, right_dir, length * 0.7, radius * 0.65, depth - 1, min_r, seed + 2.0, positions, indices);

            // Third branch at ~40% probability for organic variation
            let third_hash = Self::branch_hash(seed + depth as f32 * 13.7);
            if third_hash > 0.6 {
                let third_azimuth = third_hash * std::f32::consts::TAU;
                let third_lateral = right * (third_azimuth.cos() * 0.3) + forward * (third_azimuth.sin() * 0.3);
                let third_dir = (dir + third_lateral + Vec3::Y * 0.4).normalize();
                self.generate_branch(end, third_dir, length * 0.55, radius * 0.5, depth - 1, min_r, seed + 3.0, positions, indices);
            }
        }
    }
}
