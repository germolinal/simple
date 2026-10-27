//! IR view factors (the fractions of sky and ground a surface sees),
//! compared with analytic solutions
use geometry::{Loop3D, Point3D, Polygon3D, Vector3D};
use light::solar_surface::SolarSurface;
use light::{Float, IR_VIEW_FACTOR_RAYS};
use rendering::Scene;
use validate::{valid, ScatterValidator, ValidFunc, Validator};

/// The expected and found sky view factors, and the expected and found
/// ground view factors
type ExpectedAndFound = (Vec<Float>, Vec<Float>, Vec<Float>, Vec<Float>);

fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<ScatterValidator<Float>> {
    Box::new(ScatterValidator {
        units: Some("-"),
        expected_legend: Some("Analytic"),
        found_legend: Some("SIMPLE"),
        allowed_r2: Some(0.999),
        allowed_intersect_delta: Some(0.01),
        allowed_slope_delta: Some(0.02),
        expected,
        found,
        ..validate::ScatterValidator::default()
    })
}

/// A surface from its vertices
fn surface(vertices: &[(Float, Float, Float)]) -> Result<SolarSurface, String> {
    let mut the_loop = Loop3D::new();
    for (x, y, z) in vertices {
        the_loop.push(Point3D::new(*x, *y, *z))?;
    }
    the_loop.close()?;
    SolarSurface::new(1, &Polygon3D::new(the_loop)?, true, true)
}

/// A scene made of `polygons` (Radiance syntax, material `grey`)
fn scene(polygons: &str) -> Scene {
    let src = format!("void plastic grey 0 0 5 0.5 0.5 0.5 0 0\n{polygons}");
    let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
    scene.build_accelerator();
    scene
}

/// The (sky, ground) view factors of the side of `surface` facing
/// `direction` (the sky includes the "air" part, which is split from it
/// afterwards)
fn view_factors(
    surface: &SolarSurface,
    scene: &Scene,
    direction: Vector3D,
) -> Result<(Float, Float), String> {
    let front = surface.normal * direction > 0.;
    let v = surface.calc_view_factors(scene, front, IR_VIEW_FACTOR_RAYS)?;
    Ok((v.sky + v.air, v.ground))
}

/// The average over `[z0, z1]` of a function of the height
fn average_over_height(z0: Float, z1: Float, f: impl Fn(Float) -> Float) -> Float {
    let n = 2000;
    (0..n)
        .map(|i| f(z0 + (z1 - z0) * (i as Float + 0.5) / n as Float))
        .sum::<Float>()
        / n as Float
}

/// The fraction of the view of a point (in 2D, i.e., for infinitely long
/// geometries) between two angles from its normal: (sin φ2 − sin φ1) / 2
fn view_between(phi1: Float, phi2: Float) -> Float {
    (phi2.sin() - phi1.sin()) / 2.
}

/// A 1 × 1 m surface tilted `degrees` from facing up, with nothing around.
/// The view factors are (1 + cos β)/2 to the sky and (1 − cos β)/2 to the
/// ground.
fn tilted() -> Result<ExpectedAndFound, String> {
    let (mut exp_sky, mut found_sky, mut exp_ground, mut found_ground) =
        (vec![], vec![], vec![], vec![]);
    let empty = scene("");
    for degrees in (0..=180).step_by(15) {
        let beta = (degrees as Float).to_radians();
        // Normal: (0, -sin β, cos β)
        let s = surface(&[
            (0., 0., 0.),
            (1., 0., 0.),
            (1., beta.cos(), beta.sin()),
            (0., beta.cos(), beta.sin()),
        ])?;
        let (sky, ground) = view_factors(&s, &empty, Vector3D::new(0., -beta.sin(), beta.cos()))?;
        exp_sky.push((1. + beta.cos()) / 2.);
        exp_ground.push((1. - beta.cos()) / 2.);
        found_sky.push(sky);
        found_ground.push(ground);
    }
    Ok((exp_sky, found_sky, exp_ground, found_ground))
}

