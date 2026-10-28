//! Direct illumination from a polygon (i.e., triangles) light source,
//! compared with the analytic solution.
use geometry::{Point3D, Ray3D, Vector3D};
use rendering::{Float, PI, RayTracer, Scene};

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
        p(-50., -50., 0.),
        p(50., -50., 0.),
        p(50., 50., 0.),
        p(-50., 50., 0.),
        p(-s, -s, h),
        p(-s, s, h),
        p(s, s, h),
        p(s, -s, h),
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
    // between the floor and the light
    let origin = Point3D::new(offset.0, offset.1, offset.2 + height / 2.);
    trace_down(&scene, origin, n_shadow_samples)
}

/// The radiance seen looking down from `origin` (average of many estimates
/// of the ray tracer, with one bounce)
fn trace_down(scene: &Scene, origin: Point3D, n_shadow_samples: usize) -> Float {
    let tracer = RayTracer {
        max_depth: 1,
        n_shadow_samples,
        n_ambient_samples: 1,
    };
    let ray = Ray3D {
        origin,
        direction: Vector3D::new(0., 0., -1.),
    };
    let mut rng = rendering::rand::get_rng();
    let mut aux = [0; 32];
    let n = 4000;
    let mut sum = 0.;
    for _ in 0..n {
        sum += tracer.trace_ray(&mut rng, scene, ray, &mut aux).radiance();
    }
    sum / n as Float
}

/// Checks the ray tracer's direct illumination from a small, far triangle
/// light against the analytic solution, with the scene at the origin and far
/// from it.
///
/// This checks a fixed bug: the light's probability density used
/// `ray.origin` instead of `ray.direction`, which gave no direct light from
/// triangle lights.
#[test]
fn triangle_light_matches_analytic() {
    check_light(0.5, 2.);
}

/// Like `triangle_light_matches_analytic()`, for a large, close light (e.g.,
/// a ceiling panel). BSDF samples that hit lights in the scene used to be
/// ignored (while light samples were weighted by multiple importance
/// sampling), so such lights were underestimated (−23% here).
#[test]
fn large_triangle_light_matches_analytic() {
    check_light(2., 0.5);
}

/// The floor's radiance below the centre of a square light of side
/// `2 * half_side` at `height` must match the analytic solution (16 shadow
/// samples), with the scene at the origin and far from it.
fn check_light(half_side: Float, height: Float) {
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

/// The floor's radiance below sphere lights, small and far or large and
/// close, must match the analytic solution: ρ L (r/h)², for a sphere of
/// radius `r` centred at height `h` (≥ `r`). Sphere lights are sampled on the
/// sphere, while rays hit the triangles meshing it. With 2 shadow samples,
/// BSDF samples that hit the large sphere carry ~17% of its light.
#[test]
fn sphere_light_matches_analytic() {
    for (radius, height) in [(0.1, 2.), (1., 1.2)] as [(Float, Float); 2] {
        let src = format!(
            "void plastic floor 0 0 5 {RHO} {RHO} {RHO} 0 0
void light lamp 0 0 3 {L} {L} {L}
floor polygon f 0 0 12 -50 -50 0 50 -50 0 50 50 0 -50 50 0
lamp sphere s 0 0 4 0 0 {height} {radius}
"
        );
        let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
        scene.build_accelerator();
        let expected = RHO * L * (radius / height).powi(2);
        let origin = Point3D::new(0., 0., (height - radius) / 2.);
        let found = trace_down(&scene, origin, 2);
        let error = (found / expected - 1.).abs();
        assert!(
            error < 0.02,
            "sphere of radius {radius} at {height}: found {found}, expected {expected} ({:.1}% error)",
            100. * error
        );
    }
}
