/// Topological World Engine Model & Preset Builder
/// Enables constructing diverse topological game worlds with customizable physics and AI resonance.

use crate::engine::physics::{PhysicsEngine, UniversalPhysicsConfig};
use crate::engine::renderer::TopologicalRenderPipeline;

use crate::poc2::burn_donn_inference::BurnDonnInferenceEngine;
use crate::poc2::tda_engine::{BettiNumbers, TdaEngine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldPreset {
    OrganicAlienWorld,      // Continuous 1-Lipschitz DONN terrain with 12 peaks
    QuantumFluidOcean,      // SPH Particles cascading with Leray solenoidal bounds
    TDualSingularity,       // Scale-invariant T-Duality LOD quantum bounce world
    CyberneticMesh,         // High Betti numbers TDA mesh network
    OceanNavierStokes,      // 2D/3D Multi-frequency Gerstner & Navier-Stokes solenoidal flow
    DunesAeolian,           // Aeolian sand transport PDE with 1-Lipschitz angle of repose
    CloudsConvection,       // 3D Thermal buoyancy & turbulent vorticity field
    GlacierIceSheet,        // Viscoplastic Shallow Ice Approximation (Glen's Flow Law)
    GalaxySymplecticNBody,  // Yoshida 4th-Order N-Body with DESI 5.66 kpc Dark Matter Core
    BlackHoleTDual,         // Relativistic spacetime with T-Dual geometric bounce
}

pub struct TopologicalWorld {
    pub name: String,
    pub preset: WorldPreset,
    pub physics: PhysicsEngine,
    pub renderer: TopologicalRenderPipeline,
    pub tda: TdaEngine,
    pub donn: BurnDonnInferenceEngine,
    pub particles: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

impl TopologicalWorld {
    pub fn new(name: &str, preset: WorldPreset) -> Self {
        let mut physics_config = UniversalPhysicsConfig::default();
        let particle_count;

        match preset {
            WorldPreset::OrganicAlienWorld => {
                physics_config.lipschitz_limit = 1.0;
                particle_count = 120;
            }
            WorldPreset::QuantumFluidOcean | WorldPreset::OceanNavierStokes => {
                physics_config.enstrophy_cap = 25.0;
                physics_config.leray_projection = true;
                particle_count = 300;
            }
            WorldPreset::TDualSingularity | WorldPreset::BlackHoleTDual => {
                physics_config.alpha_prime = 1.0;
                particle_count = 80;
            }
            WorldPreset::CyberneticMesh => {
                physics_config.tda_epsilon = 3.5;
                particle_count = 200;
            }
            WorldPreset::DunesAeolian => {
                physics_config.lipschitz_limit = 0.7; // ~34 deg repose
                particle_count = 250;
            }
            WorldPreset::CloudsConvection => {
                physics_config.enstrophy_cap = 15.0;
                particle_count = 350;
            }
            WorldPreset::GlacierIceSheet => {
                physics_config.lipschitz_limit = 0.5;
                particle_count = 180;
            }
            WorldPreset::GalaxySymplecticNBody => {
                physics_config.alpha_prime = 1.0;
                particle_count = 400;
            }
        }

        let mut particles = Vec::with_capacity(particle_count);
        let mut velocities = Vec::with_capacity(particle_count);

        for i in 0..particle_count {
            let angle = (i as f32) * 0.1;
            let radius = 2.0 + (i % 5) as f32 * 1.5;
            particles.push([
                radius * angle.cos(),
                1.0 + (i % 3) as f32 * 0.5,
                radius * angle.sin(),
            ]);
            velocities.push([(i % 3) as f32 - 1.0, 0.0, (i % 2) as f32 - 0.5]);
        }

        Self {
            name: name.to_string(),
            preset,
            physics: PhysicsEngine::new(physics_config),
            renderer: TopologicalRenderPipeline::new(1.0),
            tda: TdaEngine::new(4.5),
            donn: BurnDonnInferenceEngine::new(3, 1.0),
            particles,
            velocities,
        }
    }

    /// Interactive 1-Lipschitz Mining: Subtractions density at a target point
    pub fn interact_subtract_density(&mut self, hit_point: [f32; 3], radius: f32) -> usize {
        let mut count = 0;
        for (pos, vel) in self.particles.iter_mut().zip(self.velocities.iter_mut()) {
            let dx = pos[0] - hit_point[0];
            let dy = pos[1] - hit_point[1];
            let dz = pos[2] - hit_point[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist < radius {
                // Push particles outward smoothly (1-Lipschitz crater deformation)
                let factor = (radius - dist) / radius;
                pos[1] -= factor * 1.5;
                vel[1] -= factor * 2.0;
                count += 1;
            }
        }
        count
    }

    /// Step simulation tick
    pub fn step_simulation(&mut self, dt: f32) {
        self.physics.step(dt, &mut self.velocities);

        for (pos, vel) in self.particles.iter_mut().zip(self.velocities.iter()) {
            pos[0] += vel[0] * dt;
            pos[1] += vel[1] * dt;
            pos[2] += vel[2] * dt;

            // Floor boundary collision
            if pos[1] < 0.0 {
                pos[1] = 0.0;
            }
        }
    }

    /// Analyze topology of current world state
    pub fn compute_betti_numbers(&mut self) -> BettiNumbers {
        self.tda.particles = self
            .particles
            .iter()
            .enumerate()
            .map(|(idx, pos)| crate::poc2::tda_engine::SpatialParticle {
                position: *pos,
                density: 1.0,
                cluster_id: idx,
            })
            .collect();
        self.tda.compute_vietoris_rips_betti()
    }
}

