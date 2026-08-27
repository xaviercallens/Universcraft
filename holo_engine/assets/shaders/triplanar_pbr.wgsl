#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::view

struct TerrainUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_color: vec3<f32>,
    ambient_light: vec3<f32>,
}

@group(2) @binding(0) var<uniform> uniforms: TerrainUniforms;

// Triplanar Blending Weights from Surface Normal
fn triplanar_weights(normal: vec3<f32>) -> vec3<f32> {
    let abs_n = abs(normal);
    let pow_n = pow(abs_n, vec3<f32>(4.0));
    return pow_n / (pow_n.x + pow_n.y + pow_n.z + 0.0001);
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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(mesh.world_normal);
    let weights = triplanar_weights(normal);

    // Read CPU-generated biome color from vertex attributes (COLOR)
    let base_color = mesh.color.rgb;

    // Rock Cliff Face Blend (Slope >= 0.65)
    let slope = 1.0 - abs(normal.y);
    let rock_color = vec3<f32>(0.42, 0.44, 0.48);
    let rock_blend = smoothstep(0.45, 0.65, slope);
    let blended_albedo = mix(base_color, rock_color, rock_blend);

    // Lighting (Directional Sun + Ambient Occlusion)
    let NdotL = max(dot(normal, uniforms.sun_dir), 0.15);
    
    // Simple height-based AO
    let height = mesh.world_position.y;
    let ao = clamp(0.7 + height * 0.05 - slope * 0.25, 0.35, 1.0);
    
    let diffuse = blended_albedo * uniforms.sun_color * NdotL * ao;
    let ambient = blended_albedo * uniforms.ambient_light * ao;
    let linear_color = diffuse + ambient;

    // ACES Tonemapping for Cinematic Contrast
    let final_rgb = aces_tonemap(linear_color);

    return vec4<f32>(final_rgb, 1.0);
}
