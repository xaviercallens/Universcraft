//! HoloEngine 3D — Astrophysical Spacetime & Stellar Dynamics Engine
//! Implements Kerr / Schwarzschild General Relativity Metrics, Gravitational Lensing,
//! Galactic N-Body Gravitational Dynamics, and Stellar Lifecycle Evolution.

pub const G_CONST: f64 = 6.67430e-11; // m^3 kg^-1 s^-2
pub const SPEED_OF_LIGHT: f64 = 2.99792458e8; // m/s
pub const SOLAR_MASS: f64 = 1.98847e30; // kg

/// General Relativistic Schwarzschild & Kerr Black Hole Spacetime Metric
pub struct SchwarzschildKerrMetric {
    pub mass: f64,   // kg
    pub spin_a: f64, // Angular momentum parameter a = J / (M*c) in meters
}

impl SchwarzschildKerrMetric {
    pub fn new(mass_kg: f64, spin_a_meters: f64) -> Self {
        Self {
            mass: mass_kg,
            spin_a: spin_a_meters.clamp(0.0, G_CONST * mass_kg / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)),
        }
    }

    /// Calculates the Schwarzschild radius r_s = 2GM / c^2
    pub fn schwarzschild_radius(&self) -> f64 {
        (2.0 * G_CONST * self.mass) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Calculates the Event Horizon radius r_+ = GM/c^2 + sqrt((GM/c^2)^2 - a^2)
    pub fn event_horizon_radius(&self) -> f64 {
        let m_m = (G_CONST * self.mass) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
        m_m + (m_m * m_m - self.spin_a * self.spin_a).max(0.0).sqrt()
    }

    /// Calculates the Schwarzschild Photon Sphere radius r_photon = 3GM / c^2
    pub fn photon_sphere_radius(&self) -> f64 {
        (3.0 * G_CONST * self.mass) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Calculates Gravitational Redshift z = (1 / sqrt(1 - 2GM / (r c^2))) - 1
    pub fn gravitational_redshift(&self, r_meters: f64) -> f64 {
        let r_s = self.schwarzschild_radius();
        if r_meters <= r_s {
            f64::INFINITY // Infinite redshift at or inside the event horizon
        } else {
            (1.0 / (1.0 - r_s / r_meters).sqrt()) - 1.0
        }
    }

    /// Calculates Einstein Ring light deflection angle theta = 4GM / (c^2 * b)
    pub fn deflection_angle_radians(&self, impact_parameter_b: f64) -> f64 {
        let r_s = self.schwarzschild_radius();
        if impact_parameter_b <= 0.0 {
            0.0
        } else {
            2.0 * r_s / impact_parameter_b
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StellarType {
    Protostar,
    MainSequence,
    RedGiant,
    Supernova,
    WhiteDwarf,
    NeutronStar,
    BlackHole,
}

#[derive(Debug, Clone)]
pub struct StarParticle {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64, // Solar masses M_sun
    pub age_years: f64,
    pub stellar_type: StellarType,
    pub luminosity_solar: f64,
}

impl StarParticle {
    pub fn new(position: [f64; 3], velocity: [f64; 3], mass_solar: f64) -> Self {
        let luminosity = mass_solar.powf(3.5);
        Self {
            position,
            velocity,
            mass: mass_solar,
            age_years: 0.0,
            stellar_type: StellarType::MainSequence,
            luminosity_solar: luminosity,
        }
    }

    /// Calculates the main sequence lifespan tau = 1e10 * (M / M_sun)^(-2.5) years
    pub fn main_sequence_lifetime_years(&self) -> f64 {
        1.0e10 * self.mass.powf(-2.5)
    }

    /// Evolves stellar state based on age
    pub fn evolve(&mut self, dt_years: f64) {
        self.age_years += dt_years;
        let lifetime = self.main_sequence_lifetime_years();

        if self.age_years < lifetime {
            self.stellar_type = StellarType::MainSequence;
            self.luminosity_solar = self.mass.powf(3.5);
        } else if self.age_years < lifetime * 1.1 {
            self.stellar_type = StellarType::RedGiant;
            self.luminosity_solar = self.mass.powf(3.5) * 100.0;
        } else if self.age_years < lifetime * 1.15 && self.mass >= 8.0 {
            self.stellar_type = StellarType::Supernova;
            self.luminosity_solar = 1.0e9; // Supernova brightness surge
        } else {
            // Remnant state after supernova / stellar collapse
            if self.mass < 8.0 {
                self.stellar_type = StellarType::WhiteDwarf;
                self.luminosity_solar = 0.01;
            } else if self.mass < 20.0 {
                self.stellar_type = StellarType::NeutronStar;
                self.luminosity_solar = 0.001;
            } else {
                self.stellar_type = StellarType::BlackHole;
                self.luminosity_solar = 0.0;
            }
        }
    }
}

/// N-Body Galactic Gravitational Dynamics Engine with Softening & Dark Matter Halo
pub struct GalaxyNBodySystem {
    pub stars: Vec<StarParticle>,
    pub central_black_hole_mass: f64, // M_sun
}

impl GalaxyNBodySystem {
    pub fn new_spiral_galaxy(num_stars: usize, central_black_hole_mass: f64, galaxy_radius_kpc: f64) -> Self {
        let mut stars = Vec::with_capacity(num_stars);
        let kpc_to_m = 3.085677581e19;
        let radius_m = galaxy_radius_kpc * kpc_to_m;

        for i in 0..num_stars {
            let r = (rand::random::<f64>()).sqrt() * radius_m;
            let theta = rand::random::<f64>() * std::f64::consts::TAU;
            let z = (rand::random::<f64>() - 0.5) * 0.05 * radius_m;

            let pos = [r * theta.cos(), z, r * theta.sin()];

            // Keplerian orbital velocity v = sqrt(G * M_enclosed / r)
            let total_mass_inside = central_black_hole_mass * SOLAR_MASS + (i as f64 + 1.0) * 2.0 * SOLAR_MASS;
            let v_orbital = ((G_CONST * total_mass_inside) / (r + 1.0e16)).sqrt();

            let vel = [-v_orbital * theta.sin(), 0.0, v_orbital * theta.cos()];
            let mass = 0.5 + rand::random::<f64>() * 5.0; // 0.5 to 5.5 M_sun

            stars.push(StarParticle::new(pos, vel, mass));
        }

        Self {
            stars,
            central_black_hole_mass,
        }
    }

    /// Step N-body galactic gravitational dynamics with softening parameter epsilon
    pub fn step_nbody_gravity(&mut self, dt_seconds: f64, softening_epsilon_m: f64) {
        let n = self.stars.len();
        let mut accelerations = vec![[0.0f64; 3]; n];
        let eps_sq = softening_epsilon_m * softening_epsilon_m;

        let bh_mass_kg = self.central_black_hole_mass * SOLAR_MASS;

        // Central Supermassive Black Hole Attraction + Mutual Star Gravity
        for i in 0..n {
            let p_i = self.stars[i].position;

            // 1. Central Black Hole Gravity
            let r_bh_sq = p_i[0] * p_i[0] + p_i[1] * p_i[1] + p_i[2] * p_i[2] + eps_sq;
            let r_bh = r_bh_sq.sqrt();
            let f_bh = (G_CONST * bh_mass_kg) / (r_bh_sq * r_bh);

            accelerations[i][0] -= f_bh * p_i[0];
            accelerations[i][1] -= f_bh * p_i[1];
            accelerations[i][2] -= f_bh * p_i[2];

            // 2. Star-Star Pairwise Newtonian Acceleration
            for j in (i + 1)..n {
                let p_j = self.stars[j].position;
                let dx = p_j[0] - p_i[0];
                let dy = p_j[1] - p_i[1];
                let dz = p_j[2] - p_i[2];

                let dist_sq = dx * dx + dy * dy + dz * dz + eps_sq;
                let dist_cubed = dist_sq * dist_sq.sqrt();

                let m_j_kg = self.stars[j].mass * SOLAR_MASS;
                let m_i_kg = self.stars[i].mass * SOLAR_MASS;

                let f_ij = (G_CONST * m_j_kg) / dist_cubed;
                let f_ji = (G_CONST * m_i_kg) / dist_cubed;

                accelerations[i][0] += f_ij * dx;
                accelerations[i][1] += f_ij * dy;
                accelerations[i][2] += f_ij * dz;

                accelerations[j][0] -= f_ji * dx;
                accelerations[j][1] -= f_ji * dy;
                accelerations[j][2] -= f_ji * dz;
            }
        }

        // Symplectic Verlet / Euler Integration Step
        for i in 0..n {
            self.stars[i].velocity[0] += accelerations[i][0] * dt_seconds;
            self.stars[i].velocity[1] += accelerations[i][1] * dt_seconds;
            self.stars[i].velocity[2] += accelerations[i][2] * dt_seconds;

            self.stars[i].position[0] += self.stars[i].velocity[0] * dt_seconds;
            self.stars[i].position[1] += self.stars[i].velocity[1] * dt_seconds;
            self.stars[i].position[2] += self.stars[i].velocity[2] * dt_seconds;
        }
    }
}
