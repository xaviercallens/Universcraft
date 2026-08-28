use holo_engine::client::gpu_compute::GPUComputeManager;
use image::{GenericImage, RgbaImage};
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("🎨 Generating 4x5 Gallery of all scenes...");

    let scenes = vec![
        "ocean_sunset",
        "forest_soil",
        "cloudscape",
        "crystal_cave",
        "floating_islands",
        "desert_dunes",
        "ice_glacier",
        "volcano_core",
        "alien_planet",
        "earth_orbit",
        "continental_biomes",
        "arctic_aurora",
        "volcanic_crystal_cave",
        "floating_archipelago",
        "black_hole", // Requires special pass if done via API, but let's check
    ];
    
    // We will render at 320x180 per thumbnail for speed
    let thumb_w = 480;
    let thumb_h = 270;
    let cols = 4;
    let rows = 5;
    
    let mut collage = RgbaImage::new(thumb_w * cols, thumb_h * rows);
    let compute_manager = GPUComputeManager::new(32);

    for i in 0..(cols * rows) {
        let col = i % cols;
        let row = i / cols;
        
        // Pick scene, cycle if we run out to fill 20 slots
        let scene_name = scenes[(i as usize) % scenes.len()];
        println!("Rendering [{}/{}] - {}...", i + 1, cols * rows, scene_name);
        
        let pixels = if scene_name == "black_hole" {
            let res = compute_manager.execute_advanced_scene_pass(thumb_w, thumb_h, "black_hole").await;
            res.unwrap().2
        } else {
            let res = compute_manager.execute_advanced_scene_pass(thumb_w, thumb_h, scene_name).await;
            res.unwrap().2
        };
        
        let bytes: &[u8] = bytemuck::cast_slice(&pixels);
        let img = image::RgbaImage::from_raw(thumb_w, thumb_h, bytes.to_vec()).unwrap();
        
        collage.copy_from(&img, col * thumb_w, row * thumb_h).unwrap();
    }
    
    std::fs::create_dir_all("public/output").unwrap();
    collage.save("public/output/gallery_4x5.png").unwrap();
    println!("✅ Gallery saved to public/output/gallery_4x5.png");
}
