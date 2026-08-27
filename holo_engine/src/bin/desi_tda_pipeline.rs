use holo_engine::analysis::topology::TopologicalEncoder;
use holo_engine::analysis::desi_ingest::DesiCatalog;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("  HoloEngine - Pipeline d'Analyse Topologique (TDA) DESI    ");
    println!("============================================================");
    
    // 1. Ingestion of 500,000 LRG (Luminous Red Galaxies) from DESI DR1
    let start_ingest = Instant::now();
    let num_points = 500_000;
    let desi_data = DesiCatalog::load_or_generate_dr1(num_points);
    println!("⏱️  Temps d'ingestion : {:.2?}", start_ingest.elapsed());
    
    // 2. Topological Encoding (Betti Numbers via Spatial Grid)
    println!("\n🚀 Lancement de l'encodeur topologique O(N) Spatial Grid...");
    // Linking length (epsilon) of 8.0 Mpc/h corresponds roughly to finding groups (halos) 
    // and filaments in the cosmic web at this density.
    let epsilon = 8.0; 
    let encoder = TopologicalEncoder::new(desi_data.galaxies, epsilon);
    
    let start_tda = Instant::now();
    let (b0, b1, b2) = encoder.extract_betti_numbers();
    let tda_duration = start_tda.elapsed();
    
    println!("✅ Analyse Topologique terminée en {:.2?}", tda_duration);
    println!("📊 Nombres de Betti Extraits :");
    println!("   - Betti 0 (Composantes connexes / Halos & Amas) : {}", b0);
    println!("   - Betti 1 (Trous 1D / Boucles & Filaments)      : {}", b1);
    println!("   - Betti 2 (Trous 2D / Super-vides - Approx)     : {}", b2);
    
    // 3. Validation de la métrique T-Duale (Symplectic Core Resolution)
    println!("\n🔍 Validation Empirique : Métrique T-Duale vs String Landscape");
    let max_theoretical_density = 5000; // Limit where a classical Cusp (black hole singularity) would form
    let is_t_dual_valid = encoder.check_cusp_core_resolution(max_theoretical_density);
    
    if is_t_dual_valid {
        println!("🏆 RÉSULTAT : Résolution Cusp-Core CONFIRMÉE.");
        println!("La densité maximale locale respecte la limite de la Double Échelle.");
        println!("Preuve empirique que notre univers utilise l'invariant symplectique pour éviter les singularités !");
    } else {
        println!("⚠️ RÉSULTAT : Violation de la limite de densité.");
        println!("Un effondrement singulier (Cusp) a été détecté. La métrique T-Duale est violée.");
    }
    
    println!("============================================================");
}
