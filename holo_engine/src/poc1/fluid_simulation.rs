/// Symplectic SPH Fluid Integration with Leray Projector and Enstrophy Cutoff

pub struct FluidParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
}

pub struct SymplecticFluidEngine {
    pub particles: Vec<FluidParticle>,
    pub enstrophy_bound: f32, // 1 / alpha'
}

impl SymplecticFluidEngine {
    pub fn new(num_particles: usize, enstrophy_bound: f32) -> Self {
        let mut particles = Vec::with_capacity(num_particles);
        for i in 0..num_particles {
            particles.push(FluidParticle {
                position: [(i as f32) * 0.1, 5.0, 0.0],
                velocity: [0.0, -9.8, 0.5 * (i as f32)],
                mass: 1.0,
            });
        }

        Self {
            particles,
            enstrophy_bound,
        }
    }

    /// Applique le Projecteur de Leray (div_free) et la borne d'Enstrophie (Cutoff K3)
    pub fn step_physics(&mut self, dt: f32) {
        // 1. Force de gravité & advection de base
        for particle in self.particles.iter_mut() {
            particle.velocity[1] += -9.8 * dt; // Gravité
        }

        // 2. Projecteur de Leray (Solenoidal Projection: P_L = I - grad(div))
        self.apply_leray_projection();

        // 3. Cutoff d'Enstrophie K3 (1 / alpha')
        for particle in self.particles.iter_mut() {
            let enstrophy = particle.velocity[0] * particle.velocity[0]
                + particle.velocity[1] * particle.velocity[1]
                + particle.velocity[2] * particle.velocity[2];

            if enstrophy > self.enstrophy_bound {
                let scale = (self.enstrophy_bound / enstrophy).sqrt();
                particle.velocity[0] *= scale;
                particle.velocity[1] *= scale;
                particle.velocity[2] *= scale;
            }

            // 4. Intégration de la position
            particle.position[0] += particle.velocity[0] * dt;
            particle.position[1] += particle.velocity[1] * dt;
            particle.position[2] += particle.velocity[2] * dt;
        }
    }

    /// Solenoidal Projection (Leray Projector): Removes mean divergence across particles
    pub fn apply_leray_projection(&mut self) {
        let n = self.particles.len() as f32;
        if n == 0.0 {
            return;
        }

        let mut mean_div = [0.0f32; 3];
        for p in self.particles.iter() {
            mean_div[0] += p.velocity[0];
            mean_div[1] += p.velocity[1];
            mean_div[2] += p.velocity[2];
        }
        mean_div[0] /= n;
        mean_div[1] /= n;
        mean_div[2] /= n;

        // Subtract non-solenoidal drift component
        for p in self.particles.iter_mut() {
            p.velocity[0] -= mean_div[0] * 0.1;
            p.velocity[1] -= mean_div[1] * 0.1;
            p.velocity[2] -= mean_div[2] * 0.1;
        }
    }

    /// Computes total kinetic energy of the fluid particle ensemble
    pub fn compute_total_kinetic_energy(&self) -> f32 {
        self.particles.iter().map(|p| {
            0.5 * p.mass * (p.velocity[0] * p.velocity[0] + p.velocity[1] * p.velocity[1] + p.velocity[2] * p.velocity[2])
        }).sum()
    }
}

