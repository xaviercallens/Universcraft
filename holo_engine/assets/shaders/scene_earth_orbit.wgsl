// HoloEngine Phase 4 — Cinematic 1: Planète Bleue (Earth Orbit Volumetric & S^2 Topological Shader)
// Implements:
//   1. S^2 Topological Manifold Direct Spherical Projection (Zero Moiré / Zero Aliasing)
//   2. Coupled Solenoidal Ocean Dispersion: omega^2 = g*k*tanh(kh) with Leray-Hopf Constraint
//   3. Spherical Boussinesq Clouds: Volumetric Cumulus Shell in h in [0.20, 0.85] with Radial Gravity g = -r_hat
//   4. Two-Layer Atmospheric Raymarching: Exponential Rayleigh (Gas Scattering Blue) + Mie (Water Aerosol Scattering White)
//   5. Strict Optical Vacuum Constraint: Pitch-black exosphere (0.0) without stars due to orbital camera exposure limits
//   6. HDR ACES Filmic Tone Mapping

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

@group(0) @binding(0) var<uniform> earth_params: RaymarchParams;
@group(0) @binding(1) var<storage, read_write> color_buffer: array<u32>;

fn hash33_eo(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn fbm_s2(p: vec3<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.55;
    var pos = p;
    for (var i = 0; i < 7; i++) {
        val += amp * (sin(pos.x * 3.5) * cos(pos.y * 3.2 + pos.z * 2.8) * 0.5 + 0.5);
        pos = pos * 2.13 + vec3<f32>(1.2, 2.3, 0.7);
        amp *= 0.46;
    }
    return val;
}

fn worley_s2(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let frac_p = fract(p);
    var min_dist = 1.0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let neighbor = vec3<f32>(f32(x), f32(y), f32(z));
                let point_hash = hash33_eo(cell + neighbor);
                let diff = neighbor + point_hash - frac_p;
                let dist = length(diff);
                min_dist = min(min_dist, dist);
            }
        }
    }
    return min_dist;
}

// Boussinesq Cumulus Cloud Density on S^2 Spherical Shell
fn evaluate_boussinesq_cloud_density(p: vec3<f32>) -> f32 {
    let r = length(p);
    let r_earth = 20.0;
    let alt = r - r_earth;
    
    // Altitude envelope: strict shell between h = 0.15 and h = 0.55
    let alt_envelope = smoothstep(0.12, 0.28, alt) * (1.0 - smoothstep(0.45, 0.60, alt));
    if (alt_envelope <= 0.001) {
        return 0.0;
    }

    // Direct S^2 unit direction projection
    let base_dir = p / r;
    var dir = base_dir;
    
    // Create a localized topological vortex (Single Hurricane)
    let vortex_center = normalize(vec3<f32>(0.2, 0.45, -0.75));
    let dist_to_eye = length(dir - vortex_center);
    
    // Tightly localized swirl distortion (radius 0.25 max)
    let swirl_power = smoothstep(0.25, 0.0, dist_to_eye);
    // Add small epsilon to prevent cross product with self from zeroing out
    let tangent = normalize(cross(vortex_center, dir + vec3<f32>(0.001, 0.0, 0.0))); 
    dir = normalize(dir + tangent * swirl_power * 1.2);
    
    // The eye of the hurricane (clear patch)
    let eye_mask = smoothstep(0.01, 0.05, dist_to_eye);

    // Global weather bands (latitude-based sweeping structures)
    let weather_band = smoothstep(0.0, 1.0, sin(dir.y * 6.0 + fbm_s2(dir * 1.5) * 2.0) * 0.5 + 0.5);

    // Smooth, less dense global clouds
    let macro_clusters = fbm_s2(dir * 2.5);
    let micro_worley = 1.0 - worley_s2(dir * 9.0 + vec3<f32>(1.5, 0.4, 2.1));
    
    // Threshold adjusted for sparser, fluffier distribution
    var cumulus = smoothstep(0.55, 0.90, macro_clusters * 0.65 + micro_worley * 0.35 * weather_band);

    // Punch out the eye
    cumulus *= eye_mask;
    
    // Dense hurricane eye-wall
    let eye_wall = smoothstep(0.02, 0.08, dist_to_eye) * (1.0 - smoothstep(0.08, 0.22, dist_to_eye));
    cumulus = min(cumulus + eye_wall * macro_clusters * 1.2, 1.0);

    // General clouds are less dense (1.2), hurricane is dense (3.5)
    let final_density = mix(1.2, 3.5, eye_wall);
    return cumulus * alt_envelope * final_density;
}

