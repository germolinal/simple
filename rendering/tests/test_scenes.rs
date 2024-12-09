use geometry::Point3D;
use geometry::{Sphere3D, Triangle3D, Vector3D};
use rendering::camera::{Film, Pinhole, View};
use rendering::material::*;
use rendering::primitive::Primitive;
use rendering::{Float, PI};
use rendering::{RayTracer, Scene, Spectrum};

#[test]
#[ignore]
fn laptop() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test test_scenes -- laptop --exact --nocapture --ignored
    let mut scene = Scene::new();

    const BASE_THICKNESS: Float = 0.01;
    const SCREEN_THICKNESS: Float = 0.006;
    const WIDTH: Float = 0.38;
    const DEPTH: Float = 0.3;
    const ANGLE: Float = 30. * PI / 180.0;
    const GROUND_SIZE: Float = 10.;
    const OFFSET: Float = 0.002;

    // Add light
    let glow = scene.push_material(Material::Light(Light(Spectrum([1., 1., 1.]) * 500.)));

    let s = Sphere3D::new(0.1, Point3D::new(0., 0., 5.));
    scene.push_object(glow, glow, Primitive::Sphere(s));

    // Materials
    let plastic = Material::Plastic(Plastic {
        colour: Spectrum([0.5, 0.8, 0.5]),
        specularity: 0.0,
        roughness: 0.0,
    });
    let plastic = scene.push_material(plastic);

    let screen = Material::Light(Light(Spectrum([145., 7., 205.]) * 0.03));
    let screen = scene.push_material(screen);

    let ground = Material::Plastic(Plastic {
        colour: Spectrum([0.2, 0.2, 0.2]),
        specularity: 0.0,
        roughness: 0.01,
    });
    let ground = scene.push_material(ground);

    // Ground
    let tri = Triangle3D::new(
        Point3D::new(-GROUND_SIZE, -GROUND_SIZE, 0.0),
        Point3D::new(GROUND_SIZE, -GROUND_SIZE, 0.0),
        Point3D::new(GROUND_SIZE, GROUND_SIZE, 0.0),
    )?;
    scene.push_object(ground, ground, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(-GROUND_SIZE, -GROUND_SIZE, 0.0),
        Point3D::new(GROUND_SIZE, GROUND_SIZE, 0.0),
        Point3D::new(-GROUND_SIZE, GROUND_SIZE, 0.0),
    )?;
    scene.push_object(ground, ground, Primitive::Triangle(tri));

    /* BASE */
    // Top of base
    let z = OFFSET + BASE_THICKNESS;
    let tri = Triangle3D::new(
        Point3D::new(0., 0., z),
        Point3D::new(WIDTH, 0.0, z),
        Point3D::new(0.0, DEPTH, z),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0.0, z),
        Point3D::new(WIDTH, DEPTH, z),
        Point3D::new(0.0, DEPTH, z),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // Front
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, DEPTH, OFFSET + BASE_THICKNESS),
        Point3D::new(0.0, DEPTH, OFFSET),
        Point3D::new(0.0, DEPTH, OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, DEPTH, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, DEPTH, OFFSET),
        Point3D::new(0.0, DEPTH, OFFSET),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // BACK
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(0.0, 0.0, OFFSET),
        Point3D::new(0.0, 0.0, OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, 0.0, OFFSET),
        Point3D::new(0.0, 0.0, OFFSET),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // LEFT
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0.0, OFFSET),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, DEPTH, OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0.0, OFFSET),
        Point3D::new(WIDTH, DEPTH, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, DEPTH, OFFSET),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // RIGHT
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, OFFSET),
        Point3D::new(0.0, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(0.0, DEPTH, OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, OFFSET),
        Point3D::new(0.0, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(0.0, DEPTH, OFFSET),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    /* SCREEN */

    let y: Float = DEPTH * ANGLE.cos();
    let z: Float = DEPTH * ANGLE.sin();
    // Top of screen
    let tri = Triangle3D::new(
        Point3D::new(0., 0., BASE_THICKNESS + SCREEN_THICKNESS + OFFSET),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
        Point3D::new(WIDTH, y, z + OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0., 0., OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
        Point3D::new(WIDTH, y, z + OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
        Point3D::new(0.0, y, z + OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // Left of screen
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0., OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, y, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS + z),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(WIDTH, 0., OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, y, OFFSET + BASE_THICKNESS + z),
        Point3D::new(WIDTH, y, z + OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // right of screen
    let tri = Triangle3D::new(
        Point3D::new(0., 0., OFFSET + BASE_THICKNESS),
        Point3D::new(0., y, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS + z),
        Point3D::new(0., 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0., 0., OFFSET + BASE_THICKNESS),
        Point3D::new(0., y, OFFSET + BASE_THICKNESS + z),
        Point3D::new(0., y, z + OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // back of screen
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, BASE_THICKNESS + OFFSET),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
        Point3D::new(0.0, 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, BASE_THICKNESS + OFFSET),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // Front of screen
    let tri = Triangle3D::new(
        Point3D::new(0.0, y, BASE_THICKNESS + OFFSET + z),
        Point3D::new(WIDTH, y, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS + z),
        Point3D::new(0.0, y, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS + z),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0.0, y, 0.0 + BASE_THICKNESS + OFFSET + z),
        Point3D::new(WIDTH, y, OFFSET + BASE_THICKNESS + z),
        Point3D::new(WIDTH, y, OFFSET + BASE_THICKNESS + SCREEN_THICKNESS + z),
    )?;
    scene.push_object(plastic, plastic, Primitive::Triangle(tri));

    // Bottom of screen (a.k.a. Screen)
    let tri = Triangle3D::new(
        Point3D::new(0., 0., OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, y, z + OFFSET + BASE_THICKNESS),
        Point3D::new(0.0, y, z + OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(screen, screen, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(0., 0., OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, 0.0, OFFSET + BASE_THICKNESS),
        Point3D::new(WIDTH, y, z + OFFSET + BASE_THICKNESS),
    )?;
    scene.push_object(screen, screen, Primitive::Triangle(tri));

    scene.build_accelerator();

    // Create film
    let film = Film {
        resolution: (512, 512),
    };

    // Create view
    let view_point = Point3D::new(0.9, -0.4, 0.3);
    let view_direction = (Point3D::new(0., WIDTH / 2., DEPTH) - view_point).get_normalized();
    let view = View {
        view_direction,
        view_point,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 220,
        n_shadow_samples: 1,
        max_depth: 4,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new("./tests/scenes/images/laptop.hdr"))
}

#[test]
#[ignore]
fn sponza() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test test_scenes -- sponza --ignored --exact --nocapture

    let mut scene = Scene::default();
    let gray = scene.push_material(Material::Plastic(Plastic {
        colour: Spectrum::gray(0.3),
        specularity: 0.,
        roughness: 0.,
    }));

    scene.add_from_obj("./tests/scenes/sponza.obj".to_string(), gray, gray);

    scene.add_perez_sky(
        calendar::Date {
            month: 6,
            day: 1,
            hour: 12.,
        },
        -33.,
        70.,
        65.,
        200.,
        500.,
    );

    scene.build_accelerator();

    // Create film
    let film = Film {
        resolution: (330, 330),
    };

    // Create view
    let view_point = Point3D::new(0.0, 5., 0.0);
    let view_direction = Vector3D::new(1., 0., 0.).get_normalized();
    let view_up = Vector3D::new(0., 1., 0.);
    let mut view = View {
        view_direction,
        view_point,
        view_up,
        ..View::default()
    };
    view.normalize();
    // view.infer_basis();

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 180,
        n_shadow_samples: 1,
        max_depth: 2,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new("./tests/scenes/images/sponza.hdr"))
}

#[test]
#[ignore]

fn scene_0() -> Result<(), String> {
    // cargo test --package rendering --test test_scenes -- --ignored scene_0 --exact --nocapture

    let mut scene = Scene::from_radiance("./tests/scenes/scene0.rad".to_string())?;

    scene.build_accelerator();

    // Create camera
    let film = Film {
        resolution: (512, 512),
    };

    let view_point = Point3D::new(2.25, 0.375, 1.);
    let view_direction = Vector3D::new(-0.25, 0.125, -0.125).get_normalized();

    // Create view
    let view = View {
        view_point,
        view_direction,
        field_of_view: 45.,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 20,
        n_shadow_samples: 1,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new("./tests/scenes/images/scene0.hdr"))
}

#[test]
#[ignore]
fn cornell_glass() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test test_scenes -- --ignored cornell_glass --exact --nocapture

    let mut scene = Scene::from_radiance("./tests/scenes/cornell_glass.rad".to_string())?;

    scene.build_accelerator();

    let width = 367;
    let height = (512 / 367) * width;

    // Create camera
    let film = Film {
        // resolution: (320, 240),
        resolution: (width, height),
        // resolution: (1024, 768),
        // resolution: (512, 512),
    };

    // Create view
    let view = View {
        view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
        // view_point: Point3D::new(2., 1., 1.),
        view_point: Point3D::new(3., -5., 2.25),
        field_of_view: 50.,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 300,
        n_shadow_samples: 1,
        max_depth: 8,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new(
        "./tests/scenes/images/cornell_glass.hdr",
    ))
}

#[test]
#[ignore]
fn cornell_solid() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test test_scenes -- --ignored cornell_solid --exact --nocapture

    let mut scene = Scene::from_radiance("./tests/scenes/cornell_solid.rad".to_string())?;

    scene.build_accelerator();

    // Create camera
    let film = Film {
        // resolution: (320, 240),
        resolution: (512, 367),
        // resolution: (1024, 768),
        // resolution: (512, 512),
    };

    // Create view
    let view = View {
        view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
        // view_point: Point3D::new(2., 1., 1.),
        view_point: Point3D::new(3., -5., 2.25),
        field_of_view: 50.,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 900,
        n_shadow_samples: 1,
        max_depth: 100,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new(
        "./tests/scenes/images/cornell_solid.hdr",
    ))
}

#[test]
#[ignore]
fn room() -> Result<(), String> {
    // 11 seconds
    // cargo test --features parallel --release --package rendering --test test_scenes -- room --exact --nocapture --ignored
    // oconv ../room.rad ../white_sky.rad > room.oct ;time rpict -x 512 -y 512 -vv 60 -vh 60 -ab 3 -ad 220 -aa 0 -vp 2 1 1 -vd 0 1 0 ./room.oct > rad_room.hdr

    let mut scene = Scene::from_radiance("./tests/scenes/room.rad".to_string())?;
    // scene.add_perez_sky(
    //     calendar::Date {
    //         month: 6,
    //         day: 1,
    //         hour: 12.,
    //     },
    //     -33.,
    //     70.,
    //     65.,
    //     200.,
    //     500.,
    // );

    scene.build_accelerator();

    // Create film
    let film = Film {
        resolution: (512, 512),
    };

    // Create view
    let view = View {
        view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
        view_point: Point3D::new(2., 1., 1.),
        ..View::default()
    };
    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 320,
        n_shadow_samples: 1,
        max_depth: 3,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new("./tests/scenes/images/room.hdr"))
}

#[test]
#[ignore]
fn dining() -> Result<(), String> {
    // cargo test --features parallel  --release --package rendering --test test_scenes -- dining --exact --nocapture --ignored

    let mut scene = Scene::default();
    let gray = scene.push_material(Material::Plastic(Plastic {
        colour: Spectrum::gray(0.3),
        specularity: 0.,
        roughness: 0.,
    }));

    scene.add_from_obj("./tests/scenes//casa2.obj".to_string(), gray, gray);

    let glow = scene.push_material(Material::Light(Light(Spectrum([1., 1., 1.]) * 200.)));

    let s = Sphere3D::new(0.1, Point3D::new(-2., 3.3, -1.));
    scene.push_object(glow, glow, Primitive::Sphere(s));
    let s = Sphere3D::new(0.1, Point3D::new(-2., 3.3, -1.));
    scene.push_object(glow, glow, Primitive::Sphere(s));
    let s = Sphere3D::new(0.1, Point3D::new(-0., 3.3, 1.));
    scene.push_object(glow, glow, Primitive::Sphere(s));
    let s = Sphere3D::new(0.1, Point3D::new(-0., 3.3, 1.));
    scene.push_object(glow, glow, Primitive::Sphere(s));

    scene.build_accelerator();

    // Create film
    let film = Film {
        resolution: (250, 250),
    };

    // Create view
    let view_point = Point3D::new(-4.0, 1.3, 0.);
    let view_direction = Vector3D::new(1., -0.12, 0.).get_normalized();
    let view_up = Vector3D::new(0., 1., 0.);
    let view = View {
        view_direction,
        view_point,
        view_up,
        field_of_view: 48.,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 20,
        n_shadow_samples: 1,
        max_depth: 1,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new("./tests/scenes//images/dining.hdr"))
}
