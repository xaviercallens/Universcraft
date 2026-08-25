/// Topological Render Pipeline for Scale-Invariant Quantum LOD & Surface Nets
/// Manages camera evaluation, T-Duality bounce mode switching, and 3D terrain rendering parameters.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    ContinuousRayMarch,
    DiscreteK3Fiber,
    OrganicSurfaceMesh,
    QuantumFluidParticles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalCamera {
    pub position: [f32; 3],
    pub distance_r: f32,
    pub fov_degrees: f32,
}

impl Default for TopologicalCamera {
    fn default() -> Self {
        Self {
            position: [0.0, 5.0, 15.0],
            distance_r: 15.0,
            fov_degrees: 75.0,
        }
    }
}

pub struct TopologicalRenderPipeline {
    pub alpha_prime: f32,
    pub camera: TopologicalCamera,
    pub active_mode: RenderMode,
    pub effective_r: f32,
}

impl TopologicalRenderPipeline {
    pub fn new(alpha_prime: f32) -> Self {
        Self {
            alpha_prime,
            camera: TopologicalCamera::default(),
            active_mode: RenderMode::ContinuousRayMarch,
            effective_r: 15.0,
        }
    }

    /// Evaluates T-Duality metric R_eff = max(R, alpha' / R) and updates LOD render mode
    pub fn update_camera_distance(&mut self, distance_r: f32) {
        self.camera.distance_r = distance_r;
        let min_r = distance_r.max(0.0001);
        self.effective_r = distance_r.max(self.alpha_prime / min_r);

        let sqrt_alpha = self.alpha_prime.sqrt();
        if distance_r < sqrt_alpha {
            self.active_mode = RenderMode::DiscreteK3Fiber;
        } else {
            self.active_mode = RenderMode::ContinuousRayMarch;
        }
    }
}
