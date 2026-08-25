/// Biome & Biosphere Generator: Whittaker Thermodynamic Climate Model & L-System Fractal Trees
/// Uses T-Duality (R_eff = max(R, alpha'/R)) to bound fractal tree recursion at scale sqrt(alpha').

#[cfg(feature = "full")]
use bevy::prelude::*;

#[cfg(not(feature = "full"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[cfg(not(feature = "full"))]
impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt().max(0.0001);
        Vec3 { x: self.x / len, y: self.y / len, z: self.z / len }
    }
}

#[cfg(not(feature = "full"))]
impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

#[cfg(not(feature = "full"))]
impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Vec3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

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
            &mut positions,
            &mut indices,
        );

        (positions, indices)
    }

    fn generate_branch(
        &self,
        start: Vec3,
        dir: Vec3,
        length: f32,
        radius: f32,
        depth: u32,
        min_r: f32,
        positions: &mut Vec<Vec3>,
        indices: &mut Vec<u32>,
    ) {
        // Apply T-Duality Effective Radius Bound R_eff = max(R, alpha' / R)
        let r_eff = radius.max(self.alpha_prime / radius.max(0.001));

        let base_idx = positions.len() as u32;
        let end = start + dir * length;

        // Trunk segment vertices
        positions.push(start + Vec3::new(-r_eff, 0.0, 0.0));
        positions.push(start + Vec3::new(r_eff, 0.0, 0.0));
        positions.push(end + Vec3::new(-r_eff * 0.7, 0.0, 0.0));
        positions.push(end + Vec3::new(r_eff * 0.7, 0.0, 0.0));

        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx + 1, base_idx + 3, base_idx + 2,
        ]);

        // Quantum T-Duality Bound Cutoff: Stop fractal recursion if scale reaches sqrt(alpha')
        if depth > 0 && r_eff > min_r + 0.05 {
            let left_dir = (dir + Vec3::new(-0.4, 0.3, 0.0)).normalize();
            let right_dir = (dir + Vec3::new(0.4, 0.3, 0.0)).normalize();

            self.generate_branch(end, left_dir, length * 0.7, radius * 0.65, depth - 1, min_r, positions, indices);
            self.generate_branch(end, right_dir, length * 0.7, radius * 0.65, depth - 1, min_r, positions, indices);
        }
    }
}
