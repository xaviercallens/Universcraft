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
    println!(" 🌌 HOLOENGINE HIGH-RESOLUTION MULTIVERSE OFFLINE RENDERER (1080p FHD)     ");
    println!("==========================================================================");

    let scenes = vec![
        ("01_ocean_sunset", "ocean_sunset", "🌊 Océan & Fluides (Navier-Stokes)"),
        ("02_forest_soil", "forest_soil", "🌲 Forêt & Flore L-System (K3 Whittaker)"),
        ("03_cloudscape", "cloudscape", "☁️ Nuages Volumétriques & Atmosphère (Boussinesq)"),
        ("04_crystal_cave", "crystal_cave", "💎 Grotte Cristalline & Réfraction PBR"),
        ("05_floating_islands", "floating_islands", "🏝️ Îles Flottantes (Métrique DONN)"),
        ("06_desert_dunes", "desert_dunes", "🏜️ Dunes du Désert (Transport Éolien 1-Lipschitz)"),
        ("07_ice_glacier", "ice_glacier", "🏔️ Glacier & Banquise (Loi de Fluage de Glen)"),
        ("08_volcano_core", "volcano_core", "🌋 Cœur Magmatique & Émission Thermodynamique"),
        ("09_alien_planet", "alien_planet", "🪐 Planète Extraterrestre & Ecotones Exotiques"),
        ("10_deep_space", "deep_space", "🌌 Nébuleuse Cosmique & Filaments DESI"),
        ("11_black_hole", "black_hole", "🕳️ Trou Noir de Kerr (Rebond T-Dual & Redshift)"),
        ("12_earth_orbit", "earth_orbit", "🌐 Orbite Terrestre & Atmosphère de Rayleigh-Mie"),
        ("13_continental_biomes", "continental_biomes", "🌿 Équateur Sauvage (PBR Triplanaire)"),
        ("14_arctic_aurora", "arctic_aurora", "🧊 Pôle Boréal & Aurore Polaire"),
        ("15_volcanic_crystal_cave", "volcanic_crystal_cave", "🔥 Cœur Terrestre & Fluides Magmatiques"),
        ("16_floating_archipelago", "floating_archipelago", "☁️ Archipel Suspendu & Convection Aérienne"),
    ];

    fs::create_dir_all("public/gallery").unwrap();
    fs::create_dir_all("public/output").unwrap();

    let compute_manager = GPUComputeManager::new(32);
    let target_w = 1920;
    let target_h = 1080;

    let total_start = Instant::now();
    println!("🚀 Starting High-Resolution Render Pass for {} scenes at {}x{}...", scenes.len(), target_w, target_h);

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
