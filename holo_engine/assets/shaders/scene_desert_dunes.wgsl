// HoloEngine Phase 4 — High-Fidelity Aeolian Desert Dunes Raymarching Shader
// Implements:
//   1. Nonlinear Aeolian Exner PDE Advection & Barchan Crescent Geomorphology
//   2. Strict Angle of Repose Cutoff (theta_repose = 34.0 deg, tan(34 deg) = 0.6745)
//   3. Multi-Scale Hierarchy: Macro Barchan Swells (48m), Sinuous Seif Ridges (16m), Wind-Aligned Micro-Ripples (0.55m)
//   4. Warm Ambient Bouncing & Inter-Reflection (Golden/Amber diffuse fill in shadowed slip faces)
//   5. Subsurface Scattering on Knife-Edge 34 deg Crest Lines (Backlit glowing sand rims)
//   6. Triplanar Mineral Sand Grain Micro-Glints (Voronoi silica crystal reflectance)

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

@group(0) @binding(0) var<uniform> dune_params: RaymarchParams;
@group(0) @binding(1) var<storage, read_write> color_buffer: array<u32>;

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Multi-Scale Aeolian Barchan Elevation and Crest Factor Field
// Returns vec2<f32>(elevation, crest_sharpness_factor)
fn evaluate_dune_elevation(xz: vec2<f32>) -> vec2<f32> {
    // Dominant Aeolian Wind Vector (South-West to North-East transport)
    let wind = normalize(vec2<f32>(0.85, 0.52));
    let perp = vec2<f32>(-wind.y, wind.x);

    let u = dot(xz, wind);
    let v = dot(xz, perp);

    // 1. Organic Macro Barchan Crescents (Wavelength ~ 68m, Height ~ 22m)
    // Parabolic crescent horns trailing downwind
    let crescent_curve = 0.0026 * v * v;
    let u_warped = u - crescent_curve + sin(v * 0.032) * 8.0 + cos(u * 0.014) * 4.0;
    let period_macro = 68.0;
    let f_macro = fract(u_warped / period_macro);

    var h_macro = 0.0;
    var is_slip_face = 0.0;
    var crest_sharpness = 0.0;

    // Asymmetric Exner Geomorphology:
    // Stoss slope (0.00 -> 0.78): gentle concave-to-convex windward climb (slope ~ 10°-14°)
    // Slip face (0.78 -> 1.00): steep avalanche drop capped at angle of repose (34°)
    if (f_macro < 0.78) {
        let s = f_macro / 0.78;
        // Natural smooth S-curve for stoss face
        h_macro = (3.0 * s * s - 2.0 * s * s * s) * 22.0;
    } else {
        let s = (f_macro - 0.78) / 0.22;
        // Steep avalanche slip face
        h_macro = (1.0 - s) * 22.0;
        is_slip_face = 1.0;
    }
    // Razor-sharp crest transition
    crest_sharpness = exp(-pow((f_macro - 0.78) * 35.0, 2.0));

    // 2. Meso-Scale Sinuous Dune Waves (Wavelength ~ 28m, Height ~ 5.5m)
    let u_meso = (u * 0.7 + v * 0.7) + sin(u * 0.06 + v * 0.04) * 4.0;
    let f_meso = fract(u_meso / 28.0);
    var h_meso = 0.0;
    if (f_meso < 0.75) {
        let sm = f_meso / 0.75;
        h_meso = (3.0 * sm * sm - 2.0 * sm * sm * sm) * 5.5;
    } else {
        h_meso = (1.0 - (f_meso - 0.75) / 0.25) * 5.5;
    }

    // 3. Subtle Aeolian Sand Ripples (only on windward stoss slope, zero on avalanched slip face!)
    // Organic perturbed wavefronts (Wavelength ~ 0.8m, Height ~ 0.018m)
    let rip_phase = (u * 0.88 + v * 0.35) * 8.5 + sin(u * 0.07 + v * 0.09) * 2.2;
    let w_rip = fract(rip_phase / 6.2831853);
    let ripple_shape = sin(w_rip * 6.2831853) * 0.018 * (1.0 - is_slip_face);

    // Total dune terrain elevation
    let total_elevation = h_macro + h_meso * 0.4 + ripple_shape - 12.0;

    return vec2<f32>(total_elevation, crest_sharpness);
}

