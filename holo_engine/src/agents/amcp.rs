use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

/// AMCP (Agent Mesh Communication Protocol) Message Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmcpMessage {
    /// Broadcast local topological energy perturbation
    TopologicalPerturbation {
        agent_id: String,
        position: [f32; 3],
        charge_delta: f32,
    },
    /// Mesh state negotiation proposal
    StateNegotiation {
        agent_id: String,
        consensus_vector: Vec<f32>,
    },
    /// Heartbeat and health metrics
    Heartbeat {
        agent_id: String,
        status: String,
        resonance_frequency: f32,
    },
}

/// AMCP Autonomous Agent Node representing an entity in the HoloEngine topological mesh
#[derive(Debug, Clone)]
pub struct AmcpNode {
    pub id: String,
    pub position: [f32; 3],
    pub energy: f32,
    pub resonance_frequency: f32,
    pub tx: mpsc::Sender<AmcpMessage>,
}

impl AmcpNode {
    pub fn new(id: impl Into<String>, position: [f32; 3], tx: mpsc::Sender<AmcpMessage>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str,
            position,
            energy: 1.0,
            resonance_frequency: 440.0,
            tx,
        }
    }

    /// Autonomous perception and action loop
    pub async fn step_autonomous(&mut self) -> Result<(), String> {
        // Perturb topology dynamically
        self.energy = (self.energy * 0.99).max(0.1);
        self.resonance_frequency += (rand::random::<f32>() - 0.5) * 2.0;

        let msg = AmcpMessage::Heartbeat {
            agent_id: self.id.clone(),
            status: "Autonomous_Active".to_string(),
            resonance_frequency: self.resonance_frequency,
        };

        self.tx
            .send(msg)
            .await
            .map_err(|e| format!("Failed to transmit AMCP packet: {}", e))
    }
}

/// Mesh Network State Manager for AMCP Protocol
#[derive(Default)]
pub struct AmcpMeshNetwork {
    pub active_nodes: Arc<RwLock<HashMap<String, AmcpNode>>>,
}

impl AmcpMeshNetwork {
    pub fn new() -> Self {
        Self {
            active_nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_node(&self, node: AmcpNode) {
        let mut nodes = self.active_nodes.write().await;
        nodes.insert(node.id.clone(), node);
    }
}
