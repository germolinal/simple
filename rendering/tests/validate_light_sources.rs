//! Direct lighting from light sources in the scene (polygons and spheres),
//! as calculated by the ray tracer, compared with analytic solutions
use geometry::{Point3D, Ray3D, Vector3D};
use rendering::rand::RngExt;
use rendering::{Float, PI, RayTracer, Scene};
use validate::{ScatterValidator, ValidFunc, Validator, valid};

const RHO: Float = 0.5; // reflectance of the floor
const L: Float = 100.; // radiance of the lights

fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<ScatterValidator<Float>> {
    Box::new(ScatterValidator {
        units: Some("W/m2/sr"),
        expected_legend: Some("Analytic"),
        found_legend: Some("SIMPLE"),
        allowed_r2: Some(0.999),
        allowed_intersect_delta: Some(0.5),
        allowed_slope_delta: Some(0.02),
        expected,
        found,
        ..validate::ScatterValidator::default()
    })
}

/// A scene from Radiance's syntax
fn scene(src: &str) -> Scene {
    let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
    scene.build_accelerator();
    scene
}

/// The radiance seen along `ray` (direct light and one bounce, one shadow
/// sample; average of many estimates)
fn trace(scene: &Scene, ray: Ray3D) -> Float {
    let tracer = RayTracer {
        max_depth: 1,
        n_shadow_samples: 1,
        n_ambient_samples: 1,
    };
    let mut rng = rendering::rand::get_rng();
    let mut aux = [0; 32];
    let n = 40_000;
    let mut sum = 0.;
    for _ in 0..n {
        sum += tracer.trace_ray(&mut rng, scene, ray, &mut aux).radiance();
    }
    sum / n as Float
}

/// Looking down from `origin`
fn down(origin: Point3D) -> Ray3D {
    Ray3D {
        origin,
        direction: Vector3D::new(0., 0., -1.),
    }
}

/// Configuration factor from a horizontal differential area to a parallel
/// rectangle of `2a × 2b` centred `h` above it
fn configuration_factor(a: Float, b: Float, h: Float) -> Float {
    let (x, y) = (a / h, b / h);
    let quarter = (x / (1. + x * x).sqrt() * (y / (1. + x * x).sqrt()).atan()
        + y / (1. + y * y).sqrt() * (x / (1. + y * y).sqrt()).atan())
        / (2. * PI);
    4. * quarter
}

const FLOOR: &str = "void plastic floor 0 0 5 0.5 0.5 0.5 0 0
void light lamp 0 0 3 100 100 100
floor polygon f 0 0 12 -50 -50 0 50 -50 0 50 50 0 -50 50 0
";

/// The radiance of a diffuse floor below square lights of side `2s` at height
/// `h`: ρ L F, where F is the configuration factor
fn square_lights() -> (Vec<Float>, Vec<Float>) {
    let (mut expected, mut found) = (vec![], vec![]);
    for s in [0.25, 0.5, 1., 2.] {
        for h in [0.5, 1., 2., 4.] {
            let scene = scene(&format!(
                "{FLOOR}lamp polygon l 0 0 12 -{s} -{s} {h} -{s} {s} {h} {s} {s} {h} {s} -{s} {h}\n"
            ));
            expected.push(RHO * L * configuration_factor(s, s, h));
            found.push(trace(&scene, down(Point3D::new(0., 0., h / 2.))));
        }
    }
    (expected, found)
}

/// The radiance of a diffuse floor below sphere lights of radius `r`,
/// centred at height `h`: ρ L (r/h)²
fn sphere_lights() -> (Vec<Float>, Vec<Float>) {
    let (mut expected, mut found) = (vec![], vec![]);
    for r in [0.05, 0.2, 0.5, 1.] as [Float; 4] {
        for h in [1.2, 2., 4.] {
            let scene = scene(&format!("{FLOOR}lamp sphere s 0 0 4 0 0 {h} {r}\n"));
            expected.push(RHO * L * (r / h).powi(2));
            found.push(trace(&scene, down(Point3D::new(0., 0., (h - r) / 2.))));
        }
    }
    (expected, found)
}

