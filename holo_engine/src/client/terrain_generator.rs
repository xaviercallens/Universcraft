/// Terrain Generator using Signed Distance Fields (SDF) and fast-surface-nets
/// Evaluates a 3D procedural density field and extracts a solid mesh for Bevy.

#[cfg(feature = "full")]
use bevy::prelude::*;
#[cfg(feature = "full")]
use bevy::render::mesh::{Indices, PrimitiveTopology};
#[cfg(feature = "full")]
use bevy::render::render_asset::RenderAssetUsages;
#[cfg(feature = "full")]
use fast_surface_nets::{ndshape::{ConstShape, ConstShape3u32}, surface_nets, SurfaceNetsBuffer};

#[cfg(feature = "full")]
type ChunkShape = ConstShape3u32<34, 34, 34>; // 32x32x32 usable voxels, +1 for normals/borders

#[cfg(feature = "full")]
pub struct SDFTerrainGenerator {
    pub chunk_size: f32,
    pub voxel_size: f32,
}

#[cfg(feature = "full")]
impl SDFTerrainGenerator {
    pub fn new(chunk_size: f32, voxel_size: f32) -> Self {
        Self {
            chunk_size,
            voxel_size,
        }
    }

    pub fn intersects_crater(chunk_origin: Vec3, chunk_size: f32, crater_pos: Vec3, crater_radius: f32) -> bool {
        let min = chunk_origin;
        let max = chunk_origin + Vec3::splat(chunk_size);
        let closest = crater_pos.clamp(min, max);
        (crater_pos - closest).length_squared() <= (crater_radius * crater_radius)
    }

    /// Evaluates the SDF (Signed Distance Field) for a given world position
    pub fn evaluate_sdf(x: f32, y: f32, z: f32, craters: &[(Vec3, f32)]) -> f32 {
        let harmonics = 3;
        let mut height = 0.0;
        
        for n in 1..=harmonics {
            let n_f32 = n as f32;
            height += (x * 0.1 * n_f32).sin() * 2.0 / n_f32;
            height += (z * 0.1 * n_f32 * 0.8).cos() * 1.5 / n_f32;
        }
        
        let base_height = -5.0;
        let surface_y = base_height + height;
        let mut density = surface_y - y;
        
        // 3D Cave Network Generation (Betti_1 topological wave tube subtractions)
        if y < surface_y - 1.5 {
            let cave_wave = ((x * 0.15).sin().powi(2) + (z * 0.15).cos().powi(2)).sqrt();
            let cave_sdf = 0.55 - cave_wave; // positive inside cave tubes
            if cave_sdf > 0.0 {
                // Strict 1-Lipschitz CSG subtraction: A - B = min(A, -B)
                density = density.min(-cave_sdf);
            }
        }
        
        // Subtract 1-Lipschitz smooth craters (CSG sphere subtraction)
        for (center, radius) in craters {
            let dx = x - center.x;
            let dy = y - center.y;
            let dz = z - center.z;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let crater_sdf = radius - dist; // positive inside crater sphere
            if crater_sdf > 0.0 {
                // Strict 1-Lipschitz CSG subtraction
                density = density.min(-crater_sdf);
            }
        }
        
        density
    }

    /// Generates a Bevy Mesh for a specific chunk offset
    pub fn generate_chunk_mesh(&self, offset_x: f32, offset_y: f32, offset_z: f32, craters: &[(Vec3, f32)]) -> Mesh {
        let mut voxel_data = vec![0.0f32; ChunkShape::SIZE as usize];
        let chunk_res = 34; // ChunkShape dimensions

        // Parallelize voxel data generation using Rayon for local CPU acceleration
        use rayon::prelude::*;
        voxel_data.par_chunks_mut(chunk_res as usize).enumerate().for_each(|(z_y_idx, slice)| {
            let z = (z_y_idx / chunk_res as usize) as u32;
            let y = (z_y_idx % chunk_res as usize) as u32;
            for x in 0..chunk_res {
                let world_x = offset_x + (x as f32 - 1.0) * self.voxel_size;
                let world_y = offset_y + (y as f32 - 1.0) * self.voxel_size;
                let world_z = offset_z + (z as f32 - 1.0) * self.voxel_size;
                
                let sdf_val = Self::evaluate_sdf(world_x, world_y, world_z, craters);
                slice[x as usize] = sdf_val;
            }
        });

        
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(
            &voxel_data,
            &ChunkShape {},
            [0, 0, 0],
            [33, 33, 33],
            &mut buffer
        );

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        
        let mut positions = Vec::with_capacity(buffer.positions.len());
        let mut normals = Vec::with_capacity(buffer.normals.len());
        let mut colors = Vec::with_capacity(buffer.positions.len());
        
        for (i, p) in buffer.positions.iter().enumerate() {
            // Transform local voxel coordinates back to world offset relative to chunk origin
            let lx = (p[0] - 1.0) * self.voxel_size;
            let ly = (p[1] - 1.0) * self.voxel_size;
            let lz = (p[2] - 1.0) * self.voxel_size;
            positions.push([lx, ly, lz]);
            
            let n = &buffer.normals[i];
            normals.push([n[0], n[1], n[2]]);
            
            let world_y = offset_y + ly;
            let world_x = offset_x + lx;
            let world_z = offset_z + lz;
            let up_factor = n[1].max(0.0);
            
            // Whittaker Climate Biome Evaluation (Temperature & Humidity)
            let climate_model = crate::client::biome_generator::WhittakerClimateModel::new(1.0);
            let climate = climate_model.sample_climate(world_x, world_y, world_z);
            
            if up_factor < 0.45 {
                // Steep Cliff Face -> Solid Rock Grey
                colors.push([0.55, 0.55, 0.58, 1.0]);
            } else {
                // Whittaker Biome Splatmapping (Desert, Jungle, Taiga, Tundra, Snow)
                colors.push(climate.color);
            }
        }
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(buffer.indices.clone()));
        
        mesh
    }
}
