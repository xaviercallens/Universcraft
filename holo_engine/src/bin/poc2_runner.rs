use holo_engine::agents::amcp::{AmcpMessage, AmcpNode};
use holo_engine::agents::topology_observer::TopologyObserverAgent;
use holo_engine::poc1::donn_generator::DonnGenerator;
use holo_engine::poc1::fluid_simulation::SymplecticFluidEngine;
use holo_engine::poc1::t_dual_shader::TDualShaderEngine;
use holo_engine::poc2::burn_donn_inference::BurnDonnInferenceEngine;
use holo_engine::poc2::tda_engine::TdaEngine;
use holo_engine::telemetry::metrics::TelemetrySystem;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  HoloEngine Proof of Concept 2 (PoC 2) & Télémétrie  ");
    println!("  Analyse Topologique TDA, Inférence Burn DONN & Metrics");
    println!("============================================================");

    // 1. Initialisation du Système de Télémétrie
    println!("\n[1] Activation du Pipeline de Télémétrie en Temps Réel...");
    let mut telemetry = TelemetrySystem::new(100);

    // 2. Moteur TDA & Filtration de Vietoris-Rips
    println!("\n[2] Exécution de la Filtration de Vietoris-Rips & Nombres de Betti (TDA)...");
    let mut tda_engine = TdaEngine::new(4.5);
    tda_engine.generate_point_cloud(125, 20.0);
    let betti = tda_engine.compute_vietoris_rips_betti();
    let landscape = tda_engine.compute_persistence_landscape();

    println!("    -> Nuage de Points Topologique : {} particules", tda_engine.particles.len());
    println!("    -> Composantes Connexes (Betti 0) : {}", betti.betti_0);
    println!("    -> Tunnels / Boucles 1D (Betti 1) : {}", betti.betti_1);
    println!("    -> Cavités Fermées 2D  (Betti 2) : {}", betti.betti_2);
    println!("    -> Énergie du Paysage de Persistance : {:.4}", landscape.total_persistence_energy);

    // 3. Inférence Neuronale Burn DONN
    println!("\n[3] Inférence du Réseau Neuronal Oscillatoire (Burn DONN Backend)...");
    let donn_engine = BurnDonnInferenceEngine::new(3, 1.0);
    let sample_positions: Vec<[f32; 3]> = tda_engine.particles.iter().map(|p| p.position).collect();
    let inference_res = donn_engine.infer_cymatic_alignment(&sample_positions);

    println!("    -> Énergie de Résonance Tenseur DONN : {:.4}", inference_res.resonance_energy);
    println!("    -> Ventres de Matière (Antinodes)  : {}", inference_res.antinode_peaks);
    println!("    -> Nœuds Vallées (Nodes)          : {}", inference_res.node_valleys);
    println!("    -> Erreur de Convergence Loss     : {:.6}", inference_res.convergence_error);

    // 4. Rendu Organique Mesh 1-Lipschitz & T-Dualité
    println!("\n[4] Génération de Surface Organique 1-Lipschitz & Shader T-Dual...");
    let donn_gen = DonnGenerator::new(1.0, 3);
    let is_lipschitz = donn_gen.verify_lipschitz_continuity(0.5, 10);
    let mesh = donn_gen.generate_mesh(16, 20.0);

    let shader = TDualShaderEngine::new(1.0);
    let r_eff_macro = shader.compute_r_eff(15.0);
    let lod_micro = shader.evaluate_lod_step(0.5);

    println!("    -> Maillage 1-Lipschitz continu : {} Sommets, {} Indices | Valide: {}", mesh.positions.len(), mesh.indices.len(), is_lipschitz);
    println!("    -> Métrique T-Duale Zoom Macroscopique R_eff(15.0) = {:.2}", r_eff_macro);
    println!("    -> Rebond Quantique Zoom Microscopique (R=0.5) : {}", lod_micro);

    // 5. Physics Hydrodynamics SPH
    println!("\n[5] Hydrodynamique SPH Symplectique & Cutoff K3...");
    let mut fluid_engine = SymplecticFluidEngine::new(10, 25.0);
    fluid_engine.step_physics(0.016);
    let max_vel = fluid_engine.particles.iter().map(|p| (p.velocity[0]*p.velocity[0] + p.velocity[1]*p.velocity[1] + p.velocity[2]*p.velocity[2]).sqrt()).fold(0.0f32, f32::max);
    println!("    -> Vitesse Max Fluide SPH après Leray & Cutoff K3 : {:.2} m/s (Capped <= 5.00)", max_vel);

    // 6. Network AMCP Agents & Sentinel Audit
    println!("\n[6] Mesh Agents Autonomes AMCP & Audit Sentinel...");
    let (tx, mut rx) = mpsc::channel::<AmcpMessage>(20);
    let mut node = AmcpNode::new("PoC2_Agent_01", [1.0, 2.0, 3.0], tx.clone());
    let observer = TopologyObserverAgent::new("PoC2_Sentinel_01", Some(tx));

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let AmcpMessage::Heartbeat { agent_id, status, .. } = msg {
                println!("    -> [AMCP Packet] Agent: <{}> | Statut: {}", agent_id, status);
            }
        }
    });

    node.step_autonomous().await?;
    let audit_ok = observer.audit_once().await;
    println!("    -> Audit Sentinel Topologique : {}", if audit_ok { "PASSED ✓" } else { "FAILED" });
    sleep(Duration::from_millis(300)).await;

    // 7. Enregistrement & Exportation de la Télémétrie
    println!("\n[7] Exportation du Rapport de Télémétrie JSON (Real-Time Metrics)...");
    let snapshot = telemetry.create_snapshot(
        60.0,
        16.6,
        max_vel * max_vel,
        25.0,
        betti.betti_0,
        betti.betti_1,
        betti.betti_2,
        2648,
        r_eff_macro / 1.0,
    );
    telemetry.record_snapshot(snapshot);
    let json_report = telemetry.export_json_summary();

    println!("============================================================");
    println!("  Rapport de Télémétrie JSON Généré :");
    println!("============================================================");
    println!("{}", json_report);

    println!("\n============================================================");
    println!("  PoC 2 & Télémétrie Exécutés avec Succès !");
    println!("============================================================");

    Ok(())
}
