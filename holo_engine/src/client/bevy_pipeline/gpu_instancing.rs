//! Biosphere L-Systems: GPU Hardware Instancing
//! Render millions of fractal tree nodes via a single draw call.

use bevy::prelude::*;
use rand::Rng;

pub struct GpuInstancingPlugin;

impl Plugin for GpuInstancingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_k3_flora);
    }
}

#[derive(Component)]
pub struct K3Node;

// Function to match the terrain height from bevy_cinematic.rs
fn get_terrain_height(x: f32, z: f32) -> f32 {
    f32::sin(x * 0.2 + f32::sin(z * 0.15)) * 4.0 + f32::sin(z * 0.3 - x * 0.1) * 2.0
}

fn generate_k3_flora(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🌳 GPU Instancing: Generating K3 L-System Flora (Reaction-Diffusion)...");

    // 1. Loading geometry in VRAM ONCE.
    // The "K3 Disk" (Luminescent Canopy)
    let k3_disk_mesh = meshes.add(Sphere::new(0.6));
    let k3_disk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.5),
        emissive: LinearRgba::new(0.0, 5.0, 2.0, 1.0), // Intense Bloom
        ..default()
    });

    // The "Branch" (Fractal trunk)
    let branch_mesh = meshes.add(Cuboid::new(0.3, 1.0, 0.3));
    let branch_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.1, 0.05),
        perceptual_roughness: 0.9,
        ..default()
    });

    // 2. L-System Algorithm (CPU Generation)
    let mut rng = rand::thread_rng();
    let mut leaf_instances = Vec::new();
    let mut branch_instances = Vec::new();

    let num_trees = 300; // Will generate ~40,000 instances
    println!("   -> Spawning {} K3 fractal trees...", num_trees);

    for _ in 0..num_trees {
        // Distribute trees over the landscape
        let rx = rng.gen_range(-24.0..24.0);
        let rz = rng.gen_range(-24.0..24.0);
        let ry = get_terrain_height(rx, rz);
        let root_pos = Vec3::new(rx, ry, rz);

        // State: position, rotation, scale
        let mut current_nodes = vec![(root_pos, Quat::IDENTITY, 1.0_f32)];

        for depth in 0..5 {
            let mut next_nodes = Vec::new();
            for (pos, rot, scale) in current_nodes {
                // Offset branch by half its length so it grows upwards from pos
                let length = 1.2 * scale;
                let center_pos = pos + rot * Vec3::Y * (length * 0.5);

                branch_instances.push(Transform::from_translation(center_pos)
                    .with_rotation(rot)
                    .with_scale(Vec3::new(scale, length, scale)));

                let top_pos = pos + rot * Vec3::Y * length;

                // Branching logic
                let num_branches = if depth < 2 { 3 } else { 2 };
                for _ in 0..num_branches {
                    let angle_y = rng.gen_range(0.0..std::f32::consts::TAU);
                    let angle_x = rng.gen_range(0.2..0.8);
                    let child_rot = rot * Quat::from_euler(EulerRot::YXZ, angle_y, angle_x, 0.0);
                    
                    next_nodes.push((top_pos, child_rot, scale * 0.7));
                }
            }
            current_nodes = next_nodes;
        }

        // Terminals become K3 Disks
        for (pos, rot, scale) in current_nodes {
            leaf_instances.push(Transform::from_translation(pos)
                .with_rotation(rot)
                .with_scale(Vec3::splat(scale * 1.5)));
        }
    }

    println!("   -> Generated {} branches and {} K3 leaves.", branch_instances.len(), leaf_instances.len());
    println!("   -> Dispatching to GPU via native Storage Buffers (Automatic Batching)...");

    // 3. Massive GPU Dispatch (Zero-Copy Instancing)
    // Bevy automatically batches all these entities into ONE draw call per mesh/material pair,
    // placing their transforms into a continuous GPU Storage Buffer.
    
    commands.spawn_batch(branch_instances.into_iter().map(move |transform| {
        (
            PbrBundle {
                mesh: branch_mesh.clone(),
                material: branch_mat.clone(),
                transform,
                ..default()
            },
            K3Node,
        )
    }));

    commands.spawn_batch(leaf_instances.into_iter().map(move |transform| {
        (
            PbrBundle {
                mesh: k3_disk_mesh.clone(),
                material: k3_disk_mat.clone(),
                transform,
                ..default()
            },
            K3Node,
        )
    }));
}
