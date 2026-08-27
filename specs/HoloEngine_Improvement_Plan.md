# HoloEngine — Audit Report & Detailed Improvement Plan

> **Status**: Planning Only — Do Not Implement  
> **Auditor**: Architect AI (Claude Opus 4.6)  
> **Date**: 2026-08-27  
> **Scope**: Full codebase review of `holo_engine/` (Rust core), WGSL shaders, and JS Studio

---

## 📊 Audit Summary

| Module | Files | Lines | Severity | Verdict |
|---|---|---|---|---|
| `fluid_solver.rs` | 1 | 116 | 🔴 Critical | Missing neighbor interactions — not a real SPH solver |
| `physics.rs` | 1 | 88 | 🟡 Medium | Leray projection is a scalar damping, not a divergence-free projection |
| `tda_engine.rs` | 1 | 187 | 🟡 Medium | O(n³) triangle search, Betti₂ formula is heuristic not algebraic |
| `biome_generator.rs` | 1 | 174 | 🟢 Low | Vec3 fallback is duplicated; L-System only 2D branching (XY plane) |
| `terrain_generator.rs` | 1 | 154 | 🟡 Medium | Rayon parallelization uses wrong indexing (par_chunks_mut stride mismatch) |
| `gpu_compute.rs` | 1 | 114 | 🟢 Low | WGSL string is never dispatched to a real GPU device |
| `renderer.rs` | 1 | 122 | 🟢 Low | GPUInstanceBuffer is data-only; no actual Bevy render integration |
| `atmosphere.rs` | 1 | 107 | 🟢 Low | Correct physics; could add star field / moon phase |
| `amcp.rs` | 1 | 88 | 🟢 Low | Functional async agent protocol |
| `ssfr.wgsl` | 1 | 78 | 🟢 Low | Valid shader; not wired into Bevy render graph |
| `triplanar_pbr.wgsl` | 1 | 107 | 🟢 Low | Valid shader; `weights` computed but unused in fragment |
| `topological_studio_engine.js` | 1 | 514 | 🟡 Medium | Canvas 2D CPU rendering; multiple perf bottlenecks |
| Tests | 5 | ~600 | 🟡 Medium | 23 tests pass but many test tautologies (no adversarial inputs) |

---

## 🔴 PRIORITY 1 — Critical Physics Fixes

### Task 1.1: Implement Real SPH Neighbor Search in `fluid_solver.rs`

> [!CAUTION]
> The current `step()` method applies gravity and enstrophy cap per-particle but performs **zero neighbor interactions**. A real SPH solver must compute density, pressure forces, and viscosity from neighbors.

**File**: [fluid_solver.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/client/fluid_solver.rs)

**Steps**:
1. Add a `compute_density()` method that iterates over all particle pairs within `smoothing_radius`, applying the Poly6 kernel: $W_{poly6}(r, h) = \frac{315}{64 \pi h^9}(h^2 - r^2)^3$
2. After density computation, call `compute_tait_pressure()` per particle (this part is already correct)
3. Add a `compute_pressure_force()` method using the Spiky kernel gradient: $\nabla W_{spiky}(r, h) = -\frac{45}{\pi h^6}(h - r)^2 \hat{r}$
4. Add a `compute_viscosity_force()` method using the viscosity kernel Laplacian: $\nabla^2 W_{visc}(r, h) = \frac{45}{\pi h^6}(h - r)$
5. Update `step()` to call: density → pressure → forces → Leray projection → integration
6. Update `step_parallel()` with the same logic but split into two phases (density pass is read-only, force pass is write)

**Test to add**: Create a 2-particle test where particles at distance < h must produce non-zero density > rest_density

---

### Task 1.2: Replace Scalar Damping with Proper Leray Projection in `physics.rs`

> [!WARNING]
> Lines 61-63 of `physics.rs` implement "solenoidal projection" as `vel[0] *= 0.98; vel[2] *= 0.98;` — this is simple drag, **not** a divergence-free projection.

