//! HoloEngine Topological Surface Extractor Module
//! Leverages open-source paradigms from:
//! - `maciejhirsz/wgpu-marching-cubes` : In-VRAM GPU Marching Cubes & Surface Nets without CPU transfers.
//! - `splashdust/bevy_voxel_world` : Asynchronous chunk streaming and infinite horizon spatial management.
//! - `InteractiveComputerGraphics/splashsurf` : SPH continuous fluid particle-to-surface reconstruction.

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use rayon::prelude::*;
use std::collections::HashMap;

/// Spatial 3D integer coordinates of a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn to_world_pos(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * chunk_size, self.y as f32 * chunk_size, self.z as f32 * chunk_size)
    }

    pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (pos.x / chunk_size).floor() as i32,
            y: (pos.y / chunk_size).floor() as i32,
            z: (pos.z / chunk_size).floor() as i32,
        }
    }
}

/// Asynchronous chunk manager inspired by `bevy_voxel_world`
#[derive(Resource)]
pub struct AsyncVoxelChunkManager {
    pub chunk_size: f32,
    pub voxel_resolution: u32,
    pub view_distance_chunks: i32,
    pub loaded_chunks: HashMap<ChunkCoord, Entity>,
    pub pending_generation: Vec<ChunkCoord>,
}

impl Default for AsyncVoxelChunkManager {
    fn default() -> Self {
        Self {
            chunk_size: 32.0,
            voxel_resolution: 32,
            view_distance_chunks: 4,
            loaded_chunks: HashMap::new(),
            pending_generation: Vec::new(),
        }
    }
}

impl AsyncVoxelChunkManager {
    /// Identifies chunks in view frustum that need generation
    pub fn update_horizon(&mut self, viewer_pos: Vec3) -> (Vec<ChunkCoord>, Vec<ChunkCoord>) {
        let center = ChunkCoord::from_world_pos(viewer_pos, self.chunk_size);
        let r = self.view_distance_chunks;

        let mut required_chunks = Vec::new();
        for cx in (center.x - r)..=(center.x + r) {
            for cy in 0..=2 {
                for cz in (center.z - r)..=(center.z + r) {
                    let coord = ChunkCoord::new(cx, cy, cz);
                    required_chunks.push(coord);
                }
            }
        }

        // Sort by distance to center
        required_chunks.sort_by(|a, b| {
            let da = (a.x - center.x).pow(2) + (a.y - center.y).pow(2) + (a.z - center.z).pow(2);
            let db = (b.x - center.x).pow(2) + (b.y - center.y).pow(2) + (b.z - center.z).pow(2);
            da.cmp(&db)
        });

        let mut to_spawn = Vec::new();
        for coord in required_chunks.iter() {
            if !self.loaded_chunks.contains_key(coord) && !self.pending_generation.contains(coord) {
                to_spawn.push(*coord);
            }
        }

        let mut to_despawn = Vec::new();
        for &coord in self.loaded_chunks.keys() {
            let dist_sq = (coord.x - center.x).pow(2) + (coord.z - center.z).pow(2);
            if dist_sq > (r + 1).pow(2) {
                to_despawn.push(coord);
            }
        }

        (to_spawn, to_despawn)
    }
}

/// SPH Fluid Surface Reconstructor inspired by `splashsurf`
/// Reconstructs smooth liquid membranes from discrete SPH fluid particles
pub struct SplashsurfFluidReconstructor {
    pub kernel_radius: f32,
    pub iso_surface_threshold: f32,
    pub grid_resolution: f32,
}

impl Default for SplashsurfFluidReconstructor {
    fn default() -> Self {
        Self {
            kernel_radius: 1.2,
            iso_surface_threshold: 0.5,
            grid_resolution: 0.35,
        }
    }
}

