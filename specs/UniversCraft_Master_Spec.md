# UniversCraft: Master Specification & Architecture Blueprint

## 1. Vision & Core Paradigm
**HoloEngine** is designed for UniversCraft, abandoning discrete floating-point approximations (rigid voxels, rasterized polygons) in favor of **Dual-Scale Topological Geometry**. By relying on mathematically verified topological limits (T-Duality, Vietoris-Rips Filtration, Enstrophy Bounds), the engine replaces computational singularities with geometric bounds, ensuring absolute mathematical and rendering stability.

## 2. Technology Stack
- **Language**: Rust (Memory safety, high performance).
- **Engine/ECS**: Bevy Engine (Data-driven, processing massive point-cloud arrays).
- **Graphics/Compute**: WGPU / WGSL (Compute shaders for massive fractal ray marching).
- **AI/ML**: Burn (Rust-native Deep Learning for DONNs).
- **Verification**: Lean 4 and Verus (Kernel-honest formal proofs).

## 3. Mathematical Foundations
### 3.1. Dual-Scale Metric & Quantum LOD
Instead of standard Euclidean distance, the engine uses the **T-Dual Effective Metric**: $R_{eff} = \max(R, \alpha'/R)$.
When the camera zooms in, continuous SDF (Signed Distance Field) evaluation halts at the fundamental string scale $\sqrt{\alpha'}$. The engine then projects the discrete **K3 Surface** arithmetic fiber. This ensures the GPU never crashes due to UV divergence.

### 3.2. Topological Surface Generation (TDA)
Terrain is a dynamic point cloud. Surfaces are generated using a **Vietoris-Rips Filtration**. Because the mesh generation is 1-Lipschitz continuous (proved in `TopoStability.lean`), terrain modification is perfectly smooth and mathematically guaranteed against chaotic visual tearing.

### 3.3. Symplectic Hydrodynamics & Enstrophy Bounds
Fluid dynamics are simulated via the **Leray-Hopf Projector** to guarantee incompressibility (divergence-free state). To solve the Navier-Stokes blow-up problem, fluid state vectors are projected onto the discrete topological lattice, capping enstrophy (vorticity energy) and preventing numerical blow-ups.

### 3.4. Deep Oscillatory Neural Networks (DONN)
To bring order to point-cloud generation, DONNs act as a **Topological Lock**. They apply resonance functions (cymatics) to align particles along standing waves, creating natural structures (mountains at nodes, valleys at antinodes) without rigid polygon placement.

## 4. Artificial Intelligence & Network
Entities and data streams are governed by **AMCP (Agent Mesh Communication Protocol)**. Agents act as nodes in a decentralized mesh, adapting their behaviors based on the local topology of the terrain (the Persistence Landscape) rather than rigid behavior trees.
