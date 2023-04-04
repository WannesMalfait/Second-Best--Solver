use criterion::{black_box, criterion_group, criterion_main, Criterion};
use second_best::solver;

pub fn solver_speed_begin(c: &mut Criterion) {
    let mut solver = solver::Solver::default();
    solver.be_quiet();
    c.bench_function("solver speed (depth 7)", |b| {
        b.iter(|| solver.search(black_box(7)))
    });
    c.bench_function("solver speed (depth 9)", |b| {
        b.iter(|| solver.search(black_box(9)))
    });
    // c.bench_function("solver speed (depth 12)", |b| {
    //     b.iter(|| solver.search(black_box(12)))
    // });
}

pub fn solver_speed_end(c: &mut Criterion) {
    let mut solver = solver::Solver::default();
    solver.be_quiet();
    // Position which is extremely symmetric, at the start of the second phase.
    solver
        .position
        .parse_and_play_moves(
            "0 1 2 3 4 5 6 7 1 2 3 4 5 6 7 0"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
    c.bench_function("solver speed end (depth 7)", |b| {
        b.iter(|| solver.search(black_box(7)))
    });
    c.bench_function("solver speed end (depth 9)", |b| {
        b.iter(|| solver.search(black_box(9)))
    });
    // c.bench_function("solver speed end (depth 12)", |b| {
    //     b.iter(|| solver.search(black_box(12)))
    // });
}

pub fn solver_efficiency(c: &mut Criterion) {
    let mut solver = solver::Solver::default();
    solver.be_quiet();
    // Position which is not symmetric, but no low depth solution.
    solver
        .position
        .parse_and_play_moves(
            "0 1 4 5 7 2 1 0 3 4"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
    c.bench_function("solver efficiency (depth 7)", |b| {
        b.iter(|| solver.search(black_box(7)))
    });
    c.bench_function("solver efficiency (depth 9)", |b| {
        b.iter(|| solver.search(black_box(9)))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = solver_speed_begin, solver_speed_end, solver_efficiency
}
criterion_main!(benches);
