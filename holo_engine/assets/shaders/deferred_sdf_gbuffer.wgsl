// ==============================================================================
// HoloEngine Phase 4 — Pass 1: Deferred SDF Raymarching & G-Buffer Compute Shader
// Eliminates Branch Divergence by separating Geometry/Topology from Optical Laws.
// G-Buffer Output:
//   Texture 0 (World Pos / Depth / MatID): (pos.x, pos.y, pos.z, material_id)
//   Texture 1 (Normal / Roughness / Curvature): (norm.x, norm.y, norm.z, roughness)
// ==============================================================================

struct CameraUniforms {
    cam_pos: vec3<f32>,
    time: f32,
    cam_dir: vec3<f32>,
    screen_width: u32,
    cam_up: vec3<f32>,
    screen_height: u32,
    cam_right: vec3<f32>,
    active_scene_id: u32, // 1=Ocean, 2=Forest, 4=Crystal, 6=Dune, 7=Ice, 8=Magma, 10=DESI, 11=BlackHole
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read_write> gbuffer_pos_mat: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> gbuffer_norm_rough: array<vec4<f32>>;

// Signed Distance Functions (SDF)
fn sdf_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdf_plane(p: vec3<f32>, h: f32) -> f32 {
    return p.y - h;
}

// 1-Lipschitz Noise Approximation
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(mix(hash31(i + vec3<f32>(0.0,0.0,0.0)), hash31(i + vec3<f32>(1.0,0.0,0.0)), u.x),
            mix(hash31(i + vec3<f32>(0.0,1.0,0.0)), hash31(i + vec3<f32>(1.0,1.0,0.0)), u.x), u.y),
        mix(mix(hash31(i + vec3<f32>(0.0,0.0,1.0)), hash31(i + vec3<f32>(1.0,0.0,1.0)), u.x),
            mix(hash31(i + vec3<f32>(0.0,1.0,1.0)), hash31(i + vec3<f32>(1.0,1.0,1.0)), u.x), u.y), u.z
    );
}

// Unified World SDF: Returns vec2(distance, material_id)
fn map_world(p: vec3<f32>, scene: u32) -> vec2<f32> {
    if (scene == 8u) {
        // 🔥 Scene 8: Volcano Core & Magma (ID 8.0 = Magma, ID 8.5 = Crust)
        let crust = sdf_plane(p, -1.0) - noise3d(p * 0.3) * 1.5;
        let magma_pool = sdf_sphere(p - vec3<f32>(0.0, -1.5, 0.0), 3.5) + noise3d(p * 0.8 + vec3<f32>(0.0, camera.time * 0.5, 0.0)) * 0.4;
        let d = min(crust, magma_pool);
        let mat = select(8.5, 8.0, magma_pool < crust);
        return vec2<f32>(d, mat);
    } else if (scene == 6u) {
        // 🏜️ Scene 6: Dunes 1-Lipschitz Avalanche (ID 6.0)
        let dune_h = sin(p.x * 0.2 + p.z * 0.1) * 2.0 + cos(p.z * 0.3 - p.x * 0.15) * 1.5 + noise3d(p * 0.5) * 0.5;
        let d = p.y - dune_h;
        return vec2<f32>(d, 6.0);
    } else if (scene == 4u) {
        // 💎 Scene 4: Crystal Cave (ID 4.0 = Crystal, ID 4.5 = Cave Rock)
        let cave = -sdf_sphere(p, 8.0) + noise3d(p * 0.2) * 1.5;
        let crystal1 = sdf_sphere(p - vec3<f32>(0.0, -2.0, 0.0), 1.8) + (abs(p.x) + abs(p.y) + abs(p.z)) * 0.2;
        let d = min(cave, crystal1);
        let mat = select(4.5, 4.0, crystal1 < cave);
        return vec2<f32>(d, mat);
    } else if (scene == 7u) {
        // 🏔️ Scene 7: Glen Ice Glacier (ID 7.0 = Viscoplastic Ice)
        let glacier = sdf_plane(p, -0.5) - noise3d(p * 0.25) * 2.2;
        return vec2<f32>(glacier, 7.0);
    } else if (scene == 11u) {
        // 🕳️ Scene 11: Kerr Black Hole Accretion Disk (ID 11.0 = ISCO Disk, ID 11.5 = Event Horizon)
        let r = length(p);
        let r_eff = max(r, 1.0 / max(r, 0.001)); // Lean 4 T-Dual Bounce
        let horizon = r_eff - 1.0;
        let disk = max(abs(p.y) - 0.08, sdf_sphere(p, 4.5)) - 1.8;
        let d = min(horizon, disk);
        let mat = select(11.5, 11.0, disk < horizon);
        return vec2<f32>(d, mat);
    } else {
        // Default Landscape (ID 1.0 = Generic Soil/PBR)
        let terrain = sdf_plane(p, 0.0) - noise3d(p * 0.15) * 3.0;
        return vec2<f32>(terrain, 1.0);
    }
}

// Compute SDF Normal via Tetrahedron Technique
fn calc_normal(p: vec3<f32>, scene: u32) -> vec3<f32> {
    let e = vec2<f32>(0.002, -0.002);
    return normalize(
        e.xyy * map_world(p + e.xyy, scene).x +
        e.yyx * map_world(p + e.yyx, scene).x +
        e.yxy * map_world(p + e.yxy, scene).x +
        e.xxx * map_world(p + e.xxx, scene).x
    );
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= camera.screen_width || global_id.y >= camera.screen_height) {
        return;
    }

    let pixel_idx = global_id.x + global_id.y * camera.screen_width;
    let w = f32(camera.screen_width);
    let h = f32(camera.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let ray_dir = normalize(camera.cam_dir + uv.x * camera.cam_right + uv.y * camera.cam_up);
    var ray_pos = camera.cam_pos;

    var t = 0.0;
    var t_max = 50.0;
    var hit_mat = 0.0;
    var hit = false;

    // Fast Sphere Tracing
    for (var i = 0; i < 96; i++) {
        let p = ray_pos + ray_dir * t;
        let res = map_world(p, camera.active_scene_id);
        let dist = res.x;
        hit_mat = res.y;

        if (dist < 0.002) {
            hit = true;
            break;
        }
        t += dist * 0.85;
        if (t >= t_max) {
            break;
        }
    }

    if (hit) {
        let hit_pos = ray_pos + ray_dir * t;
        let norm = calc_normal(hit_pos, camera.active_scene_id);
        let roughness = 0.35;
        
        // Write to G-Buffer Storage
        gbuffer_pos_mat[pixel_idx] = vec4<f32>(hit_pos, hit_mat);
        gbuffer_norm_rough[pixel_idx] = vec4<f32>(norm, roughness);
    } else {
        // Sky / Void background
        gbuffer_pos_mat[pixel_idx] = vec4<f32>(ray_pos + ray_dir * t_max, 0.0); // Mat ID 0 = Sky
        gbuffer_norm_rough[pixel_idx] = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
}
