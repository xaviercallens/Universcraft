# Proof of Concept 1 (PoC 1): Le Noyau Visuel Organique & Rebond T-Dual

## 1. Objectifs du PoC 1
Le PoC 1 valide la faisabilité technique de l'HoloEngine à travers une tranche verticale (*Vertical Slice*) combinant :
1. **Rendu de Terrain Organique 1-Lipschitzien** : Génération d'un champ scalaire 3D basé sur une résonance harmonique (DONN/Cymatique) converti en maillage lisse sans cubes rigides via `fast-surface-nets-rs`.
2. **Shader de Rendu Ray Marching WGSL (T-Dualité)** : Compute shader avec métrique effective $R_{eff} = \max(R, \alpha'/R)$ basculant automatiquement vers une texture de réseau K3 discret au seuil $\sqrt{\alpha'}$.
3. **Fluides Régularisés SPH** : Simulation de particules fluides via notre fork local de `salva3d` intégrant la borne d'enstrophie (Cutoff K3).
4. **Supervision Autonome AMCP** : Intégration du maillage d'agents autonomes interagissant avec le terrain topologique.

## 2. Architecture des Composants PoC 1
* `src/poc1/donn_generator.rs` : Générateur d'ondes stationnaires (Cymatique / DONN).
* `src/poc1/t_dual_shader.rs` : Pipeline et matériau WGSL pour le rebond géométrique $R_{eff}$.
* `src/poc1/fluid_simulation.rs` : Intégration SPH Salva3D avec filtre d'enstrophy.
* `src/bin/poc1_runner.rs` : Démonstrateur exécutable unifié du PoC 1.