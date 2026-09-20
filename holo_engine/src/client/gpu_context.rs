//! HoloEngine Persistent GPU Context
//! Creates a wgpu Instance/Adapter/Device/Queue once and reuses across all render calls.
//! This replaces the previous pattern of creating new GPU resources per frame,
//! which was a critical performance bottleneck.

#[cfg(feature = "wgpu")]
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
}

#[cfg(feature = "wgpu")]
impl GpuContext {
    /// Initializes the GPU context once. Call this at application startup
    /// and reuse the context for all subsequent GPU operations.
    pub async fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "No GPU adapter found".to_string())?;

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("HoloEngine Persistent GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to request WGPU device: {}", e))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            adapter_info,
        })
    }

    /// Returns the GPU adapter name (e.g., "NVIDIA GeForce RTX 2070")
    pub fn gpu_name(&self) -> &str {
        &self.adapter_info.name
    }

    /// Returns the GPU backend in use (Vulkan, DX12, Metal, etc.)
    pub fn backend(&self) -> wgpu::Backend {
        self.adapter_info.backend
    }

    /// Creates a compute pipeline from WGSL shader source
    pub fn create_compute_pipeline(
        &self,
        label: &str,
        wgsl_source: &str,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: None,
            module: &module,
            entry_point,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        })
    }

    /// Creates a GPU storage buffer with specified usage and size
    pub fn create_storage_buffer(&self, label: &str, size: u64, copy_src: bool) -> wgpu::Buffer {
        let mut usage = wgpu::BufferUsages::STORAGE;
        if copy_src {
            usage |= wgpu::BufferUsages::COPY_SRC;
        }
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Creates a GPU readback buffer (MAP_READ | COPY_DST)
    pub fn create_readback_buffer(&self, label: &str, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Submits a command encoder and reads back data from a mapped buffer.
    /// Returns the raw bytes from the readback buffer.
    pub async fn submit_and_readback(
        &self,
        encoder: wgpu::CommandEncoder,
        readback_buffer: &wgpu::Buffer,
        size: u64,
    ) -> Result<Vec<u8>, String> {
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = readback_buffer.slice(..size);
        let (tx, rx) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.await
            .map_err(|e| format!("Channel error: {:?}", e))?
            .map_err(|e| format!("Buffer map failed: {:?}", e))?;

        let data = buffer_slice.get_mapped_range();
        let bytes = data.to_vec();
        drop(data);
        readback_buffer.unmap();

        Ok(bytes)
    }
}