/// A metal floor reflecting a square light of side `2s` at 2 m, seen from
/// where its mirror direction points to the light's centre. The analytic
/// value is ∫ f(wo, wi) L cos θi dω over the light, integrated over its area.
fn glossy_reflections() -> (Vec<Float>, Vec<Float>) {
    let (mut expected, mut found) = (vec![], vec![]);
    let view_origin = Point3D::new(-3., 0., 2.);
    let view = Ray3D {
        origin: view_origin,
        direction: (Point3D::new(-1.5, 0., 0.) - view_origin).get_normalized(),
    };
    for (spec, rough) in [(0.95, 0.05), (0.9, 0.1), (0.95, 0.2)] {
        for s in [0.25, 0.5, 1.] {
            let scene = scene(&format!(
                "void metal floor 0 0 5 0.5 0.5 0.5 {spec} {rough}
void light lamp 0 0 3 {L} {L} {L}
floor polygon f 0 0 12 -50 -50 0 50 -50 0 50 50 0 -50 50 0
lamp polygon l 0 0 12 -{s} -{s} 2 -{s} {s} 2 {s} {s} 2 {s} -{s} 2
"
            ));
            // Direct integration over the light
            let mut aux = [0; 32];
            let (index, interaction) = scene.cast_ray(view, &mut aux).unwrap();
            let material = &scene.materials[scene.front_material_indexes[index]];
            let (p, normal, e1, e2) = interaction.get_triad();
            let mut rng = rendering::rand::get_rng();
            let n = 1000;
            let d_area = (2. * s / n as Float).powi(2);
            let mut sum = 0.;
            for i in 0..n {
                for j in 0..n {
                    let (u, v): (Float, Float) = rng.random();
                    let x = -s + 2. * s * (i as Float + u) / n as Float;
                    let y = -s + 2. * s * (j as Float + v) / n as Float;
                    let to_light = Point3D::new(x, y, 2.) - p;
                    let wi = to_light.get_normalized();
                    let f = material.eval_bsdf(normal, e1, e2, interaction.wo, wi, 1.0);
                    sum += f.radiance() * L * (normal * wi) * wi.z / to_light.length_squared()
                        * d_area;
                }
            }
            expected.push(sum);
            found.push(trace(&scene, view));
        }
    }
    (expected, found)
}

#[test]
fn validate_light_sources() -> Result<(), String> {
    // cargo test --release --package rendering --test validate_light_sources -- validate_light_sources --exact --nocapture
    let mut validator = Validator::new(
        "Light sources in the scene: SIMPLE vs analytic solutions",
        "../docs/validation/light_sources.html",
    );

    /// The radiance of a diffuse floor (reflectance 0.5) below square lights
    /// (radiance 100) of sides 0.5, 1, 2 and 4 m, at 0.5, 1, 2 and 4 m above
    /// it: ρ L F, where F is the configuration factor between a point and a
    /// parallel rectangle. One shadow sample.
    #[valid("Square lights above a diffuse floor")]
    fn squares() -> ValidFunc {
        let (expected, found) = square_lights();
        get_validator(expected, found)
    }

    /// The radiance of a diffuse floor (reflectance 0.5) below sphere lights
    /// (radiance 100) of radii 0.05, 0.2, 0.5 and 1 m, centred 1.2, 2 and
    /// 4 m above it: ρ L (r/h)². Spheres are sampled on the sphere, but rays
    /// hit the triangles meshing it. One shadow sample.
    #[valid("Sphere lights above a diffuse floor")]
    fn spheres() -> ValidFunc {
        let (expected, found) = sphere_lights();
        get_validator(expected, found)
    }

    /// A metal floor (specularity and roughness: 0.95 and 0.05, 0.9 and 0.1,
    /// 0.95 and 0.2) reflecting square lights of sides 0.5, 1 and 2 m, 2 m
    /// above it, seen at about 37° from the normal where the mirror direction
    /// points to the light's centre. The reference integrates the BSDF over
    /// the light's area (a million points). This combines light samples and
    /// BSDF samples (multiple importance sampling). One shadow sample.
    #[valid("Glossy reflections of lights")]
    fn glossy() -> ValidFunc {
        let (expected, found) = glossy_reflections();
        get_validator(expected, found)
    }

    validator.push(squares());
    validator.push(spheres());
    validator.push(glossy());

    validator.validate()
}