**File**: [physics.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/engine/physics.rs)

**Steps**:
1. For the particle-based physics in `physics.rs`, replace the 0.98 damping with a velocity divergence check: compute local divergence from neighbor velocity differences
2. If divergence exceeds a threshold ε, project the divergent component out by subtracting the gradient of a local pressure correction
3. For the simplified engine (no spatial grid), an acceptable approximation is: after all force updates, compute the mean radial velocity component and subtract it (enforces $\nabla \cdot \mathbf{v} \approx 0$ at the particle cluster scale)
4. Update the doc comment to clarify this is an "approximate Leray projection for particle systems" vs. a grid-based Helmholtz decomposition

**Test to add**: Assert that after `step()`, the sum of all velocity divergences is below a threshold (e.g., `< 0.1 * particle_count`)

---

## 🟡 PRIORITY 2 — Correctness & Performance Fixes

### Task 2.1: Fix Rayon Parallelization Indexing in `terrain_generator.rs`

**Problem**: The `par_chunks_mut` call splits the voxel array into chunks of size `chunk_res` (34), then derives `z` and `y` from `z_y_idx`. But the array is linearized as `x + y * res + z * res * res`, so the chunk stride doesn't correspond to Z-Y slices.

**File**: [terrain_generator.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/client/terrain_generator.rs)

**Steps**:
1. Replace `par_chunks_mut` with `rayon::par_iter` on a flat index range `0..(34*34*34)`
2. Compute `x = idx % 34`, `y = (idx / 34) % 34`, `z = idx / (34 * 34)` from flat index
3. Use `ChunkShape::linearize([x, y, z])` to write to the correct voxel slot
4. Alternative: parallelize only the outermost Z loop using `(0..chunk_res).into_par_iter()` and compute each Z-slice sequentially

**Test to add**: Compare output of parallel vs sequential evaluation for a known SDF (sphere at origin, radius 5)

---

### Task 2.2: Improve TDA Betti₂ Computation in `tda_engine.rs`

**Problem**: Line 147 computes `betti_2 = (triangles / 4).max(1)`. This is a heuristic with no mathematical basis.

**File**: [tda_engine.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/poc2/tda_engine.rs)

**Steps**:
1. Rename the current method to `compute_vietoris_rips_betti_approximate()` and add a doc comment explaining it's a heuristic
2. For Betti₂, count 4-cliques (tetrahedra) and apply the Euler characteristic: $\chi = V - E + T - K$ where $B_2 = K - T + E - V + B_0 - B_1$ (use the already-computed B0, B1, edge_count, triangle_count, and tetrahedra_count)
3. Add guard against O(n⁴) for tetrahedra by limiting particle count to 200 in the approximation path

**Test to add**: Construct a known topology (8 vertices of a cube with all faces triangulated) and assert B0=1, B1=0, B2=1

---

### Task 2.3: Make L-System Trees Branch in 3D in `biome_generator.rs`

**Problem**: `generate_branch()` only varies X and Y in branching directions. The Z component is always 0.

**File**: [biome_generator.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/client/biome_generator.rs)

**Steps**:
1. Parameterize branching with a random seed derived from the branch depth and parent position (deterministic)
2. Use 3D branching vectors: `Vec3::new(-0.4 * cos(seed), 0.3, -0.4 * sin(seed))` for left, `Vec3::new(0.4 * cos(seed+PI), 0.3, 0.4 * sin(seed+PI))` for right
3. Add a third branch direction at 40% probability for more organic look
4. Extract the `Vec3` fallback into a shared `math_types.rs` module (see Task 5.1)

**Test to add**: Assert that generated tree positions span all 3 axes (max_z - min_z > 0.1)

---

## 🟢 PRIORITY 3 — Integration & Wiring

### Task 3.1: Wire `triplanar_pbr.wgsl` to Bevy Custom Material

