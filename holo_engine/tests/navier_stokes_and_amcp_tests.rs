//! Integration Tests for UniversCraft Test Campaign Sections 4 & 5:
//! Navier-Stokes Blow-up Challenge, Leray Incompressibility, and AMCP Network Scalability.

use holo_engine::poc1::fluid_simulation::SymplecticFluidEngine;
use holo_engine::agents::topology_observer::TopologyObserverAgent;
use holo_engine::agents::amcp::{AmcpMessage, AmcpNode};
use tokio::sync::mpsc;

#[test]
fn test_4_1_navier_stokes_blowup_challenge() {
    let enstrophy_cap = 25.0; // E_max = 25.0 => max speed = 5.0
    let mut fluid_engine = SymplecticFluidEngine::new(500, enstrophy_cap);
    
    // Inject catastrophic, localized kinetic energy (speed = 1000.0 m/s)
    for p in fluid_engine.particles.iter_mut() {
        p.velocity = [1000.0, -1000.0, 500.0];
    }
    
    // Step fluid simulation using step_physics
    fluid_engine.step_physics(0.016);
    
    // Assert all velocities are truncated by Enstrophy cap (Virasoro bound)
    for (i, p) in fluid_engine.particles.iter().enumerate() {
        let speed_sq = p.velocity[0] * p.velocity[0] + p.velocity[1] * p.velocity[1] + p.velocity[2] * p.velocity[2];
        assert!(
            speed_sq <= enstrophy_cap + 1e-3,
            "Particle {} violated enstrophy cap! speed_sq = {}, limit = {}",
            i, speed_sq, enstrophy_cap
        );
        assert!(!p.velocity[0].is_nan() && !p.velocity[1].is_nan() && !p.velocity[2].is_nan(), "Velocity must not be NaN");
    }
}

#[test]
fn test_4_2_incompressibility_assertion_leray_solenoidal() {
    let enstrophy_cap = 25.0;
    let mut fluid_engine = SymplecticFluidEngine::new(100, enstrophy_cap);
    
    // Step fluid simulation to apply Leray-Hopf solenoidal projection
    fluid_engine.step_physics(0.016);
    
    // Estimate velocity field divergence div(u) across particles
    let mut total_div = 0.0;
    let particles = &fluid_engine.particles;
    
    for i in 0..particles.len() {
        let p1 = &particles[i];
        let mut div_local = 0.0;
        let mut neighbors = 0;
        
        for j in 0..particles.len() {
            if i == j { continue; }
            let p2 = &particles[j];
            let dx = p2.position[0] - p1.position[0];
            let dy = p2.position[1] - p1.position[1];
            let dz = p2.position[2] - p1.position[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            
            if dist_sq < 4.0 && dist_sq > 1e-5 {
                let dist = dist_sq.sqrt();
                let dvx = p2.velocity[0] - p1.velocity[0];
                let dvy = p2.velocity[1] - p1.velocity[1];
                let dvz = p2.velocity[2] - p1.velocity[2];
                
                div_local += (dvx * dx + dvy * dy + dvz * dz) / (dist * dist);
                neighbors += 1;
            }
        }
        
        if neighbors > 0 {
            total_div += (div_local / neighbors as f32).abs();
        }
    }
    
    let avg_div = total_div / particles.len() as f32;
    assert!(avg_div < 5.0, "Average velocity divergence div(u) should be bounded: {}", avg_div);
}

#[tokio::test]
async fn test_5_1_topology_disruption_adaptation() {
    let (tx, _rx) = mpsc::channel::<AmcpMessage>(100);
    let observer = TopologyObserverAgent::new("Sentinel_Observer_5_1", Some(tx));
    
    // Audit invariant
    let audit_passed = observer.audit_once().await;
    assert!(audit_passed, "TopologyObserver invariant audit must pass");
}

#[tokio::test]
async fn test_5_2_sovereignty_amcp_scale_test() {
    let (tx, mut rx) = mpsc::channel::<AmcpMessage>(2000);
    
    // Spawn 100 AMCP agent nodes in an autonomous mesh communication test
    for i in 0..100 {
        let node_id = format!("Agent_Node_{}", i);
        let mut node = AmcpNode::new(&node_id, [i as f32, 0.0, 0.0], tx.clone());
        node.step_autonomous().await.unwrap();
    }
    
    // Receive heartbeats from mesh network
    let mut heartbeats_count = 0;
    while let Ok(msg) = rx.try_recv() {
        if let AmcpMessage::Heartbeat { agent_id, status, .. } = msg {
            assert!(agent_id.starts_with("Agent_Node_"));
            assert_eq!(status, "Autonomous_Active");
            heartbeats_count += 1;
        }
    }
    
    assert_eq!(heartbeats_count, 100, "Should receive 100 AMCP heartbeats without packet loss or deadlocks");
}
