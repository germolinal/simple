use simple::run_simulation::{PreProcessData, prepare};
use simple::{
    EPWWeather, HVAC, Model, RhaiControlScript, SimulationModel, SimulationState, Weather,
};
use std::sync::{Arc, Mutex};

/// The model, the simulation, the controller and the state
type Setup = (
    Arc<Model>,
    PreProcessData,
    RhaiControlScript,
    Arc<Mutex<SimulationState>>,
);

/// Builds the bedroom of the book, controlled by `script`
fn setup(script: &str, research_mode: bool) -> Result<Setup, String> {
    let (model, mut state_header) = Model::from_file("./docs/book/src/examples/bedroom.spl")?;
    let weather: Weather = EPWWeather::from_file("./tests/wellington.epw")?.into();
    let data = prepare(&model, weather, 1, &mut state_header)?;
    let state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;
    let model = Arc::new(model);
    let (controller, state) = RhaiControlScript::new(&model, state, script, research_mode)?;
    Ok((model, data, controller, state))
}

/// Runs the first timestep, and returns the power of the heater
fn first_step(script: &str) -> Result<simple::Float, String> {
    let (model, data, mut controller, state) = setup(script, false)?;
    let mut memory = data.model.allocate_memory(&state.lock().unwrap())?;
    let mut period = data.sim_period;
    let date = period.next().ok_or("No dates")?;
    controller.control(date, period.day_of_week(date))?;
    let mut state = state.lock().unwrap();
    data.model
        .march(date, &data.weather, model.as_ref(), &mut state, &mut memory)?;
    let HVAC::ElectricHeater(heater) = &model.hvacs[0] else {
        return Err("Expecting an ElectricHeater".into());
    };
    heater
        .heating_cooling_consumption(&state)
        .ok_or("No power".to_string())
}

#[test]
fn physical_properties_need_research_mode() -> Result<(), String> {
    let script = r#"let s = space("Bedroom"); s.dry_bulb_temperature = 30.0;"#;
    let date = simple::Date {
        month: 1,
        day: 1,
        hour: 1.,
    };

    let (_, _, mut controller, _) = setup(script, false)?;
    let err = controller.control(date, None).unwrap_err();
    assert!(err.contains("research mode"), "{err}");

    let (model, _, mut controller, state) = setup(script, true)?;
    controller.control(date, None)?;
    let temp = model.spaces[0].dry_bulb_temperature(&state.lock().unwrap());
    assert_eq!(temp, Some(30.0));
    Ok(())
}

#[test]
fn operational_properties_can_be_set() -> Result<(), String> {
    let script = r#"let w = fenestration("North Window"); w.open_fraction = 0.5;"#;
    let (_, data, mut controller, _) = setup(script, false)?;
    let mut period = data.sim_period;
    let date = period.next().ok_or("No dates")?;
    controller.control(date, None)
}

#[test]
fn heaters_are_limited() -> Result<(), String> {
    // The bedroom heater has a max_heating_power of 1500 W
    let set = |w: &str| format!(r#"let h = hvac("Bedroom Heater"); h.power_consumption = {w};"#);
    assert_eq!(first_step(&set("99999.0"))?, 1500.0);
    assert_eq!(first_step(&set("-500.0"))?, 0.0);
    assert_eq!(first_step(&set("700.0"))?, 700.0);
    Ok(())
}

#[test]
fn date_and_randomness() -> Result<(), String> {
    // wellington.epw starts on a Sunday
    let script = r#"
        if day_of_week != "sunday" { throw `expected sunday, found ${day_of_week}`; }
        if month != 1 || day != 1 || hour != 1.0 { throw `wrong date ${month}/${day} ${hour}`; }
        let r = random();
        if r < 0.0 || r >= 1.0 { throw `random() out of range: ${r}`; }
    "#;
    first_step(script)?;
    Ok(())
}

#[test]
fn seeded_randomness_repeats() -> Result<(), String> {
    let script = "memory.r = random();";
    let date = simple::Date {
        month: 1,
        day: 1,
        hour: 1.,
    };
    let draw = |seed: Option<u64>| -> Result<String, String> {
        let (_, _, mut controller, _) = setup(script, false)?;
        if let Some(seed) = seed {
            controller.set_seed(seed);
        }
        let mut out = Vec::new();
        for _ in 0..3 {
            controller.control(date, None)?;
            out.push(format!("{:?}", controller.memory()));
        }
        Ok(out.join(","))
    };
    assert_eq!(draw(Some(42))?, draw(Some(42))?);
    assert_ne!(draw(Some(42))?, draw(Some(7))?);
    assert_ne!(draw(None)?, draw(None)?);
    Ok(())
}
