//! Benchmarks for the matrix operations that dominate whole-building
//! simulations (see `benches/simulation.rs` in the workspace root).
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use matrix::{matrix::Matrix, NDiagMatrix};

/// Solving the tridiagonal system of a no-mass chunk (e.g., a window), which
/// `heat` does in every iteration of `march_nomass`.
pub fn tri_diag_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("tri_diag_mut_gaussian");
    for n in [3, 8, 20] {
        let mut k = NDiagMatrix::<3>::new(0.0, n, n);
        for i in 0..n {
            k.set(i, i, 4.0).unwrap();
            if i > 0 {
                k.set(i, i - 1, -1.0).unwrap();
            }
            if i + 1 < n {
                k.set(i, i + 1, -1.0).unwrap();
            }
        }
        let q = vec![1.0; n];
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || (k.clone(), q.clone()),
                |(mut k, mut q)| {
                    k.mut_gaussian(&mut q).unwrap();
                    q
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

/// Daylight coefficients (surfaces x sky bins) times the sky vector, which
/// `light` does four times per timestep. 145 bins is a Reinhart sky with
/// MF=1 (box model); 2305 is MF=4 (cold apartment).
pub fn dc_times_sky(c: &mut Criterion) {
    let mut group = c.benchmark_group("dc_times_sky");
    for (n_surfaces, n_bins) in [(6, 145), (60, 2305)] {
        let dc = black_box(Matrix::new(0.123, n_surfaces, n_bins));
        let sky = black_box(Matrix::new(1.23, n_bins, 1));
        group.bench_function(format!("{n_surfaces}x{n_bins}"), |b| b.iter(|| &dc * &sky));
    }
    group.finish();
}

criterion_group!(benches, tri_diag_solve, dc_times_sky);
criterion_main!(benches);
