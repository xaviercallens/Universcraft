pub fn get_advanced_scene_wgsl(scene_id: &str) -> String {
    if scene_id == "black_hole" {
        return r#"
// HoloEngine 3D — WGSL Schwarzschild & Kerr Black Hole Relativistic Ray Marcher
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

@group(0) @binding(0) var<uniform> bh_params: RaymarchParams;
@group(0) @binding(1) var<storage, read_write> color_buffer: array<u32>;

// Hash function for stars
fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

@compute @workgroup_size(16, 16)
fn raymarch_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= bh_params.screen_width || global_id.y >= bh_params.screen_height) {
        return;
    }

    let w = f32(bh_params.screen_width);
    let h = f32(bh_params.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let forward = normalize(bh_params.camera_dir);
    let right = normalize(cross(forward, bh_params.camera_up));
    let up = cross(right, forward);

    // Initial conditions (move back to see accretion disk clearly)
    var ray_pos = bh_params.camera_pos + vec3<f32>(0.0, 3.0, 15.0); 
    // Slight downward pitch to see the disk
    let pitched_forward = normalize(forward - up * 0.2); 
    let pitched_up = normalize(cross(right, pitched_forward));
    var ray_dir = normalize(pitched_forward + uv.x * right + uv.y * pitched_up);

    let r_s = 2.0; // Schwarzschild radius
    let disk_inner = 2.5;
    let disk_outer = 8.0;

    var color = vec3<f32>(0.0);
    var trapped = false;
    var disk_color = vec3<f32>(0.0);
    var hit_disk = false;

    let dt = 0.05;
    for (var step = 0; step < i32(bh_params.max_steps); step++) {
        let r = length(ray_pos);
        if (r <= r_s) {
            trapped = true;
            break;
        }

        // Curved spacetime photon geodesic deflection acceleration: a = -1.5 * r_s / r^5 * (r x L) x r
        let grav_accel = -1.5 * r_s / (r * r * r * r * r) * cross(ray_pos, cross(ray_pos, ray_dir));
        ray_dir = normalize(ray_dir + grav_accel * dt);
        
        let next_pos = ray_pos + ray_dir * dt;

        // Check accretion disk intersection in equatorial plane y = 0
        if (ray_pos.y * next_pos.y < 0.0) { 
            let t_intersect = -ray_pos.y / ray_dir.y;
            let intersect_p = ray_pos + ray_dir * t_intersect;
            let d_center = length(intersect_p);
            
            if (d_center >= disk_inner && d_center <= disk_outer) {
                hit_disk = true;
                let v_orbit = sqrt(r_s / (2.0 * d_center)); 
                // Doppler shift approximation 
                let doppler = 1.0 + v_orbit * ray_dir.x * 2.0; 
                
                let ring = sin(d_center * 15.0) * 0.5 + 0.5;
                let intensity = (1.0 - (d_center - disk_inner) / (disk_outer - disk_inner)) * ring;
                // Thermally shifted colors
                let temp_color = vec3<f32>(1.0, 0.4, 0.1) * intensity * pow(abs(doppler), 3.0) * 2.5;
                
                disk_color += temp_color;
            }
        }
        
        ray_pos = next_pos;

        if (r > 30.0) {
            break;
        }
    }

    if (trapped) {
        color = vec3<f32>(0.0); // Event Horizon
    } else {
        let star_dir = ray_dir;
        let star_hash = hash33(floor(star_dir * 300.0));
        var star_glow = 0.0;
        if (star_hash.x > 0.99) {
            star_glow = pow(star_hash.y, 4.0) * 2.5;
        }
        
        // Milky Way galaxy band
        let gal_noise = sin(star_dir.x * 12.0) * cos(star_dir.y * 15.0 + star_dir.z * 10.0) * 0.5 + 0.5;
        let gal_band = exp(-pow(star_dir.y * 3.0, 2.0)); // Concentrate at equator
        let gal_color = vec3<f32>(0.1, 0.25, 0.5) * pow(gal_noise, 3.0) * gal_band;
        
        color = vec3<f32>(star_glow) + gal_color;
        
        if (hit_disk) {
            color += disk_color; // Additive blending
        }
    }

    // HDR Tone mapping (ACES approx)
    let a_aces = 2.51;
    let b_aces = 0.03;
    let c_aces = 2.43;
    let d_aces = 0.59;
    let e_aces = 0.14;
    color = clamp((color * (a_aces * color + b_aces)) / (color * (c_aces * color + d_aces) + e_aces), vec3<f32>(0.0), vec3<f32>(1.0));

    let pixel_idx = global_id.x + (bh_params.screen_height - 1u - global_id.y) * bh_params.screen_width;
    let r_u = u32(clamp(color.r * 255.0, 0.0, 255.0));
    let g_u = u32(clamp(color.g * 255.0, 0.0, 255.0));
    let b_u = u32(clamp(color.b * 255.0, 0.0, 255.0));
    color_buffer[pixel_idx] = (255u << 24u) | (b_u << 16u) | (g_u << 8u) | r_u;
}
        "#.to_string();
    }

    let (sdf_logic, color_logic) = match scene_id {
        "ocean_sunset" => (
            r#"
            // Scene 1: Ocean Sunset (Water)
            let water_height = sin(p.x * 2.0 + p.z * 1.5) * 0.1 + sin(p.x * 0.5) * 0.2;
            let d_water = p.y - water_height;
            return vec2<f32>(d_water, 1.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(1.0, 0.2, 1.0)); // low sunset
            
            // Sky background
            let sun_amount = pow(max(dot(ray_dir, sun_dir), 0.0), 128.0);
            let sun_disk = pow(max(dot(ray_dir, sun_dir), 0.0), 1024.0);
            let sky = mix(vec3<f32>(0.1, 0.2, 0.4), vec3<f32>(0.8, 0.4, 0.2), clamp(1.0 - ray_dir.y, 0.0, 1.0));
            let sky_final = sky + vec3<f32>(1.0, 0.6, 0.2) * sun_amount + vec3<f32>(1.0, 0.9, 0.8) * sun_disk;
            
            if (mat_id == 1.0) {
                // Water surface
                let diff = max(dot(normal, sun_dir), 0.0);
                
                // GGX approx specular
                let half_vec = normalize(sun_dir - ray_dir);
                let ndh = max(dot(normal, half_vec), 0.0);
                let specular = pow(ndh, 128.0) * 2.0;
                
                // Fresnel
                let f0 = 0.02;
                let fresnel = f0 + (1.0 - f0) * pow(1.0 - max(dot(normal, -ray_dir), 0.0), 5.0);
                
                let water_base = mix(vec3<f32>(0.01, 0.1, 0.2), vec3<f32>(0.0, 0.2, 0.3), diff);
                let reflected_sky = mix(vec3<f32>(0.1, 0.2, 0.4), vec3<f32>(0.8, 0.4, 0.2), clamp(1.0 - reflect(ray_dir, normal).y, 0.0, 1.0))
                                    + vec3<f32>(1.0, 0.8, 0.5) * specular;
                                    
                color = mix(water_base, reflected_sky, fresnel);
            }
            
            // Horizon fog blending
            let fog_factor = 1.0 - exp(-0.01 * t);
            color = mix(color, sky_final, fog_factor);
            "#
        ),
        "forest_soil" => (
            r#"
            // Scene 2: Forest & Soil (Optimized SDF)
            let soil = p.y + 1.5; // removed ground_noise from distance
            
            // Domain repetition with Poisson-like chaos
            let spacing = 8.0;
            let cell = floor((p.xz + vec2<f32>(spacing*0.5)) / spacing);
            let offset = vec2<f32>(hash21(cell), hash21(cell + vec2<f32>(1.3, 2.7))) * spacing * 0.6 - (spacing * 0.3);
            let local_p = vec3<f32>(p.x - cell.x * spacing - offset.x, p.y, p.z - cell.y * spacing - offset.y);
            
            // Tree trunk
            let trunk_r = 0.3 + hash21(cell)*0.2 - local_p.y*0.02; // tapered
            let trunk = length(local_p.xz) - trunk_r; // removed trunk_noise
            let trunk_d = max(trunk, abs(local_p.y - 1.0) - 2.5);
            
            // Canopy leaves
            let canopy_h = 3.5 + hash21(cell)*1.5;
            let c_center = vec3<f32>(0.0, canopy_h, 0.0);
            let c_dist = length(local_p - c_center);
            let leaves = c_dist - 2.0; // removed leaf_noise
            
            let d_tree = min(trunk_d, leaves);
            
            if (soil < d_tree) { return vec2<f32>(soil, 2.0); }
            if (trunk_d < leaves) { return vec2<f32>(d_tree, 3.0); }
            return vec2<f32>(d_tree, 4.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(0.8, 0.6, 0.3)); // Golden hour
            let diff = max(dot(normal, sun_dir), 0.0);
            
            // Expensive lighting only if we actually hit something (not sky)
            var ao = 1.0;
            var shadow = 1.0;
            if (t < ray_params.max_dist) {
                ao = calc_ao(hit_pos, normal);
                shadow = calc_softshadow(hit_pos + normal * 0.05, sun_dir, 0.1, 20.0, 4.0);
            }
            
            if (mat_id == 2.0) { 
                let base_color = mix(vec3<f32>(0.1, 0.08, 0.05), vec3<f32>(0.25, 0.2, 0.1), fbm3(hit_pos * 2.0));
                color = base_color * (diff * shadow * 0.8 + 0.2) * ao; 
            }
            else if (mat_id == 3.0) { 
                let bark = mix(vec3<f32>(0.1, 0.05, 0.02), vec3<f32>(0.2, 0.1, 0.05), fbm3(hit_pos * 5.0));
                color = bark * (diff * shadow * 0.8 + 0.2) * ao; 
            }
            else if (mat_id == 4.0) { 
                // Subsurface scattering
                let sss = pow(max(dot(ray_dir, sun_dir), 0.0), 6.0) * 0.4;
                let leaf_col = mix(vec3<f32>(0.05, 0.2, 0.05), vec3<f32>(0.3, 0.6, 0.1), fbm3(hit_pos * 3.0));
                color = (leaf_col * (diff * shadow * 0.6 + 0.4) + vec3<f32>(0.6, 0.9, 0.2) * sss) * ao; 
            }
            
            // Atmospheric God Rays (volumetric approx)
            let sun_amount = pow(max(dot(ray_dir, sun_dir), 0.0), 16.0);
            let sun_disk = pow(max(dot(ray_dir, sun_dir), 0.0), 512.0);
            let sky_base = mix(vec3<f32>(0.4, 0.6, 0.8), vec3<f32>(0.1, 0.3, 0.6), clamp(ray_dir.y, 0.0, 1.0));
            
            let fog_color = sky_base + vec3<f32>(1.0, 0.8, 0.5) * sun_amount * 0.8 + vec3<f32>(1.0, 1.0, 1.0) * sun_disk;
            let fog_factor = 1.0 - exp(-0.015 * t);
            color = mix(color, fog_color, fog_factor);
            "#
        ),
        "cloudscape" => (
            r#"
            // Scene 3: Cloudscape (Optimized SDF)
            // Base plane
            let d_clouds = p.y - 4.0; // removed cloud_noise
            // Worley-like cellular carving for cauliflower shapes
            let cellular = abs(sin(p.x*0.5)*sin(p.z*0.5)) * 2.0;
            return vec2<f32>(d_clouds + cellular, 5.0);
            "#,
            r#"
            if (mat_id == 5.0) {
                let light_dir = normalize(vec3<f32>(0.5, 0.8, -0.2));
                // Beer-Lambert approx for self-shadowing based on normal facing light
                let diff = max(dot(normal, light_dir), 0.0);
                
                // Silver lining (forward scattering / Henyey-Greenstein approx)
                let hg = pow(max(dot(ray_dir, light_dir), 0.0), 4.0) * 0.5;
                
                let cloud_base = vec3<f32>(0.4, 0.45, 0.5); // dark core
                let cloud_top = vec3<f32>(1.0, 1.0, 1.0); // sunlit edges
                
                color = mix(cloud_base, cloud_top, diff) + vec3<f32>(1.0, 0.9, 0.7) * hg;
            }
            "#
        ),
        "crystal_cave" => (
            r#"
            // Scene 4: Crystal Cave (Optimized SDF)
            let cave_radius = 6.0; // removed fbm3
            let cave = -(length(p.xy) - cave_radius);
            
            // Hexagonal crystals array
            let spacing = 3.0;
            let cell = floor(p.xz / spacing);
            let cx = p.x - cell.x * spacing - spacing*0.5;
            let cz = p.z - cell.y * spacing - spacing*0.5;
            
            // Hexagon SDF (approx)
            let q = abs(vec2<f32>(cx, cz));
            let hex_d = max(q.x * 0.866025 + q.y * 0.5, q.y) - (0.4 + hash21(cell)*0.4);
            // Height bounded by cave floor/ceiling
            let cr_height = 3.0 + hash21(cell + 1.5) * 4.0;
            let crystal = max(hex_d, abs(p.y) - cr_height);
            
            if (cave < crystal) { return vec2<f32>(cave, 6.0); }
            return vec2<f32>(crystal, 7.0);
            "#,
            r#"
            let light_dir = normalize(vec3<f32>(0.0, 0.0, 1.0)); // Headlamp direction
            let diff = max(dot(normal, light_dir), 0.1);
            
            if (mat_id == 6.0) { 
                // Wet cave wall
                let spec = pow(max(dot(reflect(-light_dir, normal), -ray_dir), 0.0), 16.0) * 0.3;
                color = vec3<f32>(0.1, 0.1, 0.15) * diff + vec3<f32>(0.2, 0.4, 0.5) * spec; 
            }
            else if (mat_id == 7.0) { 
                // Glowing/Refractive Crystal
                let spec = pow(max(dot(reflect(-light_dir, normal), -ray_dir), 0.0), 128.0);
                let fresnel = pow(1.0 - max(dot(normal, -ray_dir), 0.0), 4.0);
                let crystal_col = vec3<f32>(0.1, 0.8, 0.6);
                
                // Emissive internal glow
                let emissive = crystal_col * 0.8 * (0.5 + 0.5 * sin(hit_pos.y * 5.0 + t));
                
                color = crystal_col * diff * 0.5 + vec3<f32>(1.0) * spec + crystal_col * fresnel + emissive;
            }
            
            // Cave mist / fog
            let fog = 1.0 - exp(-0.02 * t);
            color = mix(color, vec3<f32>(0.05, 0.1, 0.15), fog);
            "#
        ),
        "floating_islands" => (
            r#"
            // Scene 5: Floating Islands
            let spacing = 15.0;
            let cell = floor(p.xz / spacing);
            let local_xz = p.xz - cell * spacing - spacing*0.5;
            
            // FBM erosion for inverted karst relief
            let erosion = fbm3(p * 0.8) * 2.0;
            let base_island = length(vec2<f32>(local_xz.x, local_xz.y)) - (3.0 + hash21(cell)*2.0) + erosion;
            
            // Tapering bottom and flat top
            let top_cut = p.y - (4.0 + hash21(cell+1.5)*3.0);
            let bottom_taper = -p.y + (0.0 - hash21(cell)*4.0) + length(local_xz)*0.5; // cone shape
            
            let final_island = max(max(base_island, top_cut), bottom_taper);
            
            // Sea of clouds below
            let cloud_sea = p.y + 6.0 + fbm3(p * 0.2) * 3.0;
            
            if (cloud_sea < final_island) { return vec2<f32>(cloud_sea, 5.0); }
            return vec2<f32>(final_island, 8.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
            let diff = max(dot(normal, sun_dir), 0.0);
            
            if (mat_id == 5.0) { // Sea of clouds
                color = mix(vec3<f32>(0.6, 0.7, 0.8), vec3<f32>(1.0), diff);
            }
            else if (mat_id == 8.0) { // Island
                let is_top = smoothstep(0.5, 0.8, normal.y);
                let rock_col = vec3<f32>(0.4, 0.45, 0.4) * (diff * 0.8 + 0.2);
                let grass_col = vec3<f32>(0.2, 0.6, 0.2) * (diff * 0.9 + 0.1);
                color = mix(rock_col, grass_col, is_top);
            }
            
            // Atmospheric perspective (Skybox)
            let sky_col = mix(vec3<f32>(0.4, 0.6, 0.9), vec3<f32>(0.1, 0.3, 0.6), clamp(ray_dir.y, 0.0, 1.0));
            let fog = 1.0 - exp(-0.005 * t);
            color = mix(color, sky_col, fog);
            "#
        ),
        "desert_dunes" => (
            r#"
            // Scene 6: Desert Dunes
            // Base sine waves for large dunes
            var dune = p.y + sin(p.x * 0.2 + sin(p.z * 0.15)) * 4.0;
            dune += sin(p.z * 0.3 - p.x * 0.1) * 2.0;
            
            // Micro-ripples (bump mapping base)
            let ripples = sin(p.x * 15.0 + p.z * 5.0) * 0.03;
            return vec2<f32>(dune + ripples, 9.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(0.9, 0.2, 0.3)); // Golden hour grazing sun
            
            var shadow = 1.0;
            if (t < ray_params.max_dist) {
                shadow = calc_softshadow(hit_pos + normal * 0.05, sun_dir, 0.1, 30.0, 6.0);
            }

            if (mat_id == 9.0) {
                let sky_dir = normalize(vec3<f32>(-0.5, 0.9, -0.5));
                
                let sun_diff = max(dot(normal, sun_dir), 0.0);
                let sky_diff = max(dot(normal, sky_dir), 0.0) * 0.4;
                
                let sun_col = vec3<f32>(1.0, 0.6, 0.2) * shadow;
                let sky_col = vec3<f32>(0.2, 0.4, 0.8); // Ambient blue fill
                
                let sand_albedo = vec3<f32>(0.9, 0.7, 0.4);
                
                color = sand_albedo * (sun_col * sun_diff + sky_col * sky_diff);
            }
            
            // Heat haze / Horizon dust
            let sun_disk = pow(max(dot(ray_dir, sun_dir), 0.0), 512.0) * 2.0;
            let sun_glow = pow(max(dot(ray_dir, sun_dir), 0.0), 32.0) * 0.5;
            let sky_bg = mix(vec3<f32>(0.8, 0.5, 0.3), vec3<f32>(0.2, 0.4, 0.8), clamp(ray_dir.y * 5.0, 0.0, 1.0)) 
                         + vec3<f32>(1.0, 0.8, 0.4) * sun_glow 
                         + vec3<f32>(1.0, 1.0, 1.0) * sun_disk;
                         
            let fog = 1.0 - exp(-0.01 * t);
            color = mix(color, sky_bg, fog);
            "#
        ),
        "ice_glacier" => (
            r#"
            // Scene 7: Ice Glacier (Optimized SDF)
            // Voronoi-like erosion for seracs and crevasses
            let cell = floor(p.xz * 0.3);
            let h = hash21(cell) * 2.0;
            let erosion = abs(sin(p.x * 2.0 + h) * sin(p.z * 1.5 - h)) * 1.5;
            let ice = p.y + 1.0 - erosion; // removed fbm3 from distance
            
            return vec2<f32>(ice, 10.0);
            "#,
            r#"
            if (mat_id == 10.0) {
                let sun_dir = normalize(vec3<f32>(0.2, 0.8, -0.6));
                let diff = max(dot(normal, sun_dir), 0.1);
                
                // Micro-specularity for snow sparkle
                let sparkle_noise = hash31(hit_pos * 50.0);
                let sparkle = step(0.95, sparkle_noise) * pow(max(dot(reflect(-sun_dir, normal), -ray_dir), 0.0), 32.0);
                
                let specular = pow(max(dot(reflect(-sun_dir, normal), -ray_dir), 0.0), 64.0);
                
                // Deep Ice Absorption (Cyan SSS)
                let depth_extinction = exp(-max(0.0, hit_pos.y + 1.0) * 2.0);
                let ice_col = mix(vec3<f32>(0.9, 0.95, 1.0), vec3<f32>(0.1, 0.5, 0.8), depth_extinction);
                
                color = ice_col * diff + vec3<f32>(1.0) * (specular + sparkle);
            }
            "#
        ),
        "volcano_core" => (
            r#"
            // Scene 8: Volcano Core (Optimized SDF)
            let rock_base = p.y + 1.0;
            // Voronoi plates for cracked basalt crust
            let scale = 1.5;
            let p_scaled = p.xz * scale;
            let i_p = floor(p_scaled);
            let f_p = fract(p_scaled);
            
            var min_dist = 1.0;
            for (var y = -1; y <= 1; y++) {
                for (var x = -1; x <= 1; x++) {
                    let neighbor = vec2<f32>(f32(x), f32(y));
                    let pt = vec2<f32>(hash21(i_p + neighbor), hash21(i_p + neighbor + vec2<f32>(12.3, 45.6)));
                    let diff = neighbor + pt - f_p;
                    let d = length(diff);
                    min_dist = min(min_dist, d);
                }
            }
            
            // Lava is in the cracks
            let crack_depth = smoothstep(0.1, 0.2, min_dist) * 0.5;
            let rock = rock_base - crack_depth; // removed fbm3 from distance
            
            return vec2<f32>(rock, 11.0);
            "#,
            r#"
            if (mat_id == 11.0) { 
                let scale = 1.5;
                let p_scaled = hit_pos.xz * scale;
                let i_p = floor(p_scaled);
                let f_p = fract(p_scaled);
                
                var min_dist = 1.0;
                for (var y = -1; y <= 1; y++) {
                    for (var x = -1; x <= 1; x++) {
                        let neighbor = vec2<f32>(f32(x), f32(y));
                        let pt = vec2<f32>(hash21(i_p + neighbor), hash21(i_p + neighbor + vec2<f32>(12.3, 45.6)));
                        let diff = neighbor + pt - f_p;
                        min_dist = min(min_dist, length(diff));
                    }
                }
                
                // Blackbody emission in cracks
                let is_crack = 1.0 - smoothstep(0.05, 0.15, min_dist);
                let lava_temp = is_crack * (0.8 + 0.2 * sin(t * 2.0 + hit_pos.x * 5.0)); // 0.0 to 1.0
                
                let basalt_col = vec3<f32>(0.05, 0.05, 0.05);
                let lava_col = vec3<f32>(1.0, 0.3, 0.0) * lava_temp + vec3<f32>(1.0, 0.9, 0.5) * pow(lava_temp, 4.0);
                
                let light_dir = normalize(vec3<f32>(0.0, 1.0, 0.0));
                let diff = max(dot(normal, light_dir), 0.1);
                
                color = mix(basalt_col * diff, lava_col, is_crack);
                
                // Volcanic smoke
                let smoke_dens = fbm3(hit_pos * 0.5 + vec3<f32>(0.0, t*0.5, 0.0));
                let smoke_alpha = smoothstep(0.4, 1.0, smoke_dens) * clamp(hit_pos.y * 0.2, 0.0, 1.0);
                color = mix(color, vec3<f32>(0.1, 0.08, 0.08), smoke_alpha * 0.8);
            }
            "#
        ),
        "alien_planet" => (
            r#"
            // Scene 9: Alien Planet (Optimized SDF)
            let ground = p.y + 2.0; // removed fbm3
            
            // Fractal flora (Mandelbulb-like iteration)
            var q = p;
            q.y -= -1.0; // Raise flora
            
            let spacing = 10.0;
            let cell = floor((q.xz + vec2<f32>(spacing*0.5)) / spacing);
            q.x = q.x - cell.x * spacing;
            q.z = q.z - cell.y * spacing;
            
            // Organic twist
            let angle = q.y * 0.5;
            let s = sin(angle); let c = cos(angle);
            q = vec3<f32>(q.x*c - q.z*s, q.y, q.x*s + q.z*c);
            
            // Branching distance
            let stem = length(q.xz) - 0.5 + p.y*0.1; // removed fbm3
            let bulb = length(q - vec3<f32>(0.0, 3.0, 0.0)) - 1.5; // removed fbm3
            let flora = min(max(stem, p.y - 4.0), bulb);
            
            if (ground < flora) { return vec2<f32>(ground, 13.0); }
            return vec2<f32>(flora, 14.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(0.5, 0.8, -0.5));
            let diff = max(dot(normal, sun_dir), 0.1);
            
            if (mat_id == 13.0) { // Ground
                color = vec3<f32>(0.1, 0.05, 0.2) * diff; 
            }
            else if (mat_id == 14.0) { // Flora
                let vein_noise = sin(hit_pos.y * 20.0 - t * 5.0) * sin(hit_pos.x * 20.0 + hit_pos.z * 20.0);
                let is_vein = smoothstep(0.8, 1.0, vein_noise);
                
                let plant_base = vec3<f32>(0.2, 0.0, 0.3) * diff;
                let bioluminescence = vec3<f32>(0.0, 1.0, 0.8) * is_vein * 2.0; // Cyan/Neon glow
                
                color = plant_base + bioluminescence;
            }
            
            // Multi-moon atmosphere & Ethanol mist
            let sky_col = mix(vec3<f32>(0.3, 0.0, 0.4), vec3<f32>(0.0, 0.8, 0.5), clamp(ray_dir.y, 0.0, 1.0));
            let fog = 1.0 - exp(-0.02 * t);
            color = mix(color, sky_col, fog);
            "#
        ),
        "earth_orbit" => (
            r#"
            // Cinematic 1: Earth Orbit
            let earth_radius = 6371.0;
            let earth = length(p) - earth_radius;
            
            // Basic Gerstner-like waves on surface for ocean
            let waves = sin(p.x * 0.5 + p.z * 0.2) * 0.2 + sin(p.z * 0.8 - p.x * 0.4) * 0.1;
            
            return vec2<f32>(earth + waves, 20.0);
            "#,
            r#"
            if (mat_id == 20.0) {
                let sun_dir = normalize(vec3<f32>(1.0, 0.2, 0.5));
                let diff = max(dot(normal, sun_dir), 0.0);
                
                // Fresnel specular for ocean reflection
                let specular = pow(max(dot(reflect(-sun_dir, normal), -ray_dir), 0.0), 128.0) * 2.0;
                let ocean_col = vec3<f32>(0.02, 0.1, 0.4);
                let land_col = vec3<f32>(0.1, 0.3, 0.1);
                
                // Very simple mask for land/water (FBM)
                let is_land = smoothstep(0.4, 0.6, fbm3(hit_pos * 0.005));
                let surface_col = mix(ocean_col, land_col, is_land);
                let final_spec = mix(specular, 0.0, is_land); // Only water is highly specular
                
                // City lights on night side
                let night_mask = 1.0 - smoothstep(-0.2, 0.2, dot(normal, sun_dir));
                let city_noise = step(0.8, hash31(hit_pos * 2.0)) * fbm3(hit_pos * 0.5);
                let city_lights = vec3<f32>(1.0, 0.8, 0.3) * city_noise * night_mask * is_land * 2.0;
                
                color = surface_col * diff + vec3<f32>(1.0, 0.95, 0.85) * final_spec + city_lights;
            }
            
            // Atmosphere Limb (Volumetric)
            let sun_dir = normalize(vec3<f32>(1.0, 0.2, 0.5));
            var atm_color = vec3<f32>(0.0);
            var atm_pos = ray_params.camera_pos;
            let atm_step = t / 15.0; // Reduced from 40 to 15 for performance
            
            for (var i = 0u; i < 15u; i++) {
                let r = length(atm_pos);
                if (r > 6371.0 && r < 6390.0) {
                    let alt = r - 6371.0;
                    let atm_density = exp(-alt * 0.15);
                    let rayleigh = vec3<f32>(0.1, 0.4, 1.0) * atm_density;
                    
                    // Cloud layer
                    let cloud_alt = abs(alt - 3.0);
                    var cloud_dens = 0.0;
                    if (cloud_alt < 1.0) {
                        cloud_dens = fbm3(atm_pos * 0.2) * (1.0 - cloud_alt);
                        if (cloud_dens > 0.4) {
                            atm_color += vec3<f32>(1.0) * (cloud_dens - 0.4) * atm_step * 0.2;
                        }
                    }
                    
                    atm_color += rayleigh * atm_step * 0.05;
                }
                atm_pos += ray_dir * atm_step;
            }
            
            let phase = max(dot(ray_dir, sun_dir), 0.0);
            color += atm_color * (phase * 0.8 + 0.2);
            
            // Cinematic Background (Sun & Milky Way Starfield)
            if (!hit) {
                // Milky Way Dust
                let mw_noise = fbm3(ray_dir * 15.0);
                let mw_glow = smoothstep(0.3, 0.7, mw_noise);
                let mw_color = mix(vec3<f32>(0.05, 0.02, 0.08), vec3<f32>(0.1, 0.2, 0.4), ray_dir.y * 0.5 + 0.5);
                color += mw_color * mw_glow * 0.5;
                
                // Procedural Stars
                let star_hash = hash31(ray_dir * 800.0);
                let star = step(0.998, star_hash) * pow((star_hash - 0.998) * 500.0, 2.0);
                // Twinkle
                let twinkle = sin(ray_params.camera_pos.x * 0.01 + star_hash * 100.0) * 0.5 + 0.5;
                color += vec3<f32>(star * twinkle);
                
                // Cinematic Sun Flare
                let sun_amount = pow(max(dot(ray_dir, sun_dir), 0.0), 200.0);
                let sun_core = pow(max(dot(ray_dir, sun_dir), 0.0), 2000.0);
                color += vec3<f32>(1.0, 0.8, 0.5) * sun_amount * 1.5;
                color += vec3<f32>(1.0, 1.0, 1.0) * sun_core * 5.0;
            }
            "#
        ),
        "continental_biomes" => (
            r#"
            r#"
            // Cinematic 2: Continental Biomes (Dunes to Rainforest)
            
            // 1. Organic Ecotones (Biome Blending)
            // We use X axis as a proxy for "Temperature/Humidity" gradient
            let biome_threshold = p.x * 0.1; 
            // Noise breaks the linear frontier into a natural fractal edge
            let ecotone_noise = fbm3(p * 0.5) * 2.0; 
            let blend = smoothstep(-1.5, 1.5, biome_threshold + ecotone_noise); // 0 = Desert, 1 = Jungle
            
            // 2. Dunes (Desert Biome)
            let dune = p.y + sin(p.x * 0.2 + sin(p.z * 0.15)) * 4.0 + sin(p.z * 0.3 - p.x * 0.1) * 2.0;
            let ripples = sin(p.x * 15.0 + p.z * 5.0) * 0.03;
            let d_dunes = dune + ripples;
            
            // 3. Forest (Jungle Biome)
            let soil = p.y + 1.5;
            let spacing = 6.0;
            let cell = floor((p.xz + vec2<f32>(spacing*0.5)) / spacing);
            let offset = vec2<f32>(hash21(cell), hash21(cell + 1.5)) * spacing * 0.6 - (spacing * 0.3);
            let local_p = vec3<f32>(p.x - cell.x * spacing - offset.x, p.y, p.z - cell.y * spacing - offset.y);
            let trunk = length(local_p.xz) - 0.4;
            let leaves = length(local_p - vec3<f32>(0.0, 3.5, 0.0)) - 2.5;
            let d_forest = min(soil, min(max(trunk, abs(local_p.y - 1.0) - 2.5), leaves));
            
            // Smooth Min for topological blending of the terrain
            let h = clamp(0.5 + 0.5 * (d_dunes - d_forest) / 2.0, 0.0, 1.0);
            let final_d = mix(d_dunes, d_forest, h) - 2.0 * h * (1.0 - h);
            
            // Encode the blend factor into the material ID (fractional part)
            return vec2<f32>(final_d, 21.0 + blend);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(0.9, 0.2, 0.3));
            let sky_dir = normalize(vec3<f32>(-0.5, 0.9, -0.5));
            let sun_diff = max(dot(normal, sun_dir), 0.0);
            let sky_diff = max(dot(normal, sky_dir), 0.0) * 0.4;
            
            // Extract the Biome Blend Factor
            let blend = fract(mat_id); 
            
            var shadow = 1.0;
            var ao = 1.0;
            if (t < ray_params.max_dist) {
                shadow = calc_softshadow(hit_pos + normal * 0.05, sun_dir, 0.1, 25.0, 5.0);
                ao = calc_ao(hit_pos, normal);
            }
            
            // PBR Triplanar Mapping Simulation (Fake procedural textures mapped from 3 axes)
            let bf = abs(normal);
            let tri_weights = bf / (bf.x + bf.y + bf.z);
            let tex_scale = 0.5;
            
            // Desert Triplanar Texture
            let sand_noise = fbm3(hit_pos * tex_scale) * tri_weights.x 
                           + fbm3(hit_pos.yzx * tex_scale) * tri_weights.y 
                           + fbm3(hit_pos.zxy * tex_scale) * tri_weights.z;
            let sand_albedo = mix(vec3<f32>(0.9, 0.7, 0.4), vec3<f32>(0.7, 0.5, 0.3), sand_noise);
            
            // Jungle Triplanar Texture (Soil/Moss)
            let moss_noise = fbm3(hit_pos * tex_scale * 2.0) * tri_weights.x 
                           + fbm3(hit_pos.yzx * tex_scale * 2.0) * tri_weights.y 
                           + fbm3(hit_pos.zxy * tex_scale * 2.0) * tri_weights.z;
            let moss_albedo = mix(vec3<f32>(0.1, 0.2, 0.05), vec3<f32>(0.2, 0.4, 0.1), moss_noise);
            
            // Biome Blending using the Ecotone smoothstep
            let terrain_albedo = mix(sand_albedo, moss_albedo, blend);
            
            // Final Color Compositing
            let sun_col = vec3<f32>(1.0, 0.8, 0.5) * shadow;
            let sky_col = vec3<f32>(0.2, 0.4, 0.8);
            color = terrain_albedo * (sun_col * sun_diff + sky_col * sky_diff) * ao;
            
            // Global Fog
            let fog = 1.0 - exp(-0.01 * t);
            let sky_bg = mix(vec3<f32>(0.8, 0.6, 0.4), vec3<f32>(0.2, 0.4, 0.8), clamp(ray_dir.y * 2.0, 0.0, 1.0));
            color = mix(color, sky_bg, fog);
            "#
        ),
        "arctic_aurora" => (
            r#"
            // Cinematic 3: Arctic Aurora
            let cell = floor(p.xz * 0.2);
            let h = hash21(cell) * 3.0;
            let erosion = abs(sin(p.x * 1.5 + h) * sin(p.z * 1.2 - h)) * 2.0;
            let ice = p.y + 2.0 - erosion; // removed fbm3 from distance
            
            return vec2<f32>(ice, 24.0);
            "#,
            r#"
            if (mat_id == 24.0) {
                let moon_dir = normalize(vec3<f32>(-0.5, 0.6, 0.8));
                let diff = max(dot(normal, moon_dir), 0.0);
                let specular = pow(max(dot(reflect(-moon_dir, normal), -ray_dir), 0.0), 128.0);
                let sparkle = step(0.98, hash31(hit_pos * 60.0)) * specular;
                let depth = exp(-max(0.0, hit_pos.y + 2.0) * 1.5);
                let ice_col = mix(vec3<f32>(0.8, 0.9, 1.0), vec3<f32>(0.0, 0.17, 0.28), depth);
                color = ice_col * (diff * 0.5 + 0.1) + vec3<f32>(0.8, 0.9, 1.0) * (specular + sparkle);
            }
            
            if (ray_dir.y > 0.0) {
                var aurora = 0.0;
                let uv = ray_dir.xz / ray_dir.y;
                for (var i = 0; i < 5; i++) {
                    let fi = f32(i);
                    aurora += sin(uv.x * 2.0 + fi) * sin(uv.y * 1.5 + fi) * 0.2;
                }
                let a_val = smoothstep(0.2, 0.8, abs(aurora));
                let a_col = mix(vec3<f32>(0.0, 1.0, 0.53), vec3<f32>(0.88, 0.0, 1.0), ray_dir.y);
                color += a_col * a_val * exp(-ray_dir.y * 2.0);
            }
            "#
        ),
        "volcanic_crystal_cave" => (
            r#"
            // Cinematic 4: Volcanic Crystal Cave
            let cave = -(length(p.xy) - 8.0);
            let magma = p.y + 6.0 + sin(p.x*2.0)*0.2;
            
            let hex_q = abs(vec2<f32>(p.x - 4.0, p.z - 2.0));
            let hex = max(hex_q.x * 0.866 + hex_q.y * 0.5, hex_q.y) - 1.0;
            let crystal = max(hex, abs(p.y) - 4.0);
            
            if (magma < cave && magma < crystal) { return vec2<f32>(magma, 25.0); }
            if (crystal < cave) { return vec2<f32>(crystal, 26.0); }
            return vec2<f32>(cave, 27.0);
            "#,
            r#"
            if (mat_id == 25.0) {
                let temp = fbm3(hit_pos * 2.0);
                let is_crack = smoothstep(0.4, 0.6, temp);
                color = mix(vec3<f32>(0.02), vec3<f32>(1.0, 0.2, 0.0) + vec3<f32>(1.0, 0.8, 0.0) * temp, is_crack);
            }
            else if (mat_id == 26.0) {
                let crystal_col = mix(vec3<f32>(0.5, 0.0, 1.0), vec3<f32>(0.0, 0.95, 1.0), hit_pos.y * 0.1 + 0.5);
                let emissive = crystal_col;
                let spec = pow(max(dot(reflect(-normalize(vec3<f32>(0.0, -1.0, 1.0)), normal), -ray_dir), 0.0), 128.0);
                color = crystal_col * 0.2 + emissive + spec;
            }
            else {
                let magma_glow = vec3<f32>(1.0, 0.3, 0.0) * max(dot(normal, normalize(vec3<f32>(0.0, -1.0, 0.0))), 0.0);
                color = vec3<f32>(0.05) + magma_glow * 0.5;
            }
            "#
        ),
        "floating_archipelago" => (
            r#"
            // Cinematic 5: Floating Archipelago
            let spacing = 20.0;
            let cell = floor(p.xz / spacing);
            let local_xz = p.xz - cell * spacing - spacing*0.5;
            
            let cone = length(local_xz) - (4.0 + hash21(cell)*2.0) + p.y*0.5;
            let top = p.y - 2.0;
            let island = max(cone, top);
            
            let clouds = p.y + 10.0;
            
            if (clouds < island) { return vec2<f32>(clouds, 28.0); }
            return vec2<f32>(island, 29.0);
            "#,
            r#"
            let sun_dir = normalize(vec3<f32>(1.0, 0.4, 0.5));
            let diff = max(dot(normal, sun_dir), 0.0);
            
            if (mat_id == 28.0) {
                let c_noise = fbm3(hit_pos * 0.1);
                color = mix(vec3<f32>(0.3, 0.4, 0.5), vec3<f32>(1.0, 0.8, 0.6), diff + c_noise*0.5);
            }
            else if (mat_id == 29.0) {
                let rock = vec3<f32>(0.3, 0.3, 0.3);
                let grass = vec3<f32>(0.2, 0.6, 0.2);
                let is_top = smoothstep(0.6, 0.8, normal.y);
                color = mix(rock, grass, is_top) * (diff * 0.8 + 0.2);
            }
            
            let sky_col = mix(vec3<f32>(0.8, 0.5, 0.3), vec3<f32>(0.2, 0.4, 0.7), clamp(ray_dir.y, 0.0, 1.0));
            color = mix(color, sky_col, 0.2); // Removed global fog based on t since t can be max_dist
            "#
        ),
        _ => (
            r#"
            let d = p.y + 1.0;
            return vec2<f32>(d, 0.0);
            "#,
            r#"
            let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.5));
            let diff = max(dot(normal, light_dir), 0.1);
            color = vec3<f32>(0.5) * diff;
            "#
        )
    };

    format!(r#"
struct RaymarchParams {{
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
}};

@group(0) @binding(0) var<uniform> ray_params: RaymarchParams;
@group(0) @binding(1) var<storage, read_write> color_buffer: array<u32>;

fn hash21(p: vec2<f32>) -> f32 {{
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q += dot(q, q + 45.32);
    return fract(q.x * q.y);
}}

fn hash31(p: vec3<f32>) -> f32 {{
    var q = fract(p * vec3<f32>(123.34, 456.21, 789.92));
    q += dot(q, q + 45.32);
    return fract(q.x * q.y * q.z);
}}

fn noise3(p: vec3<f32>) -> f32 {{
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(mix(hash31(i + vec3<f32>(0.0,0.0,0.0)), hash31(i + vec3<f32>(1.0,0.0,0.0)), u.x),
            mix(hash31(i + vec3<f32>(0.0,1.0,0.0)), hash31(i + vec3<f32>(1.0,1.0,0.0)), u.x), u.y),
        mix(mix(hash31(i + vec3<f32>(0.0,0.0,1.0)), hash31(i + vec3<f32>(1.0,0.0,1.0)), u.x),
            mix(hash31(i + vec3<f32>(0.0,1.0,1.0)), hash31(i + vec3<f32>(1.0,1.0,1.0)), u.x), u.y), u.z);
}}

