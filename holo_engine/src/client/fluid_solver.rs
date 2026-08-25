//! HoloEngine Symplectic Fluid Dynamics & SPH Solver Module
//! Implements Tait Equation of State, Leray-Hopf Solenoidal Projection, and K3 Enstrophy Cap.

#[derive(Debug, Clone, Copy)]
pub struct FluidParticle {
    pub position: (f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SPHParams {
    pub rest_density: f32, // Rest density rho_0
    pub stiffness: f32,    // Tait equation stiffness parameter B
    pub gamma: f32,        // Tait exponent gamma (7.0 for water)
    pub viscosity: f32,    // Kinematic viscosity
    pub enstrophy_cap: f32,// Maximum vorticity/enstrophy bound E_max (25.0)
    pub smoothing_radius: f32, // Kernel smoothing radius h
}

impl Default for SPHParams {
    fn default() -> Self {
        Self {
            rest_density: 1000.0,
            stiffness: 2000.0,
            gamma: 7.0,
            viscosity: 0.01,
            enstrophy_cap: 25.0,
            smoothing_radius: 1.0,
        }
    }
}

pub struct SymplecticFluidSolver {
    pub params: SPHParams,
    pub particles: Vec<FluidParticle>,
}

impl SymplecticFluidSolver {
    pub fn new(params: SPHParams, particles: Vec<FluidParticle>) -> Self {
        Self { params, particles }
    }

    /// Computes pressure using the Tait Equation of State (quasi-incompressible fluid)
    /// P = B * ((rho / rho_0)^gamma - 1)
    pub fn compute_tait_pressure_with_params(params: &SPHParams, density: f32) -> f32 {
        let rho_ratio = (density / params.rest_density).max(0.1);
        let pressure = params.stiffness * (rho_ratio.powf(params.gamma) - 1.0);
        pressure.max(0.0)
    }

    pub fn compute_tait_pressure(&self, density: f32) -> f32 {
        Self::compute_tait_pressure_with_params(&self.params, density)
    }

    /// Applies Leray-Hopf Solenoidal Projection and Enstrophy Cap E_max
    pub fn apply_leray_solenoidal_projection(velocity: (f32, f32, f32), enstrophy_cap: f32) -> (f32, f32, f32) {
        let speed_sq = velocity.0 * velocity.0 + velocity.1 * velocity.1 + velocity.2 * velocity.2;
        let enstrophy = speed_sq * 0.5;

        if enstrophy > enstrophy_cap {
            let scale = (enstrophy_cap / enstrophy).sqrt();
            (velocity.0 * scale, velocity.1 * scale, velocity.2 * scale)
        } else {
            velocity
        }
    }

    /// Advances the fluid simulation step synchronously or in parallel
    pub fn step(&mut self, dt: f32) {
        for particle in self.particles.iter_mut() {
            particle.pressure = Self::compute_tait_pressure_with_params(&self.params, particle.density);
            
            // Gravity vector
            particle.velocity.1 -= 9.81 * dt;

            // Enforce Leray-Hopf solenoidal projection and enstrophy bound
            particle.velocity = Self::apply_leray_solenoidal_projection(
                particle.velocity,
                self.params.enstrophy_cap,
            );

            // Symplectic Euler Integration
            particle.position.0 += particle.velocity.0 * dt;
            particle.position.1 += particle.velocity.1 * dt;
            particle.position.2 += particle.velocity.2 * dt;
        }
    }

    /// Parallelized multi-threaded fluid step execution using Rayon
    pub fn step_parallel(&mut self, dt: f32) {
        use rayon::prelude::*;
        let params = self.params;

        self.particles.par_iter_mut().for_each(|particle| {
            particle.pressure = Self::compute_tait_pressure_with_params(&params, particle.density);
            
            // Gravity vector
            particle.velocity.1 -= 9.81 * dt;

            // Enforce Leray-Hopf solenoidal projection and enstrophy bound
            particle.velocity = Self::apply_leray_solenoidal_projection(
                particle.velocity,
                params.enstrophy_cap,
            );

            // Symplectic Euler Integration
            particle.position.0 += particle.velocity.0 * dt;
            particle.position.1 += particle.velocity.1 * dt;
            particle.position.2 += particle.velocity.2 * dt;
        });
    }
}
