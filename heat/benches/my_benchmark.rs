use criterion::{black_box, criterion_group, criterion_main, Criterion};

use heat::surface::{rk4, ChunkMemory};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut memory = black_box(ChunkMemory::new(0, 23));
    c.bench_function("rk4", |b| b.iter(|| rk4(&mut memory).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
