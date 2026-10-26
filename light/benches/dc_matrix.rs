//! Multiplying daylight coefficient matrices by a sky vector, as `SolarModel`
//! does in every timestep: the generic `Matrix` product against `DCMatrix`.
use criterion::{criterion_group, criterion_main, Criterion};
use light::{DCMatrix, Float};
use matrix::Matrix;
use std::hint::black_box;

/// Reads a daylight coefficient matrix from cached optical data
fn read_dc(file: &str, name: &str) -> Matrix {
    let s = std::fs::read_to_string(file).expect("Could not read optical data");
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    serde_json::from_value(v[name].clone()).unwrap()
}

/// A matrix in which every row has (roughly) `density` non-zero values
fn synthetic(nrows: usize, ncols: usize, density: Float) -> Matrix {
    let mut state: u64 = 3;
    let mut next = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 11) as f64 / (1u64 << 53) as f64) as Float
    };
    let mut m = Matrix::new(0.0, nrows, ncols);
    for r in 0..nrows {
        for c in 0..ncols {
            if next() < density {
                m.set(r, c, next()).unwrap();
            }
        }
    }
    m
}

fn compare(c: &mut Criterion, name: &str, m: &Matrix) {
    let (nrows, ncols) = m.size();
    let sky_vec: Vec<Float> = (0..ncols).map(|i| (i % 7) as Float + 0.5).collect();
    let sky = Matrix::from_data(ncols, 1, sky_vec.clone());
    let dc = DCMatrix::from(m);
    let mut into = vec![0.0; nrows];

    let mut group = c.benchmark_group(format!("dc_times_sky/{name}"));
    group.bench_function("matrix", |b| b.iter(|| black_box(m) * black_box(&sky)));
    group.bench_function("dc_matrix", |b| {
        b.iter(|| dc.mul_vec_into(black_box(&sky_vec), &mut into).unwrap())
    });
    group.finish();
}

pub fn real(c: &mut Criterion) {
    let cold = "../tests/cold_apartment/cold_optical_data.json";
    compare(c, "cold_front_surfaces", &read_dc(cold, "front_surfaces_dc"));
    compare(c, "cold_back_surfaces", &read_dc(cold, "back_surfaces_dc"));
    compare(c, "cold_front_fenestrations", &read_dc(cold, "front_fenestrations_dc"));
    let bx = "../tests/box/optical_data.json";
    compare(c, "box_front_surfaces", &read_dc(bx, "front_surfaces_dc"));
}

pub fn synthetic_densities(c: &mut Criterion) {
    for density in [0.25, 0.4, 0.5, 0.6, 0.75, 1.0] {
        let m = synthetic(60, 2306, density);
        compare(c, &format!("60x2306_density_{density}"), &m);
    }
}

criterion_group!(benches, real, synthetic_densities);
criterion_main!(benches);
