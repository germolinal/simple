use simple::{run_simulation::*, Model};

fn main() -> Result<(), String> {
    // cargo instruments --features parallel --template 'CPU Profiler' --release --example cold_apartment

    // time cargo run  --release --example cold_apartment

    let options = SimOptions {
        input_file: "./tests/cold_apartment/cold.spl".into(),
        weather_file: Some("./tests/wellington.epw".into()),
        output: Some("./tests/cold_apartment/check.csv".into()),
        control_file: None,
        research_mode: false,
        n: 4,
        .. SimOptions::default()
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

    Ok(())
}
