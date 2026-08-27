//! HoloEngine 3D — Baroclinic Multi-Layer Global Climate Engine
//! Implements 3D Hydrostatic Atmospheric Pressure Gradients, Coriolis Deflection,
//! Hadley/Ferrel Circulation Cells, Moisture Transport, and Condensation Phase Changes.

pub const EARTH_OMEGA: f32 = 7.292115e-5; // rad/s (Earth rotation rate)
pub const GRAVITY: f32 = 9.80665; // m/s^2
pub const SPECIFIC_HEAT_AIR: f32 = 1004.0; // J/(kg K)
pub const LATENT_HEAT_VAPORIZATION: f32 = 2.26e6; // J/kg

#[derive(Debug, Clone)]
pub struct AtmosphericCell {
    pub temperature_k: f32,
    pub pressure_pa: f32,
    pub specific_humidity: f32, // kg water / kg air
    pub wind_velocity: [f32; 3], // [u (east), v (north), w (up)]
    pub cloud_water_content: f32,
}

pub struct BaroclinicClimateGrid {
    pub dim_x: usize,
    pub dim_y: usize,
    pub dim_z: usize,
    pub grid_spacing_m: f32,
    pub cells: Vec<AtmosphericCell>,
}

impl BaroclinicClimateGrid {
    pub fn new(dim_x: usize, dim_y: usize, dim_z: usize, grid_spacing_m: f32) -> Self {
        let total = dim_x * dim_y * dim_z;
        let mut cells = Vec::with_capacity(total);

        for z in 0..dim_z {
            let height_m = z as f32 * grid_spacing_m;
            // Standard atmosphere lapse rate T(z) = 288.15 - 0.0065 * z
            let t_z = (288.15 - 0.0065 * height_m).max(210.0);
            // Hydrostatic barometric formula P(z) = P0 * exp(-M g z / R T)
            let p_z = 101325.0 * (-GRAVITY * height_m / (287.05 * t_z)).exp();

            for y in 0..dim_y {
                let lat_rad = ((y as f32 / dim_y as f32) - 0.5) * std::f32::consts::PI;
                // Equator warmer than poles
                let equator_warmth = lat_rad.cos() * 25.0;

                for _x in 0..dim_x {
                    cells.push(AtmosphericCell {
                        temperature_k: t_z + equator_warmth,
                        pressure_pa: p_z,
                        specific_humidity: (0.015 * lat_rad.cos()).max(0.0001),
                        wind_velocity: [0.0, 0.0, 0.0],
                        cloud_water_content: 0.0,
                    });
                }
            }
        }

        Self {
            dim_x,
            dim_y,
            dim_z,
            grid_spacing_m,
            cells,
        }
    }

    #[inline]
    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.dim_x + z * self.dim_x * self.dim_y
    }

    /// Computes Coriolis acceleration vector F_cor = -2 * Omega x v
    pub fn compute_coriolis_acceleration(&self, y_index: usize, velocity: [f32; 3]) -> [f32; 3] {
        let lat_rad = ((y_index as f32 / self.dim_y as f32) - 0.5) * std::f32::consts::PI;
        let f = 2.0 * EARTH_OMEGA * lat_rad.sin(); // Coriolis parameter f = 2 Omega sin(phi)

        // u_accel = f * v, v_accel = -f * u
        [f * velocity[1], -f * velocity[0], 0.0]
    }

    /// Executes baroclinic multi-layer atmosphere update step
    pub fn step_baroclinic_circulation(&mut self, dt_seconds: f32) {
        let mut new_cells = self.cells.clone();

        for z in 0..self.dim_z {
            for y in 1..(self.dim_y - 1) {
                for x in 0..self.dim_x {
                    let idx = self.get_index(x, y, z);
                    let cell = &self.cells[idx];

                    // 1. Pressure Gradient Force: -1/rho * grad(P)
                    let x_next = (x + 1) % self.dim_x;
                    let x_prev = if x == 0 { self.dim_x - 1 } else { x - 1 };

                    let idx_east = self.get_index(x_next, y, z);
                    let idx_west = self.get_index(x_prev, y, z);
                    let idx_north = self.get_index(x, y + 1, z);
                    let idx_south = self.get_index(x, y - 1, z);

                    let rho = cell.pressure_pa / (287.05 * cell.temperature_k); // Ideal gas law rho = P / R T
                    let dp_dx = (self.cells[idx_east].pressure_pa - self.cells[idx_west].pressure_pa) / (2.0 * self.grid_spacing_m);
                    let dp_dy = (self.cells[idx_north].pressure_pa - self.cells[idx_south].pressure_pa) / (2.0 * self.grid_spacing_m);

                    let pgf_x = -dp_dx / rho;
                    let pgf_y = -dp_dy / rho;

                    // 2. Coriolis Acceleration
                    let cor = self.compute_coriolis_acceleration(y, cell.wind_velocity);

                    // 3. Update Wind Velocities
                    let new_u = cell.wind_velocity[0] + (pgf_x + cor[0]) * dt_seconds;
                    let new_v = cell.wind_velocity[1] + (pgf_y + cor[1]) * dt_seconds;

                    new_cells[idx].wind_velocity[0] = new_u * 0.99; // Damping
                    new_cells[idx].wind_velocity[1] = new_v * 0.99;

                    // 4. Moisture Condensation & Latent Heat Release
                    let sat_humidity = (17.27 * (cell.temperature_k - 273.15) / (cell.temperature_k - 35.85)).exp() * 0.0038;
                    if cell.specific_humidity > sat_humidity {
                        let excess = cell.specific_humidity - sat_humidity;
                        new_cells[idx].specific_humidity -= excess * 0.1;
                        new_cells[idx].cloud_water_content += excess * 0.1;
                        let latent_warmth = (excess * LATENT_HEAT_VAPORIZATION / SPECIFIC_HEAT_AIR) * 0.1;
                        new_cells[idx].temperature_k += latent_warmth;
                    }
                }
            }
        }

        self.cells = new_cells;
    }
}
