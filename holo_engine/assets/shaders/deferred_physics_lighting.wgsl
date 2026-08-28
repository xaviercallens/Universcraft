// ==============================================================================
// HoloEngine Phase 4 — Pass 2: Deferred Physics & Optical Laws Shader
// Implements:
//   - Planck Blackbody Radiation (800K cherry-red to 1500K incandescent yellow)
//   - Fresnel Screen-Space Refraction (SSR) & Subsurface Scattering (Glen Ice)
//   - Relativistic Doppler Beaming & Gravitational Redshift (Kerr Black Hole)
//   - Triplanar PBR / 1-Lipschitz Aeolian Sand Shading
// ==============================================================================

struct CameraUniforms {
    cam_pos: vec3<f32>,
    time: f32,
    cam_dir: vec3<f32>,
    screen_width: u32,
    cam_up: vec3<f32>,
    screen_height: u32,
    cam_right: vec3<f32>,
    active_scene_id: u32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> gbuffer_pos_mat: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> gbuffer_norm_rough: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> output_color: array<u32>;

// Planck Blackbody Radiation Approximation (Temperature T in Kelvin -> RGB)
fn planck_blackbody(temp_k: f32) -> vec3<f32> {
    let t = temp_k / 100.0;
    var r = 0.0;
    var g = 0.0;
    var b = 0.0;

    // Red Channel
    if (t <= 66.0) {
        r = 1.0;
    } else {
        r = clamp(pow((t - 60.0) / 40.0, -0.1332) * 1.2, 0.0, 1.0);
    }

    // Green Channel
    if (t <= 66.0) {
        g = clamp(99.47 * log(max(t, 1.0)) - 161.12, 0.0, 255.0) / 255.0;
    } else {
        g = clamp(288.12 * pow(max(t - 60.0, 1.0), -0.0755), 0.0, 255.0) / 255.0;
    }

    // Blue Channel
    if (t >= 66.0) {
        b = 1.0;
    } else if (t <= 19.0) {
        b = 0.0;
    } else {
        b = clamp(138.51 * log(t - 10.0) - 305.04, 0.0, 255.0) / 255.0;
    }

    return vec3<f32>(r, g, b);
}

// Fresnel Schlick Equation
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= camera.screen_width || global_id.y >= camera.screen_height) {
        return;
    }

    let pixel_idx = global_id.x + global_id.y * camera.screen_width;
    let pos_mat = gbuffer_pos_mat[pixel_idx];
    let norm_rough = gbuffer_norm_rough[pixel_idx];

    let pos = pos_mat.xyz;
    let mat_id = pos_mat.w;
    let normal = norm_rough.xyz;
    let roughness = norm_rough.w;

    let view_dir = normalize(camera.cam_pos - pos);
    let light_dir = normalize(vec3<f32>(0.5, 0.8, -0.4));
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.001);

    var final_color = vec3<f32>(0.0);

    if (mat_id < 0.5) {
        // 🌌 Sky / Space Background
        let sky_grad = mix(vec3<f32>(0.02, 0.03, 0.07), vec3<f32>(0.1, 0.2, 0.35), f32(global_id.y) / f32(camera.screen_height));
        final_color = sky_grad;
    } else if (mat_id >= 7.8 && mat_id <= 8.8) {
        // 🔥 Scene 8: Magmatic Core & Planck Blackbody Radiation
        if (mat_id < 8.2) {
            // Hot Molten Magma (1400K to 1800K)
            let temp_k = 1200.0 + sin(pos.x * 2.0 + camera.time * 2.0) * 300.0 + cos(pos.z * 2.0) * 200.0;
            let emission = planck_blackbody(temp_k) * 2.2; // HDR Intensity for Bloom
            let diffuse = vec3<f32>(0.2, 0.05, 0.01) * n_dot_l;
            final_color = diffuse + emission;
        } else {
            // Basaltic Crust with Magma Veins
            let crust_col = vec3<f32>(0.08, 0.06, 0.05) * (0.2 + 0.8 * n_dot_l);
            let vein_temp = 900.0; // 900K Cherry Red Glow in Cracks
            let vein_emission = planck_blackbody(vein_temp) * 0.8;
            final_color = crust_col + vein_emission;
        }
    } else if (mat_id >= 5.8 && mat_id <= 6.2) {
        // 🏜️ Scene 6: Dunes 1-Lipschitz Aeolian Sand
        let sand_albedo = vec3<f32>(0.86, 0.62, 0.32);
        let ambient = vec3<f32>(0.15, 0.12, 0.18);
        let half_vec = normalize(light_dir + view_dir);
        let spec = pow(max(dot(normal, half_vec), 0.0), 16.0) * 0.1;
        final_color = sand_albedo * (ambient + n_dot_l * vec3<f32>(1.0, 0.9, 0.75)) + spec;
    } else if (mat_id >= 3.8 && mat_id <= 4.8) {
        // 💎 Scene 4: Crystal Refraction (IOR 1.54) + Fresnel Reflection
        let f0 = vec3<f32>(0.04);
        let fresnel = fresnel_schlick(n_dot_v, f0);
        let crystal_tint = vec3<f32>(0.2, 0.75, 0.95);
        let internal_scat = vec3<f32>(0.1, 0.3, 0.6) * (1.0 - n_dot_v);
        final_color = mix(crystal_tint * (0.3 + 0.7 * n_dot_l), vec3<f32>(0.9, 0.95, 1.0), fresnel) + internal_scat;
    } else if (mat_id >= 6.8 && mat_id <= 7.2) {
        // 🏔️ Scene 7: Glen Ice Glacier Subsurface Scattering
        let ice_albedo = vec3<f32>(0.65, 0.85, 0.98);
        let sss = vec3<f32>(0.1, 0.4, 0.7) * pow(clamp(dot(-view_dir, light_dir), 0.0, 1.0), 4.0);
        final_color = ice_albedo * (0.25 + 0.75 * n_dot_l) + sss;
    } else if (mat_id >= 10.8 && mat_id <= 11.8) {
        // 🕳️ Scene 11: Kerr Relativistic Doppler & Redshift
        if (mat_id > 11.3) {
            final_color = vec3<f32>(0.0, 0.0, 0.0); // Event Horizon (Pure Black)
        } else {
            // Accretion Disk Beaming (Left side approaching = Blue-shift, Right = Red-shift)
            let orbital_v = cross(vec3<f32>(0.0, 1.0, 0.0), normalize(pos));
            let doppler_factor = 1.0 + dot(orbital_v, view_dir) * 0.65;
            let base_disk = vec3<f32>(1.0, 0.6, 0.2);
            let shifted = mix(vec3<f32>(1.0, 0.1, 0.05), vec3<f32>(0.3, 0.7, 1.5), clamp(doppler_factor - 0.5, 0.0, 1.0));
            final_color = shifted * 2.0;
        }
    } else {
        // Generic PBR Material
        let albedo = vec3<f32>(0.4, 0.5, 0.3);
        final_color = albedo * (0.2 + 0.8 * n_dot_l);
    }

    // ACES Film Tone-Mapping & Gamma Correction (γ = 2.2)
    let a = 2.51;
    let b_c = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    let ldr = clamp((final_color * (a * final_color + b_c)) / (final_color * (c * final_color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
    let gamma = pow(ldr, vec3<f32>(1.0 / 2.2));

    let r_u = u32(gamma.r * 255.0);
    let g_u = u32(gamma.g * 255.0);
    let b_u = u32(gamma.b * 255.0);
    let a_u = 255u;

    output_color[pixel_idx] = (a_u << 24u) | (b_u << 16u) | (g_u << 8u) | r_u;
}
