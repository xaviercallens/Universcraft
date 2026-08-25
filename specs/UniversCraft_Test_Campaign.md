# UniversCraft: Test Campaign Strategy

This test campaign verifies that the HoloEngine strictly adheres to its mathematical boundaries and does not suffer from traditional game engine failures (memory leaks, float overflows, mesh tearing).

## 1. Formal Verification (Zero-Sorry Audits)
* **1.1 Lean 4 Kernel Validation**: Compile all mathematical models (`DualScale.lean`, `TopoStability.lean`, `FourierStateZ3.lean`) using Lean 4. Target: `0 warnings, 0 errors, 0 sorry axioms`.
* **1.2 Leray-Hopf Axiom Audit**: Run `#print axioms` on `leray_projector_divergence_free` and `leray_projector_idempotent` to ensure only permitted axioms (e.g., `propext`, `Classical.choice`) are utilized.
* **1.3 Verus Code Contracts**: Run Verus static analysis on critical Rust core modules to mathematically prove memory safety and loop invariances.

## 2. Rendering & T-Dual LOD Tests
* **2.1 UV Divergence Prevention**: Force the camera to zoom infinitely into a continuous SDF fractal.
  * *Expected Result*: Ray marcher dynamically switches to the discrete K3 surface at distance $\sqrt{\alpha'}$. GPU frame times remain stable (< 16ms).
* **2.2 Compute Shader Load Test**: Spawn 10 million particles in the ray marcher. Ensure WGSL compute shaders properly cull non-intersecting regions based on the effective metric.

## 3. TDA & Surface Stability Tests
* **3.1 Lipschitz Continuity Validation**: Introduce massive, chaotic modifications (simulated explosions/mining) to the point cloud.
  * *Expected Result*: The `Splashsurf` reconstructor generates smooth transitions. No mesh tearing, overlapping polygons, or infinite loops occur.
* **3.2 Vietoris-Rips Boundary Test**: Test the threshold distance $\le t$ for simplices connection to ensure proper topological hole closure (no "leaking" geometry).

## 4. Symplectic Hydrodynamics Tests
* **4.1 Navier-Stokes Blow-up Challenge**: Inject catastrophic, localized kinetic energy into a simulated ocean.
  * *Expected Result*: The Enstrophy cap (Virasoro central charge bound) truncates the energy cascade. The simulation safely disperses the energy topologially. The engine does not crash.
* **4.2 Incompressibility Assertion**: Continually measure the divergence of the velocity field after the Leray-Hopf projector is applied.
  * *Expected Result*: $\nabla \cdot \vec{u} = 0$ at all times (allowing for minimal acceptable floating-point drift, though strictly bounded by the lattice projection).

## 5. Network & AMCP Tests
* **5.1 Topology Disruption Adaptation**: Suddenly alter the terrain (Persistence Landscape) under a swarm of AMCP-driven agents.
  * *Expected Result*: Agents immediately renegotiate their state based on the new topological data without requiring rigid behavior tree resets.
* **5.2 Sovereignty Algorithm Scale**: Simulate a macro-scale node interaction (10,000 agents communicating energy transfers). Check for deadlocks or packet storms.
