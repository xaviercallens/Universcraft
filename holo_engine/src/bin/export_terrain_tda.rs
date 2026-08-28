use std::fs::File;
use std::io::Write;
use rayon::prelude::*;
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};

type ChunkShape = ConstShape3u32<34, 34, 34>;

fn main() {
    println!("Extracting UniversCraft SDF Terrain to Point Cloud for TDA/TNN analysis...");

    let chunk_size = 32.0;
    let voxel_size = 1.0;
    let mut all_points = Vec::new();

    for cx in -1..=1 {
        for cz in -1..=1 {
            let offset_x = cx as f32 * chunk_size;
            let offset_z = cz as f32 * chunk_size;
            
            let mut sdf_data = vec![0.0f32; ChunkShape::SIZE as usize];
            
            sdf_data.par_iter_mut().enumerate().for_each(|(i, val)| {
                let [x, y, z] = <ChunkShape as ConstShape<3>>::delinearize(i as u32);
                let wx = offset_x + (x as f32 - 1.0) * voxel_size;
                let wy = -10.0 + (y as f32 - 1.0) * voxel_size;
                let wz = offset_z + (z as f32 - 1.0) * voxel_size;
                
                let dune = wy + f32::sin(wx * 0.2 + f32::sin(wz * 0.15)) * 4.0 + f32::sin(wz * 0.3 - wx * 0.1) * 2.0;
                *val = dune;
            });

            let mut buffer = fast_surface_nets::SurfaceNetsBuffer::default();
            fast_surface_nets::surface_nets(&sdf_data, &ChunkShape {}, [0, 0, 0], [33, 33, 33], &mut buffer);

            for p in buffer.positions.iter() {
                let px = offset_x + (p[0] - 1.0) * voxel_size;
                let py = -10.0 + (p[1] - 1.0) * voxel_size;
                let pz = offset_z + (p[2] - 1.0) * voxel_size;
                all_points.push(format!("{},{},{}", px, py, pz));
            }
        }
    }

    let mut file = File::create("/tmp/universcraft_terrain.csv").unwrap();
    file.write_all(b"x,y,z\n").unwrap();
    file.write_all(all_points.join("\n").as_bytes()).unwrap();
    println!("Saved {} terrain points to /tmp/universcraft_terrain.csv", all_points.len());

    let mut trees = Vec::new();
    let num_trees = 300;
    for _ in 0..num_trees {
        let rx = (rand::random::<f32>() - 0.5) * 48.0;
        let rz = (rand::random::<f32>() - 0.5) * 48.0;
        let ry = f32::sin(rx * 0.2 + f32::sin(rz * 0.15)) * 4.0 + f32::sin(rz * 0.3 - rx * 0.1) * 2.0 - 10.0;
        trees.push(format!("{},{},{}", rx, ry, rz));
    }
    let mut file2 = File::create("/tmp/universcraft_trees.csv").unwrap();
    file2.write_all(b"x,y,z\n").unwrap();
    file2.write_all(trees.join("\n").as_bytes()).unwrap();
    println!("Saved {} tree roots to /tmp/universcraft_trees.csv", trees.len());
}
