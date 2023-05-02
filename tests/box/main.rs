use simple::{run_simulation::*,  Model};

#[test]
#[ignore]
fn some_fun() {
    // cargo test --release --package simple --test box -- some_fun --exact --nocapture --ignored
    let options = SimOptions {
        // input_file: "./tests/box/input.spl".into(),
        input_file: "./tests/box/cold.spl".into(),
        weather_file: "./tests/wellington.epw".into(),
        output: Some("./check.csv".into()),
        control_file: None,
        research_mode: false,
        n: 4,
    };

    // Create model
    let (simple_model, mut state_header) =
        Model::from_file(options.input_file.to_string()).unwrap();

    let controller = simple::void_control::VoidControl {};
    // let controller = simple::OccupantBehaviour::new(&simple_model).unwrap();

    let res = match &options.output {
        Some(v) => {
            let out = std::fs::File::create(v).unwrap();
            run(
                &simple_model,
                &mut state_header,
                // state,
                &options,
                out,
                controller,
            )
        }
        None => run(
            &simple_model,
            &mut state_header,
            // state,
            &options,
            std::io::stdout().lock(),
            controller,
        ),
    };

    if let Err(e) = res {
        panic!("{}", e);
    }
}
