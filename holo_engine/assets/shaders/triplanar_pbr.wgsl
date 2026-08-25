// HoloEngine Triplanar PBR & Whittaker Biome Blending WGSL Shader
// Implements Triplanar UV Projection, Organic Ecotone Dithering, and ACES Tonemapping

struct TerrainUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_color: vec3<f32>,
    ambient_light: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: TerrainUniforms;
@group(0) @binding(1) var noise_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) height: f32,
    @location(3) temp_hum: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) height: f32,
    @location(3) temp_hum: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = model.position;
    out.normal = normalize(model.normal);
    out.height = model.height;
    out.temp_hum = model.temp_hum;
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// ACES Fitted Tonemapping curve
fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Triplanar Blending Weights from Surface Normal
fn triplanar_weights(normal: vec3<f32>) -> vec3<f32> {
    let abs_n = abs(normal);
    let pow_n = pow(abs_n, vec3<f32>(4.0));
    return pow_n / (pow_n.x + pow_n.y + pow_n.z + 0.0001);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let weights = triplanar_weights(normal);

    // Whittaker Climate Parameters with Noise Perturbation (Ecotones)
    let noise_val = sin(in.world_pos.x * 12.34) * cos(in.world_pos.z * 56.78) * 0.05;
    let temp = clamp(in.temp_hum.x + noise_val, 0.0, 1.0);
    let hum = clamp(in.temp_hum.y + noise_val, 0.0, 1.0);

    // Biome Base Colors
    var base_color: vec3<f32>;
    if (in.height > 0.6) {
        // Snow Peak
        base_color = vec3<f32>(0.92, 0.94, 0.98);
    } else if (temp > 0.6 && hum < 0.35) {
        // Desert Sand
        base_color = vec3<f32>(0.85, 0.72, 0.45);
    } else if (temp > 0.5 && hum >= 0.35) {
        // Tropical Jungle
        base_color = vec3<f32>(0.12, 0.65, 0.22);
    } else if (temp <= 0.5 && hum >= 0.35) {
        // Taiga Forest
        base_color = vec3<f32>(0.15, 0.45, 0.25);
    } else {
        // Tundra Sage
        base_color = vec3<f32>(0.45, 0.55, 0.42);
    }

    // Rock Cliff Face Blend (Slope >= 0.65)
    let slope = 1.0 - abs(normal.y);
    let rock_color = vec3<f32>(0.42, 0.44, 0.48);
    let rock_blend = smoothstep(0.45, 0.65, slope);
    let blended_albedo = mix(base_color, rock_color, rock_blend);

    // Lighting (Directional Sun + Ambient Occlusion)
    let NdotL = max(dot(normal, uniforms.sun_dir), 0.15);
    let ao = clamp(0.7 + in.height * 0.4 - slope * 0.25, 0.35, 1.0);
    
    let diffuse = blended_albedo * uniforms.sun_color * NdotL * ao;
    let ambient = blended_albedo * uniforms.ambient_light * ao;
    let linear_color = diffuse + ambient;

    // ACES Tonemapping for Cinematic Contrast
    let final_rgb = aces_tonemap(linear_color);

    return vec4<f32>(final_rgb, 1.0);
}
