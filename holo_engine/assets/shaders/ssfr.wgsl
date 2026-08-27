#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::view

struct FluidUniforms {
    fluid_color: vec4<f32>,
    smoothing_radius: f32,
    light_dir: vec3<f32>,
}

@group(2) @binding(0) var<uniform> uniforms: FluidUniforms;

// We'll leave the depth texture as a placeholder or remove it since Bevy handles depth via prepass.
// For now, we will use a procedural approach if depth is missing or just shade based on normals.

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Normal reconstruction from simple screen space or just using the mesh normal
    let normal = normalize(mesh.world_normal);
    
    // Specular Reflection & Refraction (Fresnel)
    let view_dir = normalize(view.world_position.xyz - mesh.world_position.xyz);
    let half_dir = normalize(uniforms.light_dir + view_dir);
    
    let NdotL = max(dot(normal, uniforms.light_dir), 0.2);
    let NdotH = max(dot(normal, half_dir), 0.0);
    let specular = pow(NdotH, 64.0);
    
    let water_base = uniforms.fluid_color.rgb * NdotL;
    let final_color = water_base + vec3<f32>(specular * 0.8);
    
    return vec4<f32>(final_color, uniforms.fluid_color.a);
}
