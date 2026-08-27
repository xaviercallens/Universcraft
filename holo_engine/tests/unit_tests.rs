#[cfg(test)]
mod tests {
    use holo_engine::agents::amcp::{AmcpMessage, AmcpNode};
    use holo_engine::agents::topology_observer::TopologyObserverAgent;
    use holo_engine::poc1::donn_generator::DonnGenerator;
    use holo_engine::poc1::fluid_simulation::SymplecticFluidEngine;
    use holo_engine::poc1::t_dual_shader::TDualShaderEngine;
    use tokio::sync::mpsc;

    #[test]
    fn test_donn_generator_scalar_field_and_mesh() {
        let generator = DonnGenerator::new(1.0, 3);
        let val1 = generator.evaluate_scalar_field(0.0, 0.0, 0.0);
        let val2 = generator.evaluate_scalar_field(5.0, 5.0, 5.0);
        
        // Field must evaluate to deterministic finite values
        assert!(!val1.is_nan() && !val1.is_infinite());
        assert!(!val2.is_nan() && !val2.is_infinite());
        assert_ne!(val1, val2);

        // Test 1-Lipschitz continuity check
        let is_lipschitz = generator.verify_lipschitz_continuity(0.5, 10);
        assert!(is_lipschitz, "DONN field must preserve 1-Lipschitz continuity");

        // Test Organic Terrain Mesh generation
        let mesh = generator.generate_mesh(16, 20.0);
        assert!(!mesh.positions.is_empty(), "Mesh generation should produce positions");
        assert_eq!(mesh.positions.len(), mesh.normals.len());
    }

    #[test]
    fn test_t_dual_effective_metric_bound_and_symmetry() {
        let shader_engine = TDualShaderEngine::new(1.0); // sqrt_alpha_prime = 1.0
        
        // Test R_eff = max(R, alpha' / R)
        let r_macro = shader_engine.compute_r_eff(10.0);
        assert_eq!(r_macro, 10.0);

        let r_micro = shader_engine.compute_r_eff(0.1);
        assert_eq!(r_micro, 10.0); // max(0.1, 1.0 / 0.1) = 10.0

        // Test T-Duality Symmetry R_eff(R) == R_eff(alpha' / R)
        let symmetry_ok = shader_engine.verify_t_duality_symmetry(4.0);
        assert!(symmetry_ok, "T-Duality symmetry must hold");

        // Test LOD decision logic
        let lod_macro = shader_engine.evaluate_lod_step(5.0);
        assert!(lod_macro.contains("CONTINUOUS_SDF_RAYMARCH"));

        let lod_micro = shader_engine.evaluate_lod_step(0.5);
        assert!(lod_micro.contains("DISCRETE_K3_FIBER_PROJECTION"));
    }

    #[test]
    fn test_symplectic_fluid_leray_and_enstrophy_bound() {
        let enstrophy_bound = 25.0; // max vel norm squared = 25.0 (speed = 5.0)
        let mut fluid_engine = SymplecticFluidEngine::new(5, enstrophy_bound);

        // Inject extreme kinetic energy to trigger enstrophy cap
        for particle in fluid_engine.particles.iter_mut() {
            particle.velocity = [100.0, 200.0, 300.0];
        }

        fluid_engine.step_physics(0.016);

        // Verify all particle velocities are strictly truncated to sqrt(enstrophy_bound)
        for particle in fluid_engine.particles.iter() {
            let speed_sq = particle.velocity[0] * particle.velocity[0]
                + particle.velocity[1] * particle.velocity[1]
                + particle.velocity[2] * particle.velocity[2];
            
            assert!(speed_sq <= enstrophy_bound + 1e-3, "Enstrophy bound violated: speed_sq = {}", speed_sq);
        }

        let ke = fluid_engine.compute_total_kinetic_energy();
        assert!(ke > 0.0, "Kinetic energy must be positive");
    }

