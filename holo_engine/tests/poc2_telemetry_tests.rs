#[cfg(test)]
mod poc2_tests {
    use holo_engine::poc2::burn_donn_inference::BurnDonnInferenceEngine;
    use holo_engine::poc2::tda_engine::TdaEngine;
    use holo_engine::telemetry::metrics::TelemetrySystem;

    #[test]
    fn test_tda_vietoris_rips_betti_numbers() {
        let mut tda_engine = TdaEngine::new(5.0);
        tda_engine.generate_point_cloud(64, 15.0);
        assert!(!tda_engine.particles.is_empty(), "Point cloud generation should produce spatial particles");

        let betti = tda_engine.compute_vietoris_rips_betti();
        assert!(betti.betti_0 > 0, "Betti 0 (connected components) must be positive");
        
        let landscape = tda_engine.compute_persistence_landscape();
        assert!(!landscape.pairs.is_empty(), "Persistence landscape should contain persistence pairs");
        assert!(landscape.total_persistence_energy > 0.0, "Total persistence energy must be positive");
    }

    #[test]
    fn test_burn_donn_tensor_inference() {
        let donn_engine = BurnDonnInferenceEngine::new(3, 1.0);
        let sample_pos = [[0.0, 0.0, 0.0], [5.0, 5.0, 5.0], [10.0, 10.0, 10.0]];
        
        let single_val = donn_engine.forward_tensor_pass(sample_pos[0]);
        assert!(!single_val.is_nan() && !single_val.is_infinite());

        let res = donn_engine.infer_cymatic_alignment(&sample_pos);
        assert!(res.resonance_energy >= 0.0);
        assert!(res.convergence_error < 1e-2);
    }

    #[test]
    fn test_telemetry_pipeline_recording_and_export() {
        let mut telemetry = TelemetrySystem::new(10);
        let snapshot = telemetry.create_snapshot(60.0, 16.6, 20.0, 25.0, 14, 12, 3, 2648, 15.0);
        
        telemetry.record_snapshot(snapshot);
        assert_eq!(telemetry.snapshots.len(), 1);

        let json_report = telemetry.export_json_summary();
        assert!(json_report.contains("\"fps\": 60.0"));
        assert!(json_report.contains("\"betti_0\": 14"));
        assert!(json_report.contains("\"amcp_packet_throughput\": 2648"));
    }
}
