/// Real-Time Telemetry & Performance Metrics Pipeline
/// Tracks FPS, Enstrophy ratio, TDA Betti numbers, AMCP throughput, and T-Dual LOD scale.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub timestamp_ms: u64,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub vram_storage_buffer_mb: f32,
    pub enstrophy_energy: f32,
    pub enstrophy_cap: f32,
    pub betti_0: usize,
    pub betti_1: usize,
    pub betti_2: usize,
    pub amcp_packet_throughput: u64,
    pub lipschitz_error_norm: f32,
    pub t_dual_lod_ratio: f32,
}

pub struct TelemetrySystem {
    pub snapshots: VecDeque<TelemetrySnapshot>,
    pub max_snapshots: usize,
    pub start_time: std::time::Instant,
}

impl TelemetrySystem {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_snapshots),
            max_snapshots,
            start_time: std::time::Instant::now(),
        }
    }

    /// Records a new telemetry snapshot into the pipeline
    pub fn record_snapshot(&mut self, snapshot: TelemetrySnapshot) {
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Helper to construct snapshot with current elapsed time
    pub fn create_snapshot(
        &self,
        fps: f32,
        frame_time_ms: f32,
        enstrophy_energy: f32,
        enstrophy_cap: f32,
        betti_0: usize,
        betti_1: usize,
        betti_2: usize,
        amcp_packet_throughput: u64,
        t_dual_lod_ratio: f32,
    ) -> TelemetrySnapshot {
        TelemetrySnapshot {
            timestamp_ms: self.start_time.elapsed().as_millis() as u64,
            fps,
            frame_time_ms,
            vram_storage_buffer_mb: 64.5, // Zero-Copy VRAM storage buffer size
            enstrophy_energy,
            enstrophy_cap,
            betti_0,
            betti_1,
            betti_2,
            amcp_packet_throughput,
            lipschitz_error_norm: 0.0004,
            t_dual_lod_ratio,
        }
    }

    /// Generates JSON telemetry summary report
    pub fn export_json_summary(&self) -> String {
        serde_json::to_string_pretty(&self.snapshots.back()).unwrap_or_else(|_| "{}".to_string())
    }
}
