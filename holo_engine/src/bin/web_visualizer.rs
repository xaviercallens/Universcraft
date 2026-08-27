use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use holo_engine::client::gpu_compute::GPUComputeManager;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    // We could put shared resources here if needed
}

#[derive(Deserialize)]
struct GenerateRequest {
    scene: String, // "raymarch" or "black_hole"
    resolution_width: u32,
    resolution_height: u32,
    // Add other parameters later (e.g. bh_mass, spin)
}

#[derive(Serialize)]
struct GenerateResponse {
    success: bool,
    image_url: String,
    render_time_ms: f32,
    message: String,
}

#[tokio::main]
async fn main() {
    println!("============================================================");
    println!("  HoloEngine 3D — Remote Web Visualizer & GPU Render Node   ");
    println!("============================================================");

    // Create public and output directories
    std::fs::create_dir_all("public/output").unwrap();

    let state = AppState {};

    // Build our application with routes
    let app = Router::new()
        .route("/api/generate", post(handle_generate))
        .nest_service("/", ServeDir::new("public"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Run our app with hyper on 0.0.0.0:8080
    // Bind to 0.0.0.0 so it can be accessed from outside the VM
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_generate(
    State(_state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> impl IntoResponse {
    let width = payload.resolution_width.clamp(256, 3840);
    let height = payload.resolution_height.clamp(256, 2160);

    let compute_manager = GPUComputeManager::new(32); // 32 is dummy for 2D passes

    let result = {
        println!("Generating scene '{}' at {}x{}", payload.scene, width, height);
        #[cfg(feature = "wgpu")]
        {
            compute_manager.execute_advanced_scene_pass(width, height, &payload.scene).await
        }
        #[cfg(not(feature = "wgpu"))]
        {
            Err("wgpu feature not enabled".to_string())
        }
    };

    match result {
        Ok((adapter, duration_ms, pixels)) => {
            println!("GPU Adapter used: {}", adapter.name);
            println!("GPU Backend: {:?}", adapter.backend);
            println!("GPU pass finished in {} ms. Array len: {}", duration_ms, pixels.len());
            // Save to PNG
            let filename = format!("{}_{}x{}_{}.png", payload.scene, width, height, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
            let path = PathBuf::from("public/output").join(&filename);
            
            // Cast [u32] to [u8]
            let bytes: &[u8] = bytemuck::cast_slice(&pixels);
            println!("Casted to bytes, length: {}. Converting to RgbaImage...", bytes.len());
            
            if let Some(img) = image::RgbaImage::from_raw(width, height, bytes.to_vec()) {
                println!("RgbaImage created. Spawning save thread...");
                let path_clone = path.clone();
                let save_res = std::thread::Builder::new()
                    .stack_size(32 * 1024 * 1024)
                    .spawn(move || {
                        img.save(&path_clone)
                    })
                    .unwrap()
                    .join()
                    .unwrap();

                if let Err(e) = save_res {
                    return Json(GenerateResponse {
                        success: false,
                        image_url: "".to_string(),
                        render_time_ms: duration_ms,
                        message: format!("Failed to save image: {}", e),
                    });
                }
            } else {
                return Json(GenerateResponse {
                    success: false,
                    image_url: "".to_string(),
                    render_time_ms: duration_ms,
                    message: "Failed to create image buffer".to_string(),
                });
            }

            Json(GenerateResponse {
                success: true,
                image_url: format!("/output/{}", filename),
                render_time_ms: duration_ms,
                message: format!("Scene rendered successfully in {:.2} ms", duration_ms),
            })
        }
        Err(e) => {
            Json(GenerateResponse {
                success: false,
                image_url: "".to_string(),
                render_time_ms: 0.0,
                message: format!("GPU compute failed: {}", e),
            })
        }
    }
}
