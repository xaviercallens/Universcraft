#[cfg(feature = "full")]
use bevy::prelude::*;
#[cfg(feature = "full")]
use bevy::render::mesh::Indices;
#[cfg(feature = "full")]
use bevy::render::render_asset::RenderAssetUsages;
#[cfg(feature = "full")]
use bevy::render::render_resource::PrimitiveTopology;
#[cfg(feature = "full")]
use fast_surface_nets::{ndshape::{ConstShape, ConstShape3u32}, surface_nets, SurfaceNetsBuffer};

#[cfg(feature = "full")]
type SampleShape = ConstShape3u32<32, 32, 32>;

fn main() {
    #[cfg(feature = "full")]
    {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_systems(Startup, setup)
            .run();
    }
    #[cfg(not(feature = "full"))]
    {
        println!("HoloEngine Core initialized. Run with `--features full` to launch full Bevy 3D renderer.");
    }
}

#[cfg(feature = "full")]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut sdf = [1.0; SampleShape::SIZE as usize];
    for z in 0..32u32 {
        for y in 0..32u32 {
            for x in 0..32u32 {
                let i = SampleShape::linearize([x, y, z]) as usize;
                let dx = x as f32 - 16.0;
                let dy = y as f32 - 16.0;
                let dz = z as f32 - 16.0;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt() - 10.0;
                sdf[i] = dist;
            }
        }
    }

    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
        &sdf,
        &SampleShape {},
        [0, 0, 0],
        [31, 31, 31],
        &mut buffer,
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffer.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, buffer.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; buffer.positions.len()]);
    mesh.insert_indices(Indices::U32(buffer.indices.clone()));

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.6),
            ..default()
        }),
        transform: Transform::from_xyz(-16.0, -16.0, -16.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