/// A 3 m high facade facing a parallel building of height `h` across a
/// street of width `w` (both 400 m long, i.e., 2D). From a point at height
/// `z`, the building spans the angles from atan(−z/w) to atan((h − z)/w);
/// the sky is above it, and the ground below.
fn canyon() -> Result<ExpectedAndFound, String> {
    const FACADE: Float = 3.;
    let (mut exp_sky, mut found_sky, mut exp_ground, mut found_ground) =
        (vec![], vec![], vec![], vec![]);
    let facade = surface(&[(0., 0., 0.), (0., 1., 0.), (0., 1., FACADE), (0., 0., FACADE)])?;
    let half_pi = Float::from(std::f32::consts::FRAC_PI_2);
    for w in [1., 2., 4., 8., 16.] {
        for h in [3., 9., 30.] {
            let other = scene(&format!(
                "grey polygon b 0 0 12 {w} -200 0 {w} 200 0 {w} 200 {h} {w} -200 {h}\n"
            ));
            let (sky, ground) = view_factors(&facade, &other, Vector3D::new(1., 0., 0.))?;
            exp_sky.push(average_over_height(0., FACADE, |z| {
                view_between(((h - z) / w).atan(), half_pi)
            }));
            exp_ground.push(average_over_height(0., FACADE, |z| {
                view_between(-half_pi, (-z / w).atan())
            }));
            found_sky.push(sky);
            found_ground.push(ground);
        }
    }
    Ok((exp_sky, found_sky, exp_ground, found_ground))
}

/// A 1.5 m high window (from 0.5 to 2 m) under an overhang of depth `d`,
/// 0.1 m above it (400 m long, i.e., 2D). From a point at height `z`, the sky
/// is seen between the horizon and the overhang's edge, at atan((2.1 −
/// z)/d); the ground is half of the view.
fn overhang() -> Result<ExpectedAndFound, String> {
    let (mut exp_sky, mut found_sky, mut exp_ground, mut found_ground) =
        (vec![], vec![], vec![], vec![]);
    let window = surface(&[(0., 0., 0.5), (0., 1., 0.5), (0., 1., 2.), (0., 0., 2.)])?;
    for d in [0.25, 0.5, 1., 2., 4.] {
        let other = scene(&format!(
            "grey polygon o 0 0 12 0 -200 2.1 {d} -200 2.1 {d} 200 2.1 0 200 2.1\n"
        ));
        let (sky, ground) = view_factors(&window, &other, Vector3D::new(1., 0., 0.))?;
        exp_sky.push(average_over_height(0.5, 2., |z| {
            view_between(0., ((2.1 - z) / d).atan())
        }));
        exp_ground.push(0.5);
        found_sky.push(sky);
        found_ground.push(ground);
    }
    Ok((exp_sky, found_sky, exp_ground, found_ground))
}

#[test]
fn validate_view_factors() -> Result<(), String> {
    // cargo test --release --package light --test validate_view_factors -- validate_view_factors --exact --nocapture
    let mut validator = Validator::new(
        "IR view factors: SIMPLE vs analytic solutions",
        "../docs/validation/view_factors.html",
    );

    /// Unobstructed surfaces tilted from 0° (facing up) to 180° (facing
    /// down), every 15°. Sky view factor: (1 + cos β)/2.
    #[valid("Tilted surfaces - Sky")]
    fn tilted_sky() -> Result<ValidFunc, String> {
        let (expected, found, _, _) = tilted()?;
        Ok(get_validator(expected, found))
    }

    /// Unobstructed surfaces tilted from 0° (facing up) to 180° (facing
    /// down), every 15°. Ground view factor: (1 − cos β)/2.
    #[valid("Tilted surfaces - Ground")]
    fn tilted_ground() -> Result<ValidFunc, String> {
        let (_, _, expected, found) = tilted()?;
        Ok(get_validator(expected, found))
    }

    /// A 3 m high facade facing a parallel building (3, 9 and 30 m high)
    /// across a street (1, 2, 4, 8 and 16 m wide), long enough to be 2D. The
    /// analytic sky view factor is the average, over the facade's height, of
    /// the fraction of the view above the other building.
    #[valid("Street canyon - Sky")]
    fn canyon_sky() -> Result<ValidFunc, String> {
        let (expected, found, _, _) = canyon()?;
        Ok(get_validator(expected, found))
    }

    /// Same as the previous one, for the ground: the fraction of the view
    /// below the bottom of the other building.
    #[valid("Street canyon - Ground")]
    fn canyon_ground() -> Result<ValidFunc, String> {
        let (_, _, expected, found) = canyon()?;
        Ok(get_validator(expected, found))
    }

    /// A 1.5 m high window under an overhang 0.1 m above it (0.25, 0.5, 1, 2
    /// and 4 m deep), long enough to be 2D. The analytic sky view factor is
    /// the average, over the window's height, of the fraction of the view
    /// between the horizon and the edge of the overhang.
    #[valid("Window under an overhang - Sky")]
    fn overhang_sky() -> Result<ValidFunc, String> {
        let (expected, found, _, _) = overhang()?;
        Ok(get_validator(expected, found))
    }

    validator.push(tilted_sky()?);
    validator.push(tilted_ground()?);
    validator.push(canyon_sky()?);
    validator.push(canyon_ground()?);
    validator.push(overhang_sky()?);

    validator.validate()
}
