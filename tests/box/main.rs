use simple::{run_simulation::*, Model};
use validate::{valid, ScatterValidator, ValidFunc, Validator};

#[test]
fn box_sim() -> Result<(), String> {
    // cargo test --release --package simple --test box -- box_sim --exact --nocapture
    let p = "./docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }
    let target_file = format!("{}/cold_wellington_box.html", p);
    let mut validations = Validator::new("Simulation of a single room", &target_file);

    #[valid("Simulate a single-zone building in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes:     
    /// * Dynamic Heat Transfer through walls
    /// * Convection Coefficients
    /// * Long wave radiation exchange with the sky
    /// * Direct and Diffuse Solar Radiation
    fn series() -> Result<ValidFunc, String> {
        let options = SimOptions {
            input_file: "./tests/box/box.spl".into(),
            weather_file: "./tests/wellington.epw".into(),
            output: Some("./tests/box/check.csv".into()),
            control_file: None,
            research_mode: false,
            n: 4,
        };

        // Create model
        let (simple_model, mut state_header) = Model::from_file(&options.input_file)?;

        let controller = simple::void_control::VoidControl {};

        let res = &options.output.clone().ok_or("No output")?;
        let out = std::fs::File::create(res).map_err(|e| e.to_string())?;
        run(
            &simple_model,
            &mut state_header,
            // state,
            &options,
            out,
            controller,
        )?;

        // Load produced data
        let found = validate::from_csv::<simple::Float>(res, &[1]);
        let expected = validate::from_csv::<simple::Float>("tests/box/cold_box_eplus.csv", &[0]);

        Ok(Box::new(ScatterValidator {
            chart_title: Some("Dry Bulb Temperature - SIMPLE vs EnergyPlus"),
            units: Some("C"),
            expected_legend: Some("EnergyPlus-calculated temperature"),
            expected: expected[0].iter().skip(20).map(|v| *v).collect(),
            found_legend: Some("SIMPLE-calculated temperature"),
            found: found[0].iter().skip(20).map(|v| *v).collect(),

            // allowed_r2: Some(0.93),
            // allowed_intersect_delta: Some(0.5),
            // allowed_slope_delta: Some(0.06),
            ..Default::default()
        }))
    }

    validations.push(series()?);

    validations.validate()
}