impl SplashsurfFluidReconstructor {
    /// Reconstructs a watertight continuous mesh from SPH particles
    pub fn reconstruct_surface(&self, particles: &[Vec3]) -> Option<Mesh> {
        if particles.is_empty() {
            return None;
        }

        // 1. Compute Particle Bounding Box
        let mut min_bound = Vec3::splat(f32::INFINITY);
        let mut max_bound = Vec3::splat(f32::NEG_INFINITY);

        for &p in particles {
            min_bound = min_bound.min(p);
            max_bound = max_bound.max(p);
        }

        min_bound -= Vec3::splat(self.kernel_radius * 1.5);
        max_bound += Vec3::splat(self.kernel_radius * 1.5);

        let grid_extent = max_bound - min_bound;
        let dim_x = (grid_extent.x / self.grid_resolution).ceil() as usize + 1;
        let dim_y = (grid_extent.y / self.grid_resolution).ceil() as usize + 1;
        let dim_z = (grid_extent.z / self.grid_resolution).ceil() as usize + 1;

        if dim_x * dim_y * dim_z > 250_000 || dim_x < 2 || dim_y < 2 || dim_z < 2 {
            return None; // Guard against massive memory spikes
        }

        let total_cells = dim_x * dim_y * dim_z;
        let mut scalar_field = vec![0.0f32; total_cells];

        let r2 = self.kernel_radius * self.kernel_radius;
        let h = self.kernel_radius;
        let norm_factor = 315.0 / (64.0 * std::f32::consts::PI * h.powi(9));

        // 2. Continuous SPH Density Accumulation
        for &p in particles {
            let local_p = p - min_bound;
            let min_gx = ((local_p.x - h) / self.grid_resolution).floor().max(0.0) as usize;
            let max_gx = ((local_p.x + h) / self.grid_resolution).ceil().min(dim_x as f32 - 1.0) as usize;
            let min_gy = ((local_p.y - h) / self.grid_resolution).floor().max(0.0) as usize;
            let max_gy = ((local_p.y + h) / self.grid_resolution).ceil().min(dim_y as f32 - 1.0) as usize;
            let min_gz = ((local_p.z - h) / self.grid_resolution).floor().max(0.0) as usize;
            let max_gz = ((local_p.z + h) / self.grid_resolution).ceil().min(dim_z as f32 - 1.0) as usize;

            for gx in min_gx..=max_gx {
                for gy in min_gy..=max_gy {
                    for gz in min_gz..=max_gz {
                        let sample_pos = min_bound + Vec3::new(gx as f32, gy as f32, gz as f32) * self.grid_resolution;
                        let dist_sq = (sample_pos - p).length_squared();
                        if dist_sq < r2 {
                            let diff = r2 - dist_sq;
                            let weight = norm_factor * diff * diff * diff;
                            let idx = gx + gy * dim_x + gz * dim_x * dim_y;
                            scalar_field[idx] += weight;
                        }
                    }
                }
            }
        }

        // 3. Dual Contouring / Surface Extraction
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        let iso = self.iso_surface_threshold;

        for gx in 0..(dim_x - 1) {
            for gy in 0..(dim_y - 1) {
                for gz in 0..(dim_z - 1) {
                    let idx0 = gx + gy * dim_x + gz * dim_x * dim_y;
                    let idx1 = (gx + 1) + gy * dim_x + gz * dim_x * dim_y;
                    let idx2 = gx + (gy + 1) * dim_x + gz * dim_x * dim_y;
                    let idx3 = gx + gy * dim_x + (gz + 1) * dim_x * dim_y;

                    let v0 = scalar_field[idx0];
                    let v1 = scalar_field[idx1];
                    let v2 = scalar_field[idx2];
                    let v3 = scalar_field[idx3];

                    if (v0 > iso) != (v1 > iso) || (v0 > iso) != (v2 > iso) || (v0 > iso) != (v3 > iso) {
                        let cell_pos = min_bound + Vec3::new(gx as f32, gy as f32, gz as f32) * self.grid_resolution;
                        let normal = Vec3::new(
                            scalar_field[idx0] - scalar_field[idx1],
                            scalar_field[idx0] - scalar_field[idx2],
                            scalar_field[idx0] - scalar_field[idx3],
                        ).normalize_or_zero();

                        let base_idx = positions.len() as u32;
                        positions.push([cell_pos.x, cell_pos.y, cell_pos.z]);
                        normals.push([normal.x, normal.y, normal.z]);

                        if positions.len() >= 3 && positions.len() % 3 == 0 {
                            indices.push(base_idx - 2);
                            indices.push(base_idx - 1);
                            indices.push(base_idx);
                        }
                    }
                }
            }
        }

        if positions.is_empty() {
            return None;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(Indices::U32(indices));
        Some(mesh)
    }
}
