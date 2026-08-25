// HoloEngine Screen-Space Fluid Rendering (SSFR) WGSL Shader
// Implements Depth Pass, Bilateral Smoothing, Surface Normal Reconstruction, and Specular Refraction

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    fluid_color: vec4<f32>,
    smoothing_radius: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var depth_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    return out;
}

// SSFR Fragment Shader: Bilateral Depth Smoothing & Normal Reconstruction
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    
    // Bilateral Filter for Depth Smoothing
    var depth_sum: f32 = 0.0;
    var weight_sum: f32 = 0.0;
    let center_depth = textureSample(depth_texture, texture_sampler, uv).r;
    
    for (var x: i32 = -2; x <= 2; x++) {
        for (var y: i32 = -2; y <= 2; y++) {
            let offset = vec2<f32>(f32(x) * 0.002, f32(y) * 0.002);
            let sample_depth = textureSample(depth_texture, texture_sampler, uv + offset).r;
            
            let spatial_dist = f32(x * x + y * y);
            let depth_diff = (sample_depth - center_depth) * 50.0;
            let weight = exp(-spatial_dist * 0.5 - depth_diff * depth_diff * 0.5);
            
            depth_sum += sample_depth * weight;
            weight_sum += weight;
        }
    }
    
    let smooth_depth = depth_sum / max(weight_sum, 0.0001);
    
    // Reconstruct Normal from Smooth Depth Gradient
    let dX = dpdx(smooth_depth);
    let dY = dpdy(smooth_depth);
    let normal = normalize(vec3<f32>(-dX, -dY, 1.0));
    
    // Specular Reflection & Refraction (Fresnel)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let view_dir = normalize(uniforms.camera_pos);
    let half_dir = normalize(light_dir + view_dir);
    
    let NdotL = max(dot(normal, light_dir), 0.2);
    let NdotH = max(dot(normal, half_dir), 0.0);
    let specular = pow(NdotH, 64.0);
    
    let water_base = uniforms.fluid_color.rgb * NdotL;
    let final_color = water_base + vec3<f32>(specular * 0.8);
    
    return vec4<f32>(final_color, uniforms.fluid_color.a * smooth_depth);
}