fn fbm3(p: vec3<f32>) -> f32 {{
    var f = 0.0;
    var scale = 0.5;
    var q = p;
    for (var i = 0; i < 3; i++) {{
        f += scale * noise3(q);
        q *= 2.0;
        scale *= 0.5;
    }}
    return f;
}}

fn evaluate_scene(p: vec3<f32>) -> vec2<f32> {{
    {}
}}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {{
    let e = max(0.001, length(p) * 0.00005);
    let dx = evaluate_scene(p + vec3<f32>(e, 0.0, 0.0)).x - evaluate_scene(p - vec3<f32>(e, 0.0, 0.0)).x;
    let dy = evaluate_scene(p + vec3<f32>(0.0, e, 0.0)).x - evaluate_scene(p - vec3<f32>(0.0, e, 0.0)).x;
    let dz = evaluate_scene(p + vec3<f32>(0.0, 0.0, e)).x - evaluate_scene(p - vec3<f32>(0.0, 0.0, e)).x;
    return normalize(vec3<f32>(dx, dy, dz));
}}

fn calc_softshadow(ro: vec3<f32>, rd: vec3<f32>, mint: f32, maxt: f32, k: f32) -> f32 {{
    var res: f32 = 1.0;
    var t: f32 = mint;
    for(var i: i32 = 0; i < 16; i++) {{
        if(t > maxt) {{ break; }}
        let h = evaluate_scene(ro + rd * t).x;
        if(h < 0.001) {{ return 0.0; }}
        res = min(res, k * h / t);
        t += clamp(h, 0.02, 0.20);
    }}
    return clamp(res, 0.0, 1.0);
}}

