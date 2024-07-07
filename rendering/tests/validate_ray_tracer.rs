use geometry::{Point3D, Ray3D, Vector3D};

use rendering::{Float, RayTracer, Scene};
use validate::{valid, SeriesValidator, ValidFunc, Validator};
const MAX_DEPTH: usize = 30;

fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<SeriesValidator<Float>> {
    Box::new(SeriesValidator {
        x_label: Some("Sensor".into()),
        y_label: Some("Luminance".into()),
        y_units: Some("cd/m2"),
        expected_legend: Some("Radiance"),
        found_legend: Some("SIMPLE"),
        expected,
        found,
        ..validate::SeriesValidator::default()
    })
}

fn load_expected_results(filename: String) -> Result<Vec<Float>, String> {
    let s = std::fs::read_to_string(filename).map_err(|e| e.to_string())?;
    let r = s
        .lines()
        .map(|line| {
            let a: Vec<Float> = line
                .split_ascii_whitespace()
                .into_iter()
                .map(|x| x.parse::<Float>().expect("This should never fail"))
                .collect();
            assert_eq!(a.len(), 1);
            a[0]
        })
        .collect();

    Ok(r)
}

fn load_rays(filename: &str) -> Result<Vec<Ray3D>, String> {
    let s = std::fs::read_to_string(filename).map_err(|e| e.to_string())?;
    let r = s
        .lines()
        .map(|line| {
            let a: Vec<Float> = line
                .split_ascii_whitespace()
                .into_iter()
                .map(|x| x.parse::<Float>().expect("This should never fail"))
                .collect();

            Ray3D {
                origin: Point3D::new(a[0], a[1], a[2]),
                direction: Vector3D::new(a[3], a[4], a[5]).get_normalized(),
            }
        })
        .collect();

    Ok(r)
}

fn get_simple_results(dir: &str, max_depth: usize) -> Result<(Vec<Float>, Vec<Float>), String> {
    let mut scene = Scene::from_radiance(format!("./tests/ray_tracer/{}/box.rad", dir))
        .expect("Could not read");
    scene.build_accelerator();

    let n_ambient_samples = 29000;

    let integrator = RayTracer {
        n_ambient_samples,
        n_shadow_samples: 1,
        max_depth,
        ..RayTracer::default()
    };
    let mut aux = [0; 32];
    let mut rng = rendering::rand::get_rng();

    let rays = load_rays("./tests/points.pts")?;

    let found: Vec<Float> = rays
        .into_iter()
        .map(|ray| {
            let c = integrator.trace_ray(&mut rng, &scene, ray, &mut aux);
            c.radiance()
        })
        .collect();

    let expected = if max_depth == 0 {
        load_expected_results(format!("./tests/ray_tracer/{dir}/direct_results.txt"))?
    } else {
        load_expected_results(format!("./tests/ray_tracer/{dir}/global_results.txt"))?
    };
    println!("{} | {}", dir, max_depth);
    println!("exp,found");
    for i in 0..expected.len() {
        println!("{},{}", expected[i], found[i])
    }
    Ok((expected, found))
}

fn plastic(validator: &mut Validator) -> Result<(), String> {
    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse plastic,
    /// without specularity or roughness.
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Diffuse Plastic - Global illumination")]
    fn plastic_diffuse_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_diffuse", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse plastic,
    /// without specularity or roughness. It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Diffuse Plastic - Direct illumination")]
    fn plastic_diffuse_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_diffuse", 0)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse plastic with some roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Rough Plastic - Global illumination")]
    fn plastic_rough_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_rough", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse plastic with some roughness.
    /// It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Rough Plastic - Direct illumination")]
    fn plastic_rough_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_rough", 0)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular plastic with no roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Specular Plastic - Global illumination")]
    fn plastic_specular_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_specular", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular plastic with no roughness.
    /// It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Specular Plastic - Direct illumination")]
    fn plastic_specular_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_specular", 0)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular plastic with some roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Full Plastic - Global illumination")]
    fn plastic_full_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_full", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular plastic with some roughness.
    /// It only takes into account direct lighting
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Full Plastic - Direct illumination")]
    fn plastic_full_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("plastic_box_full", 0)?;
        Ok(get_validator(expected, found))
    }

    validator.push(plastic_diffuse_global()?);
    validator.push(plastic_diffuse_direct()?);

    validator.push(plastic_specular_global()?);
    validator.push(plastic_specular_direct()?);

    validator.push(plastic_rough_global()?);
    validator.push(plastic_rough_direct()?);

    validator.push(plastic_full_global()?);
    validator.push(plastic_full_direct()?);

    Ok(())
}

fn metal(validator: &mut Validator) -> Result<(), String> {
    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse metal,
    /// without specularity or roughness.
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Diffuse Metal - Global illumination")]
    fn metal_diffuse_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_diffuse", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse metal,
    /// without specularity or roughness. It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Diffuse Metal - Direct illumination")]
    fn metal_diffuse_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_diffuse", 0)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse metal with some roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Rough Metal - Global illumination")]
    fn metal_rough_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_rough", MAX_DEPTH)?;

        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of diffuse metal with some roughness.
    /// It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Rough Metal - Direct illumination")]
    fn metal_rough_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_rough", 0)?;

        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular metal with no roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Specular Metal - Global illumination")]
    fn metal_specular_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_specular", MAX_DEPTH)?;

        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular metal with no roughness.
    /// It only takes into account direct lighting (no bounces)
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Specular Metal - Direct illumination")]
    fn metal_specular_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_specular", 0)?;

        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular metal with some roughness
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Full Metal - Global illumination")]
    fn metal_full_global() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_full", MAX_DEPTH)?;
        Ok(get_validator(expected, found))
    }

    /// Contrasts the results of SIMPLE and Radiance, in a box made of a partially specular metal with some roughness.
    /// It only takes into account direct lighting
    ///
    /// Radiance parameters used are ` rtrace -h -ab 0 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Full Metal - Direct illumination")]
    fn metal_full_direct() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("metal_box_full", 0)?;
        Ok(get_validator(expected, found))
    }

    validator.push(metal_diffuse_global()?);
    validator.push(metal_diffuse_direct()?);

    validator.push(metal_specular_global()?);
    validator.push(metal_specular_direct()?);

    validator.push(metal_rough_global()?);
    validator.push(metal_rough_direct()?);

    validator.push(metal_full_global()?);
    validator.push(metal_full_direct()?);

    Ok(())
}

fn glass(validator: &mut Validator) -> Result<(), String> {
    /// Contrasts the results of SIMPLE and Radiance, in a box made out of a glass.
    ///
    /// Radiance parameters used are ` rtrace -h -ab 30 -aa 0 -lr -0 -lw 1e-10 -ad 30000 $OCTREE `
    #[valid("Glass")]
    fn glass_full() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results("glass_box", MAX_DEPTH)?;

        Ok(get_validator(expected, found))
    }
    validator.push(glass_full()?);

    Ok(())
}

#[test]
fn validate_ray_tracer() -> Result<(), String> {
    // cargo test --package rendering --test validate_ray_tracer -- validate_ray_tracer --exact --nocapture
    let mut validator = Validator::new("Validate Ray Tracer", "../docs/validation/ray_tracer.html");

    metal(&mut validator)?;
    plastic(&mut validator)?;
    glass(&mut validator)?;

    validator.validate()
}
