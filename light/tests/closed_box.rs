use geometry::{Point3D, Ray3D, Vector3D};
use model::Model;
use rendering::rand::*;
use rendering::{Float, Scene, Wavelengths};

/// Rays cast from inside a closed room must not reach the sky.
///
/// The box used for validation once had a wall whose polygon left a
/// triangular hole above its window, which let ~2% of the rays out (and
/// tripled the solar radiation reaching the inside of the room).
#[test]
fn closed_box_does_not_leak() {
    let (model, _) = Model::from_file("../tests/box/box.spl").unwrap();
    let mut scene = Scene::from_simple_model(&model, Wavelengths::Solar).unwrap();
    scene.build_accelerator();

    // The box is 15.24 x 15.24 x 4.572 m
    let (l, h) = (15.24, 4.572);
    let mut rng = get_rng();
    let mut aux = [0usize; 32];
    let n = 200_000;
    let mut escaped = 0;
    for _ in 0..n {
        let x: Float = rng.random();
        let y: Float = rng.random();
        let z: Float = rng.random();
        let origin = Point3D::new(x * l, y * l, z * h);

        let z: Float = 2. * rng.random::<Float>() - 1.;
        let phi: Float = 2. * rendering::PI * rng.random::<Float>();
        let r = (1. - z * z).sqrt();
        let direction = Vector3D::new(r * phi.cos(), r * phi.sin(), z);

        if scene.cast_ray(Ray3D { origin, direction }, &mut aux).is_none() {
            escaped += 1;
        }
    }
    // (a few rays slip through the edges between triangles)
    assert!(
        (escaped as Float) < 1e-3 * n as Float,
        "{escaped} out of {n} rays escaped the box"
    );
}
