use simple::run_simulation::prepare;
use simple::{EPWWeather, HVAC, Model, RhaiControlScript, SimulationModel, Weather};
use std::sync::Arc;

/// The examples in the "Getting started" and "Controlling the building"
/// chapters of the book (docs/book/src/examples) parse, and work together.
#[test]
fn book_examples() -> Result<(), String> {
    let (model, mut state_header) = Model::from_file("./docs/book/src/examples/bedroom.spl")?;
    let weather: Weather = EPWWeather::from_file("./tests/wellington.epw")?.into();

    let n = 60;
    let data = prepare(&model, weather, n, &mut state_header)?;
    let state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;
    let mut memory = data.model.allocate_memory(&state)?;

    let model = Arc::new(model);
    let script = std::fs::read_to_string("./docs/book/src/examples/thermostat.rhai")
        .map_err(|e| e.to_string())?;
    let (mut controller, state) = RhaiControlScript::new(&model, state, script, false)?;

    // Simulate the first two days... it is summer in Wellington, but the
    // nights are cold enough to need some heating
    let mut heated = false;
    for date in data.sim_period.take(48 * n) {
        controller.control(date)?;
        let mut state = state.lock().unwrap();
        data.model
            .march(date, &data.weather, model.as_ref(), &mut state, &mut memory)?;

        let HVAC::ElectricHeater(heater) = &model.hvacs[0] else {
            return Err("Expecting an ElectricHeater".into());
        };
        let power = heater
            .heating_cooling_consumption(&state)
            .ok_or("No power consumption")?;
        heated |= power > 0.0;
    }
    assert!(heated, "The heater was never turned on");

    Ok(())
}
