use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use fast_surface_nets::{
    ndshape::{ConstShape, ConstShape3u32, Shape},
    surface_nets, SurfaceNetsBuffer,
};
use rayon::prelude::*;

pub struct MarchingCubesPlugin;

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {}
}

// 32x32x32 chunk + 2 padding for normals calculation = 34
type ChunkShape = ConstShape3u32<34, 34, 34>;

pub fn extract_surface_mesh(chunk_offset: Vec3, voxel_size: f32) -> Mesh {
    let mut sdf_data = vec![0.0f32; ChunkShape::SIZE as usize];
    
    // 1. Parallel CPU evaluation of the continuous SDF (DONN equation)
    sdf_data.par_iter_mut().enumerate().for_each(|(i, val)| {
        let [x, y, z] = <ChunkShape as ConstShape<3>>::delinearize(i as u32);
        let wx = chunk_offset.x + (x as f32 - 1.0) * voxel_size;
        let wy = chunk_offset.y + (y as f32 - 1.0) * voxel_size;
        let wz = chunk_offset.z + (z as f32 - 1.0) * voxel_size;
        
        let dune = wy + f32::sin(wx * 0.2 + f32::sin(wz * 0.15)) * 4.0 + f32::sin(wz * 0.3 - wx * 0.1) * 2.0;
        *val = dune;
    });

    // 2. High-speed topology extraction via Surface Nets
    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(&sdf_data, &ChunkShape {}, [0, 0, 0], [33, 33, 33], &mut buffer);

    let mut positions = Vec::with_capacity(buffer.positions.len());
    let mut normals = Vec::with_capacity(buffer.normals.len());

    for (p, n) in buffer.positions.iter().zip(buffer.normals.iter()) {
        let px = (p[0] - 1.0) * voxel_size;
        let py = (p[1] - 1.0) * voxel_size;
        let pz = (p[2] - 1.0) * voxel_size;
        positions.push([px, py, pz]);
        normals.push([n[0], n[1], n[2]]);
    }

    // 3. Zero-Copy transition to Bevy Mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(buffer.indices));
    mesh
}
