use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionSettings},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use holo_engine::client::bevy_pipeline::triplanar_material::TriplanarMaterial;

fn main() {
    println!("🎬 Launching Bevy Cinematic Viewer (Triplanar, SSAO, Bloom, CSM, ACES)");
    
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
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .run();
}

fn create_deformed_terrain_mesh() -> Mesh {
    let size = 50.0;
    let subdivisions = 100;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let step = size / subdivisions as f32;
    let half_size = size / 2.0;

    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let px = x as f32 * step - half_size;
            let pz = z as f32 * step - half_size;
            
            // Dune equation
            let py = f32::sin(px * 0.2 + f32::sin(pz * 0.15)) * 4.0 + f32::sin(pz * 0.3 - px * 0.1) * 2.0;
            
            positions.push([px, py, pz]);
            normals.push([0.0, 1.0, 0.0]); // Will be recomputed later
            uvs.push([x as f32 / subdivisions as f32, z as f32 / subdivisions as f32]);
        }
    }

    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let start = z * (subdivisions + 1) + x;
            indices.push(start);
            indices.push(start + subdivisions + 1);
            indices.push(start + 1);

            indices.push(start + 1);
            indices.push(start + subdivisions + 1);
            indices.push(start + subdivisions + 2);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals(); // Bevy function to compute smooth normals!
    
    // In Bevy 0.14, we need tangents if normal maps are used!
    // mesh.generate_tangents().unwrap(); // Wait, triplanar normal mapping doesn't use standard UV tangents. We map manually in shader.
    
    mesh
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TriplanarMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            transform: Transform::from_xyz(-10.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::default(),
        ScreenSpaceAmbientOcclusionBundle::default(),
    ));

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

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(create_deformed_terrain_mesh()),
        material: mat,
        ..default()
    });
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds() * 0.1));
    }
}