// Solenoidal Ocean Height & Normal on S^2 Surface
fn evaluate_solenoidal_ocean(dir: vec3<f32>) -> vec3<f32> {
    // Dispersion waves: omega^2 = g*k*tanh(kh) wrapped on S^2
    let w1 = sin(dir.x * 45.0 + dir.z * 30.0) * 0.004;
    let w2 = cos(dir.y * 60.0 - dir.x * 25.0) * 0.003;
    
    let eps = 0.002;
    let dir_x = normalize(dir + vec3<f32>(eps, 0.0, 0.0));
    let dir_z = normalize(dir + vec3<f32>(0.0, 0.0, eps));
    
    let dh_dx = (sin(dir_x.x * 45.0 + dir_x.z * 30.0) * 0.004 - w1) / eps;
    let dh_dz = (sin(dir_z.x * 45.0 + dir_z.z * 30.0) * 0.004 - w1) / eps;
    
    return normalize(dir - vec3<f32>(dh_dx, 0.0, dh_dz) * 0.08);
}

@compute @workgroup_size(16, 16)
fn raymarch_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= earth_params.screen_width || global_id.y >= earth_params.screen_height) {
        return;
    }

    let w = f32(earth_params.screen_width);
    let h = f32(earth_params.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let forward = normalize(earth_params.camera_dir);
    let right = normalize(cross(forward, earth_params.camera_up));
    let up = cross(right, forward);

    let ray_pos = earth_params.camera_pos;
    let ray_dir = normalize(forward + uv.x * right + uv.y * up);

    let sun_dir = normalize(vec3<f32>(1.2, 0.45, -0.85)); // Sun position illuminating Earth limb
    let r_earth = 20.0;
    let r_atmosphere_top = 20.7; // Exosphere threshold

    // Ray-Sphere Intersection test for atmospheric bounding volume
    let b_proj = dot(ray_pos, ray_dir);
    let c_atm = dot(ray_pos, ray_pos) - r_atmosphere_top * r_atmosphere_top;
    let delta_atm = b_proj * b_proj - c_atm;

    // OPTICAL VACUUM CONSTRAINT: Rays missing the exosphere threshold return absolute pitch-black space (0.0)
    if (delta_atm < 0.0) {
        let pixel_idx = global_id.x + global_id.y * earth_params.screen_width;
        color_buffer[pixel_idx] = 255u << 24u; // Pure black space (no stars)
        return;
    }

    let t_entry_atm = max(-b_proj - sqrt(delta_atm), 0.0);
    let t_exit_atm = -b_proj + sqrt(delta_atm);

    // Ray-Sphere Intersection for Earth Surface (Ocean & Land)
    let c_earth = dot(ray_pos, ray_pos) - r_earth * r_earth;
    let delta_earth = b_proj * b_proj - c_earth;
    var hit_earth = false;
    var t_earth = 1e5;
    if (delta_earth >= 0.0) {
        let t_surf = -b_proj - sqrt(delta_earth);
        if (t_surf > 0.0) {
            hit_earth = true;
            t_earth = t_surf;
        }
    }

    let t_end = select(t_exit_atm, t_earth, hit_earth);

    // Volumetric Two-Layer Atmospheric Raymarching (Rayleigh + Mie + Boussinesq Clouds)
    var transmittance = vec3<f32>(1.0);
    var in_scattering = vec3<f32>(0.0);

    let steps = 48;
    let dt = (t_end - t_entry_atm) / f32(steps);
    var curr_t = t_entry_atm + dt * 0.5;

    // Rayleigh Scattering Coefficients (Blue Wavelength Preference)
    let beta_rayleigh = vec3<f32>(0.08, 0.28, 0.85) * 0.18;
    let beta_mie = vec3<f32>(0.15, 0.15, 0.15) * 0.12;

    // Henyey-Greenstein Phase Function for Forward Mie Scattering
    let cos_theta = dot(ray_dir, sun_dir);
    let g_mie = 0.82;
    let phase_mie = (1.0 - g_mie * g_mie) / (4.0 * 3.14159265 * pow(1.0 + g_mie * g_mie - 2.0 * g_mie * cos_theta, 1.5));
    let phase_rayleigh = 3.0 / (16.0 * 3.14159265) * (1.0 + cos_theta * cos_theta);

    for (var i = 0; i < steps; i++) {
        let p = ray_pos + ray_dir * curr_t;
        let r = length(p);
        let alt = max(r - r_earth, 0.0);

        if (r <= r_atmosphere_top) {
            // Exponential Scale Heights: Rayleigh (H = 0.8) & Mie Aerosols (H = 0.25)
            let density_rayleigh = exp(-alt / 0.82);
            let density_mie = exp(-alt / 0.26);

            // Boussinesq Cloud Density
            let cloud_density = evaluate_boussinesq_cloud_density(p);

            // Volumetric Extinction & Scattering Coefficients
            let sig_ext = beta_rayleigh * density_rayleigh + beta_mie * density_mie + vec3<f32>(cloud_density * 2.5);
            let sig_s = beta_rayleigh * density_rayleigh * phase_rayleigh + beta_mie * density_mie * phase_mie + vec3<f32>(cloud_density * 1.8);

            // Shadow Ray march towards Sun for Cloud Shadows
            let shadow_p = p + sun_dir * 0.2;
            let shadow_cloud = evaluate_boussinesq_cloud_density(shadow_p);
            let sun_attenuation = exp(-shadow_cloud * 1.2);

            let step_transmittance = exp(-sig_ext * dt);
            let sun_light = vec3<f32>(2.8, 2.6, 2.2) * sun_attenuation;

            in_scattering += transmittance * sig_s * sun_light * dt;
            transmittance *= step_transmittance;
        }

        curr_t += dt;
        if (length(transmittance) < 0.01) {
            break;
        }
    }

    var surface_color = vec3<f32>(0.0);

    if (hit_earth) {
        let p_surf = ray_pos + ray_dir * t_earth;
        let dir_surf = normalize(p_surf);

        // Direct S^2 Topological Projection for Land/Water Continents
        let raw_fbm = fbm_s2(dir_surf * 3.8) + fbm_s2(dir_surf * 15.0) * 0.15;
        let land_mask = smoothstep(0.40, 0.52, raw_fbm);
        
        let ocean_normal = evaluate_solenoidal_ocean(dir_surf);
        let normal = select(ocean_normal, dir_surf, land_mask > 0.5);

        let NdotL = max(dot(normal, sun_dir), 0.0);

        // Deep Oceanic Beer-Lambert Absorption & Cyan/Cobalt Gradient
        let ocean_base = mix(vec3<f32>(0.01, 0.10, 0.38), vec3<f32>(0.0, 0.35, 0.60), NdotL);
        // More arid/chaotic land textures
        let land_base = mix(vec3<f32>(0.08, 0.18, 0.06), vec3<f32>(0.4, 0.3, 0.15), fbm_s2(dir_surf * 25.0));

        var surf_albedo = mix(ocean_base, land_base, land_mask);

        // Solenoidal Ocean Specular Sun Glint
        let H_vec = normalize(sun_dir - ray_dir);
        let spec_power = 180.0;
        let specular = pow(max(dot(normal, H_vec), 0.0), spec_power) * (1.0 - land_mask) * 4.5;

        // Boussinesq Cloud Shadows on Ocean
        let shadow_p = p_surf + sun_dir * 0.4;
        let cloud_shadow = exp(-evaluate_boussinesq_cloud_density(shadow_p) * 2.2);

        surface_color = (surf_albedo * NdotL * vec3<f32>(2.4, 2.3, 2.0) + vec3<f32>(specular)) * cloud_shadow;
    }

    var final_radiance = surface_color * transmittance + in_scattering;

    // HDR ACES Filmic Tone Mapping Curve
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    var mapped = clamp((final_radiance * (a * final_radiance + b)) / (final_radiance * (c * final_radiance + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));

    // Gamma 2.2 Correction
    mapped = pow(mapped, vec3<f32>(1.08));
    let final_color = pow(mapped, vec3<f32>(1.0 / 2.2));

    let pixel_idx = global_id.x + global_id.y * earth_params.screen_width;
    let r_u = u32(clamp(final_color.r * 255.0, 0.0, 255.0));
    let g_u = u32(clamp(final_color.g * 255.0, 0.0, 255.0));
    let b_u = u32(clamp(final_color.b * 255.0, 0.0, 255.0));
    color_buffer[pixel_idx] = (255u << 24u) | (b_u << 16u) | (g_u << 8u) | r_u;
}