    #[tokio::test]
    async fn test_amcp_agent_mesh_communication_and_observer() {
        let (tx, mut rx) = mpsc::channel::<AmcpMessage>(10);
        let mut node = AmcpNode::new("Test_Agent_01", [1.0, 2.0, 3.0], tx.clone());
        let observer = TopologyObserverAgent::new("Test_Sentinel_01", Some(tx));

        node.step_autonomous().await.unwrap();

        let received = rx.recv().await;
        assert!(received.is_some());

        if let Some(AmcpMessage::Heartbeat { agent_id, status, .. }) = received {
            assert_eq!(agent_id, "Test_Agent_01");
            assert_eq!(status, "Autonomous_Active");
        } else {
            panic!("Expected AMCP Heartbeat message");
        }

        let audit_passed = observer.audit_once().await;
        assert!(audit_passed, "TopologyObserver invariant audit must pass");
    }

    #[test]
    fn test_whittaker_climate_and_t_dual_l_systems() {
        use holo_engine::client::biome_generator::{WhittakerClimateModel, BiomeType};

        let climate_model = WhittakerClimateModel::new(1.0); // alpha_prime = 1.0

        // Test Desert vs Snow Peak classification
        let desert_sample = climate_model.sample_climate(0.0, 0.0, 0.0);
        assert!(desert_sample.temperature >= 0.0 && desert_sample.temperature <= 1.0);
        assert!(desert_sample.humidity >= 0.0 && desert_sample.humidity <= 1.0);

        let snow_sample = climate_model.sample_climate(0.0, 10.0, 0.0);
        assert_eq!(snow_sample.biome, BiomeType::SnowPeak);

        // Test T-Dual L-System Fractal Tree vertex truncation at R_eff = sqrt(alpha')
        let (positions, indices) = climate_model.build_t_dual_fractal_tree(2.0, 5);
        assert!(!positions.is_empty(), "Fractal tree must generate positions");
        assert!(!indices.is_empty(), "Fractal tree must generate indices");
    }

    #[test]
    fn test_wgsl_gpu_compute_sdf_shader_and_voxels() {
        use holo_engine::client::gpu_compute::GPUComputeManager;

        let manager = GPUComputeManager::new(32);
        assert!(manager.validate_wgsl_shader() > 500, "WGSL shader string must be valid");
        assert_eq!(manager.total_voxels(), 32 * 32 * 32);

        let voxels = manager.cpu_fallback_eval((0.0, 0.0, 0.0), 32.0);
        assert_eq!(voxels.len(), 32_768);
    }

    #[test]
    fn test_atmospheric_scattering_and_sun_cycle() {
        use holo_engine::client::atmosphere::AtmosphericEngine;
        use holo_engine::math_types::Vec3;

        let noon_engine = AtmosphericEngine::new(12.0);
        let sun = noon_engine.compute_sun_position();
        assert!(sun.elevation > 0.5, "Noon sun elevation should be high");

        let sky = noon_engine.evaluate_sky_color(Vec3::new(0.0, 1.0, 0.0));
        assert!(sky.x >= 0.0 && sky.y >= 0.0 && sky.z >= 0.0, "Sky color RGB components must be non-negative");

        let fog = noon_engine.compute_volumetric_fog(50.0, 0.0);
        assert!(fog >= 0.0 && fog <= 1.0, "Volumetric fog factor must be bounded [0, 1]");

        // Adversarial: downward view direction must not produce negative RGB
        let down_sky = noon_engine.evaluate_sky_color(Vec3::new(0.0, -1.0, 0.0));
        assert!(down_sky.x >= 0.0 && down_sky.y >= 0.0 && down_sky.z >= 0.0, "Downward sky must be non-negative");
    }

    #[test]
    fn test_tait_eos_and_leray_solenoidal_fluid_solver() {
        use holo_engine::client::fluid_solver::{SymplecticFluidSolver, SPHParams, FluidParticle};
        use holo_engine::math_types::Vec3;

        let params = SPHParams::default();
        let particle = FluidParticle::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(10.0, 10.0, 10.0), // High kinetic energy > E_max
            1.0,
        );

        let mut solver = SymplecticFluidSolver::new(params, vec![particle]);
        let pressure = solver.compute_tait_pressure(1050.0);
        assert!(pressure > 0.0, "Tait EOS pressure must be positive for compressed fluid");

