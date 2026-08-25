# HoloEngine — Sprint V4.0 : "Zero-Copy GPU Pipeline & Rendu Cinématographique" [✅ COMPLÉTÉ]

Ce plan d'implémentation est une évolution majeure de la Phase 4, actant la transition complète vers une architecture **100% GPU-Driven**. L'objectif est de briser le goulet d'étranglement du CPU (visant un bond de ~25 FPS à +60-120 FPS constants) tout en introduisant un rendu photoréaliste de type "Next-Gen" (PBR).

---

## ⚙️ CHANTIER 1 : Optimisation Extrême (Le Paradigme "GPU-First") [✅ VALIDÉ]

### 1. La Lithosphère sur Compute Shaders (SDF & Maillage WGPU)
- **Objectif :** Déporter l'évaluation de l'onde DONN, du champ SDF et de l'algorithme d'extraction (ex: Marching Cubes / Surface Nets) intégralement sur la VRAM.
- **Action Technique :** 
  - Déploiement d'un pipeline de *Compute Shaders* WGSL. Le CPU ne gérera plus que les clics de minage (1-Lipschitz), tandis que le GPU recalculera les *Chunks* modifiés.
  - *Étude de composants* : Inspiration sur l'architecture asynchrone des chunks de `bevy_voxel_world` et les extracteurs WGPU de type `wgpu-marching-cubes`.

### 2. La Biosphère L-Systems (GPU Hardware Instancing)
- **Objectif :** Annuler la surcharge des *Draw Calls* causée par les milliers de branches fractales et disques K3.
- **Action Technique :**
  - Chargement unique (en VRAM) des modèles 3D (Tronc unitaire + Disque K3).
  - Transmission d'un *Storage Buffer* contenant les matrices de transformation (coordonnées calculées par la réaction-diffusion et limitées par $R_{eff}$). L'API d'Instanciation GPU (`bevy::render::mesh::InstancedMesh`) dessinera la forêt entière en un seul appel.

### 3. Hydrodynamique Symplectique (SSFR)
- **Objectif :** Éliminer l'Overdraw massif causé par les sphères SPH translucides.
- **Action Technique :**
  - Rendu des particules dans un *Depth Buffer* offscreen.
  - Application d'un lissage bilatéral (Bilateral Blur) puis reconstruction des normales pour afficher une surface liquide continue. Injection du plafond d'enstrophie Leray directement dans le pipeline Compute WGPU.

---

## 🎨 CHANTIER 2 : Visualisation Next-Gen (Shader Whittaker & PBR)

### 1. Matériaux PBR et Triplanar Mapping
- **Objectif :** Appliquer des textures réalistes (Albedo, Normal, Roughness) sur une topologie procédurale dépourvue d'UVs natifs.
- **Action Technique :**
  - Développement d'un *Custom Material WGSL* dans Bevy utilisant la projection Triplanaire (échantillonnage depuis X, Y et Z mélangé selon la normale).
  - Intégration de textures libres de droits (PolyHaven) pour la Roche, la Terre, le Sable et la Neige.

### 2. Écotones Organiques (Biome Blending GPU)
- **Objectif :** Remplacer les frontières mathématiques abruptes par des lisières naturelles.
- **Action Technique :**
  - Injection des paramètres $T$ (Température) et $H$ (Humidité) de Whittaker au sein du Shader PBR.
  - Lissage des bordures via un bruit fractal et la fonction `smoothstep` : `mix(biome_A, biome_B, smoothstep(..., biome_threshold + noise))`

### 3. Post-Processing Cinématographique (Bevy Native)
- **Objectif :** Offrir une profondeur et des contrastes cinématographiques à coût CPU quasi nul.
- **Action Technique :**
  - **Cascaded Shadow Maps** : Ombres portées dynamiques couplées à la trajectoire solaire vectorisée.
  - **Tonemapping ACES Fitted** : Conversion des couleurs linéaires pour un rendu vibrant.
  - **Bloom & SSAO** : Halos sur la flore K3 et ombrage physiquement réaliste (Ambient Occlusion) pour ancrer les arbres et assombrir les grottes $Betti_1$.

---

## 📋 ROADMAP & SCRIPT DE DÉPLOIEMENT

**Semaine 1 (Quick Wins Visuels Bevy) :**
- Implémentation du Shader PBR Triplanaire.
- Activation de la pile Post-Processing native Bevy (Ombres, SSAO, ACES, Bloom). Le rendu est transformé immédiatement.

**Semaine 2 (L'Instanciation VRAM) :**
- Développement et activation du GPU Hardware Instancing pour la flore (arbres fractals et feuillages K3). Soulagement massif du CPU.

**Semaine 3-4 (Le Grand Saut GPU) :**
- Migration des algorithmes de base vers les Compute Shaders : Extraction du SDF et solveur SPH. Atteinte de l'architecture "Zero-Copy" finale et des **60 FPS minimum garantis**.
