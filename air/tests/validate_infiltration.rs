use air::{air_model::AirFlowModel, Float};
use calendar::{Date, Period};
use communication::{MetaOptions, SimulationModel};
use model::{Building, Infiltration, Model, SimulationStateHeader, Space};
use validate::{valid, ScatterValidator, ValidFunc, Validator};
use weather::{EPWWeather, Weather};

fn get_validator(
    expected: Vec<Float>,
    found: Vec<Float>,
    expected_legend: &'static str,
) -> Box<ScatterValidator<Float>> {
    Box::new(ScatterValidator {
        units: Some("m3/s"),

        expected_legend: Some(expected_legend),
        found_legend: Some("SIMPLE"),
        expected,
        found,

        // allowed_intersect_delta: Some(1e-2),
        // allowed_slope_delta: Some(1e-2),
        ..validate::ScatterValidator::default()
    })
}

fn get_model(inf: Infiltration) -> (Model, SimulationStateHeader) {
    let mut model = Model::default();
    let mut state_header = SimulationStateHeader::new();

    let bname = "the building";
    let mut building = Building::new(bname);
    let cs = 0.000145;
    let cw = 0.000174;
    building.set_stack_coefficient(cs).set_wind_coefficient(cw);
    model.add_building(building);

    let mut space = Space::new("the space");
    space
        .set_volume(1061.88)
        .set_infiltration(inf)
        .set_building(bname)
        .set_dry_bulb_temperature_index(0)
        .unwrap();

    state_header
        .push(
            model::SimulationStateElement::SpaceDryBulbTemperature(0),
            20.0,
        )
        .unwrap();

    model.add_space(space);
    (model, state_header)
}

/// returns expected and found
fn sim(inf: Infiltration, dir: &str) -> Result<(Vec<Float>, Vec<Float>), String> {
    // Load EPlus data
    let path_string = format!("./tests/{}/eplusout.csv", dir);
    let path = path_string.as_str();
    let cols = validate::from_csv(path, &[1, 2, 3]);

    let temp = &cols[1];
    let vol = &cols[2];

    let (model, mut state_header) = get_model(inf);
    let meta_options = MetaOptions::default();
    let options = ();

    let weather = EPWWeather::from_file("./tests/wellington.epw")?;
    let weather: Weather = weather.into();
    let physics = AirFlowModel::new(&meta_options, options, &model, &mut state_header, 1)?;
    let mut state = state_header.take_values().expect("could not take values");
    let mut mem = physics.allocate_memory(&state)?;

    let mut exp = Vec::with_capacity(weather.data.len());
    let mut found = Vec::with_capacity(weather.data.len());

    let start = Date {
        day: 1,
        month: 1,
        hour: 0.0,
    };

    let end = Date {
        day: 31,
        month: 12,
        hour: 23.99999,
    };

    let dt = 60. * 60. / 4.0; // 15 minutes

    let sim_period = Period::new(start, end, dt);
    for (i, date) in sim_period.into_iter().enumerate() {
        // Set interior temperature
        model.spaces[0].set_dry_bulb_temperature(&mut state, temp[i])?;
        physics.march(date, &weather, &model, &mut state, &mut mem)?;

        let inf = model.spaces[0]
            .infiltration_volume(&state)
            .expect("No infiltration");
        exp.push(vol[i]);
        found.push(inf)
    }

    Ok((exp, found))
}

fn effective_air_leakage_area(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    /// This test intends to test non-vertical convection coefficients and their correct placement
    #[valid("Effective Air Leakage Area")]
    fn aux() -> Result<ValidFunc, String> {
        let inf = Infiltration::EffectiveAirLeakageArea { area: 0.05 };

        let (expected, found) = sim(inf, "effective_air_leakage")?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    validations.push(aux()?);

    Ok(())
}

fn design_flow_rate(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    /// This test intends to test non-vertical convection coefficients and their correct placement
    /// 
    /// > **NOTE:** This algorithm is still under development. Do not utilize.
    #[valid("Design flow rate")]
    fn aux() -> Result<ValidFunc, String> {
        let inf = Infiltration::DesignFlowRate {
            a: 0.6060000,
            b: 3.6359996E-02,
            c: 0.1177165,
            d: 0.0,
            phi: 0.5,
        };

        let (expected, found) = sim(inf, "design_flow_rate")?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    validations.push(aux()?);

    Ok(())
}

#[test]
fn validate() -> Result<(), String> {
    // cargo test --package air --test validate_infiltration -- validate --exact --nocapture
    let p = "../docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }

    let target_file = format!("{}/infiltration.html", p);
    let mut validations =
        Validator::new("SIMPLE Air - Infiltration validation report", &target_file);

    effective_air_leakage_area(&mut validations)?;
    design_flow_rate(&mut validations)?;

    validations.validate()
}
