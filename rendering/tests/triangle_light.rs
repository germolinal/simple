//! Direct illumination from a polygon (i.e., triangles) light source,
//! compared with the analytic solution.
use geometry::{Point3D, Ray3D, Vector3D};
use rendering::{Float, RayTracer, Scene, PI};

const RHO: Float = 0.5; // floor reflectance
const L: Float = 100.; // light radiance

/// Configuration factor from a horizontal differential area to a parallel
/// rectangle of `2a × 2b` centred `h` above it
fn configuration_factor(a: Float, b: Float, h: Float) -> Float {
    let (x, y) = (a / h, b / h);
    let quarter = (x / (1. + x * x).sqrt() * (y / (1. + x * x).sqrt()).atan()
        + y / (1. + y * y).sqrt() * (x / (1. + y * y).sqrt()).atan())
        / (2. * PI);
    4. * quarter
}

/// A large diffuse floor, and a square light of side `2 * half_side` at
/// `height` above its centre; everything moved by `offset`
fn scene(offset: (Float, Float, Float), half_side: Float, height: Float) -> Scene {
    let (dx, dy, dz) = offset;
    let p = |x: Float, y: Float, z: Float| format!("{} {} {}", x + dx, y + dy, z + dz);
    let (s, h) = (half_side, height);
    let src = format!(
        "void plastic floor 0 0 5 {RHO} {RHO} {RHO} 0 0
void light lamp 0 0 3 {L} {L} {L}
floor polygon f 0 0 12 {} {} {} {}
lamp polygon l 0 0 12 {} {} {} {}
",
        p(-50., -50., 0.), p(50., -50., 0.), p(50., 50., 0.), p(-50., 50., 0.),
        p(-s, -s, h), p(-s, s, h), p(s, s, h), p(s, -s, h),
    );
    let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
    scene.build_accelerator();
    scene
}

/// The floor's radiance below the light's centre, as calculated by the ray
/// tracer (average of many estimates)
fn floor_radiance(
    offset: (Float, Float, Float),
    n_shadow_samples: usize,
    half_side: Float,
    height: Float,
) -> Float {
    let scene = scene(offset, half_side, height);
    let tracer = RayTracer {
        max_depth: 1,
        n_shadow_samples,
        n_ambient_samples: 1,
    };
    let ray = Ray3D {
        // between the floor and the light
        origin: Point3D::new(offset.0, offset.1, offset.2 + height / 2.),
        direction: Vector3D::new(0., 0., -1.),
    };
    let mut rng = rendering::rand::get_rng();
    let mut aux = [0; 32];
    let n = 4000;
    let mut sum = 0.;
    for _ in 0..n {
        sum += tracer.trace_ray(&mut rng, &scene, ray, &mut aux).radiance();
    }
    sum / n as Float
}

/// Checks the ray tracer's direct illumination from triangle lights against
/// the analytic solution, for a small, far light and a large, close one
/// (e.g., a ceiling panel), with the scene at the origin and far from it.
///
/// This checks two fixed bugs: the light's probability density used
/// `ray.origin` instead of `ray.direction` (which gave no direct light from
/// triangle lights), and samples of lights in the scene were weighted as if
/// BSDF samples that hit them were also added (which underestimated large,
/// close lights).
#[test]
fn triangle_light_matches_analytic() {
    for (half_side, height) in [(0.5, 2.), (2., 0.5)] {
        let expected = RHO * L * configuration_factor(half_side, half_side, height);
        for offset in [(0., 0., 0.), (1000., 1000., 0.)] {
            let found = floor_radiance(offset, 16, half_side, height);
            let error = (found / expected - 1.).abs();
            assert!(
                error < 0.02,
                "light of side {}, at {height}; offset {offset:?}: found {found}, expected {expected} ({:.1}% error)",
                2. * half_side,
                100. * error
            );
        }
    }
}
