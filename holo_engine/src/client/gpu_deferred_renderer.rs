/// HoloEngine Phase 4 — High-Performance Deferred SDF & Physics GPU Renderer
/// Implements 2-Pass Compute Pipeline:
///   Pass 1: SDF Sphere-Tracing into G-Buffer (WorldPos/MatID, Normal/Roughness)
///   Pass 2: Deferred Optical Physics (Planck Blackbody, Fresnel SSR, Kerr Doppler, 1-Lipschitz PBR)
///   Pass 3: 1-Lipschitz Avalanche Dune Relaxation Compute Pass

use std::time::Instant;
#[cfg(feature = "wgpu")]
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DeferredCameraUniform {
    pub cam_pos: [f32; 3],
    pub time: f32,
    pub cam_dir: [f32; 3],
    pub screen_width: u32,
    pub cam_up: [f32; 3],
    pub screen_height: u32,
    pub cam_right: [f32; 3],
    pub active_scene_id: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DuneComputeUniform {
    pub grid_size: u32,
    pub cell_spacing: f32,
    pub max_slope_tan: f32,
    pub wind_angle_rad: f32,
    pub wind_strength: f32,
    pub relaxation_rate: f32,
    pub time_step: f32,
    pub pad: f32,
}

pub struct DeferredGpuRenderer;

impl DeferredGpuRenderer {
    #[cfg(feature = "wgpu")]
    pub async fn execute_deferred_pipeline(
        width: u32,
        height: u32,
        scene_id: u32,
        time: f32,
    ) -> Result<(wgpu::AdapterInfo, f32, Vec<u32>), String> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Failed to acquire WGPU Adapter".to_string())?;

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("HoloEngine Phase 4 Deferred Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {:?}", e))?;

        let gbuffer_shader_src = include_str!("../../assets/shaders/deferred_sdf_gbuffer.wgsl");
        let lighting_shader_src = include_str!("../../assets/shaders/deferred_physics_lighting.wgsl");

        let gbuffer_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GBuffer SDF Module"),
            source: wgpu::ShaderSource::Wgsl(gbuffer_shader_src.into()),
        });

        let lighting_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Lighting Module"),
            source: wgpu::ShaderSource::Wgsl(lighting_shader_src.into()),
        });

        let pixel_count = (width * height) as usize;
        let gbuffer_vec4_size = (pixel_count * std::mem::size_of::<[f32; 4]>()) as u64;
        let color_buffer_size = (pixel_count * std::mem::size_of::<u32>()) as u64;

        // G-Buffer Buffers
        let gbuffer_pos_mat = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GBuffer Pos Mat"),
            size: gbuffer_vec4_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let gbuffer_norm_rough = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GBuffer Norm Rough"),
            size: gbuffer_vec4_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let output_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Color Buffer"),
            size: color_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: color_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Setup Uniform
        let cam_uniform = DeferredCameraUniform {
            cam_pos: [0.0, 1.5, 4.0],
            time,
            cam_dir: [0.0, -0.2, -1.0],
            screen_width: width,
            cam_up: [0.0, 1.0, 0.0],
            screen_height: height,
            cam_right: [1.0, 0.0, 0.0],
            active_scene_id: scene_id,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[cam_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Pass 1: G-Buffer Pipeline
        let gbuffer_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GBuffer BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let gbuffer_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GBuffer BG"),
            layout: &gbuffer_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: gbuffer_pos_mat.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: gbuffer_norm_rough.as_entire_binding() },
            ],
        });

        let gbuffer_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GBuffer PL"),
            bind_group_layouts: &[&gbuffer_bgl],
            push_constant_ranges: &[],
        });

        let gbuffer_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("GBuffer Pipeline"),
            layout: Some(&gbuffer_pl),
            module: &gbuffer_module,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        // Pass 2: Deferred Lighting Pipeline
        let lighting_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lighting BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let lighting_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lighting BG"),
            layout: &lighting_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: gbuffer_pos_mat.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: gbuffer_norm_rough.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: output_color_buffer.as_entire_binding() },
            ],
        });

        let lighting_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lighting PL"),
            bind_group_layouts: &[&lighting_bgl],
            push_constant_ranges: &[],
        });

        let lighting_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Lighting Pipeline"),
            layout: Some(&lighting_pl),
            module: &lighting_module,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        // Dispatch Both Passes
        let start_time = Instant::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Deferred Command Encoder"),
        });

        let wg_x = (width + 15) / 16;
        let wg_y = (height + 15) / 16;

        {
            // Pass 1: GBuffer Compute
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pass 1: GBuffer Compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&gbuffer_pipeline);
            pass.set_bind_group(0, &gbuffer_bg, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        {
            // Pass 2: Deferred Lighting Compute
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pass 2: Deferred Lighting Compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&lighting_pipeline);
            pass.set_bind_group(0, &lighting_bg, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        encoder.copy_buffer_to_buffer(
            &output_color_buffer,
            0,
            &readback_buffer,
            0,
            color_buffer_size,
        );

        queue.submit(Some(encoder.finish()));

        // Map and Readback
        let buffer_slice = readback_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.await.map_err(|e| format!("Channel rx error: {:?}", e))?
            .map_err(|e| format!("Map buffer error: {:?}", e))?;

        let data = buffer_slice.get_mapped_range();
        let pixels: &[u32] = bytemuck::cast_slice(&data);
        let result_vec = pixels.to_vec();
        drop(data);
        readback_buffer.unmap();

        let elapsed_ms = start_time.elapsed().as_secs_f32() * 1000.0;
        Ok((adapter_info, elapsed_ms, result_vec))
    }

    #[cfg(feature = "wgpu")]
    pub async fn execute_dune_avalanche_pass(
        grid_size: u32,
        heights: &[f32],
    ) -> Result<(f32, Vec<f32>), String> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Failed to acquire WGPU Adapter".to_string())?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Dune Avalanche Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {:?}", e))?;

        let shader_src = include_str!("../../assets/shaders/dune_avalanche_relaxation.wgsl");
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dune Avalanche Module"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        let byte_size = (grid_size * grid_size * 4) as u64;

        let in_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dune In Buffer"),
            contents: bytemuck::cast_slice(heights),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dune Out Buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dune Readback Buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params = DuneComputeUniform {
            grid_size,
            cell_spacing: 0.5,
            max_slope_tan: 0.6745085, // tan(34°)
            wind_angle_rad: 0.785398, // 45°
            wind_strength: 1.2,
            relaxation_rate: 0.85,
            time_step: 0.016,
            pad: 0.0,
        };

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dune Uniform Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Dune BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dune BG"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: in_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: out_buf.as_entire_binding() },
            ],
        });

        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Dune PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Dune Pipeline"),
            layout: Some(&pl),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let start = Instant::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Dune Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Dune Avalanche Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups((grid_size + 15) / 16, (grid_size + 15) / 16, 1);
        }

        encoder.copy_buffer_to_buffer(&out_buf, 0, &readback, 0, byte_size);
        queue.submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.await.map_err(|e| format!("{:?}", e))?
            .map_err(|e| format!("{:?}", e))?;

        let data = slice.get_mapped_range();
        let res_heights: &[f32] = bytemuck::cast_slice(&data);
        let out_vec = res_heights.to_vec();
        drop(data);
        readback.unmap();

        let duration_ms = start.elapsed().as_secs_f32() * 1000.0;
        Ok((duration_ms, out_vec))
    }
}