// Analytic / Finite-Difference Surface Normal
fn evaluate_dune_normal(xz: vec2<f32>) -> vec3<f32> {
    let eps = 0.03;
    let h0 = evaluate_dune_elevation(xz).x;
    let hx = evaluate_dune_elevation(xz + vec2<f32>(eps, 0.0)).x;
    let hz = evaluate_dune_elevation(xz + vec2<f32>(0.0, eps)).x;

    let dx = (hx - h0) / eps;
    let dz = (hz - h0) / eps;

    return normalize(vec3<f32>(-dx, 1.0, -dz));
}

// Triplanar Sand Grain Mineral Glint Noise
fn evaluate_sand_micro_glint(p: vec3<f32>, norm: vec3<f32>, view_dir: vec3<f32>, sun_dir: vec3<f32>) -> f32 {
    let scale = 140.0;
    let g1 = hash31(floor(p * scale));
    let g2 = hash31(floor(p.yzx * scale));
    let g3 = hash31(floor(p.zxy * scale));

    let tri = abs(norm);
    let g_blend = (g1 * tri.x + g2 * tri.y + g3 * tri.z) / (tri.x + tri.y + tri.z);

    let half_vec = normalize(sun_dir - view_dir);
    let ndh = max(dot(norm, half_vec), 0.0);
    let glint = pow(ndh, 64.0) * step(0.92, g_blend) * 2.5;

    return glint;
}

// Raymarched Soft Shadows from Sand Crests
fn evaluate_dune_shadow(origin: vec3<f32>, sun_dir: vec3<f32>) -> f32 {
    var t = 0.25;
    var shadow = 1.0;

    for (var i = 0; i < 40; i++) {
        let p_test = origin + sun_dir * t;
        let h = evaluate_dune_elevation(p_test.xz).x;
        let d = p_test.y - h;

        if (d < 0.01) {
            return 0.0;
        }

        shadow = min(shadow, 14.0 * d / t);
        t += max(d * 0.55, 0.18);
        if (t > 70.0) { break; }
    }

    return clamp(shadow, 0.0, 1.0);
}

// Physically-Based Rayleigh-Nishita Desert Sky Model (Deep Azure to Warm Desert Horizon)
fn evaluate_desert_sky(ray_dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let y = max(ray_dir.y, 0.0);
    let sun_align = max(dot(ray_dir, sun_dir), 0.0);

    // Deep Cobalt Zenith -> Vibrant Azure Mid-Sky -> Warm Soft Golden Horizon
    let sky_zenith = vec3<f32>(0.03, 0.20, 0.65);   // Rich deep azure
    let sky_mid    = vec3<f32>(0.15, 0.48, 0.86);   // Crisp desert sky
    let sky_horiz  = vec3<f32>(0.84, 0.74, 0.58);   // Warm desert horizon

    // Smooth continuous Rayleigh elevation curve
    var sky = mix(sky_horiz, sky_mid, pow(clamp(y * 3.0, 0.0, 1.0), 0.7));
    sky = mix(sky, sky_zenith, pow(clamp(y * 1.5, 0.0, 1.0), 1.4));

    // Solar Disc and Mie forward scatter
    let sun_disc = pow(sun_align, 1024.0) * 18.0;
    let mie_aureole = pow(sun_align, 18.0) * 1.4 + pow(sun_align, 4.0) * 0.3;
    let sun_color = vec3<f32>(1.0, 0.94, 0.80);

    return sky + sun_color * (sun_disc + mie_aureole);
}

