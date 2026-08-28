use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle},
    prelude::*,
};
use holo_engine::client::bevy_pipeline::{
    gpu_instancing::GpuInstancingPlugin,
    marching_cubes::{extract_surface_mesh, MarchingCubesPlugin},
    triplanar_material::TriplanarMaterial,
};

fn main() {
    println!("🎬 Launching Bevy Cinematic Viewer (Phase 4 - Zero-Copy GPU)");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "HoloEngine - Cinematic Viewer (Phase 4)".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MaterialPlugin::<TriplanarMaterial>::default())
        // Include our new architecture modules
        .add_plugins(GpuInstancingPlugin)
        .add_plugins(MarchingCubesPlugin)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TriplanarMaterial>>,
) {
    // 1. Post-Processing & Cinematic Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            transform: Transform::from_xyz(-10.0, 15.0, 20.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
            ..default()
        },
        BloomSettings::default(),
        ScreenSpaceAmbientOcclusionBundle::default(),
    ));

    // 2. Cascaded Shadow Maps (Sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::PI / 4., -std::f32::consts::PI / 4.)),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 100.0,
            ..default()
        }.into(),
        ..default()
    });

    // 3. Load Triplanar Materials
    let mat = materials.add(TriplanarMaterial {
        sand_color: asset_server.load("textures/sand_color.jpg"),
        sand_normal: asset_server.load("textures/sand_normal.jpg"),
        sand_rough: asset_server.load("textures/sand_rough.jpg"),
        moss_color: asset_server.load("textures/moss_color.jpg"),
        moss_normal: asset_server.load("textures/moss_normal.jpg"),
        moss_rough: asset_server.load("textures/moss_rough.jpg"),
        blend_sharpness: 2.0,
        texture_scale: 1.0,
    });

    // 4. Dispatch the multithreaded Marching Cubes (fast-surface-nets)
    // Generate a massive 9-chunk terrain (3x3 grid) via the Zero-Copy pipeline
    let chunk_size = 32.0;
    let voxel_size = 1.0;
    
    println!("⛰️ Generating SDF Terrain (Marching Cubes) via fast-surface-nets + Rayon...");
    
    for cx in -1..=1 {
        for cz in -1..=1 {
            let offset = Vec3::new(cx as f32 * chunk_size, -10.0, cz as f32 * chunk_size);
            // This runs the extremely fast Rayon par_iter under the hood
            let mesh = extract_surface_mesh(offset, voxel_size);
            
            commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: mat.clone(),
                transform: Transform::from_translation(offset),
                ..default()
            });
        }
    }
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds() * 0.1));
    }
}
