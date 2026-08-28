#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_vertex_output::MeshVertexOutput,
}

@group(2) @binding(0) var sand_color_tex: texture_2d<f32>;
@group(2) @binding(1) var sand_samp: sampler;
@group(2) @binding(2) var sand_normal_tex: texture_2d<f32>;
@group(2) @binding(3) var sand_normal_samp: sampler;
@group(2) @binding(4) var sand_rough_tex: texture_2d<f32>;
@group(2) @binding(5) var sand_rough_samp: sampler;

@group(2) @binding(6) var moss_color_tex: texture_2d<f32>;
@group(2) @binding(7) var moss_samp: sampler;
@group(2) @binding(8) var moss_normal_tex: texture_2d<f32>;
@group(2) @binding(9) var moss_normal_samp: sampler;
@group(2) @binding(10) var moss_rough_tex: texture_2d<f32>;
@group(2) @binding(11) var moss_rough_samp: sampler;

@group(2) @binding(12) var<uniform> blend_sharpness: f32;
@group(2) @binding(13) var<uniform> texture_scale: f32;

fn sample_triplanar(tex: texture_2d<f32>, samp: sampler, pos: vec3<f32>, normal: vec3<f32>, scale: f32) -> vec4<f32> {
    let w = abs(normal);
    let weights = w / (w.x + w.y + w.z);
    let cx = textureSample(tex, samp, pos.yz * scale) * weights.x;
    let cy = textureSample(tex, samp, pos.xz * scale) * weights.y;
    let cz = textureSample(tex, samp, pos.xy * scale) * weights.z;
    return cx + cy + cz;
}

fn hash(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(q.x) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i + vec2<f32>(0.0,0.0)), hash(i + vec2<f32>(1.0,0.0)), u.x),
               mix(hash(i + vec2<f32>(0.0,1.0)), hash(i + vec2<f32>(1.0,1.0)), u.x), u.y);
}

@fragment
fn fragment(
    mesh: MeshVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let world_pos = mesh.world_position.xyz;
    let world_normal = normalize(mesh.world_normal);
    
    let n = noise(world_pos.xz * 0.2) * 2.0 - 1.0;
    let threshold = world_pos.x * 0.05 + n * blend_sharpness;
    let biome_blend = smoothstep(-0.5, 0.5, threshold);
    
    let s_color = sample_triplanar(sand_color_tex, sand_samp, world_pos, world_normal, texture_scale);
    let m_color = sample_triplanar(moss_color_tex, moss_samp, world_pos, world_normal, texture_scale * 2.0);
    let base_color = mix(s_color, m_color, biome_blend);
    
    let s_rough = sample_triplanar(sand_rough_tex, sand_rough_samp, world_pos, world_normal, texture_scale);
    let m_rough = sample_triplanar(moss_rough_tex, moss_rough_samp, world_pos, world_normal, texture_scale * 2.0);
    let roughness = mix(s_rough, m_rough, biome_blend).r;
    
    var pbr_input = pbr_input_from_standard_material(mesh, is_front);
    pbr_input.material.base_color = base_color;
    pbr_input.material.perceptual_roughness = roughness;
    pbr_input.material.metallic = 0.0;
    pbr_input.material.reflectance = 0.2;
    
    // In Bevy 0.14 apply_pbr_lighting handles everything
    return bevy_pbr::pbr_functions::apply_pbr_lighting(pbr_input);
}
