// ==============================================================================
// HoloEngine Phase 4 — 1-Lipschitz Avalanche & Aeolian Dune Relaxation Shader
// Enforces mathematical angle-of-repose stability (tan(34°) ≈ 0.6745).
// Excess elevation exceeding 1-Lipschitz gradient flows to neighboring cells.
// ==============================================================================

struct DuneSimulationParams {
    grid_size: u32,
    cell_spacing: f32,
    max_slope_tan: f32, // tan(34°) = 0.6745085
    wind_angle_rad: f32,
    wind_strength: f32,
    relaxation_rate: f32,
    time_step: f32,
    pad: f32,
};

@group(0) @binding(0) var<uniform> params: DuneSimulationParams;
@group(0) @binding(1) var<storage, read> height_in: array<f32>;
@group(0) @binding(2) var<storage, read_write> height_out: array<f32>;

fn get_idx(x: u32, y: u32, size: u32) -> u32 {
    let cx = clamp(x, 0u, size - 1u);
    let cy = clamp(y, 0u, size - 1u);
    return cx + cy * size;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    let n = params.grid_size;

    if (x >= n || y >= n) {
        return;
    }

    let idx = get_idx(x, y, n);
    let h_c = height_in[idx];

    // Neighbor coordinates with boundary clamp in pure WGSL
    let x_sub = select(0u, x - 1u, x > 0u);
    let y_sub = select(0u, y - 1u, y > 0u);
    let x_add = min(x + 1u, n - 1u);
    let y_add = min(y + 1u, n - 1u);

    let h_w = height_in[get_idx(x_sub, y, n)];
    let h_e = height_in[get_idx(x_add, y, n)];
    let h_s = height_in[get_idx(x, y_sub, n)];
    let h_n = height_in[get_idx(x, y_add, n)];

    // 1. Aeolian Wind Advection (Exner Equation)
    let wind_dir = vec2<f32>(cos(params.wind_angle_rad), sin(params.wind_angle_rad));
    let grad_x = (h_e - h_w) / (2.0 * params.cell_spacing);
    let grad_y = (h_n - h_s) / (2.0 * params.cell_spacing);
    let advection = -params.wind_strength * (wind_dir.x * grad_x + wind_dir.y * grad_y) * params.time_step;

    var new_h = h_c + advection;

    // 2. 1-Lipschitz Avalanche Relaxation (Repose Angle Bound)
    let max_delta = params.max_slope_tan * params.cell_spacing;
    var outflow = 0.0;

    if (new_h - h_w > max_delta) { outflow += (new_h - h_w - max_delta) * 0.25; }
    if (new_h - h_e > max_delta) { outflow += (new_h - h_e - max_delta) * 0.25; }
    if (new_h - h_s > max_delta) { outflow += (new_h - h_s - max_delta) * 0.25; }
    if (new_h - h_n > max_delta) { outflow += (new_h - h_n - max_delta) * 0.25; }

    var inflow = 0.0;
    if (h_w - new_h > max_delta) { inflow += (h_w - new_h - max_delta) * 0.25; }
    if (h_e - new_h > max_delta) { inflow += (h_e - new_h - max_delta) * 0.25; }
    if (h_s - new_h > max_delta) { inflow += (h_s - new_h - max_delta) * 0.25; }
    if (h_n - new_h > max_delta) { inflow += (h_n - new_h - max_delta) * 0.25; }

    new_h = new_h - outflow * params.relaxation_rate + inflow * params.relaxation_rate;

    height_out[idx] = new_h;
}
