use holo_engine::agents::amcp::{AmcpMessage, AmcpNode};
use holo_engine::agents::topology_observer::TopologyObserverAgent;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  HoloEngine Autonomous AMCP Agent Mesh Daemon");
    println!("============================================================");

    let (tx, mut rx) = mpsc::channel::<AmcpMessage>(100);

    // Spawn Topology Observer Agent Task
    let sentinel_tx = tx.clone();
    tokio::spawn(async move {
        let observer = TopologyObserverAgent::new("Topological_Sentinel_01", Some(sentinel_tx));
        observer.run_autonomous_loop().await;
    });


    // Spawn Mesh Listener Agent Task
    let listener_tx = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                AmcpMessage::Heartbeat { agent_id, status, resonance_frequency } => {
                    println!(
                        "[AMCP Mesh] Packet received from Agent <{}> | Status: {} | Resonance: {:.2} Hz",
                        agent_id, status, resonance_frequency
                    );
                }
                AmcpMessage::TopologicalPerturbation { agent_id, position, charge_delta } => {
                    println!(
                        "[AMCP Mesh] Perturbation event from <{}> at {:?} with delta {:.4}",
                        agent_id, position, charge_delta
                    );
                }
                AmcpMessage::StateNegotiation { agent_id, .. } => {
                    println!("[AMCP Mesh] State negotiation requested by <{}>", agent_id);
                }
            }
        }
    });

    // Spawn 5 Autonomous AMCP Agent Nodes
    for i in 1..=5 {
        let agent_id = format!("AMCP_Entity_Node_{:02}", i);
        let node_tx = listener_tx.clone();
        tokio::spawn(async move {
            let mut node = AmcpNode::new(agent_id, [i as f32 * 2.0, 0.0, 0.0], node_tx);
            loop {
                if let Err(e) = node.step_autonomous().await {
                    eprintln!("Error in agent step: {}", e);
                }
                sleep(Duration::from_millis(1500 + (i * 200) as u64)).await;
            }
        });
    }

    println!("[Daemon] All 6 autonomous agents initialized and running mesh protocol.");

    // Keep daemon running continuously
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
