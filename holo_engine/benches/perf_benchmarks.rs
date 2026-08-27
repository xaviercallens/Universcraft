use criterion::{black_box, criterion_group, criterion_main, Criterion};
use holo_engine::client::fluid_solver::{FluidParticle, SPHParams, SymplecticFluidSolver};
use holo_engine::math_types::Vec3;
use holo_engine::poc2::tda_engine::TdaEngine;

fn bench_fluid_solver(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fluid Solver");

    let params = SPHParams::default();
    let particles: Vec<FluidParticle> = (0..500)
        .map(|i| {
            FluidParticle::new(
                Vec3::new((i % 10) as f32 * 0.5, (i / 10) as f32 * 0.5, 0.0),
                Vec3::new(1.0, 0.0, -1.0),
                1.0,
            )
        })
        .collect();

    let mut solver = SymplecticFluidSolver::new(params, particles);

    group.bench_function("step_sequential_500", |b| {
        b.iter(|| {
            solver.step(black_box(0.016));
        })
    });

    group.bench_function("step_parallel_500", |b| {
        b.iter(|| {
            solver.step_parallel(black_box(0.016));
        })
    });

    group.finish();
}

fn bench_tda_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("TDA Engine");

    let mut tda = TdaEngine::new(4.5);
    tda.generate_point_cloud(250, 20.0);

    group.bench_function("vietoris_rips_betti_250", |b| {
        b.iter(|| {
            black_box(tda.compute_vietoris_rips_betti());
        })
    });

    group.finish();
}

fn bench_galaxy_dynamics(c: &mut Criterion) {
    let mut group = c.benchmark_group("Astrophysics Engine");

    let mut galaxy = holo_engine::client::astrophysics::GalaxyNBodySystem::new_spiral_galaxy(1000, 1.0e6, 5.0);

    group.bench_function("nbody_gravity_1000_stars", |b| {
        b.iter(|| {
            galaxy.step_nbody_gravity(black_box(31536000.0), black_box(1.0e12));
        })
    });

    group.finish();
}

fn bench_climate_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("Climate Engine");

    let mut grid = holo_engine::client::advanced_climate::BaroclinicClimateGrid::new(20, 20, 5, 1000.0);

    group.bench_function("baroclinic_step_20x20x5", |b| {
        b.iter(|| {
            grid.step_baroclinic_circulation(black_box(3600.0));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_fluid_solver, bench_tda_engine, bench_galaxy_dynamics, bench_climate_grid);
criterion_main!(benches);
