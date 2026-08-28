#[cfg(feature = "wgpu")]
use holo_engine::client::bevy_pipeline::tnn_physics_coupling::TnnPhysicsRegistry;
#[cfg(feature = "wgpu")]
use holo_engine::client::gpu_compute::GPUComputeManager;
use image::{GenericImage, RgbaImage};
use std::fs;
use std::time::Instant;

#[tokio::main]
async fn main() {
#[cfg(feature = "wgpu")]
{
    println!("==========================================================================");
    println!(" 🌌 HOLOENGINE TDA/TNN CERTIFIED HIGH-RESOLUTION OFFLINE RENDERER (1080p) ");
    println!("==========================================================================");

    let physics_reg = TnnPhysicsRegistry::load_from_assets("assets/physics");
    println!("📊 Loaded Certified TNN/TDA Invariants from assets/physics:");
    if let Some(ref ocean) = physics_reg.ocean {
        println!("  🌊 Ocean JHTDB FNO-3D: g={:.2} m/s², ν={:.2e} m²/s, β₁={}", ocean.gravity_m_s2, ocean.kinematic_viscosity_m2_s, ocean.target_b1_vortices);
    }
    if let Some(ref dunes) = physics_reg.dunes {
        println!("  🏜️ Dunes Exner PINN: θ_repose={:.1}°, slope≤{:.4}, β₁={}, β₂={}", dunes.repose_angle_deg, dunes.max_slope_tan, dunes.target_b1_loops, dunes.target_b2_voids);
    }
    if let Some(ref astro) = physics_reg.astrophysics {
        println!("  🌌 Astrophysics DESI SympNet: R_c={:.2} kpc, Stars={}, β₁={}, Z={:+.2}σ", astro.dark_matter_core_rc_kpc, astro.stars_count, astro.target_b1_filaments, astro.z_score_vs_poisson);
    }
    if let Some(ref bh) = physics_reg.black_hole {
        println!("  🕳️ Black Hole EHT T-Dual: M={:.1e} M☉, a={:.2}, ISCO={:.2} Rg, T-Dual Metric={}", bh.mass_solar_masses, bh.dimensionless_spin_a, bh.isco_radius_rg, bh.t_dual_effective_metric);
    }

    let scenes = vec![
        ("01_ocean_sunset", "ocean_sunset", "🌊 Océan & Fluides (Navier-Stokes / FNO-3D)"),
        ("02_forest_soil", "forest_soil", "🌲 Forêt & Flore L-System (K3 Whittaker / Murray Law)"),
        ("03_cloudscape", "cloudscape", "☁️ Nuages Volumétriques & Atmosphère (Boussinesq / ERA5)"),
        ("04_crystal_cave", "crystal_cave", "💎 Grotte Cristalline & Réfraction PBR (COD Fd-3m)"),
        ("05_floating_islands", "floating_islands", "🏝️ Îles Flottantes (Métrique DONN & TDA)"),
        ("06_desert_dunes", "desert_dunes", "🏜️ Dunes du Désert (Transport Éolien Exner 1-Lipschitz)"),
        ("07_ice_glacier", "ice_glacier", "🏔️ Glacier & Banquise (Loi de Fluage de Glen n=3)"),
        ("08_volcano_core", "volcano_core", "🌋 Cœur Magmatique & Émission Thermodynamique (Planck)"),
        ("09_alien_planet", "alien_planet", "🪐 Planète Extraterrestre & Écotones Exotiques"),
        ("10_deep_space", "deep_space", "🌌 Nébuleuse Cosmique & Filaments DESI (SympNets)"),
        ("11_black_hole", "black_hole", "🕳️ Trou Noir de Kerr (Rebond T-Dual Sans Singularité & Redshift)"),
        ("12_earth_orbit", "earth_orbit", "🌐 Orbite Terrestre & Atmosphère Rayleigh-Mie"),
        ("13_continental_biomes", "continental_biomes", "🌿 Équateur Sauvage (PBR Triplanaire & Whittaker)"),
        ("14_arctic_aurora", "arctic_aurora", "🧊 Pôle Boréal & Aurore Polaire (Coriolis / Magneto)"),
        ("15_volcanic_crystal_cave", "volcanic_crystal_cave", "🔥 Cœur Terrestre & Fluides Magmatiques Visqueux"),
        ("16_floating_archipelago", "floating_archipelago", "☁️ Archipel Suspendu & Convection Aérienne"),
    ];

    fs::create_dir_all("public/gallery").unwrap();
    fs::create_dir_all("public/output").unwrap();

    let compute_manager = GPUComputeManager::new(32);
    let target_w = 1920;
    let target_h = 1080;

    let total_start = Instant::now();
    println!("\n🚀 Starting High-Resolution Render Pass for {} scenes at {}x{}...", scenes.len(), target_w, target_h);

    for (idx, (file_prefix, scene_id, label)) in scenes.iter().enumerate() {
        println!("\n[{}/{}] Rendering: {} (Scene ID: '{}')...", idx + 1, scenes.len(), label, scene_id);

        let render_res = compute_manager.execute_advanced_scene_pass(target_w, target_h, scene_id).await;
        
        match render_res {
            Ok((adapter, duration_ms, pixels)) => {
                let bytes: &[u8] = bytemuck::cast_slice(&pixels);
                if let Some(img) = RgbaImage::from_raw(target_w, target_h, bytes.to_vec()) {
                    let out_path = format!("public/gallery/{}.png", file_prefix);
                    img.save(&out_path).unwrap();
                    println!("  ✅ Saved: {} (GPU: {}, Latency: {:.2} ms)", out_path, adapter.name, duration_ms);
                } else {
                    eprintln!("  ❌ Failed to create image buffer for {}", scene_id);
                }
            }
            Err(e) => {
                eprintln!("  ❌ Error rendering {}: {}", scene_id, e);
            }
        }
    }

    println!("\n🖼️ Generating High-Resolution 4x4 Master Collage (3840x2160)...");
    let thumb_w: u32 = 960;
    let thumb_h: u32 = 540;
    let cols: usize = 4;
    let rows: usize = 4;
    let mut collage = RgbaImage::new(thumb_w * (cols as u32), thumb_h * (rows as u32));

    for (idx, (file_prefix, _, _)) in scenes.iter().enumerate() {
        let col = (idx % cols) as u32;
        let row = (idx / cols) as u32;
        let img_path = format!("public/gallery/{}.png", file_prefix);
        if let Ok(img) = image::open(&img_path) {
            let resized = image::imageops::resize(&img, thumb_w, thumb_h, image::imageops::FilterType::Lanczos3);
            collage.copy_from(&resized, col * thumb_w, row * thumb_h).unwrap();
        }
    }

    collage.save("public/gallery_4x5.png").unwrap();
    collage.save("public/output/gallery_4x5.png").unwrap();
    println!("✅ Saved Master High-Res Collage to public/gallery_4x5.png");

    println!("\n🎉 High-Resolution Offline Processing Completed in {:.2}s!", total_start.elapsed().as_secs_f32());
}
#[cfg(not(feature = "wgpu"))]
{
    eprintln!("wgpu feature required for highres generation");
}
}
