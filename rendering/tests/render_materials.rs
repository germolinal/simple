use geometry::Point3D;
use geometry::{Sphere3D, Triangle3D};
use rendering::camera::{Film, Pinhole, View};
use rendering::material::Material;
use rendering::material::*;
use rendering::primitive::Primitive;
use rendering::Float;
use rendering::{RayTracer, Scene, Spectrum};

fn render_ball(mat: Material, filename: &str) -> Result<(), String> {
    let mut scene = Scene::new();

    // Add room
    const HALF_ROOM_SIZE: Float = 2.5;

    let gray = Material::Plastic(Plastic {
        colour: Spectrum::gray(0.2),
        specularity: 0.0,
        roughness: 0.0,
    });
    let gray = scene.push_material(gray);

    let red = Material::Plastic(Plastic {
        colour: Spectrum([0.9, 0.36, 0.36]),
        specularity: 0.0,
        roughness: 0.0,
    });
    let red = scene.push_material(red);

    let blue = Material::Plastic(Plastic {
        colour: Spectrum([0.36, 0.36, 0.9]),
        specularity: 0.0,
        roughness: 0.0,
    });
    let blue = scene.push_material(blue);

    // Floor
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));

    // ceiling
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));

    // Back
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(gray, gray, Primitive::Triangle(tri));

    // Left
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(red, red, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(-HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(-HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(red, red, Primitive::Triangle(tri));

    // Right
    let tri = Triangle3D::new(
        Point3D::new(HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(blue, blue, Primitive::Triangle(tri));
    let tri = Triangle3D::new(
        Point3D::new(HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 0.0),
        Point3D::new(HALF_ROOM_SIZE, HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
        Point3D::new(HALF_ROOM_SIZE, -HALF_ROOM_SIZE, 2. * HALF_ROOM_SIZE),
    )?;
    scene.push_object(blue, blue, Primitive::Triangle(tri));

    // Add ball
    let centre = Point3D::new(0., 0., 1.5);
    let mat = scene.push_material(mat);
    let s = Sphere3D::new(0.85, centre);
    scene.push_object(mat, mat, Primitive::Sphere(s));

    // Add light
    let glow = scene.push_material(Material::Light(Light(
        //145, 7, 205
        Spectrum::ONE * 10000.,
    )));

    let s = Sphere3D::new(0.1, Point3D::new(4., -15., 5.));
    scene.push_object(glow, glow, Primitive::Sphere(s));

    scene.build_accelerator();

    // Create film
    let film = Film {
        resolution: (200, 200),
    };

    // Create view
    let view_point = Point3D::new(0., -1.1 * HALF_ROOM_SIZE, 4.1);
    let view_direction = (centre - view_point).get_normalized();
    let view = View {
        view_direction,
        view_point,
        ..View::default()
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: 60,
        n_shadow_samples: 1,
        max_depth: 1,
        ..RayTracer::default()
    };

    let buffer = integrator.render(&scene, &camera);
    buffer.save_hdre(std::path::Path::new(filename))
}

#[test]
#[ignore]
fn test_render_specular_plastic() -> Result<(), String> {
    // cargo test --package rendering --test render_materials -- test_render_specular_plastic --exact --nocapture --ignored

    let plastic = Material::Plastic(Plastic {
        colour: Spectrum([0.9, 0.5, 0.5]),
        specularity: 0.09,
        roughness: 0.05,
    });

    render_ball(plastic, "./tests/scenes/images/specular_plastic.hdr")
}

#[test]
#[ignore]
fn test_render_specular_metal() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test render_materials -- test_render_specular_metal --ignored --exact --nocapture

    let metal = Material::Metal(Metal {
        colour: Spectrum([0.0, 0.5, 0.5]),
        specularity: 0.28,
        roughness: 0.05,
    });

    render_ball(metal, "./tests/scenes/images/specular_metal.hdr")
}

#[test]
#[ignore]
fn test_render_glass() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test render_materials -- test_render_glass --ignored --exact --nocapture
    let metal = Material::Glass(Glass {
        colour: Spectrum([0.9, 0.9, 0.9]),
        refraction_index: 1.52,
    });

    render_ball(metal, "./tests/scenes/images/glass.hdr")
}

#[test]
#[ignore]
fn test_render_mirror() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test render_materials -- test_render_mirror --ignored --exact --nocapture

    let plastic = Material::Mirror(Mirror(Spectrum::gray(0.5)));

    render_ball(plastic, "./tests/scenes/images/mirror.hdr")
}

#[test]
#[ignore]
fn test_render_dielectric() -> Result<(), String> {
    // cargo test --features parallel --release --package rendering --test render_materials -- test_render_dielectric --exact --nocapture --ignored

    let dielectric = Material::Dielectric(Dielectric {
        colour: Spectrum::gray(0.95),
        refraction_index: 1.6,
    });

    render_ball(dielectric, "./tests/scenes/images/dielectric.hdr")
}
