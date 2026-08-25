# HoloEngine 3D — Plan d'Implémentation : Phase 2 (Optimisation GPU & Écotones)

Ce document détaille la feuille de route technique pour transformer le prototype mathématique (Phase 1) en un moteur de rendu temps réel hautes performances capable de tenir **60 FPS constants** lors des interactions dynamiques (minage 1-Lipschitz) et du rendu d'environnements denses.

---

## 🛠️ Partie 1 : Améliorations de la Phase 1 (Polissage Visuel) [✅ COMPLÉTÉ]

### 1. Écotones : Transitions Douces de Biomes (Dithering Organique) [✅ VALIDÉ]
**Objectif :** Éviter les frontières nettes entre les biomes de Whittaker (ex: passage abrupt Désert $\rightarrow$ Jungle).
**Implémentation :**
- **Perturbation Thermodynamique :** Injecter un bruit haute fréquence dans les équations d'échantillonnage de la Température ($T$) et de l'Humidité ($H$).
- **Formule :** `T_eff = T + (perlin_noise(x, z) * 0.05)` et `H_eff = H + (perlin_noise(x, z) * 0.05)`.
- **Fichiers ciblés :**
  - `holo_engine/src/client/biome_generator.rs` (Rust)
  - `specs/topological_studio_engine.js` (Web)

---

## 🚀 Partie 2 : Phase 2 — Le Sprint "Zero-Copy" & GPU Compute [✅ COMPLÉTÉ]

L'objectif de cette phase est de résoudre le goulet d'étranglement lié aux calculs asynchrones globaux sur le CPU, responsable des chutes de framerate lors du minage.

### 1. "Chunking" Spatial (Sparse Voxel Octree / Secteurs) [✅ VALIDÉ]
**Objectif :** Localiser les recalculs géométriques (Surface Nets) uniquement aux zones altérées par le joueur.
**Implémentation :**
- Découper le monde infini en *Chunks* de `32x32x32` mètres.
- Implémenter un gestionnaire de Chunks `ChunkManager` qui garde en mémoire cache les maillages non modifiés.
- Lors d'un clic de minage (Raymarching), identifier le *Chunk* intersecté et ne relancer la génération SDF/Mesh que sur ce bloc précis.
- **Impact :** Les temps de recalcul post-minage chuteront drastiquement (passage de O(N) global à O(1) local).

### 2. Déportation Totale sur GPU (Compute Shaders WGSL) [✅ VALIDÉ]
**Objectif :** Libérer le CPU en calculant la densité SDF et l'extraction *Surface Nets* sur la carte graphique en parallèle massif.
**Implémentation :**
- **SDF Compute Shader :** Écrire un shader `terrain_sdf.wgsl` (`gpu_compute.rs`) qui évalue l'équation $f(x,y,z)$ pour chaque voxel du chunk (32,768 threads parallèles).
- **Surface Nets WGSL :** Transférer l'algorithme d'extraction de surface sur GPU.
- **Architecture Zero-Copy :** Les sommets (Vertices) et indices générés par le Compute Shader restent dans la VRAM et sont passés directement au pipeline de rendu Bevy sans transiter par la RAM CPU.
- **Impact :** Génération de terrain quasi instantanée (< 1 ms par chunk).

### 3. GPU Hardware Instancing pour la Biosphère Algorithmique [✅ VALIDÉ]
**Objectif :** Afficher 5000+ arbres fractals sans provoquer de *Draw Call Bottleneck* entre le CPU et le GPU.
**Implémentation :**
- **Mesh Unique :** Charger la géométrie d'une branche L-System et du disque *K3 Fiber Billboard* une seule fois dans la mémoire vidéo.
- **Instance Buffer :** Le système de Réaction-Diffusion (TDA) génère une liste de matrices de transformation (positions, rotations, échelles quantiques).
- **Rendu par Instanciation :** Utiliser la méthode `renderBatchedFlora` pour dessiner toute la forêt en 2 instructions de dessin batchées.
- **Impact :** Soulagement majeur du CPU et libération de bande passante PCI-Express.

---

## 🏆 Cible de Sortie (Milestone : Phase 2 Complete) [✅ ATTEINTE]
**Critère de succès :** Le moteur maintient `60.0 FPS` constants avec :
- Le minage 1-Lipschitz en rafale actif.
- Un horizon de vue de 8 chunks (256 mètres).
- Une forêt de plus de 5000 entités florales générées.

Une fois ce jalon matériel validé, la **Phase 3 (Atmosphère volumétrique et Ciel Astrophysique)** pourra être entamée.
