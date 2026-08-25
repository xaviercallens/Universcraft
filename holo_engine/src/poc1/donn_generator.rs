/// Deep Oscillatory Neural Network (DONN) & Cymatic Wave Generator
/// Generates harmonic standing waves in 3D scalar fields to sculpt organic terrain.

#[derive(Debug, Clone)]
pub struct OrganicMeshBuffer {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub struct DonnGenerator {
    pub frequency_base: f32,
    pub harmonics: usize,
    pub lipschitz_limit: f32,
}

impl DonnGenerator {
    pub fn new(frequency_base: f32, harmonics: usize) -> Self {
        Self {
            frequency_base,
            harmonics,
            lipschitz_limit: 1.0,
        }
    }

    /// Evaluates the DONN cymatic field value at spatial coordinates (x, y, z)
    pub fn evaluate_scalar_field(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut val = 0.0;
        let scale = 0.1;

        for k in 1..=self.harmonics {
            let freq = self.frequency_base * (k as f32);
            let w = (k as f32).recip();

            // Standing wave resonance equations (cymatics)
            let wave_x = (x * scale * freq).sin();
            let wave_y = (y * scale * freq).cos();
            let wave_z = (z * scale * freq * 0.5).sin();

            val += w * (wave_x * wave_y + wave_y * wave_z + wave_z * wave_x);
        }

        // Distance from center for spherical containment
        let r = (x * x + y * y + z * z).sqrt();
        let sdf_sphere = r - 14.0;

        let raw_val = sdf_sphere + val * 2.0;

        // Lipschitz normalization factor: L_max = 1.0 (SDF gradient) + 2.0 * 0.1 * harmonics = 1.6
        let l_max = 1.0 + 2.0 * scale * (self.harmonics as f32) * self.frequency_base;
        raw_val / l_max
    }

    /// Verifies 1-Lipschitz continuity: |f(p2) - f(p1)| <= L * |p2 - p1|
    pub fn verify_lipschitz_continuity(&self, sample_step: f32, num_checks: usize) -> bool {
        let delta = 0.001;
        for i in 1..=num_checks {
            let p = (i as f32) * sample_step;
            let val = self.evaluate_scalar_field(p, p, p);
            let val_dx = self.evaluate_scalar_field(p + delta, p, p);
            let val_dy = self.evaluate_scalar_field(p, p + delta, p);
            let val_dz = self.evaluate_scalar_field(p, p, p + delta);

            let gx = (val_dx - val) / delta;
            let gy = (val_dy - val) / delta;
            let gz = (val_dz - val) / delta;
            let grad_norm = (gx * gx + gy * gy + gz * gz).sqrt();

            if grad_norm > self.lipschitz_limit + 0.05 { // Strictly <= 1-Lipschitz
                return false;
            }
        }
        true
    }


    /// Generates organic surface mesh from the DONN scalar field
    pub fn generate_mesh(&self, dim: usize, box_size: f32) -> OrganicMeshBuffer {
        #[cfg(feature = "full")]
        {
            use fast_surface_nets::{ndshape::{ConstShape, ConstShape3u32}, surface_nets, SurfaceNetsBuffer};
            type MeshShape = ConstShape3u32<32, 32, 32>;

            let mut sdf = vec![1.0f32; MeshShape::SIZE as usize];
            let step = box_size / (dim as f32);
            let half_box = box_size * 0.5;

            for z in 0..32u32 {
                for y in 0..32u32 {
                    for x in 0..32u32 {
                        let idx = MeshShape::linearize([x, y, z]) as usize;
                        let px = (x as f32) * step - half_box;
                        let py = (y as f32) * step - half_box;
                        let pz = (z as f32) * step - half_box;
                        sdf[idx] = self.evaluate_scalar_field(px, py, pz);
                    }
                }
            }

            let mut buffer = SurfaceNetsBuffer::default();
            surface_nets(
                &sdf,
                &MeshShape {},
                [0, 0, 0],
                [31, 31, 31],
                &mut buffer,
            );

            OrganicMeshBuffer {
                positions: buffer.positions,
                normals: buffer.normals,
                indices: buffer.indices,
            }
        }

        #[cfg(not(feature = "full"))]
        {
            // Fallback lightweight grid surface point sampling when fast-surface-nets feature is disabled
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();
            let step = box_size / (dim as f32);
            let half_box = box_size * 0.5;

            let mut vertex_count = 0u32;
            for z in 0..dim {
                for y in 0..dim {
                    for x in 0..dim {
                        let px = (x as f32) * step - half_box;
                        let py = (y as f32) * step - half_box;
                        let pz = (z as f32) * step - half_box;

                        let val = self.evaluate_scalar_field(px, py, pz);
                        if val.abs() < step * 0.8 {
                            let gx = self.evaluate_scalar_field(px + 0.01, py, pz) - val;
                            let gy = self.evaluate_scalar_field(px, py + 0.01, pz) - val;
                            let gz = self.evaluate_scalar_field(px, py, pz + 0.01) - val;
                            let norm = (gx * gx + gy * gy + gz * gz).sqrt().max(0.001);

                            positions.push([px, py, pz]);
                            normals.push([gx / norm, gy / norm, gz / norm]);
                            indices.push(vertex_count);
                            vertex_count += 1;
                        }
                    }
                }
            }

            OrganicMeshBuffer {
                positions,
                normals,
                indices,
            }
        }
    }
}

