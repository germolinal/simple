use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use heat::surface::{ChunkMemory, rk4};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut memory = black_box(ChunkMemory::new(0, 23));
    c.bench_function("rk4", |b| b.iter(|| rk4(&mut memory).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
