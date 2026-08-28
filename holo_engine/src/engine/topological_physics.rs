/// Unified Topological Physics System (Universcraft Engine)
/// Integrates all 6 physical biomes, computes real-time TDA Betti invariants,
/// and streams physical observables to rendering and game loops.

use crate::engine::physical_biomes::{
    BlackHoleConfig, BlackHoleSpacetime, CloudConfig, CloudField, CrystallographyMagmaConfig,
    CrystallographyMagmaState, DuneConfig, DuneGrid, EcologicalFloraConfig, EcologicalFloraState,
    GalaxySymplecticSystem, GlacierConfig, GlacierModel, OceanConfig, OceanState,
};
use crate::poc2::tda_engine::{BettiNumbers, TdaEngine, SpatialParticle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveBiome {
    Ocean,
    Dunes,
    Clouds,
    Glacier,
    Galaxy,
    BlackHole,
    Crystallography,
    EcologicalFlora,
}

pub struct TopologicalPhysicsSystem {
    pub active_biome: ActiveBiome,
    pub ocean: OceanState,
    pub dunes: DuneGrid,
    pub clouds: CloudField,
    pub glacier: GlacierModel,
    pub galaxy: GalaxySymplecticSystem,
    pub black_hole: BlackHoleSpacetime,
    pub crystallography: CrystallographyMagmaState,
    pub ecological_flora: EcologicalFloraState,
    pub tda: TdaEngine,
    pub step_counter: usize,
    pub current_hamiltonian: f32,
    pub initial_hamiltonian: f32,
    pub latest_betti: BettiNumbers,
}

impl TopologicalPhysicsSystem {
    pub fn new(active_biome: ActiveBiome) -> Self {
        let galaxy = GalaxySymplecticSystem::new(250, 5.66); // 5.66 kpc DESI Dark Matter Core
        let init_h = galaxy.compute_hamiltonian();

        Self {
            active_biome,
            ocean: OceanState::new(OceanConfig::default()),
            dunes: DuneGrid::new(64, 0.5, DuneConfig::default()),
            clouds: CloudField::new(CloudConfig::default()),
            glacier: GlacierModel::new(64, 1.0, GlacierConfig::default()),
            galaxy,
            black_hole: BlackHoleSpacetime::new(BlackHoleConfig::default()),
            crystallography: CrystallographyMagmaState::new(CrystallographyMagmaConfig::default()),
            ecological_flora: EcologicalFloraState::new(EcologicalFloraConfig::default()),
            tda: TdaEngine::new(4.5),
            step_counter: 0,
            current_hamiltonian: init_h,
            initial_hamiltonian: init_h,
            latest_betti: BettiNumbers::default(),
        }
    }

    /// Step the active physics biome by dt
    pub fn step(&mut self, dt: f32) {
        self.step_counter += 1;

        match self.active_biome {
            ActiveBiome::Ocean => {
                self.ocean.step(dt);
            }
            ActiveBiome::Dunes => {
                self.dunes.step(dt);
            }
            ActiveBiome::Clouds => {
                self.clouds.step(dt);
            }
            ActiveBiome::Glacier => {
                self.glacier.step(dt);
            }
            ActiveBiome::Galaxy => {
                self.galaxy.step_yoshida(dt);
                self.current_hamiltonian = self.galaxy.compute_hamiltonian();
            }
            ActiveBiome::BlackHole => {
                // Relativistic spacetime geodesics are analytical
            }
            ActiveBiome::Crystallography => {
                self.crystallography.time += dt;
            }
            ActiveBiome::EcologicalFlora => {
                self.ecological_flora.time += dt;
            }
        }

        // Periodically extract TDA topological invariants
        if self.step_counter % 30 == 0 {
            self.update_topological_invariants();
        }
    }

    /// Extract TDA Betti numbers (b0: clusters/halos, b1: loops/filaments, b2: voids)
    pub fn update_topological_invariants(&mut self) {
        let mut particles = Vec::new();

        match self.active_biome {
            ActiveBiome::Galaxy => {
                for (idx, b) in self.galaxy.bodies.iter().enumerate() {
                    particles.push(SpatialParticle {
                        position: b.position,
                        density: b.mass,
                        cluster_id: idx,
                    });
                }
            }
            ActiveBiome::Ocean => {
                for i in 0..100 {
                    let x = (i % 10) as f32 * 2.0 - 10.0;
                    let z = (i / 10) as f32 * 2.0 - 10.0;
                    let (pos, _) = self.ocean.sample_surface_and_velocity(x, z);
                    particles.push(SpatialParticle {
                        position: pos,
                        density: 1.0,
                        cluster_id: i,
                    });
                }
            }
            ActiveBiome::Crystallography => {
                for i in 0..64 {
                    let x = ((i % 4) as f32 - 1.5) * 1.5;
                    let y = (((i / 4) % 4) as f32 - 1.5) * 1.5;
                    let z = ((i / 16) as f32 - 1.5) * 1.5;
                    particles.push(SpatialParticle {
                        position: [x, y, z],
                        density: 1.0,
                        cluster_id: i,
                    });
                }
            }
            _ => {
                // Sample generic probe points
                for i in 0..50 {
                    particles.push(SpatialParticle {
                        position: [(i as f32 * 0.5).sin() * 5.0, (i as f32 * 0.2).cos() * 2.0, (i as f32 * 0.3).sin() * 5.0],
                        density: 1.0,
                        cluster_id: i,
                    });
                }
            }
        }

        self.tda.particles = particles;
        self.latest_betti = self.tda.compute_vietoris_rips_betti();
    }

    /// Relative Energy Invariant drift: delta H / H_0 (should be <= 1e-6 via Yoshida integrator)
    pub fn get_hamiltonian_drift(&self) -> f32 {
        if self.initial_hamiltonian.abs() > 1e-6 {
            (self.current_hamiltonian - self.initial_hamiltonian).abs() / self.initial_hamiltonian.abs()
        } else {
            0.0
        }
    }
}

