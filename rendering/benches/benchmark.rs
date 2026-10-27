use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use rendering::{rand::RngExt, Scene, Spectrum};

// Reference targets: https://github.com/svenstaro/bvh
pub fn criterion_benchmark(c: &mut Criterion) {
    // Setup (no component of the direction is zero, as in most rays)
    let mut aux = [0; 32];
    let ray = black_box(geometry::Ray3D {
        direction: geometry::Vector3D::new(0.3, 1., 2.).get_normalized(),
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
            ) = rng.random();

            // Triangles of up to ~1 m, centred anywhere in a 30 m cube (if
            // their vertices could be anywhere in the cube, they would all
            // overlap each other)
            const SCALE: rendering::Float = 30.;
            let (cx, cy, cz): (rendering::Float, rendering::Float, rendering::Float) =
                rng.random();
            let centre = geometry::Point3D::new((cx - 0.5) * SCALE, (cy - 0.5) * SCALE, (cz - 0.5) * SCALE);
            let vertex = |x: rendering::Float, y: rendering::Float, z: rendering::Float| {
                centre + geometry::Vector3D::new(x - 0.5, y - 0.5, z - 0.5)
            };
            if let Ok(tri) =
                geometry::Triangle3D::new(vertex(x1, y1, z1), vertex(x2, y2, z2), vertex(x3, y3, z3))
            {
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

    // SPONZA, with no component of the direction being zero (this was
    // called "dining", but it loaded Sponza too)

    let ray = black_box(geometry::Ray3D {
        direction: geometry::Vector3D::new(1., 0.1, 0.05).get_normalized(),
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

    c.bench_function("intersect_sponza_oblique", |b| {
        b.iter(|| scene.cast_ray(ray, &mut aux))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
