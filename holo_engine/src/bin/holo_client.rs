//! HoloEngine Bevy 3D Client
//! Implements the 3D immersive prototype for Sprint V3.0

#[cfg(feature = "full")]
use bevy::prelude::*;
#[cfg(feature = "full")]
use holo_engine::client::terrain_generator::SDFTerrainGenerator;

#[cfg(not(feature = "full"))]
fn main() {
    println!("Please run this binary with --features full to enable the Bevy 3D engine.");
}

#[cfg(feature = "full")]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<TerrainCraters>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (fly_camera, update_sph_particles, mine_terrain_system))
        .run();
}

#[cfg(feature = "full")]
#[derive(Component)]
struct FlyCamera {
    pitch: f32,
    yaw: f32,
}

#[cfg(feature = "full")]
#[derive(Resource, Default)]
struct TerrainCraters {
    craters: Vec<(Vec3, f32)>,
}

#[cfg(feature = "full")]
#[derive(Component)]
struct TerrainChunkComponent {
    origin: Vec3,
    size: f32,
}

#[cfg(feature = "full")]
#[derive(Component)]
struct WaterParticle {
    velocity: Vec3,
}

#[cfg(feature = "full")]
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Ambient Light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });

    // Spatial Chunking Grid (2x2 grid of 16m chunks = 32m x 32m world)
    let chunk_size = 16.0;
    let voxel_size = 1.0;
    let terrain_gen = SDFTerrainGenerator::new(chunk_size, voxel_size);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        ..default()
    });

    let chunk_offsets = [
        Vec3::new(-16.0, -5.0, -16.0),
        Vec3::new(0.0, -5.0, -16.0),
        Vec3::new(-16.0, -5.0, 0.0),
        Vec3::new(0.0, -5.0, 0.0),
    ];

    for origin in chunk_offsets {
        let chunk_mesh = terrain_gen.generate_chunk_mesh(origin.x, origin.y, origin.z, &[]);
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(chunk_mesh),
                material: material_handle.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            TerrainChunkComponent {
                origin,
                size: chunk_size,
            },
        ));
    }

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera {
            pitch: 0.0,
            yaw: 0.0,
        },
    ));

    // Spawn SPH Water Particles
    let sphere_mesh = meshes.add(Sphere::new(0.3));
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.8, 1.0, 0.7),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        ..default()
    });

    for i in 0..100 {
        let x = (i % 10) as f32 * 1.5 - 7.5;
        let z = (i / 10) as f32 * 1.5 - 7.5;
        commands.spawn((
            PbrBundle {
                mesh: sphere_mesh.clone(),
                material: water_material.clone(),
                transform: Transform::from_xyz(x, 15.0 + (i as f32 * 0.1), z),
                ..default()
            },
            WaterParticle {
                velocity: Vec3::new(0.0, -2.0, 0.0),
            },
        ));
    }
}

#[cfg(feature = "full")]
use bevy::input::mouse::MouseMotion;

#[cfg(feature = "full")]
fn fly_camera(
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let speed = 15.0 * time.delta_seconds();
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    for (mut transform, mut cam) in &mut query {
        let forward = transform.forward();
        let right = transform.right();
        
        let mut translation = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) { translation += *forward; }
        if keys.pressed(KeyCode::KeyS) { translation -= *forward; }
        if keys.pressed(KeyCode::KeyD) { translation += *right; }
        if keys.pressed(KeyCode::KeyA) { translation -= *right; }
        if keys.pressed(KeyCode::Space) { translation += Vec3::Y; }
        if keys.pressed(KeyCode::ShiftLeft) { translation -= Vec3::Y; }
        
        transform.translation += translation * speed;
        
        // Mouse Look
        cam.yaw -= mouse_delta.x * 0.002;
        cam.pitch -= mouse_delta.y * 0.002;
        cam.pitch = cam.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
        
        transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
    }
}

#[cfg(feature = "full")]
fn update_sph_particles(
    mut query: Query<(&mut Transform, &mut WaterParticle)>,
    mut craters: ResMut<TerrainCraters>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let gravity = -9.81;
    
    for (mut transform, mut particle) in &mut query {
        particle.velocity.y += gravity * dt;
        transform.translation += particle.velocity * dt;
        
        let p = transform.translation;
        
        // Continuous River/Rain recycling: if water falls below world boundary, respawn at mountain peak
        if p.y < -15.0 || p.x.abs() > 25.0 || p.z.abs() > 25.0 {
            transform.translation = Vec3::new(
                (rand::random::<f32>() - 0.5) * 15.0,
                12.0 + rand::random::<f32>() * 3.0,
                (rand::random::<f32>() - 0.5) * 15.0,
            );
            particle.velocity = Vec3::ZERO;
            continue;
        }
        
        let sdf = SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z, &craters.craters);
        
        // Terrain collision & Hydraulic Erosion
        if sdf > -0.2 {
            let eps = 0.1;
            let nx = SDFTerrainGenerator::evaluate_sdf(p.x + eps, p.y, p.z, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x - eps, p.y, p.z, &craters.craters);
            let ny = SDFTerrainGenerator::evaluate_sdf(p.x, p.y + eps, p.z, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x, p.y - eps, p.z, &craters.craters);
            let nz = SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z + eps, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z - eps, &craters.craters);
            let norm = Vec3::new(nx, ny, nz).normalize_or_zero();
            
            transform.translation += norm * (sdf + 0.2);
            let v = particle.velocity;
            let speed = v.length();
            
            // Hydraulic Erosion: Fast flowing water carves small canyon channels over time
            if speed > 6.0 && craters.craters.len() < 50 {
                craters.craters.push((transform.translation, 0.4));
            }
            
            particle.velocity = (v - 2.0 * v.dot(norm) * norm) * 0.35; // bounce with dampening
        }
    }
}

#[cfg(feature = "full")]
fn mine_terrain_system(
    mouse: Res<ButtonInput<MouseButton>>,
    cam_query: Query<&Transform, With<FlyCamera>>,
    mut craters: ResMut<TerrainCraters>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_query: Query<(&Handle<Mesh>, &TerrainChunkComponent)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(cam_transform) = cam_query.get_single() {
            let origin = cam_transform.translation;
            let dir = *cam_transform.forward();
            
            // Sphere Tracing Raymarcher (Adaptive Step)
            let mut t = 0.0;
            let max_dist = 50.0;
            
            while t < max_dist {
                let p = origin + dir * t;
                let sdf = SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z, &craters.craters);
                
                if sdf > 0.0 {
                    // Hit ground! Add crater radius 3.0
                    let hit_point = p;
                    let crater_radius = 3.0;
                    craters.craters.push((hit_point, crater_radius));
                    
                    // Zero-Copy Spatial Chunking: Remesh ONLY affected chunks
                    for (mesh_handle, chunk) in &chunk_query {
                        if SDFTerrainGenerator::intersects_crater(chunk.origin, chunk.size, hit_point, crater_radius) {
                            let terrain_gen = SDFTerrainGenerator::new(chunk.size, 1.0);
                            let new_mesh = terrain_gen.generate_chunk_mesh(chunk.origin.x, chunk.origin.y, chunk.origin.z, &craters.craters);
                            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                                *mesh = new_mesh;
                            }
                        }
                    }
                    break;
                }
                
                // Adaptive step: sdf is negative in air. Step forward based on distance to surface.
                let step = (-sdf * 0.5).clamp(0.1, 3.0);
                t += step;
            }
        }
    }
}
