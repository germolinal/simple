//! The Ward BSDF must reflect the same amount of light whether it is
//! evaluated (as when sampling lights) or sampled (as when tracing bounces).
use geometry::{Point3D, Ray3D, Triangle3D, Vector3D};
use rendering::material::{Material, Metal, Plastic};
use rendering::primitive::Primitive;
use rendering::rand::RngExt;
use rendering::{Float, Scene, Spectrum, PI};

/// A large horizontal plane made of `material`
fn plane(material: Material) -> Scene {
    let mut scene = Scene::new();
    let m = scene.push_material(material);
    let p = |x: Float, y: Float| Point3D::new(x, y, 0.);
    for t in [
        Triangle3D::new(p(-100., -100.), p(100., -100.), p(100., 100.)).unwrap(),
        Triangle3D::new(p(-100., -100.), p(100., 100.), p(-100., 100.)).unwrap(),
    ] {
        scene.push_object(m, m, Primitive::Triangle(t));
    }
    scene.build_accelerator();
    scene
}

/// Directional-hemispherical reflectance for light arriving at `degrees` from
/// the normal: (by sampling the BSDF, by integrating its evaluation)
fn reflectances(material: Material, degrees: Float) -> (Float, Float) {
    let scene = plane(material);
    let material = &scene.materials[0];
    let mut rng = rendering::rand::get_rng();
    let theta = degrees.to_radians();
    let ray = Ray3D {
        origin: Point3D::new(-2. * theta.sin(), 0., 2. * theta.cos()),
        direction: Vector3D::new(theta.sin(), 0., -theta.cos()),
    };
    let mut aux = [0; 32];
    let (_, interaction) = scene.cast_ray(ray, &mut aux).unwrap();
    let (_, normal, e1, e2) = interaction.get_triad();

    // As when tracing bounces: average of BSDF * cos / pdf
    let n = 200_000;
    let mut sampled = 0.;
    for _ in 0..n {
        let mut it = interaction;
        let mut eta = 1.0;
        if let Some(s) = material.sample_bsdf(ray.direction, &mut it, &mut eta, &mut rng) {
            if s.pdf > 0. {
                sampled += s.spectrum.radiance() * (normal * s.wi).abs() / s.pdf;
            }
        }
    }

    // As when sampling lights: the integral of BSDF * cos over the hemisphere
    // (stratified, so that narrow lobes are covered evenly)
    let side = 633;
    let n_eval = side * side;
    let mut evaluated = 0.;
    for k in 0..n_eval {
        let (j1, j2): (Float, Float) = rng.random();
        let u1 = ((k / side) as Float + j1) / side as Float;
        let u2 = ((k % side) as Float + j2) / side as Float;
        let (z, phi) = (u1, 2. * PI * u2);
        let r = (1. - z * z).max(0.).sqrt();
        let wi = e1 * (r * phi.cos()) + e2 * (r * phi.sin()) + normal * z;
        let f = material.eval_bsdf(normal, e1, e2, interaction.wo, wi, 1.0);
        evaluated += f.radiance() * z * 2. * PI;
    }
    (sampled / n as Float, evaluated / n_eval as Float)
}

/// Evaluating the specular lobe used to give up to 4 times too much light
/// (normalized half vector, and an extra division by cos θo), so highlights
/// of light sources on glossy surfaces were too bright.
#[test]
fn ward_evaluation_matches_sampling() {
    let materials = [
        Material::Metal(Metal { colour: Spectrum::gray(0.5), specularity: 0.95, roughness: 0.05 }),
        Material::Metal(Metal { colour: Spectrum::gray(0.5), specularity: 0.9, roughness: 0.1 }),
        Material::Plastic(Plastic { colour: Spectrum::gray(0.45), specularity: 0.05, roughness: 0.1 }),
    ];
    for material in materials {
        for degrees in [0., 30., 60.] {
            let (sampled, evaluated) = reflectances(material.clone(), degrees);
            assert!(
                // The bug was a factor of 2-4; this leaves room for noise
                (evaluated / sampled - 1.).abs() < 0.1,
                "{material:?} at {degrees}°: sampled {sampled}, evaluated {evaluated}"
            );
        }
    }
}
