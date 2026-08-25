#[cfg(test)]
mod engine_tests {
    use holo_engine::engine::{RenderMode, TopologicalWorld, WorldPreset};

    #[test]
    fn test_engine_world_creation_and_simulation() {
        let mut world = TopologicalWorld::new("TestAlienWorld", WorldPreset::OrganicAlienWorld);
        assert!(!world.particles.is_empty());

        world.step_simulation(0.016);
        assert!(world.physics.metrics.is_enstrophy_bounded);
    }

    #[test]
    fn test_engine_interactive_lipschitz_mining() {
        let mut world = TopologicalWorld::new("MiningWorld", WorldPreset::OrganicAlienWorld);
        let count = world.interact_subtract_density([0.0, 1.0, 0.0], 5.0);
        assert!(count > 0, "Mining should deform nearby particles");
    }

    #[test]
    fn test_engine_t_duality_lod_switching() {
        let mut world = TopologicalWorld::new("TDualWorld", WorldPreset::TDualSingularity);

        world.renderer.update_camera_distance(10.0);
        assert_eq!(world.renderer.active_mode, RenderMode::ContinuousRayMarch);

        world.renderer.update_camera_distance(0.2);
        assert_eq!(world.renderer.active_mode, RenderMode::DiscreteK3Fiber);
    }
}
