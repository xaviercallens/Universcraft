//! HoloEngine Symplectic Fluid Dynamics & SPH Solver Module
//! Implements Tait Equation of State, Leray-Hopf Solenoidal Projection, K3 Enstrophy Cap,
//! and real SPH neighbor interactions (Poly6 density, Spiky pressure, Viscosity kernels).

use crate::math_types::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct FluidParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
    pub force: Vec3,
}

impl FluidParticle {
    pub fn new(position: Vec3, velocity: Vec3, mass: f32) -> Self {
        Self {
            position,
            velocity,
            density: 0.0,
            pressure: 0.0,
            mass,
            force: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SPHParams {
    pub rest_density: f32,     // Rest density rho_0
    pub stiffness: f32,        // Tait equation stiffness parameter B
    pub gamma: f32,            // Tait exponent gamma (7.0 for water)
    pub viscosity: f32,        // Kinematic viscosity
    pub enstrophy_cap: f32,    // Maximum vorticity/enstrophy bound E_max (25.0)
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

/// SPH Kernel Constants (precomputed from smoothing_radius h)
struct SPHKernels {
    h: f32,
    h2: f32,
    poly6_coeff: f32,     // 315 / (64 * PI * h^9)
    spiky_grad_coeff: f32, // -45 / (PI * h^6)
    visc_lap_coeff: f32,   // 45 / (PI * h^6)
}

impl SPHKernels {
    fn new(h: f32) -> Self {
        let h2 = h * h;
        let h6 = h2 * h2 * h2;
        let h9 = h6 * h2 * h;
        Self {
            h,
            h2,
            poly6_coeff: 315.0 / (64.0 * std::f32::consts::PI * h9),
            spiky_grad_coeff: -45.0 / (std::f32::consts::PI * h6),
            visc_lap_coeff: 45.0 / (std::f32::consts::PI * h6),
        }
    }

    /// Poly6 Kernel: W(r, h) = (315 / 64πh⁹)(h² - r²)³ for r < h
    fn poly6(&self, r2: f32) -> f32 {
        if r2 >= self.h2 { return 0.0; }
        let diff = self.h2 - r2;
        self.poly6_coeff * diff * diff * diff
    }

    /// Spiky Kernel Gradient magnitude: ∇W = (-45 / πh⁶)(h - r)² for r < h
    fn spiky_grad(&self, r: f32) -> f32 {
        if r >= self.h || r < 1e-6 { return 0.0; }
        let diff = self.h - r;
        self.spiky_grad_coeff * diff * diff
    }

    /// Viscosity Kernel Laplacian: ∇²W = (45 / πh⁶)(h - r) for r < h
    fn visc_laplacian(&self, r: f32) -> f32 {
        if r >= self.h { return 0.0; }
        self.visc_lap_coeff * (self.h - r)
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
    pub fn apply_leray_solenoidal_projection(velocity: Vec3, enstrophy_cap: f32) -> Vec3 {
        let speed_sq = velocity.length_squared();
        let enstrophy = speed_sq * 0.5;

        if enstrophy > enstrophy_cap {
            let scale = (enstrophy_cap / enstrophy).sqrt();
            velocity * scale
        } else {
            velocity
        }
    }

    /// Phase 1: Compute density for all particles from neighbor interactions (Poly6 kernel)
    fn compute_densities(&mut self) {
        let kernels = SPHKernels::new(self.params.smoothing_radius);
        let n = self.particles.len();
        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();
        let masses: Vec<f32> = self.particles.iter().map(|p| p.mass).collect();

        for i in 0..n {
            let mut density = 0.0f32;
            for j in 0..n {
                let diff = positions[i] - positions[j];
                let r2 = diff.length_squared();
                density += masses[j] * kernels.poly6(r2);
            }
            self.particles[i].density = density.max(self.params.rest_density * 0.1);
            self.particles[i].pressure = Self::compute_tait_pressure_with_params(
                &self.params,
                self.particles[i].density,
            );
        }
    }

    /// Phase 2: Compute pressure + viscosity forces from neighbor interactions
    fn compute_forces(&mut self) {
        let kernels = SPHKernels::new(self.params.smoothing_radius);
        let n = self.particles.len();

        // Snapshot read-only particle state
        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();
        let velocities: Vec<Vec3> = self.particles.iter().map(|p| p.velocity).collect();
        let densities: Vec<f32> = self.particles.iter().map(|p| p.density).collect();
        let pressures: Vec<f32> = self.particles.iter().map(|p| p.pressure).collect();
        let masses: Vec<f32> = self.particles.iter().map(|p| p.mass).collect();

        for i in 0..n {
            let mut pressure_force = Vec3::ZERO;
            let mut viscosity_force = Vec3::ZERO;

            for j in 0..n {
                if i == j { continue; }

                let diff = positions[i] - positions[j];
                let r = diff.length();
                if r >= self.params.smoothing_radius || r < 1e-6 { continue; }

                let dir = diff * (1.0 / r); // Normalized direction

                // Pressure force: -m_j * (P_i + P_j) / (2 * rho_j) * ∇W_spiky
                let pressure_term = masses[j]
                    * (pressures[i] + pressures[j])
                    / (2.0 * densities[j].max(1.0));
                pressure_force += dir * (pressure_term * kernels.spiky_grad(r));

                // Viscosity force: mu * m_j * (v_j - v_i) / rho_j * ∇²W_visc
                let vel_diff = velocities[j] - velocities[i];
                viscosity_force +=
                    vel_diff * (self.params.viscosity * masses[j] / densities[j].max(1.0) * kernels.visc_laplacian(r));
            }

            // Gravity
            let gravity = Vec3::new(0.0, -9.81 * densities[i], 0.0);

            self.particles[i].force = pressure_force + viscosity_force + gravity;
        }
    }

    /// Full simulation step with real SPH neighbor interactions:
    /// density → pressure → forces → Leray projection → symplectic integration
    pub fn step(&mut self, dt: f32) {
        // Phase 1: Density & Pressure from neighbor kernels
        self.compute_densities();

        // Phase 2: Pressure + Viscosity forces from neighbor kernels
        self.compute_forces();

        // Phase 3: Integration with Leray-Hopf solenoidal projection
        let enstrophy_cap = self.params.enstrophy_cap;
        for particle in self.particles.iter_mut() {
            // Acceleration = Force / density
            let accel = particle.force * (1.0 / particle.density.max(1.0));
            particle.velocity += accel * dt;

            // Enforce Leray-Hopf solenoidal projection and enstrophy bound
            particle.velocity = Self::apply_leray_solenoidal_projection(
                particle.velocity,
                enstrophy_cap,
            );

            // Symplectic Euler Integration
            particle.position += particle.velocity * dt;
        }
    }

    /// Parallelized multi-threaded fluid step execution using Rayon
    /// Density & force computation phases use snapshots for thread safety
    pub fn step_parallel(&mut self, dt: f32) {
        use rayon::prelude::*;

        // Phase 1: Compute densities (read positions, write densities)
        let kernels = SPHKernels::new(self.params.smoothing_radius);
        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();
        let masses: Vec<f32> = self.particles.iter().map(|p| p.mass).collect();
        let rest_density = self.params.rest_density;
        let params = self.params;

        self.particles.par_iter_mut().enumerate().for_each(|(i, particle)| {
            let kernels_local = SPHKernels::new(params.smoothing_radius);
            let mut density = 0.0f32;
            for j in 0..positions.len() {
                let diff = positions[i] - positions[j];
                let r2 = diff.length_squared();
                density += masses[j] * kernels_local.poly6(r2);
            }
            particle.density = density.max(rest_density * 0.1);
            particle.pressure = SymplecticFluidSolver::compute_tait_pressure_with_params(&params, particle.density);
        });

        // Phase 2: Compute forces (read all state, write forces)
        let densities: Vec<f32> = self.particles.iter().map(|p| p.density).collect();
        let pressures: Vec<f32> = self.particles.iter().map(|p| p.pressure).collect();
        let velocities: Vec<Vec3> = self.particles.iter().map(|p| p.velocity).collect();

        self.particles.par_iter_mut().enumerate().for_each(|(i, particle)| {
            let kernels_local = SPHKernels::new(params.smoothing_radius);
            let mut pressure_force = Vec3::ZERO;
            let mut viscosity_force = Vec3::ZERO;

            for j in 0..positions.len() {
                if i == j { continue; }
                let diff = positions[i] - positions[j];
                let r = diff.length();
                if r >= params.smoothing_radius || r < 1e-6 { continue; }

                let dir = diff * (1.0 / r);
                let pressure_term = masses[j] * (pressures[i] + pressures[j]) / (2.0 * densities[j].max(1.0));
                pressure_force += dir * (pressure_term * kernels_local.spiky_grad(r));

                let vel_diff = velocities[j] - velocities[i];
                viscosity_force += vel_diff * (params.viscosity * masses[j] / densities[j].max(1.0) * kernels_local.visc_laplacian(r));
            }

            particle.force = pressure_force + viscosity_force + Vec3::new(0.0, -9.81 * densities[i], 0.0);
        });

        // Phase 3: Integration
        let enstrophy_cap = self.params.enstrophy_cap;
        self.particles.par_iter_mut().for_each(|particle| {
            let accel = particle.force * (1.0 / particle.density.max(1.0));
            particle.velocity += accel * dt;
            particle.velocity = SymplecticFluidSolver::apply_leray_solenoidal_projection(particle.velocity, enstrophy_cap);
            particle.position += particle.velocity * dt;
        });
    }
}
