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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    pub enable_ssao: bool,
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub tonemapping_aces: bool,
    pub enable_cascaded_shadows: bool,
    pub shadow_map_resolution: u32,
}

impl Default for PostProcessingConfig {
    fn default() -> Self {
        Self {
            enable_ssao: true,
            enable_bloom: true,
            bloom_intensity: 0.15,
            tonemapping_aces: true,
            enable_cascaded_shadows: true,
            shadow_map_resolution: 2048,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GPUInstanceTransform {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub color_tint: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct GPUInstanceBuffer {
    pub instance_count: usize,
    pub transforms: Vec<GPUInstanceTransform>,
}

impl GPUInstanceBuffer {
    pub fn new() -> Self {
        Self {
            instance_count: 0,
            transforms: Vec::new(),
        }
    }

    pub fn push_instance(&mut self, transform: GPUInstanceTransform) {
        self.transforms.push(transform);
        self.instance_count += 1;
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.instance_count = 0;
    }
}

pub struct TopologicalRenderPipeline {
    pub alpha_prime: f32,
    pub camera: TopologicalCamera,
    pub active_mode: RenderMode,
    pub effective_r: f32,
    pub post_processing: PostProcessingConfig,
    pub flora_instance_buffer: GPUInstanceBuffer,
}

impl TopologicalRenderPipeline {
    pub fn new(alpha_prime: f32) -> Self {
        Self {
            alpha_prime,
            camera: TopologicalCamera::default(),
            active_mode: RenderMode::ContinuousRayMarch,
            effective_r: 15.0,
            post_processing: PostProcessingConfig::default(),
            flora_instance_buffer: GPUInstanceBuffer::new(),
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
