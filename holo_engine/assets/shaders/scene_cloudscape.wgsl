// HoloEngine Phase 4 — High-Fidelity Volumetric Cloud Raymarching Shader
// Implements:
//   1. Continuous Density Participating Medium (No Hard SDF Surface)
//   2. 3D Worley Cellular Cauliflower Clustering + 3D FBM Micro-Erosion
//   3. Dual-Lobe Henyey-Greenstein Forward Phase & Beer-Lambert Light Marching
//   4. Dual-Source Multi-Scattering (Warm Solar Direct + Rayleigh Blue-Violet Sky Ambient)

struct RaymarchParams {
    camera_pos: vec3<f32>,
    fov: f32,
    camera_dir: vec3<f32>,
    screen_width: u32,
    camera_up: vec3<f32>,
    screen_height: u32,
    max_steps: u32,
    max_dist: f32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> params: RaymarchParams;
@group(0) @binding(1) var<storage, read_write> color_buffer: array<u32>;

// Pseudo-random hash functions
fn hash13(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

// 3D Smooth Value Noise
fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash13(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash13(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash13(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash13(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash13(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash13(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash13(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash13(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);

    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);

    return mix(nxy0, nxy1, u.z);
}

// 3D Fractional Brownian Motion (4 octaves)
fn fbm3d(p: vec3<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i++) {
        val += amp * noise3d(pos);
        pos = pos * 2.02 + vec3<f32>(1.3, 2.7, 4.1);
        amp *= 0.5;
    }
    return val;
}

// 3D Worley Cellular Noise for Cauliflower Billows (8-corner unrolled for LLVM JIT & WebGPU speed)
fn worley3d(p: vec3<f32>) -> f32 {
    let id = floor(p);
    let fd = fract(p);
    var min_dist = 1.0;

    let n000 = hash33(id + vec3<f32>(0.0, 0.0, 0.0)) - fd;
    min_dist = min(min_dist, dot(n000, n000));
    let n100 = hash33(id + vec3<f32>(1.0, 0.0, 0.0)) + vec3<f32>(1.0, 0.0, 0.0) - fd;
    min_dist = min(min_dist, dot(n100, n100));
    let n010 = hash33(id + vec3<f32>(0.0, 1.0, 0.0)) + vec3<f32>(0.0, 1.0, 0.0) - fd;
    min_dist = min(min_dist, dot(n010, n010));
    let n110 = hash33(id + vec3<f32>(1.0, 1.0, 0.0)) + vec3<f32>(1.0, 1.0, 0.0) - fd;
    min_dist = min(min_dist, dot(n110, n110));
    let n001 = hash33(id + vec3<f32>(0.0, 0.0, 1.0)) + vec3<f32>(0.0, 0.0, 1.0) - fd;
    min_dist = min(min_dist, dot(n001, n001));
    let n101 = hash33(id + vec3<f32>(1.0, 0.0, 1.0)) + vec3<f32>(1.0, 0.0, 1.0) - fd;
    min_dist = min(min_dist, dot(n101, n101));
    let n011 = hash33(id + vec3<f32>(0.0, 1.0, 1.0)) + vec3<f32>(0.0, 1.0, 1.0) - fd;
    min_dist = min(min_dist, dot(n011, n011));
    let n111 = hash33(id + vec3<f32>(1.0, 1.0, 1.0)) + vec3<f32>(1.0, 1.0, 1.0) - fd;
    min_dist = min(min_dist, dot(n111, n111));

    return clamp(1.0 - sqrt(min_dist), 0.0, 1.0);
}

// Cumulus Congestus Volumetric Density Field
fn sample_cloud_density(pos: vec3<f32>) -> f32 {
    // Altitude envelope [y = 0.5 to y = 13.5]
    let y_base = 0.5;
    let y_top = 13.5;
    if (pos.y < y_base || pos.y > y_top) {
        return 0.0;
    }

    let h_rel = (pos.y - y_base) / (y_top - y_base);
    // Asymmetric vertical density profile: sharp flat base, anvil/cauliflower top
    let height_gradient = smoothstep(0.0, 0.18, h_rel) * smoothstep(1.0, 0.45, h_rel);

    // Primary Cumulus Congestus cluster formation
    let p_scaled = pos * vec3<f32>(0.28, 0.22, 0.28);
    let center_dist = length(p_scaled - vec3<f32>(0.0, 1.2, 0.0));
    let cluster_body = clamp(1.85 - center_dist, 0.0, 2.0);

    // Secondary billowing lobes
    let lobe1 = clamp(1.3 - length(p_scaled - vec3<f32>(1.4, 0.8, -0.6)), 0.0, 1.5);
    let lobe2 = clamp(1.4 - length(p_scaled - vec3<f32>(-1.3, 1.0, 0.8)), 0.0, 1.5);
    let lobe3 = clamp(1.2 - length(p_scaled - vec3<f32>(0.3, 1.6, 1.1)), 0.0, 1.5);
    let macro_clusters = max(cluster_body, max(lobe1, max(lobe2, lobe3)));

    if (macro_clusters <= 0.05) {
        return 0.0;
    }

    // Worley Cauliflower billow carving
    let w_macro = worley3d(pos * 0.45);
    let w_micro = worley3d(pos * 1.2);
    let worley_blend = w_macro * 0.7 + w_micro * 0.3;

    // High-frequency wispy FBM erosion
    let fbm_erosion = fbm3d(pos * 1.8) * 0.38;

    // Synthesized Participating Media Density
    let raw_density = (macro_clusters + worley_blend * 0.95 - fbm_erosion - 0.45) * height_gradient;
    return clamp(raw_density * 2.8, 0.0, 1.0);
}

// Henyey-Greenstein Scattering Phase Function
fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    return (1.0 - g2) / (4.0 * 3.1415926535 * pow(max(1.0 + g2 - 2.0 * g * cos_theta, 0.001), 1.5));
}

@compute @workgroup_size(16, 16)
fn raymarch_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.screen_width || global_id.y >= params.screen_height) {
        return;
    }

    let w = f32(params.screen_width);
    let h = f32(params.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let forward = normalize(params.camera_dir);
    let right = normalize(cross(forward, params.camera_up));
    let up = cross(right, forward);

    let fov_rad = params.fov * 0.01745329;
    var ray_dir = normalize(forward + (uv.x * right + uv.y * up) * tan(fov_rad * 0.5));
    var ray_pos = params.camera_pos;

    // Solar lighting direction (warm afternoon sun)
    let sun_dir = normalize(vec3<f32>(0.65, 0.52, 0.55));
    let cos_theta = dot(ray_dir, sun_dir);

    // Dual-Lobe Henyey-Greenstein (strong forward silver lining + slight back-scatter)
    let phase_forward = henyey_greenstein(cos_theta, 0.82);
    let phase_backward = henyey_greenstein(cos_theta, -0.22);
    let phase_total = mix(phase_forward, phase_backward, 0.22);

    // Radiance Colors
    let sun_radiance = vec3<f32>(3.2, 2.7, 2.1); // Warm solar irradiance
    let sky_zenith = vec3<f32>(0.12, 0.32, 0.72); // Rayleigh blue zenith
    let sky_horizon = vec3<f32>(0.62, 0.74, 0.92); // Atmospheric horizon haze
    let sky_ambient = mix(vec3<f32>(0.42, 0.48, 0.68), vec3<f32>(0.32, 0.35, 0.54), clamp(-ray_dir.y * 1.5 + 0.4, 0.0, 1.0)); // Rich blue-violet ambient

    // Volumetric Raymarching Integration Loop
    var accum_radiance = vec3<f32>(0.0);
    var transmittance = 1.0;

    let step_size = 0.35;
    let max_march_dist = 45.0;
    var t = 2.5;

    for (var i = 0; i < 40; i++) {
        if (t > max_march_dist || transmittance < 0.02) {
            break;
        }

        let p = ray_pos + ray_dir * t;
        let density = sample_cloud_density(p);

        if (density > 0.005) {
            // Secondary light march toward the sun (Beer-Lambert attenuation)
            let l_pos1 = p + sun_dir * 0.9;
            let l_pos2 = p + sun_dir * 2.0;
            let light_tau = (sample_cloud_density(l_pos1) + sample_cloud_density(l_pos2)) * 0.9;

            // Beer-Lambert Direct Solar Transmittance
            let sun_attenuation = exp(-light_tau * 1.8);

            // Powder sugar effect (brightens translucent thin outer boundaries)
            let powder_effect = 1.0 - exp(-density * 2.8);

            // 1. Direct Sun In-Scattering (Golden / Pure White Incandescent Rims)
            let direct_sun = sun_radiance * sun_attenuation * phase_total * powder_effect;

            // 2. Ambient Sky In-Scattering (Deep Blue/Purple Shadow Tones)
            let height_fill = clamp((p.y - 0.5) / 10.0, 0.0, 1.0);
            let ambient_fill = sky_ambient * (0.35 + 0.65 * height_fill) * exp(-density * 0.4);

            let total_scatter = direct_sun + ambient_fill;

            // Primary Ray Extinction & Radiance Accumulation
            let extinction = density * 1.5;
            let delta_tau = extinction * step_size;
            let step_transmittance = exp(-delta_tau);

            accum_radiance += transmittance * total_scatter * (1.0 - step_transmittance);
            transmittance *= step_transmittance;
        }

        t += step_size;
    }

    // Sky Background Composition
    let sun_align = max(dot(ray_dir, sun_dir), 0.0);
    let sun_disk = pow(sun_align, 768.0) * 12.0;
    let sun_corona = pow(sun_align, 12.0) * 0.8;
    let sky_bg = mix(sky_horizon, sky_zenith, clamp(ray_dir.y * 1.8 + 0.1, 0.0, 1.0)) + vec3<f32>(1.0, 0.92, 0.75) * (sun_disk + sun_corona);

    var final_color = accum_radiance + transmittance * sky_bg;

    // ACES Film Tone Mapping (Preserves dynamic range and highlight purity)
    let a_aces = 2.51;
    let b_aces = 0.03;
    let c_aces = 2.43;
    let d_aces = 0.59;
    let e_aces = 0.14;
    final_color = clamp((final_color * (a_aces * final_color + b_aces)) / (final_color * (c_aces * final_color + d_aces) + e_aces), vec3<f32>(0.0), vec3<f32>(1.0));

    // Gamma correction (gamma = 2.2)
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    let pixel_idx = global_id.x + global_id.y * params.screen_width;
    let r_u = u32(clamp(final_color.r * 255.0, 0.0, 255.0));
    let g_u = u32(clamp(final_color.g * 255.0, 0.0, 255.0));
    let b_u = u32(clamp(final_color.b * 255.0, 0.0, 255.0));
    color_buffer[pixel_idx] = (255u << 24u) | (b_u << 16u) | (g_u << 8u) | r_u;
}
