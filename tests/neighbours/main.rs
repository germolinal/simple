use simple::{run_simulation::*, Model};

#[test]
#[ignore]
fn neighbours_sim() -> Result<(), String> {
    // cargo test --features parallel --release --package simple --test neighbours -- neighbours_sim --exact --nocapture --ignored

    let options = SimOptions {
        input_file: "./tests/neighbours/neighbours.spl".into(),
        weather_file: Some("./tests/wellington.epw".into()),
        output: Some("./tests/neighbours/check.csv".into()),
        control_file: None,
        research_mode: false,
        n: 1,
        ..SimOptions::default()
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
    )
}
