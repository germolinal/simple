use criterion::{black_box, criterion_group, criterion_main, Criterion};
use matrix::matrix::Matrix;

pub fn add_into(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function(
        "
    add_into",
        |b| b.iter(|| this.add_into(&other, &mut into)),
    );
}

pub fn sub_into(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("sub_into", |b| b.iter(|| this.sub_into(&other, &mut into)));
}

pub fn scale_into(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("scale_into", |b| {
        b.iter(|| this.scale_into(black_box(22.0), &mut into))
    });
}

pub fn prod_into(c: &mut Criterion) {
    let ncols = 30;
    let nrows = 30;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("prod_into", |b| {
        b.iter(|| this.prod_into(&other, &mut into))
    });
}

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
        b.iter(|| this.prod_tri_diag_into(&other, &mut into))
    });
}

pub fn from_prod_n_diag(c: &mut Criterion) {
    let ncols = 25;
    let nrows = 25;
    let this = black_box(Matrix::new(1.23123, nrows, ncols));
    let other = black_box(Matrix::new(1.23123, nrows, ncols));
    let mut into = black_box(Matrix::new(1.23123, nrows, ncols));

    c.bench_function("from_prod_n_diag", |b| {
        b.iter(|| into = this.from_prod_n_diag(&other, 3).unwrap())
    });
}

criterion_group!(
    benches,
    add_into,
    sub_into,
    scale_into,
    prod_into,
    from_prod_n_diag,
    prod_tri_diag_into,
);
criterion_main!(benches);
