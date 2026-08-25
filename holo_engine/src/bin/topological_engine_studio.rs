/// Topological Engine Studio - Next-Generation Game Engine Demonstrator
/// Instantiates multiple topological world models and runs interactive simulation steps.

use holo_engine::engine::{RenderMode, TopologicalWorld, WorldPreset};

fn main() {
    println!("============================================================");
    println!("   HoloEngine Framework - Studio de Mondes Topologiques    ");
    println!("   Prochaine Génération de Moteur de Jeu & Invariants Physiques");
    println!("============================================================\n");

    let presets = [
        ("Monde 1: Terrain Organique 1-Lipschitz (DONN)", WorldPreset::OrganicAlienWorld),
        ("Monde 2: Océan Quantique (Particules SPH & Leray)", WorldPreset::QuantumFluidOcean),
        ("Monde 3: Singularité Quantique T-Duale (LOD Rebound)", WorldPreset::TDualSingularity),
        ("Monde 4: Maillage Cybernétique (Topologie TDA)", WorldPreset::CyberneticMesh),
    ];

    for (desc, preset) in presets {
        println!("------------------------------------------------------------");
        println!("[+] Initialisation du modèle : {}", desc);
        let mut world = TopologicalWorld::new("StudioWorld", preset);

        // Run initial simulation step
        world.step_simulation(0.016);
        let betti = world.compute_betti_numbers();

        println!("    -> Particules / Points de Masse : {}", world.particles.len());
        println!("    -> Énergie Cinétique Totale   : {:.4}", world.physics.metrics.total_kinetic_energy);
        println!("    -> Enstrophie Bornée (Strict) : {}", world.physics.metrics.is_enstrophy_bounded);
        println!("    -> Nombres de Betti TDA       : B0={} | B1={} | B2={}", betti.betti_0, betti.betti_1, betti.betti_2);

        // Test Interactive 1-Lipschitz Mining on Organic World
        if preset == WorldPreset::OrganicAlienWorld {
            let hit = [0.0, 1.0, 0.0];
            let affected = world.interact_subtract_density(hit, 3.0);
            println!("    -> [Minage 1-Lipschitz] Impact au point {:?} | Particules déplacées : {}", hit, affected);
        }

        // Test T-Duality Zoom Rebound on Singularity World
        if preset == WorldPreset::TDualSingularity {
            world.renderer.update_camera_distance(15.0);
            println!("    -> [Zoom Macroscopique] Dist R=15.0 => Mode: {:?}", world.renderer.active_mode);
            world.renderer.update_camera_distance(0.5);
            println!("    -> [Zoom Microscopique] Dist R=0.5  => Mode: {:?}", world.renderer.active_mode);
            assert_eq!(world.renderer.active_mode, RenderMode::DiscreteK3Fiber);
        }
        println!();
    }

    println!("============================================================");
    println!("   Tous les modèles de mondes topologiques sont prêts !    ");
    println!("============================================================");
}
