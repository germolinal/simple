use simple::{run_simulation::*, Model};
use validate::{valid, ScatterValidator, Validate, Validator};

#[test]
#[ignore]
fn box_sim() {
    // cargo test --release --package simple --test box -- box_sim --exact --nocapture
    let p = "./docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).unwrap();
    }
    let target_file = format!("{}/cold_wellington_box.html", p);
    let mut validations = Validator::new("Simulation of a single room", &target_file);

    #[valid(Simulate a single-zone building in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes:     
    /// * Dynamic Heat Transfer through walls
    /// * Convection Coefficients
    /// * Long wave radiation exchange with the sky
    /// * Direct and Diffuse Solar Radiation
    fn series() -> Box<dyn Validate> {
        let options = SimOptions {
            input_file: "./tests/box/box.spl".into(),
            weather_file: "./tests/wellington.epw".into(),
            output: Some("./tests/box/check.csv".into()),
            control_file: None,
            research_mode: false,
            n: 4,
        };

        // Create model
        let (simple_model, mut state_header) =
            Model::from_file(options.input_file.to_string()).unwrap();

        let controller = simple::void_control::VoidControl {};
        // let controller = simple::OccupantBehaviour::new(&simple_model).unwrap();

        let res = &options.output.clone().unwrap();
        let out = std::fs::File::create(res).unwrap();
        run(
            &simple_model,
            &mut state_header,
            // state,
            &options,
            out,
            controller,
        )
        .unwrap();

        // Load produced data
        let found = validate::from_csv::<simple::Float>(res, &[1]);
        let expected = validate::from_csv::<simple::Float>("tests/box/cold_box_eplus.csv", &[0]);

        Box::new(ScatterValidator {
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
        })
    }

    validations.push(series());

    validations.validate().unwrap();
}
