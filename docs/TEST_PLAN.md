# UniversCraft HoloEngine — Formal Test & Verification Plan

This document outlines the strict testing strategy mandated by the HoloEngine Phase 4 Improvement Plan. Because this engine relies on advanced mathematical topology, standard code coverage is insufficient. We require invariant-based testing.

## 1. Mathematical Invariant Tests

The core of our testing philosophy relies on asserting physical and topological realities:

### 1.1 Incompressible Flow (Leray-Hopf)
*   **Test:** Divergence strictly bounds to zero.
*   **Condition:** After the pressure-projection solver step, `sum(divergence(v))` must be `< 1e-5`.
*   **Status:** Needs Implementation (Blocked by proper Poisson solver replacing the current heuristic).

### 1.2 Kinetic Energy Bounds
*   **Test:** Absolute velocity limits.
*   **Condition:** `0.5 * m * ||v||^2` for any particle must never exceed `max_kinetic_energy`.
*   **Status:** ✅ Implemented. `apply_kinetic_energy_cap()` enforces this exact boundary.

### 1.3 Symplectic Conservation (Hamiltonian)
*   **Test:** Yoshida N-body solver must conserve energy.
*   **Condition:** Over 10,000 steps, total Hamiltonian drift must be `abs(ΔH/H0) < 1e-6`.
*   **Status:** Needs Implementation.

### 1.4 Persistent Homology (TDA)
*   **Test:** Exact Betti numbers on known geometries.
*   **Condition:** 
    *   Unit Circle: $B_0=1$, $B_1=1$, $B_2=0$
    *   Unit Sphere: $B_0=1$, $B_1=0$, $B_2=1$
    *   Torus: $B_0=1$, $B_1=2$, $B_2=1$
*   **Status:** Needs Implementation (Current algorithm approximates via Euler characteristic shortcut which fails for complex geometries).

---

## 2. Structural & Architectural Tests

### 2.1 GPU Context Memory Leak Prevention
*   **Test:** The `GpuContext` must initialize exactly once per client instance.
*   **Condition:** Calling rendering functions must utilize the cached Device/Adapter. Recreating devices must fail the test.
*   **Status:** ✅ Implemented conceptually. Needs a test harness to mock the GPU adapter.

### 2.2 Spatial Hashing Validation
*   **Test:** $O(N)$ lookup validation.
*   **Condition:** Querying neighbors within radius $R$ via the hash grid must yield the exact same `Vec<ParticleId>` as a naive $O(N^2)$ distance check.
*   **Status:** Blocked by Phase C3 (Spatial Hashing Implementation).

---

## 3. Formal Verification (Future Phase E)

We are committing to migrating critical core logic into theorem provers:

1.  **Lean 4 Integration:** 
    *   Goal: Prove that the T-Dual metric transformation function is smooth (at least $C^1$ continuous) across the string scale $R = \sqrt{\alpha'}$.
2.  **Verus for Rust:**
    *   Goal: Annotate the `SymplecticFluidSolver::step()` function. Prove memory safety and bounds checking mathematically, completely removing the need for runtime panics during array iterations.

## Execution Matrix

| Test Suite | Framework | Target Files | Priority |
|------------|-----------|--------------|----------|
| Topology | `cargo test` | `topological_physics.rs`, `tda_engine.rs` | High |
| Fluid Math | `cargo test` | `fluid_solver.rs`, `navier_stokes_and_amcp_tests.rs` | Critical |
| Astrophysics | `cargo test` | `astrophysics.rs`, `physics.rs` | Medium |
| Shaders | `wgpu` runner | `scene_*.wgsl` via `test_rtx_gpu.py` | High |
| Proofs | Lean 4 | `specs/proofs/*.lean` (TBD) | Low |
