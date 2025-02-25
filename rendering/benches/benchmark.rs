use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rendering::{rand::Rng, Scene, Spectrum};

// Reference targets: https://github.com/svenstaro/bvh
pub fn criterion_benchmark(c: &mut Criterion) {
    // Setup
    let mut aux = [0; 32];
    let ray = black_box(geometry::Ray3D {
        direction: geometry::Vector3D::new(0., 1., 2.).get_normalized(),
        origin: geometry::Point3D::new(1., 2., 1.),
    });

    // ROOM

    let mut room = black_box(Scene::from_radiance("./tests/scenes/room.rad".to_string()))
        .expect("Could not load file");
    room.build_accelerator();

    c.bench_function("intersect_room", |b| {
        b.iter(|| room.cast_ray(ray, &mut aux))
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

    fn get_triangle_scene(n: usize) -> Scene {
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
        while i < n {
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
        triangles
    }

    let tri_1200 = get_triangle_scene(1200);
    c.bench_function("intersect_1200_triangles", |b| {
        b.iter(|| tri_1200.cast_ray(ray, &mut aux))
    });

    let tri_12k = get_triangle_scene(12_000);
    c.bench_function("intersect_12k_triangles", |b| {
        b.iter(|| tri_12k.cast_ray(ray, &mut aux))
    });

    let tri_120k = get_triangle_scene(120_000);
    c.bench_function("intersect_120k_triangles", |b| {
        b.iter(|| tri_120k.cast_ray(ray, &mut aux))
    });

    // c.bench_function("unobstructed_triangles", |b| {
    //     b.iter(|| {
    //         triangles.unobstructed_distance(&ray.geometry, rendering::Float::MAX, &mut aux)
    //     })
    // }

    // SPONZA

    let ray = black_box(geometry::Ray3D {
        direction: geometry::Vector3D::new(1., 0., 0.).get_normalized(),
        origin: geometry::Point3D::new(0., 5., 0.),
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
        b.iter(|| scene.cast_ray(ray, &mut aux))
    });

    // DINING

    let ray = black_box(geometry::Ray3D {
        direction: geometry::Vector3D::new(1., 0., 0.).get_normalized(),
        origin: geometry::Point3D::new(-4.0, 1., 0.),
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
        b.iter(|| scene.cast_ray(ray, &mut aux))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
