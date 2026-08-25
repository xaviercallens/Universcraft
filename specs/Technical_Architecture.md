# Architecture Technique : HoloEngine

Ce document définit l'architecture technique stabilisée pour l'ensemble du projet UniversCraft, maximisant la réutilisation des dépôts open-source stratégiques. L'architecture est conçue autour du paradigme "Zero-Copy" et de la sécurité mémoire (Rust), intégrant les limites topologiques pour éviter les crashs de calcul.

## 1. Vue d'Ensemble des Couches (Layers)

L'architecture est structurée en 5 couches interconnectées fonctionnant de manière asynchrone pour éviter tout goulot d'étranglement (bottleneck) sur le thread principal.

### Couche 1 : L'Orchestrateur Central (Data & ECS)
*   **Technologie** : `bevyengine/bevy`
*   **Rôle** : Gérer toutes les données du jeu via le pattern *Entity Component System* (ECS). Bevy orchestre l'exécution parallèle des systèmes (physique, IA, rendu).
*   **Pattern Asynchrone** : Utilisation du `AsyncComputeTaskPool` de Bevy pour diviser le monde en "Chunks". Le calcul lourd (TDA, génération de surface) est déporté sur des threads d'arrière-plan.

### Couche 2 : Génération Procédurale et Verrou Topologique
*   **Technologie** : `tracel-ai/burn`
*   **Rôle** : Framework Deep Learning natif Rust s'exécutant directement dans les systèmes Bevy.
*   **Fonctionnement** : Les réseaux de neurones oscillatoires (DONN) infèrent des ondes stationnaires (cymatique) pour déterminer la densité d'information de l'espace, remplaçant ainsi les algorithmes de bruit classique (Perlin) pour une géométrie plus harmonieuse.

### Couche 3 : Reconstruction Topologique (Le "Nouveau Voxel")
*   **Technologies** : `InteractiveComputerGraphics/splashsurf` (pour les fluides et surfaces complexes) et `bonsairobo/fast-surface-nets-rs` (pour l'extraction rapide du terrain).
*   **Rôle** : Convertir le nuage de points/données (généré par la couche 2) en un maillage (Mesh) organique, lisse et 1-Lipschitzien. 
*   **Fonctionnement** : Lit la filtration de Vietoris-Rips et génère en temps réel des surfaces sans l'aspect cubique (Minecraft) ou les défauts de déchirement visuel (Marching Cubes standard).

### Couche 4 : Moteur Physique Symplectique
*   **Technologies** : `dimforge/salva` (Hydrodynamique) et `dimforge/rapier` (Collisions/Solides).
*   **Rôle** : Simuler les fluides continus et les interactions physiques avec l'environnement.
*   **L'Injection Mathématique (Le Hack de Navier-Stokes)** :
    *   **Projecteur de Leray** : Intégré au cœur de `salva`, ce projecteur garantit l'incompressibilité absolue des fluides (div_free).
    *   **Coupure K3 (Enstrophy Bound)** : Limiteur intégré qui tronque l'énergie des tourbillons (vorticité) si elle dépasse la limite physique ($1/\alpha'$), garantissant qu'aucune tempête ne fasse exploser le CPU.

### Couche 5 : Rendu Visuel Quantique (Zero-Copy Pipeline)
*   **Technologie** : `gfx-rs/wgpu` (Intégré via Bevy).
*   **Rôle** : Pipeline de rendu de bas niveau et Compute Shaders.
*   **Fonctionnement** : Implémente le système LOD par T-Dualité. Les Compute Shaders évaluent l'espace fractal en Ray Marching. Si la distance tombe sous le seuil $\sqrt{\alpha'}$, le shader cesse l'évaluation continue et affiche la topologie K3 discrète.

---

## 2. Le Flux de Données "Zero-Copy" (Data Pipeline)

Pour garantir plus de 60 FPS constants lors des simulations massives, l'information ne doit pas faire d'allers-retours coûteux entre la RAM (CPU) et la VRAM (GPU).

```mermaid
graph TD
    A[Burn (DONN) - Génère le Nuage de Points] -->|Écriture directe| B[Storage Buffer VRAM]
    A -->|Traitement Async| C[Fast-Surface-Nets / Splashsurf]
    C -->|Génère Mesh 3D local| D[Bevy ECS / WGPU]
    B -->|Lecture instantanée| E[WGPU Compute Shader Ray Marching]
    D --> F[Affichage Joueur]
    E --> F
    G[Salva - Fluides SPH] -->|Projecteur Leray + Coupure K3| H[Mise à jour Vélocités VRAM]
    H --> D
```

1.  **Génération (GPU-First)** : L'IA (`burn`) calcule la densité spatiale et écrit directement dans un Storage Buffer sur la VRAM.
2.  **Rendu Lointain** : Le Ray Marching de Bevy (`wgpu`) lit ce buffer immédiatement (Zero-Copy) pour le paysage lointain.
3.  **Terrain Local** : Pour l'interaction proche du joueur, les tâches asynchrones (`fast-surface-nets-rs`) maillent les chunks locaux.
4.  **Physique Constrainte** : Les particules SPH (`salva`) coulent sur les meshes locaux générés par `splashsurf` et subissent des collisions avec `rapier`.

---

## 3. Stratégie de Construction et Stabilité

Pour pérenniser le développement de cette architecture, la règle d'or est la **Validation Verticale** :

*   **Intégration Native** : N'utilisez pas de wrappers lourds. Tout doit compiler via Cargo.
*   **Verification Formelle (Lean 4)** : Tout changement dans les limites physiques (`salva` cut-off, LOD `wgpu`) doit correspondre formellement aux théorèmes prouvés dans `DualScale.lean` et `FourierStateZ3.lean`.
*   **Isolement de la Mémoire** : La mutabilité (emprunts Rust) garantira qu'un chunk en cours de re-maillage (reconstruction par Splashsurf après une explosion) ne bloque pas la boucle physique de Rapier/Salva.
