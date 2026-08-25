# HoloEngine 3D — Plan d'Implémentation : Phase 4 (Hydrodynamique Symplectique & SSFR) [✅ COMPLÉTÉ]

Ce document définit la feuille de route pour la **Phase 4**. L'objectif est d'implémenter une simulation fluide (eau, vent) physiquement réaliste, asynchrone, et topologiquement protégée contre le crash (blow-up Navier-Stokes).

---

## 🌊 1. Intégration du Solveur SPH (Smoothed-Particle Hydrodynamics) [✅ VALIDÉ]

**Objectif :** Déployer un solveur fluide complet dans le moteur Rust (Bevy) pour simuler de larges masses d'eau, en migrant depuis la simulation basique SPH du Studio Web.

**Spécifications techniques :**
- **Densité et Pression :** Implémentation des équations d'état (Tait Equation) pour un fluide quasi-incompressible.
- **Viscosité et Tension de Surface :** Modélisation de la cohésion de l'eau.
- **Multithreading Asynchrone :** Le calcul hydrodynamique doit s'exécuter sur un thread séparé (Async Compute Task Pool) pour ne pas bloquer le thread de rendu.

---

## 🛡️ 2. Projection de Leray-Hopf & Plafond d'Enstrophie (Invariants) [✅ VALIDÉ]

**Objectif :** Garantir mathématiquement la stabilité du fluide à grande échelle.

**Spécifications techniques :**
- **Projection Solenoidale :** Forcer le champ de vélocité à être à divergence nulle ($\nabla \cdot \vec{v} = 0$) à chaque étape de la simulation, assurant l'incompressibilité parfaite.
- **Cutoff d'Enstrophie K3 :** Appliquer la limite de l'énergie tourbillonnaire ($E_{max} = 25.0$). Si l'enstrophie dépasse ce seuil, les vélocités locales sont instantanément amorties via le projecteur de Leray, garantissant un système "Zero-Crash".

---

## 🎨 3. Rendu SSFR (Screen-Space Fluid Rendering) [✅ VALIDÉ]

**Objectif :** Remplacer le rendu particulaire actuel (sphères translucides qui se chevauchent) par une nappe liquide unifiée et performante, conformément aux recommandations de l'Architecte IA.

**Spécifications techniques :**
- **Depth Pass :** Rendre les particules fluides uniquement sous forme de profondeurs (Depth) dans un tampon d'écran.
- **Flou Bilatéral :** Lisser la profondeur (Smoothness) sans perdre les bords (Bilateral Blur).
- **Calcul des Normales :** Reconstruire les normales de surface à partir de la profondeur lissée pour appliquer des reflets spéculaires et une réfraction photoréaliste.

---

## 📂 Plan d'Action & Fichiers Ciblés [✅ INTÉGRÉ & TESTÉ]
1. `holo_engine/src/client/fluid_solver.rs` (Cœur SPH et projecteurs Leray en Rust).
2. `holo_engine/assets/shaders/ssfr.wgsl` (Compute/Render Shader pour le Screen-Space Fluid Rendering).
3. `holo_engine/tests/unit_tests.rs` (Tests unitaires de la dynamique des fluides et de la pression de Tait).