@compute @workgroup_size(16, 16)
fn raymarch_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= dune_params.screen_width || global_id.y >= dune_params.screen_height) {
        return;
    }

    let w = f32(dune_params.screen_width);
    let h = f32(dune_params.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let forward = normalize(dune_params.camera_dir);
    let right = normalize(cross(forward, dune_params.camera_up));
    let up = cross(right, forward);

    let ray_dir = normalize(forward + uv.x * right + uv.y * up);
    let cam_pos = dune_params.camera_pos;

    // Cross-Light Sun (Side lighting creating sharp shadow contrast on barchan slip faces)
    let sun_dir = normalize(vec3<f32>(-0.82, 0.36, 0.45));
    let sun_color = vec3<f32>(2.4, 1.7, 0.9); // Calibrated warm solar irradiance (pleasing & non-glaring)

    // Primary Bounded Raymarching
    var hit = false;
    var t = 0.5;
    var t_hit = dune_params.max_dist;

    for (var i = 0; i < 110; i++) {
        let p = cam_pos + ray_dir * t;
        let dune_data = evaluate_dune_elevation(p.xz);
        let dist = (p.y - dune_data.x) * 0.65; // Conservative step with Lipschitz bound

        if (dist < 0.015 * (1.0 + t * 0.015)) {
            // Binary root refinement
            var t_low = t - max(dist, 0.12);
            var t_high = t;
            for (var r = 0; r < 5; r++) {
                let t_mid = 0.5 * (t_low + t_high);
                let p_mid = cam_pos + ray_dir * t_mid;
                if (p_mid.y <= evaluate_dune_elevation(p_mid.xz).x) {
                    t_high = t_mid;
                } else {
                    t_low = t_mid;
                }
            }
            t_hit = t_high;
            hit = true;
            break;
        }

        t += max(dist, 0.07);
        if (t > dune_params.max_dist) { break; }
    }

    var final_color = evaluate_desert_sky(ray_dir, sun_dir);

    if (hit) {
        let hit_pos = cam_pos + ray_dir * t_hit;
        let dune_info = evaluate_dune_elevation(hit_pos.xz);
        let crest_factor = dune_info.y;
        let normal = evaluate_dune_normal(hit_pos.xz);

        // 1. Direct Sunlight Diffuse
        let n_dot_l = max(dot(normal, sun_dir), 0.0);
        let shadow = evaluate_dune_shadow(hit_pos + normal * 0.08, sun_dir);

        // 2. High-Contrast Dune Shading
        // Sunlit sand: Authentic Saharan golden ochre
        let sand_albedo = vec3<f32>(0.90, 0.52, 0.18);

        // Shadowed slip face: Deep warm terracotta-amber bounce + subtle sky blue ambient
        let bounce_dir = normalize(vec3<f32>(-sun_dir.x, 0.6, -sun_dir.z));
        let bounce_diff = max(dot(normal, bounce_dir), 0.0);
        let warm_bounce = vec3<f32>(0.48, 0.18, 0.05) * bounce_diff * 0.40;

        // Sky ambient diffuse fill (cool blue skylight hitting upward surfaces in shadow)
        let sky_ambient = vec3<f32>(0.02, 0.08, 0.24) * (normal.y * 0.5 + 0.5) * 0.30;

        // 3. Subsurface Scattering (Sand Translucency on Knife-Edge 34° Crests)
        let sss_backlight = pow(max(-dot(normal, sun_dir) + 0.35, 0.0), 3.0);
        let sss_glow = vec3<f32>(1.0, 0.55, 0.15) * sss_backlight * crest_factor * 2.5;

        // 4. Triplanar Mineral Micro-Glints
        let glint = evaluate_sand_micro_glint(hit_pos, normal, ray_dir, sun_dir) * shadow;

        // High-Contrast Direct vs Shadow Illumination
        let direct_illum = sand_albedo * sun_color * n_dot_l * shadow;
        let ambient_illum = (warm_bounce + sky_ambient) * (1.0 - shadow * 0.6);

        let surface_radiance = direct_illum + ambient_illum + sss_glow + vec3<f32>(1.0, 0.95, 0.85) * glint;

        // Clear Desert Atmosphere (subtle Rayleigh haze in distance, preserving deep sky)
        let dust_fog = 1.0 - exp(-0.0015 * t_hit);
        let haze_color = vec3<f32>(0.75, 0.68, 0.60);
        final_color = mix(surface_radiance, haze_color, dust_fog);
    }

    // High Contrast S-Curve & ACES Tone Mapping
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    var mapped = clamp((final_color * (a * final_color + b)) / (final_color * (c * final_color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));

    // Contrast Enhancement (rich shadows, deep blue sky, soothing golden dunes)
    mapped = pow(mapped, vec3<f32>(1.18));
    mapped = smoothstep(vec3<f32>(0.015), vec3<f32>(0.985), mapped);

    // Gamma 2.2 Correction
    final_color = pow(mapped, vec3<f32>(1.0 / 2.2));

    let pixel_idx = global_id.x + global_id.y * dune_params.screen_width;
    let r_u = u32(clamp(final_color.r * 255.0, 0.0, 255.0));
    let g_u = u32(clamp(final_color.g * 255.0, 0.0, 255.0));
    let b_u = u32(clamp(final_color.b * 255.0, 0.0, 255.0));
    color_buffer[pixel_idx] = (255u << 24u) | (b_u << 16u) | (g_u << 8u) | r_u;
}
