use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(feature = "texture")]
use geometry::round_error::ApproxFloat;

pub fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("get_perpendicular", |b| b.iter(|| get_perpendicular(black_box(Vector3D::new(1., 2., 3.) )) ));

    // #[cfg(feature = "texture")]
    // let n1 = black_box(ApproxFloat::from_value_and_error(2.12312, 12.));
    // #[cfg(feature = "texture")]
    // let n2 = black_box(ApproxFloat::from_value_and_error(-2.12312, 2.));

    // #[cfg(not(feature = "texture"))]
    // let n1 = 12.12131;
    // #[cfg(not(feature = "texture"))]
    // let n2 = 0.12141;

    // c.bench_function("mul", |b| {
    //     b.iter(|| {
    //         let _ = black_box(n1 * n2);
    //     })
    // });

    // c.bench_function("bbox_intersection", |b| {
    //     b.iter(|| {
    //         let _ = black_box(n1 * n2);
    //     })
    // });

    let bbox = black_box(geometry::BBox3D::new(
        geometry::Point3D::new(0., 0., 0.),
        geometry::Point3D::new(1., 1., 1.),
    ));
    let (x, y, z) = (0.5, 0.5, -2.);
    let ray = black_box(geometry::Ray3D {
        origin: geometry::Point3D::new(x, y, z),
        direction: geometry::Vector3D::new(0., 0., 1.),
    });
    let inv_dir = black_box(geometry::Vector3D::new(1. / x, 1. / y, 1. / z));

    c.bench_function("bbox_intersection", |b| {
        b.iter(|| {
            let _ = black_box(bbox.intersect(ray, &inv_dir));
        })
    });

    // let v = black_box([1., 2., -9.1, 12.]);
    // c.bench_function("max_min", |b| {
    //     b.iter(|| {

    //         let  _ = black_box(geometry::round_error::max_min(&v));
    //     })
    // });

    // let m1 = black_box([
    //     1., 1., 2., 3., 4., 5., 1., 2., 3., 1., 5., 1., 8., 7., 2., 1.,
    // ]);
    // let m2 = black_box([
    //     1., 1., 2., 3., 4., 5., 1., 2., 3., 11., 25., 1., 8., -7., 2., 12.,
    // ]);
    // c.bench_function("mul4x4", |b| {
    //     b.iter(|| {
    //         let _ = black_box(geometry::transform::mul4x4(&m1, &m2));
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