fn calc_ao(pos: vec3<f32>, nor: vec3<f32>) -> f32 {{
    var occ: f32 = 0.0;
    var sca: f32 = 1.0;
    for(var i: i32 = 0; i < 5; i++) {{
        let h = 0.01 + 0.12 * f32(i) / 4.0;
        let d = evaluate_scene(pos + h * nor).x;
        occ += (h - d) * sca;
        sca *= 0.95;
        if(occ > 0.35) {{ break; }}
    }}
    return clamp(1.0 - 3.0 * occ, 0.0, 1.0) * (0.5 + 0.5 * nor.y);
}}

fn aces_film(x: vec3<f32>) -> vec3<f32> {{
    let a = 2.51f;
    let b = 0.03f;
    let c = 2.43f;
    let d = 0.59f;
    let e = 0.14f;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}}

@compute @workgroup_size(16, 16)
fn raymarch_main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    if (global_id.x >= ray_params.screen_width || global_id.y >= ray_params.screen_height) {{
        return;
    }}

    let w = f32(ray_params.screen_width);
    let h = f32(ray_params.screen_height);
    let uv = vec2<f32>(f32(global_id.x) - 0.5 * w, 0.5 * h - f32(global_id.y)) / h;

    let forward = normalize(ray_params.camera_dir);
    let right = normalize(cross(forward, ray_params.camera_up));
    let up = cross(right, forward);

    let ray_dir = normalize(forward + uv.x * right + uv.y * up);

    var t: f32 = 0.0;
    var hit: bool = false;
    var hit_pos: vec3<f32> = ray_params.camera_pos;
    var mat_id: f32 = 0.0;

    for (var i: u32 = 0u; i < ray_params.max_steps; i = i + 1u) {{
        let p = ray_params.camera_pos + ray_dir * t;
        let res = evaluate_scene(p);
        let d = res.x;
        
        // P2 Optimization: Distance-adapted epsilon (Cone tracing tolerance)
        let epsilon = max(0.0005, 0.0001 * t);
        if (abs(d) < epsilon) {{
            hit = true;
            hit_pos = p;
            mat_id = res.y;
            break;
        }}
        t += d;
        if (t > ray_params.max_dist) {{
            break;
        }}
    }}

    var color: vec3<f32> = vec3<f32>(0.01, 0.01, 0.02); // Deep Space Sky
    var normal: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
    
    if (hit) {{
        normal = calc_normal(hit_pos);
    }} else {{
        // If no hit, t is max_dist
        t = ray_params.max_dist;
        hit_pos = ray_params.camera_pos + ray_dir * t;
    }}
    
    {} // Insert color_logic
    
    // Apply ACES cinematic tonemapping and Gamma correction (sRGB)
    color = aces_film(color);
    color = pow(color, vec3<f32>(1.0 / 2.2));

    let r = u32(clamp(color.r * 255.0, 0.0, 255.0));
    let g = u32(clamp(color.g * 255.0, 0.0, 255.0));
    let b = u32(clamp(color.b * 255.0, 0.0, 255.0));
    let a = 255u;

    let pixel_idx = global_id.x + (ray_params.screen_height - 1u - global_id.y) * ray_params.screen_width;
    color_buffer[pixel_idx] = (a << 24u) | (b << 16u) | (g << 8u) | r;
}}
    "#, sdf_logic, color_logic)
}