**Problem**: The shader file exists but is never loaded or used by any Rust code.

**File**: New file `holo_engine/src/client/terrain_material.rs`

**Steps**:
1. Create a `TerrainMaterial` struct implementing Bevy's `Material` trait
2. Use `#[derive(AsBindGroup)]` to bind `TerrainUniforms` as `@group(0) @binding(0)`
3. Reference the shader via `Asset::load("shaders/triplanar_pbr.wgsl")`
4. Register the material plugin in `main.rs` setup function
5. Replace `StandardMaterial` in terrain chunk spawning with `TerrainMaterial`

---

### Task 3.2: Wire `ssfr.wgsl` into the Bevy Render Graph

**Problem**: The SSFR shader exists but has no render pass integration.

**File**: New file `holo_engine/src/client/ssfr_pass.rs`

**Steps**:
1. Create a custom Bevy render pass that renders fluid particles as point sprites to a depth-only framebuffer
2. Create a second fullscreen pass that reads the depth texture and applies the bilateral blur + normal reconstruction from `ssfr.wgsl`
3. Add the render graph node after the main opaque pass but before the transparent pass
4. Wire the `fluid_solver.rs` particle positions into a Bevy `GpuBuffer` that the depth pass reads

---

### Task 3.3: Connect `GPUInstanceBuffer` to Bevy Instanced Rendering

**Problem**: `GPUInstanceBuffer` in `renderer.rs` is a CPU-side data structure with no GPU upload path.

**File**: New file `holo_engine/src/client/flora_instancing.rs`

**Steps**:
1. In the Bevy `Update` system, populate the `GPUInstanceBuffer` from Whittaker-validated flora positions
2. Convert `GPUInstanceTransform` to a `bytemuck::Pod`-compatible struct
3. Upload to a Bevy `StorageBuffer` or use Bevy's `InstanceMaterialData` API
4. Render using `draw_mesh_instanced` with the trunk mesh and K3 billboard mesh

---

### Task 3.4: Integrate `gpu_compute.rs` WGSL Shader with WGPU Device

**Problem**: `TERRAIN_SDF_WGSL` is a string constant but `GPUComputeManager` never creates a WGPU device, pipeline, or dispatches the shader.

**File**: [gpu_compute.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/src/client/gpu_compute.rs)

**Steps**:
1. Add a method `async fn init_device(&mut self)` that calls `wgpu::Instance::new()` → `request_adapter()` → `request_device()`
2. Create compute pipeline from `TERRAIN_SDF_WGSL` source
3. Add `dispatch_sdf_evaluation()` that creates uniform/storage buffers, binds them, and dispatches workgroups `(grid_res/8, grid_res/8, grid_res/8)`
4. Add `readback_results()` that maps the output buffer and returns `Vec<f32>`
5. Gate behind `#[cfg(feature = "full")]` or a new `gpu` feature flag

---

## 🟡 PRIORITY 4 — Test Hardening

### Task 4.1: Add Adversarial Edge-Case Tests