        solver.step(0.016);
        let updated = &solver.particles[0];
        let enstrophy = 0.5 * updated.velocity.length_squared();
        assert!(enstrophy <= params.enstrophy_cap + 0.001, "Enstrophy must be bounded by E_max (25.0)");
    }

    #[test]
    fn test_gpu_instancing_and_post_processing_config() {
        use holo_engine::engine::renderer::{TopologicalRenderPipeline, GPUInstanceTransform};

        let mut pipeline = TopologicalRenderPipeline::new(1.0);
        assert!(pipeline.post_processing.enable_ssao, "SSAO must be enabled by default");
        assert!(pipeline.post_processing.tonemapping_aces, "ACES tonemapping must be enabled");

        pipeline.flora_instance_buffer.push_instance(GPUInstanceTransform {
            position: [1.0, 2.0, 3.0],
            scale: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color_tint: [0.1, 0.8, 0.2, 1.0],
        });

        assert_eq!(pipeline.flora_instance_buffer.instance_count, 1);
        pipeline.flora_instance_buffer.clear();
        assert_eq!(pipeline.flora_instance_buffer.instance_count, 0);
    }

    #[test]
    fn test_parallel_sph_fluid_solver() {
        use holo_engine::client::fluid_solver::{SymplecticFluidSolver, SPHParams, FluidParticle};
        use holo_engine::math_types::Vec3;

        let params = SPHParams::default();
        let particles: Vec<FluidParticle> = (0..50).map(|i| FluidParticle::new(
            Vec3::new(i as f32 * 0.1, 5.0, 0.0),
            Vec3::new(5.0, 5.0, 5.0),
            1.0,
        )).collect();

        let mut solver = SymplecticFluidSolver::new(params, particles);
        solver.step_parallel(0.016);
        assert_eq!(solver.particles.len(), 50);
        for p in &solver.particles {
            let enstrophy = 0.5 * p.velocity.length_squared();
            assert!(enstrophy <= params.enstrophy_cap + 0.1, "Enstrophy must be bounded");
        }
    }

    #[test]
    fn test_sph_neighbor_density_interaction() {
        use holo_engine::client::fluid_solver::{SymplecticFluidSolver, SPHParams, FluidParticle};
        use holo_engine::math_types::Vec3;

        let mut params = SPHParams::default();
        params.smoothing_radius = 2.0;

        // Two particles within smoothing radius must produce density > rest_density * 0.1
        let particles = vec![
            FluidParticle::new(Vec3::new(0.0, 0.0, 0.0), Vec3::ZERO, 1.0),
            FluidParticle::new(Vec3::new(0.5, 0.0, 0.0), Vec3::ZERO, 1.0),
        ];

        let mut solver = SymplecticFluidSolver::new(params, particles);
        solver.step(0.016);

        // After step, density must have been computed from neighbor interaction
        for p in &solver.particles {
            assert!(!p.density.is_nan(), "Density must not be NaN");
            assert!(p.density > 0.0, "Density must be positive after neighbor interaction");
        }
    }

    #[test]
    fn test_zero_density_no_nan() {
        use holo_engine::client::fluid_solver::{SymplecticFluidSolver, SPHParams};

        let params = SPHParams::default();
        let pressure = SymplecticFluidSolver::compute_tait_pressure_with_params(&params, 0.0);
        assert!(!pressure.is_nan(), "Pressure must not be NaN for zero density");
        assert!(!pressure.is_infinite(), "Pressure must not be infinite for zero density");
    }

    #[test]
    fn test_3d_lsystem_tree_branches_span_z_axis() {
        use holo_engine::client::biome_generator::WhittakerClimateModel;

        let model = WhittakerClimateModel::new(1.0);
        let (positions, _indices) = model.build_t_dual_fractal_tree(2.0, 5);
        assert!(!positions.is_empty());

        let min_z = positions.iter().map(|p| p.z).fold(f32::INFINITY, f32::min);
        let max_z = positions.iter().map(|p| p.z).fold(f32::NEG_INFINITY, f32::max);
        assert!(max_z - min_z > 0.01, "3D L-System tree must branch across Z axis, span = {}", max_z - min_z);
    }
}

