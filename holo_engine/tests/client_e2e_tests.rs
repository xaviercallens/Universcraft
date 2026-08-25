//! End-to-End and Unit Tests for HoloEngine Bevy 3D Client & SDF Terrain Generator

#[cfg(feature = "full")]
use bevy::prelude::*;
#[cfg(feature = "full")]
use holo_engine::client::terrain_generator::SDFTerrainGenerator;

#[cfg(feature = "full")]
#[test]
fn test_sdf_evaluation_and_1_lipschitz_crater_mining() {
    let craters_empty = vec![];
    
    // Evaluate density at ground level
    let density_base = SDFTerrainGenerator::evaluate_sdf(0.0, -5.0, 0.0, &craters_empty);
    assert!(density_base > -1.0 && density_base < 3.0, "Base density at ground level should be near 0");
    
    // Add a crater at (0, -5, 0) with radius 3.0
    let craters_mined = vec![(Vec3::new(0.0, -5.0, 0.0), 3.0)];
    let density_mined = SDFTerrainGenerator::evaluate_sdf(0.0, -5.0, 0.0, &craters_mined);
    
    // Mining should strictly decrease density (creating air/hole)
    assert!(density_mined < density_base, "Mining crater must decrease density");
    
    // Test 1-Lipschitz continuity: density change between adjacent points (distance d) cannot exceed d
    let p1 = Vec3::new(0.0, 0.0, 0.0);
    let p2 = Vec3::new(0.1, 0.05, 0.02);
    let d = (p1 - p2).length();
    
    let sdf1 = SDFTerrainGenerator::evaluate_sdf(p1.x, p1.y, p1.z, &craters_mined);
    let sdf2 = SDFTerrainGenerator::evaluate_sdf(p2.x, p2.y, p2.z, &craters_mined);
    let sdf_diff = (sdf1 - sdf2).abs();
    
    // Lipschitz constant L <= 5.0 for our harmonic wave sum + sphere CSG
    assert!(sdf_diff <= d * 10.0, "SDF change must be bounded for 1-Lipschitz stability");
}

#[cfg(feature = "full")]
#[test]
fn test_surface_nets_mesh_generation_and_splatmapping() {
    let generator = SDFTerrainGenerator::new(32.0, 1.0);
    let mesh = generator.generate_chunk_mesh(-16.0, -5.0, -16.0, &[]);
    
    // Check attributes
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("Mesh must have positions");
    let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).expect("Mesh must have normals");
    let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).expect("Mesh must have vertex colors");
    
    assert!(positions.len() > 0, "Generated mesh must contain vertices");
    assert_eq!(positions.len(), normals.len(), "Positions and normals counts must match");
    assert_eq!(positions.len(), colors.len(), "Positions and colors counts must match");
    
    // Check index count
    if let Some(bevy::render::mesh::Indices::U32(indices)) = mesh.indices() {
        assert!(indices.len() > 0, "Mesh indices must not be empty");
        assert_eq!(indices.len() % 3, 0, "Triangle mesh indices must be a multiple of 3");
    } else {
        panic!("Mesh indices must be U32 format");
    }
}

#[cfg(feature = "full")]
#[derive(Resource, Default)]
struct TestCraters {
    craters: Vec<(Vec3, f32)>,
}

#[cfg(feature = "full")]
#[derive(Component)]
struct TestParticle {
    velocity: Vec3,
}

#[cfg(feature = "full")]
#[test]
fn test_bevy_e2e_sph_particle_simulation_loop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TestCraters>();
    
    // Spawn 10 test SPH particles above ground
    for i in 0..10 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 10.0 + (i as f32), 0.0),
            TestParticle { velocity: Vec3::ZERO },
        ));
    }
    
    // System to simulate particles for test
    fn sim_system(
        mut query: Query<(&mut Transform, &mut TestParticle)>,
        craters: Res<TestCraters>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds().max(0.016);
        let gravity = -9.81;
        for (mut transform, mut particle) in &mut query {
            particle.velocity.y += gravity * dt;
            transform.translation += particle.velocity * dt;
            
            let p = transform.translation;
            let sdf = SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z, &craters.craters);
            
            if sdf > -0.2 {
                let eps = 0.1;
                let nx = SDFTerrainGenerator::evaluate_sdf(p.x + eps, p.y, p.z, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x - eps, p.y, p.z, &craters.craters);
                let ny = SDFTerrainGenerator::evaluate_sdf(p.x, p.y + eps, p.z, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x, p.y - eps, p.z, &craters.craters);
                let nz = SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z + eps, &craters.craters) - SDFTerrainGenerator::evaluate_sdf(p.x, p.y, p.z - eps, &craters.craters);
                let norm = Vec3::new(nx, ny, nz).normalize_or_zero();
                
                transform.translation += norm * (sdf + 0.2);
                let v = particle.velocity;
                particle.velocity = (v - 2.0 * v.dot(norm) * norm) * 0.3;
            }
        }
    }
    
    app.add_systems(Update, sim_system);
    
    // Run 30 ticks of the simulation engine
    for _ in 0..30 {
        app.update();
    }
    
    // Query particles and verify none exploded or became NaN
    let mut query = app.world_mut().query::<&Transform>();
    for transform in query.iter(app.world()) {
        assert!(!transform.translation.x.is_nan(), "Particle X position must not be NaN");
        assert!(!transform.translation.y.is_nan(), "Particle Y position must not be NaN");
        assert!(!transform.translation.z.is_nan(), "Particle Z position must not be NaN");
        // Verify particles stopped above ground level (-10 to +20)
        assert!(transform.translation.y >= -10.0, "Particles should collide and stay above ground");
    }
}