**File**: [unit_tests.rs](file:///home/xavkal/xdev/UniversCraft/Universcraft/holo_engine/tests/unit_tests.rs)

**Steps**:
1. **Enstrophy stress test**: Create 1000 particles with velocity = (100, 100, 100). Assert ALL velocities are clamped after `step()`
2. **Zero-density SPH test**: Set density = 0.0 and assert no NaN/Inf in pressure computation
3. **Negative height atmosphere test**: Call `evaluate_sky_color` with `view_dir = (0, -1, 0)` and assert no negative RGB
4. **Empty world TDA test**: Call `compute_vietoris_rips_betti()` with 0 particles and assert B0=0
5. **Extreme T-Duality test**: Call `compute_r_eff(0.0)` and assert no division by zero

### Task 4.2: Add Integration Benchmarks

**File**: New file `holo_engine/benches/perf_benchmarks.rs`

**Steps**:
1. Add `criterion` dependency to `Cargo.toml` under `[dev-dependencies]`
2. Benchmark `fluid_solver.step()` with 1000 particles (target: < 1ms)
3. Benchmark `fluid_solver.step_parallel()` with 10000 particles
4. Benchmark `tda_engine.compute_vietoris_rips_betti()` with 100 particles
5. Benchmark `donn_generator.evaluate_scalar_field()` for 32³ grid

---

## 🟢 PRIORITY 5 — Code Quality & Architecture

### Task 5.1: Extract Shared Math Types

> [!IMPORTANT]
> `Vec3` is defined in 3 different ways: tuple `(f32, f32, f32)` in `fluid_solver.rs` and `atmosphere.rs`, custom struct in `biome_generator.rs`, and `[f32; 3]` in `world.rs`.

**File**: New file `holo_engine/src/math_types.rs`

**Steps**:
1. Create a single `Vec3` struct with `Add`, `Sub`, `Mul<f32>`, `Div<f32>`, `Neg`, `normalize()`, `dot()`, `cross()`, `length()`, `length_squared()`
2. When `feature = "full"`, re-export `bevy::math::Vec3` via a type alias
3. When `feature != "full"`, use the custom implementation
4. Migrate `fluid_solver.rs` from tuples to `Vec3`
5. Migrate `atmosphere.rs` from tuples to `Vec3`
6. Remove the inline `Vec3Normalize` trait from `atmosphere.rs`
7. Remove the inline `Vec3` struct from `biome_generator.rs`

### Task 5.2: Add `#[cfg]` Guards to All Binary Targets

**Problem**: Some `bin/*.rs` files have unguarded Bevy imports that break `cargo test --lib --tests` compilation.

**Files**: All files in `holo_engine/src/bin/`

**Steps**:
1. Audit every `use bevy::*` import in all binary files
2. Ensure every Bevy import and function is wrapped in `#[cfg(feature = "full")]`
3. Add a `#[cfg(not(feature = "full"))] fn main() {}` fallback to each binary
4. Verify with `cargo check` (no features) and `cargo check --features full`

### Task 5.3: Add `clippy` and `rustfmt` CI Lints

**File**: New file `.github/workflows/ci.yml`

**Steps**:
1. Create a GitHub Actions workflow that runs on push to `main`
2. Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --tests`
3. Add a `rustfmt.toml` with `max_width = 120` and `edition = "2024"`

---

## 📋 Execution Order (Sprint Backlog)

| # | Task | Priority | Effort | Dependencies |
|---|---|---|---|---|
| 1 | 1.1 Real SPH Neighbor Search | 🔴 P1 | 3h | None |
| 2 | 1.2 Proper Leray Projection | 🔴 P1 | 2h | None |
| 3 | 2.1 Fix Rayon Indexing | 🟡 P2 | 1h | None |
| 4 | 5.1 Extract Math Types | 🟢 P5 | 2h | None |
| 5 | 2.3 3D L-System Trees | 🟡 P2 | 1h | 5.1 |
| 6 | 2.2 Improve Betti₂ | 🟡 P2 | 2h | None |
| 7 | 4.1 Adversarial Tests | 🟡 P4 | 1.5h | 1.1, 1.2 |
| 8 | 5.2 Fix cfg Guards | 🟢 P5 | 1h | None |
| 9 | 3.1 Wire Triplanar PBR | 🟢 P3 | 3h | `full` feature |
| 10 | 3.2 Wire SSFR Pass | 🟢 P3 | 3h | `full` feature |
| 11 | 3.3 Flora Instancing | 🟢 P3 | 2h | 3.1 |
| 12 | 3.4 GPU Compute Dispatch | 🟢 P3 | 3h | `full` feature |
| 13 | 4.2 Criterion Benchmarks | 🟡 P4 | 1.5h | 1.1 |
| 14 | 5.3 CI Pipeline | 🟢 P5 | 1h | None |

**Estimated Total**: ~26 hours of engineering work.
