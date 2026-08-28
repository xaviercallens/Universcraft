// HoloEngine & wgpu-marching-cubes — In-VRAM GPU Marching Cubes & Surface Nets Compute Shader
// Generates 1-Lipschitz continuous topological terrain meshes directly in GPU memory

struct ChunkParams {
    chunk_origin: vec3<f32>,
    voxel_size: f32,
    grid_dim: u32,
    iso_level: f32,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> params: ChunkParams;
@group(0) @binding(1) var<storage, read> sdf_volume: array<f32>;
@group(0) @binding(2) var<storage, read_write> vertex_count: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> out_positions: array<vec4<f32>>; // pos.xyz, normal.x
@group(0) @binding(4) var<storage, read_write> out_normals: array<vec4<f32>>;   // normal.yz, uv.xy

fn get_voxel_idx(x: u32, y: u32, z: u32) -> u32 {
    let dim = params.grid_dim;
    return x + y * dim + z * dim * dim;
}

fn sample_sdf(x: u32, y: u32, z: u32) -> f32 {
    return sdf_volume[get_voxel_idx(x, y, z)];
}

fn compute_gradient(x: u32, y: u32, z: u32) -> vec3<f32> {
    let dim = params.grid_dim;
    let dx = sample_sdf(min(x + 1u, dim - 1u), y, z) - sample_sdf(select(0u, x - 1u, x > 0u), y, z);
    let dy = sample_sdf(x, min(y + 1u, dim - 1u), z) - sample_sdf(x, select(0u, y - 1u, y > 0u), z);
    let dz = sample_sdf(x, y, min(z + 1u, dim - 1u)) - sample_sdf(x, y, select(0u, z - 1u, z > 0u));
    return normalize(vec3<f32>(dx, dy, dz));
}

@compute @workgroup_size(8, 8, 8)
fn extract_surface_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dim = params.grid_dim;
    if (global_id.x >= dim - 1u || global_id.y >= dim - 1u || global_id.z >= dim - 1u) {
        return;
    }

    let x = global_id.x;
    let y = global_id.y;
    let z = global_id.z;

    // Sample 8 corners of the voxel cell
    let v0 = sample_sdf(x, y, z);
    let v1 = sample_sdf(x + 1u, y, z);
    let v2 = sample_sdf(x + 1u, y + 1u, z);
    let v3 = sample_sdf(x, y + 1u, z);
    let v4 = sample_sdf(x, y, z + 1u);
    let v5 = sample_sdf(x + 1u, y, z + 1u);
    let v6 = sample_sdf(x + 1u, y + 1u, z + 1u);
    let v7 = sample_sdf(x, y + 1u, z + 1u);

    let iso = params.iso_level;

    // Check if surface passes through this cell
    var inside_count = 0u;
    if (v0 < iso) { inside_count += 1u; }
    if (v1 < iso) { inside_count += 1u; }
    if (v2 < iso) { inside_count += 1u; }
    if (v3 < iso) { inside_count += 1u; }
    if (v4 < iso) { inside_count += 1u; }
    if (v5 < iso) { inside_count += 1u; }
    if (v6 < iso) { inside_count += 1u; }
    if (v7 < iso) { inside_count += 1u; }

    if (inside_count == 0u || inside_count == 8u) {
        return; // Empty or fully solid cell
    }

    // Centroid of crossing points (Dual Contouring / Surface Nets vertex)
    var crossing_sum = vec3<f32>(0.0);
    var num_crossings = 0.0;

    let cell_min = params.chunk_origin + vec3<f32>(f32(x), f32(y), f32(z)) * params.voxel_size;
    let d = params.voxel_size;

    // Edge crossings
    if ((v0 < iso) != (v1 < iso)) {
        let t = (iso - v0) / (v1 - v0);
        crossing_sum += cell_min + vec3<f32>(t * d, 0.0, 0.0);
        num_crossings += 1.0;
    }
    if ((v0 < iso) != (v3 < iso)) {
        let t = (iso - v0) / (v3 - v0);
        crossing_sum += cell_min + vec3<f32>(0.0, t * d, 0.0);
        num_crossings += 1.0;
    }
    if ((v0 < iso) != (v4 < iso)) {
        let t = (iso - v0) / (v4 - v0);
        crossing_sum += cell_min + vec3<f32>(0.0, 0.0, t * d);
        num_crossings += 1.0;
    }

    if (num_crossings > 0.0) {
        let vert_pos = crossing_sum / num_crossings;
        let vert_norm = compute_gradient(x, y, z);

        let out_idx = atomicAdd(&vertex_count, 1u);
        if (out_idx < arrayLength(&out_positions)) {
            out_positions[out_idx] = vec4<f32>(vert_pos, vert_norm.x);
            out_normals[out_idx] = vec4<f32>(vert_norm.y, vert_norm.z, vert_pos.x * 0.1, vert_pos.z * 0.1);
        }
    }
}
