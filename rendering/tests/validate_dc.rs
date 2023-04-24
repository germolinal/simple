use geometry::{Point3D, Ray3D, Vector3D};
use rendering::{colour_matrix, ColourMatrix, DCFactory, Float, Scene};
use validate::{valid, ScatterValidator, Validate, Validator};

fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<ScatterValidator<Float>> {
    Box::new(ScatterValidator {
        units: Some("cd/m2"),
        expected,
        found,
        expected_legend: Some("Radiance"),
        found_legend: Some("SIMPLE"),
        ..validate::ScatterValidator::default()
    })
}

fn flatten_matrix(m: &ColourMatrix) -> Vec<Float> {
    let (nrows, ncols) = m.size();
    let mut v: Vec<Float> = Vec::with_capacity(nrows * ncols);
    for row in 0..nrows {
        for col in 0..ncols {
            let value = m.get(row, col).unwrap().radiance();
            v.push(value);
        }
    }
    v
}

fn load_rays(filename: &str) -> Vec<Ray3D> {
    let s = std::fs::read_to_string(filename).unwrap();
    s.lines()
        .map(|line| {
            let a: Vec<Float> = line
                .split_ascii_whitespace()
                .into_iter()
                .map(|x| x.parse::<Float>().unwrap())
                .collect();

            Ray3D {
                origin: Point3D::new(a[0], a[1], a[2]),
                direction: Vector3D::new(a[3], a[4], a[5]).get_normalized(),
            }
        })
        .collect()
}

fn load_expected_results(filename: String) -> Vec<Float> {
    let path = std::path::Path::new(&filename);
    let m = colour_matrix::read_colour_matrix(path).unwrap();

    flatten_matrix(&m)
}

fn get_simple_results(dir: &str, max_depth: usize, with_glass: bool) -> (Vec<Float>, Vec<Float>) {
    let mut scene = if with_glass {
        Scene::from_radiance(format!("./tests/dc/{dir}/scene.rad"))
    } else {
        Scene::from_radiance(format!("./tests/dc/{dir}/room.rad"))
    };
    scene.build_accelerator();

    let integrator = DCFactory {
        n_ambient_samples: 51020,
        max_depth,
        limit_weight: 1e-9,
        ..DCFactory::default()
    };

    let rays = load_rays("./tests/points.pts");
    let found_matrix = integrator.calc_dc(&rays, &scene);
    let found = flatten_matrix(&found_matrix);

    let expected = if max_depth == 0 {
        if with_glass {
            load_expected_results(format!("./tests/dc/{dir}/direct_results_glass.txt"))
        } else {
            load_expected_results(format!("./tests/dc/{dir}/direct_results_no_glass.txt"))
        }
    } else {
        if with_glass {
            load_expected_results(format!("./tests/dc/{dir}/global_results_glass.txt"))
        } else {
            load_expected_results(format!("./tests/dc/{dir}/global_results_no_glass.txt"))
        }
    };

    // Filter infinites
    let temp: Vec<(Float, Float)> = expected
        .iter()
        .zip(found.iter())
        .filter_map(|(e, f)| {
            if f.is_infinite() {
                None
            } else {
                Some((*e, *f))
            }
        })
        .collect();
    let expected = temp.iter().map(|(e, _f)| *e).collect();
    let found = temp.iter().map(|(_e, f)| *f).collect();

    (expected, found)
}

/// Calculate the Daylight Coefficients in a room with a glass window, considering ambient bounces
#[valid(Room With Glass - With Bounces)]
fn room_global_with_glass() -> Box<dyn Validate> {
    let (expected, found) = get_simple_results("room", 12, true);
    get_validator(expected, found)
}

/// Calculate the Daylight Coefficients in a room with a window with no glass, considering ambient bounces
#[valid(Room With No Glass - With Bounces)]
fn room_global_with_no_glass() -> Box<dyn Validate> {
    let (expected, found) = get_simple_results("room", 12, false);
    get_validator(expected, found)
}

/// Calculate the Daylight Coefficients in a room with a Glass window, with zero bounces
#[valid(Room With Glass - Direct)]
fn room_direct_with_glass() -> Box<dyn Validate> {
    let (expected, found) = get_simple_results("room", 0, true);
    get_validator(expected, found)
}

/// Calculate the Daylight Coefficients in a room, with zero bounces
#[valid(Room With No Glass - Direct)]
fn room_direct_with_no_glass() -> Box<dyn Validate> {
    let (expected, found) = get_simple_results("room", 0, false);
    get_validator(expected, found)
}

fn room(validator: &mut Validator) {
    validator.push(room_direct_with_no_glass());
    validator.push(room_direct_with_glass());
    validator.push(room_global_with_no_glass());
    validator.push(room_global_with_glass());
}

#[ignore]
#[test]
fn validate_dc() {
    // cargo test --release --features parallel --package rendering --test validate_dc -- validate_dc --exact --nocapture --ignored
    let mut validator = Validator::new(
        "Validate Daylight Coefficients",
        "./docs/validation/daylight_coefficient.html",
    );

    room(&mut validator);

    validator.validate().unwrap();
}
