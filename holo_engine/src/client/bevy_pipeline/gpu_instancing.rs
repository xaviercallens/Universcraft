//! Biosphere L-Systems: GPU Hardware Instancing
//! Render millions of fractal tree nodes via a single draw call.

pub struct GpuInstancer {
    // Scaffold for Bevy hardware instancing
    pub instance_count: usize,
}

impl GpuInstancer {
    pub fn new(count: usize) -> Self {
        println!("🌳 Initializing GPU Hardware Instancing ({} L-System nodes)", count);
        Self { instance_count: count }
    }
}
