use criterion::{black_box, criterion_group, criterion_main, Criterion};
use matrix::{matrix::Matrix, NDiagMatrix};

pub fn prod_n_diag_into(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("prod_n_diag_into", |b| {
        b.iter(|| this.prod_n_diag_into(&other, 3, &mut into))
    });
}

pub fn prod_tri_diag_into(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("prod_tri_diag_into", |b| {
        b.iter(|| this.prod_tri_diag_into(&other, &mut into).unwrap())
    });
}

pub fn prod_tri_diag_into_2(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(NDiagMatrix::<3>::new(1.23123, nrows, ncols));
    let other = black_box(vec![0.0; ncols]);
    let mut into = black_box(vec![0.0; ncols]);

    c.bench_function("prod_tri_diag_into_2", |b| {
        b.iter(|| this.column_prod_into(&other, &mut into).unwrap())
    });
}

pub fn from_prod_n_diag(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("from_prod_n_diag", |b| {
        b.iter(|| {
            into = this
                .from_prod_n_diag(&other, 3)
                .expect("could not multiply")
        })
    });
}

criterion_group!(
    benches,
    prod_n_diag_into,
    from_prod_n_diag,
    prod_tri_diag_into,
    prod_tri_diag_into_2,
);
criterion_main!(benches);
