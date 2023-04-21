use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rendering::{rand::Rng, Ray, Scene, Spectrum};

// Reference targets: https://github.com/svenstaro/bvh
pub fn criterion_benchmark(c: &mut Criterion) {
    // Setup
    let mut aux: Vec<usize> = vec![0; 10];
    let mut ray = black_box(Ray {
        geometry: geometry::Ray3D {
            direction: geometry::Vector3D::new(0., 1., 2.).get_normalized(),
            origin: geometry::Point3D::new(1., 2., 1.),
        },
        ..Ray::default()
    });

    // ROOM

    let mut room = black_box(Scene::from_radiance("./tests/scenes/room.rad".to_string()));
    room.build_accelerator();

    c.bench_function("intersect_room", |b| {
        b.iter(|| black_box(room.cast_ray(&mut ray, &mut aux)))
    });

    // c.bench_function("unobstructed_room", |b| {
    //     b.iter(|| {
    //         room.unobstructed_distance(&ray.geometry, rendering::Float::MAX, &mut aux)
    //     })
    // });

    // CORNELL

    // let mut cornell = black_box(Scene::from_radiance("./tests/scenes/cornell.rad".to_string()));
    // cornell.build_accelerator();

    // c.bench_function("intersect_cornell", |b| {
    //     b.iter(|| {
    //         cornell.cast_ray(&mut ray, &mut aux)
    //     })
    // });

    // c.bench_function("unobstructed_cornell", |b| {
    //     b.iter(|| {
    //         cornell.unobstructed_distance(&ray.geometry, rendering::Float::MAX, &mut aux)
    //     })
    // });

    // TRIANGLES
    let mut triangles = black_box(Scene::new());
    let plastic = rendering::material::Material::Plastic(rendering::material::Plastic {
        colour: Spectrum::gray(0.5),
        specularity: 0.05,
        roughness: 0.1,
    });
    let mut rng = rendering::rand::get_rng();
    let plastic = triangles.push_material(plastic);
    let mut i = 0;
    while i < 120_000 {
        let (x1, y1, z1, x2, y2, z2, x3, y3, z3): (
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
            rendering::Float,
        ) = rng.gen();

        const SCALE: rendering::Float = 30.;
        if let Ok(tri) = geometry::Triangle3D::new(
            geometry::Point3D::new((x1 - 0.5) * SCALE, (y1 - 0.5) * SCALE, (z1 - 0.5) * SCALE),
            geometry::Point3D::new((x2 - 0.5) * SCALE, (y2 - 0.5) * SCALE, (z2 - 0.5) * SCALE),
            geometry::Point3D::new((x3 - 0.5) * SCALE, (y3 - 0.5) * SCALE, (z3 - 0.5) * SCALE),
        ) {
            i += 1;
            triangles.push_object(
                plastic,
                plastic,
                rendering::primitive::Primitive::Triangle(tri),
            );
        };
    }
    triangles.build_accelerator();

    c.bench_function("intersect_triangles", |b| {
        b.iter(|| black_box(triangles.cast_ray(&mut ray, &mut aux)))
    });

    // c.bench_function("unobstructed_triangles", |b| {
    //     b.iter(|| {
    //         triangles.unobstructed_distance(&ray.geometry, rendering::Float::MAX, &mut aux)
    //     })
    // }

    // SPONZA

    let mut ray = black_box(Ray {
        geometry: geometry::Ray3D {
            direction: geometry::Vector3D::new(1., 0., 0.).get_normalized(),
            origin: geometry::Point3D::new(0., 5., 0.),
        },
        ..Ray::default()
    });

    let mut scene = black_box(Scene::default());
    let gray = scene.push_material(rendering::material::Material::Plastic(
        rendering::material::Plastic {
            colour: Spectrum::gray(0.3),
            specularity: 0.,
            roughness: 0.,
        },
    ));
    scene.add_from_obj("./tests/scenes/sponza.obj".to_string(), gray, gray);
    scene.build_accelerator();

    c.bench_function("intersect_sponza", |b| {
        b.iter(|| black_box(scene.cast_ray(&mut ray, &mut aux)))
    });

    // DINING

    let mut ray = black_box(Ray {
        geometry: geometry::Ray3D {
            direction: geometry::Vector3D::new(1., 0., 0.).get_normalized(),
            origin: geometry::Point3D::new(-4.0, 1., 0.),
        },
        ..Ray::default()
    });

    let mut scene = black_box(Scene::default());
    let gray = scene.push_material(rendering::material::Material::Plastic(
        rendering::material::Plastic {
            colour: Spectrum::gray(0.3),
            specularity: 0.,
            roughness: 0.,
        },
    ));
    scene.add_from_obj("./tests/scenes/sponza.obj".to_string(), gray, gray);
    scene.build_accelerator();

    c.bench_function("intersect_dining", |b| {
        b.iter(|| black_box(scene.cast_ray(&mut ray, &mut aux)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
