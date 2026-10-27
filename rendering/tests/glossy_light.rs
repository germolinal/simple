//! A glossy surface reflecting a light: the ray tracer, and each of its
//! sampling strategies, against a direct integration of the BSDF over the
//! light's area
use geometry::{Point3D, Ray3D};
use rendering::interaction::Interaction;
use rendering::material::Material;
use rendering::rand::RngExt;
use rendering::{Float, RayTracer, Scene};

const L: Float = 100.; // light radiance

/// (specularity, roughness) of the metals tested
const METALS: [(Float, Float); 3] = [(0.95, 0.05), (0.9, 0.1), (0.95, 0.2)];

/// A metal floor (`spec`, `rough`), and a 1 × 1 m light 2 m above the origin
fn scene(spec: Float, rough: Float) -> Scene {
    let src = format!(
        "void metal floor 0 0 5 0.5 0.5 0.5 {spec} {rough}
void light lamp 0 0 3 {L} {L} {L}
floor polygon f 0 0 12 -50 -50 0 50 -50 0 50 50 0 -50 50 0
lamp polygon l 0 0 12 -0.5 -0.5 2 -0.5 0.5 2 0.5 0.5 2 0.5 -0.5 2
"
    );
    let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
    scene.build_accelerator();
    scene
}

/// A ray hitting the floor at (-1.5, 0, 0), whose mirror direction points to
/// the light's centre
fn view_ray() -> Ray3D {
    let origin = Point3D::new(-3., 0., 2.);
    Ray3D {
        origin,
        direction: (Point3D::new(-1.5, 0., 0.) - origin).get_normalized(),
    }
}

/// Where the view ray hits the floor, and the floor's material
fn floor_hit(scene: &Scene) -> (Interaction, &Material) {
    let mut aux = [0; 32];
    let (index, interaction) = scene.cast_ray(view_ray(), &mut aux).unwrap();
    (
        interaction,
        &scene.materials[scene.front_material_indexes[index]],
    )
}

/// Whether a ray from `p` in direction `wi` hits the light
fn hits_light(p: Point3D, wi: geometry::Vector3D) -> bool {
    if wi.z <= 0. {
        return false;
    }
    let q = p + wi * ((2. - p.z) / wi.z);
    q.x.abs() < 0.5 && q.y.abs() < 0.5
}

/// ∫ f(wo, wi) L cos θi dω over the light, as a sum over its area
fn reference(scene: &Scene) -> Float {
    let (interaction, material) = floor_hit(scene);
    let (p, normal, e1, e2) = interaction.get_triad();
    let n = 1000;
    let d_area = 1. / (n * n) as Float;
    let mut rng = rendering::rand::get_rng();
    let mut sum = 0.;
    for i in 0..n {
        for j in 0..n {
            let (u, v): (Float, Float) = rng.random();
            let x = -0.5 + (i as Float + u) / n as Float;
            let y = -0.5 + (j as Float + v) / n as Float;
            let to_light = Point3D::new(x, y, 2.) - p;
            let d2 = to_light.length_squared();
            let wi = to_light.get_normalized();
            let cos_light = wi.z; // the light faces down
            let f = material.eval_bsdf(normal, e1, e2, interaction.wo, wi, 1.0);
            sum += f.radiance() * L * (normal * wi) * cos_light / d2 * d_area;
        }
    }
    sum
}

/// The ray tracer's estimate (direct light plus one bounce)
fn traced(scene: &Scene, n_shadow_samples: usize) -> Float {
    let tracer = RayTracer {
        max_depth: 1,
        n_shadow_samples,
        n_ambient_samples: 1,
    };
    let mut rng = rendering::rand::get_rng();
    let mut aux = [0; 32];
    let n = 40_000;
    let mut sum = 0.;
    for _ in 0..n {
        sum += tracer
            .trace_ray(&mut rng, scene, view_ray(), &mut aux)
            .radiance();
    }
    sum / n as Float
}

/// The ray tracer combines light and BSDF samples with multiple importance
/// sampling. It used to be 10-55% too bright here, because sampled Ward
/// directions were not reflected around the sampled half vector, and the pdf
/// did not describe them; and, with one shadow sample, because shadow rays
/// went to the light's centre.
#[test]
fn glossy_reflection_of_light() {
    for (spec, rough) in METALS {
        let scene = scene(spec, rough);
        let expected = reference(&scene);
        for n_shadow in [1, 4] {
            let found = traced(&scene, n_shadow);
            let error = found / expected - 1.;
            assert!(
                error.abs() < 0.02,
                "metal spec {spec} rough {rough}, {n_shadow} shadow samples: found {found}, expected {expected} ({:+.1}%)",
                100. * error
            );
        }
    }
}

/// Each sampling strategy alone must give the right answer, and the
/// directions sampled for the BSDF must be distributed as its pdf says
#[test]
fn glossy_strategies() {
    for (spec, rough) in METALS {
        let scene = scene(spec, rough);
        let expected = reference(&scene);
        let (interaction, material) = floor_hit(&scene);
        let (p, normal, e1, e2) = interaction.get_triad();
        let mut rng = rendering::rand::get_rng();
        let n = 400_000;

        // BSDF sampling
        let (mut bsdf, mut n_hits) = (0., 0);
        for _ in 0..n {
            let mut it = interaction;
            let mut eta = 1.0;
            let ray = view_ray();
            if let Some(sample) = material.sample_bsdf(ray.direction, &mut it, &mut eta, &mut rng) {
                if hits_light(p, sample.wi) {
                    n_hits += 1;
                    bsdf += sample.spectrum.radiance() * (normal * sample.wi) / sample.pdf * L;
                }
            }
        }
        let bsdf = bsdf / n as Float;

        // Light sampling, over all the triangles of the light
        let mut light_sum = 0.;
        for light in &scene.lights {
            for _ in 0..n {
                let wi = light.primitive.sample_direction(&mut rng, p);
                let shadow = Ray3D {
                    origin: p,
                    direction: wi,
                };
                if let Some(info) = light.primitive.intersect(&shadow) {
                    let pdf = light.primitive.solid_angle_pdf(&info, &shadow);
                    if pdf > 0. {
                        let f = material.eval_bsdf(normal, e1, e2, interaction.wo, wi, 1.0);
                        light_sum += f.radiance() * L * (normal * wi) / pdf;
                    }
                }
            }
        }
        let light = light_sum / n as Float;

        // Share of BSDF samples hitting the light vs. ∫ pdf() over it
        let m = 600;
        let mut int_pdf = 0.;
        for i in 0..m {
            for j in 0..m {
                let x = -0.5 + (i as Float + 0.5) / m as Float;
                let y = -0.5 + (j as Float + 0.5) / m as Float;
                let to = Point3D::new(x, y, 2.) - p;
                let wi = to.get_normalized();
                let pdf = material.pdf(normal, e1, e2, wi, interaction.wo, 1.0);
                int_pdf += pdf * wi.z / to.length_squared() / (m * m) as Float;
            }
        }
        let share = n_hits as Float / n as Float;

        for (name, found) in [("BSDF sampling", bsdf), ("light sampling", light)] {
            let error = found / expected - 1.;
            assert!(
                error.abs() < 0.02,
                "metal spec {spec} rough {rough}, {name}: found {found}, expected {expected} ({:+.1}%)",
                100. * error
            );
        }
        assert!(
            (share - int_pdf).abs() < 0.01,
            "metal spec {spec} rough {rough}: {share} of the BSDF samples hit the light, but the pdf integrates to {int_pdf} over it"
        );
    }
}
