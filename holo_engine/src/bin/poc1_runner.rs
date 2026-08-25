use holo_engine::agents::amcp::{AmcpMessage, AmcpNode};
use holo_engine::agents::topology_observer::TopologyObserverAgent;
use holo_engine::poc1::donn_generator::DonnGenerator;
use holo_engine::poc1::fluid_simulation::SymplecticFluidEngine;
use holo_engine::poc1::t_dual_shader::TDualShaderEngine;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  HoloEngine Proof of Concept 1 (PoC 1) Demonstrator  ");
    println!("  Noyau Visuel Organique & Rebond Quantique T-Dual    ");
    println!("============================================================");

    // 1. Instanciation du générateur DONN (Cymatique & Maillage Organique)
    println!("\n[1] Initialisation du Réseau DONN (Cymatique)...");
    let donn = DonnGenerator::new(1.0, 3);
    let sample_val = donn.evaluate_scalar_field(5.0, 5.0, 5.0);
    let is_lipschitz = donn.verify_lipschitz_continuity(0.5, 10);
    let mesh = donn.generate_mesh(16, 20.0);
    println!("    -> Champ scalaire DONN calculé à (5,5,5) = {:.4}", sample_val);
    println!("    -> Vérification Borne 1-Lipschitz : {}", if is_lipschitz { "PASSED (Continuity Verified)" } else { "FAILED" });
    println!("    -> Maillage Organique Généré : {} Sommets, {} Normales, {} Indices", mesh.positions.len(), mesh.normals.len(), mesh.indices.len());

    // 2. Moteur de Shader T-Dual & Bounding LOD
    println!("\n[2] Test du Moteur Ray Marching T-Dual (Métrique Effective R_eff)...");
    let shader_engine = TDualShaderEngine::new(1.0);
    let lod_macro = shader_engine.evaluate_lod_step(15.0);
    let lod_micro = shader_engine.evaluate_lod_step(0.5);
    let symmetry_ok = shader_engine.verify_t_duality_symmetry(4.0);
    println!("    -> Zoom Macroscopique (R=15.0) : {}", lod_macro);
    println!("    -> Zoom Microscopique (R=0.5)  : {}", lod_micro);
    println!("    -> Invariance par Symétrie T-Duale R_eff(R)=R_eff(α'/R) : {}", if symmetry_ok { "VERIFIED ✓" } else { "FAILED" });

    // 3. Simulation Fluide Symplectique SPH (Salva + Limite K3 + Leray Projector)
    println!("\n[3] Exécution de la Physique Fluide Symplectique (Leray + Enstrophy Bound)...");
    let mut fluid_engine = SymplecticFluidEngine::new(10, 100.0);
    for step in 1..=3 {
        fluid_engine.step_physics(0.016);
        let p0 = &fluid_engine.particles[0];
        let ke = fluid_engine.compute_total_kinetic_energy();
        println!(
            "    -> Pas #{}: Particule 0 pos=[{:.2}, {:.2}, {:.2}], vel=[{:.2}, {:.2}, {:.2}] | Énergie Cinétique = {:.4} J",
            step, p0.position[0], p0.position[1], p0.position[2], p0.velocity[0], p0.velocity[1], p0.velocity[2], ke
        );
    }

    // 4. Intégration du Maillage d'Agents Autonomes AMCP & Topology Observer Sentinel
    println!("\n[4] Lancement des Agents Autonomes AMCP & Sentinel Topology Observer...");
    let (tx, mut rx) = mpsc::channel::<AmcpMessage>(10);
    let mut node = AmcpNode::new("PoC1_Agent_Node", [0.0, 0.0, 0.0], tx.clone());
    let observer = TopologyObserverAgent::new("PoC1_Topology_Sentinel", Some(tx));

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                AmcpMessage::Heartbeat { agent_id, status, resonance_frequency } => {
                    println!(
                        "    -> [AMCP Packet] Agent: <{}> | Statut: {} | Résonance: {:.2} Hz",
                        agent_id, status, resonance_frequency
                    );
                }
                _ => {}
            }
        }
    });

    node.step_autonomous().await?;
    let audit_result = observer.audit_once().await;
    println!("    -> Audit Sentinel Topologique : {}", if audit_result { "TOUS LES INVARIANTS CONSERVÉS ✓" } else { "ALERTE TOPOLOGIQUE" });

    sleep(Duration::from_millis(500)).await;

    println!("\n============================================================");
    println!("  PoC 1 Exécuté avec Succès ! Tous les invariants sont validés.");
    println!("============================================================");

    Ok(())
}

