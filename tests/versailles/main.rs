use simple::{run_simulation::*, Model};
use validate::{valid, ScatterValidator, ValidFunc, Validator};

#[test]
#[ignore]
fn versailles_sim() -> Result<(), String> {
    // cargo test --release --features parallel --package simple --test versailles -- versailles_sim --exact --nocapture --ignored
    let p = "./docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }
    let target_file = format!("{}/versailles_energy.html", p);
    let mut validations = Validator::new(
        "Simulation of an entire townhouse in Wellington New Zealand",
        &target_file,
    );

    let options = SimOptions {
        input_file: "./tests/versailles/versailles.spl".into(),
        // weather_file: "./tests/versailles/api_weather.sw".into(),
        weather_file: "./tests/wellington.epw".into(),
        output: Some("./tests/versailles/check.csv".into()),
        control_file: None,
        research_mode: false,
        n: 4,
    };

    // Create model
    let (simple_model, mut state_header) = Model::from_file(&options.input_file)?;

    let controller = simple::occupant_behaviour::OccupantBehaviour {};

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

    fn process_space(i: usize) -> ValidFunc {
        // Load produced data
        let mut found = validate::from_csv::<simple::Float>(
            "./tests/cold_apartment/check.csv",
            &[1, 2, 3, 4, 5, 6, 7, 8],
        );
        let mut expected = validate::from_csv::<simple::Float>(
            "tests/cold_apartment/eplusout.csv",
            &[1, 2, 3, 4, 5, 6, 7, 8],
        );

        expected[i].pop();
        found[i].pop();


        let skip = 2000; // This is warmup period
        // let take = expected[0].len() - skip - 1000;
        Box::new(ScatterValidator {
            chart_title: Some("Dry Bulb Temperature - SIMPLE vs EnergyPlus"),
            units: Some("C"),
            expected_legend: Some("EnergyPlus-calculated temperature"),
            expected: expected[i]
                .iter()
                .skip(skip)
                // .take(take)
                .map(|v| *v)
                .collect(),
            found_legend: Some("SIMPLE-calculated temperature"),
            found: found[i].iter()
                .skip(skip)
                // .take(take)
                .map(|v| *v)
                .collect(),

            // allowed_r2: Some(0.93),
            // allowed_intersect_delta: Some(0.8),
            // allowed_slope_delta: Some(0.12),
            ..Default::default()
        })
    }

    #[valid("Kids bedroom in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn kids_bedroom() -> ValidFunc {
        process_space(0)
    }

    #[valid("Bathrooom in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn bathroom() -> ValidFunc {
        process_space(1)
    }

    #[valid("Storage in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn storage() -> ValidFunc {
        process_space(2)
    }

    #[valid("Kitchen in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn kitchen() -> ValidFunc {
        process_space(3)
    }

    #[valid("Laundry in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn laundry() -> ValidFunc {
        process_space(4)
    }

    #[valid("Livingroom in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn livingroom() -> ValidFunc {
        process_space(5)
    }

    #[valid("Main Bedroom in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn main_bedroom() -> ValidFunc {
        process_space(6)
    }

    #[valid("Hallway in an apartment in Wellington, New Zealand")]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    ///
    /// It includes pretty much everything
    fn hallway() -> ValidFunc {
        process_space(7)
    }

    validations.push(kids_bedroom());
    validations.push(bathroom());
    validations.push(storage());
    validations.push(kitchen());
    validations.push(laundry());
    validations.push(livingroom());
    validations.push(main_bedroom());
    validations.push(hallway());

    validations.validate()
}
