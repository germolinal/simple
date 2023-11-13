use communication::MetaOptions;
use communication::SimulationModel;
use heat::heat_model::ThermalModel;
use heat::Float;

use calendar::Date;
use model::{Model, SimulationStateElement, SimulationStateHeader, HVAC};
use schedule::ScheduleConstant;
use test_models::{get_single_zone_test_building, SingleZoneTestBuildingOptions, TestMat};
use validate::{valid, SeriesValidator, ValidFunc, Validator};
use weather::SyntheticWeather;

fn get_validator(
    expected: Vec<Float>,
    found: Vec<Float>,
    expected_legend: &'static str,
) -> Box<SeriesValidator<Float>> {
    Box::new(SeriesValidator {
        x_label: Some("time step"),
        y_label: Some("Zone Temperature"),
        y_units: Some("C"),

        expected_legend: Some(expected_legend),
        found_legend: Some("SIMPLE"),
        expected,
        found,

        allowed_mean_bias_error: Some(0.2),
        allowed_root_mean_squared_error: Some(0.2),
        ..validate::SeriesValidator::default()
    })
}

/// A single-zone test model with walls assumed to have
/// no mass. It has a closed solution, which is nice.
///
/// There is no sun.
#[derive(Default)]
struct SingleZoneTestModel {
    /// volume of the zone (m3)
    zone_volume: Float,

    /// Facade area (m2)
    surface_area: Float,

    /// the R-value of the facade
    facade_r: Float,

    /// Infiltration rate (m3/s)
    infiltration_rate: Float,

    /// Heating power (Watts)
    heating_power: Float,

    /// Lighting power (Watts)
    lighting_power: Float,

    /// Temperature outside of the zone
    temp_out: Float,

    /// Temperature at the beginning
    temp_start: Float,
}

impl SingleZoneTestModel {
    fn get_closed_solution(&self) -> Box<impl Fn(Float) -> Float> {
        // heat balance in the form
        // of C*dT/dt = A - B*T
        let air = heat::gas::AIR;
        let rho = air.density(22. + 273.15); //kg/m3
        let cp = air.heat_capacity(22. + 273.15); //J/kg.K
        let u = 1. / self.facade_r;

        let c = self.zone_volume * rho * cp;

        let a = self.heating_power
            + self.lighting_power
            + self.temp_out * u * self.surface_area
            + self.infiltration_rate * rho * cp * self.temp_out;

        let b = u * self.surface_area + rho * self.infiltration_rate * cp;

        let k1 = self.temp_start - a / b;

        let f = move |t: Float| -> Float { a / b + k1 * (-b * t / c).exp() };

        Box::new(f)
    }
}

const META_OPTIONS: MetaOptions = MetaOptions {
    latitude: 0.,
    longitude: 0.,
    standard_meridian: 0.,
    elevation: 0.0,
};

fn march_with_window() -> Result<(Vec<Float>, Vec<Float>), String> {
    let surface_height = 2.;
    let surface_width = 2.;
    let window_height = 1.;
    let window_width = 1.;
    let zone_volume = 40.;

    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            window_height,
            window_width,
            construction: vec![TestMat::Polyurethane(0.02)],
            emissivity: 0.0,
            ..Default::default()
        },
    )?;

    // Finished model the Model
    let n: usize = 6;
    let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;

    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut memory = thermal_model.allocate_memory(&state)?;

    // START TESTING.
    let hs_front = 10.;
    let hs_back = 10.;
    thermal_model.surfaces[0].front_hs = Some(hs_front);
    thermal_model.surfaces[0].back_hs = Some(hs_back);
    thermal_model.fenestrations[0].front_hs = Some(hs_front);
    thermal_model.fenestrations[0].back_hs = Some(hs_back);

    let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

    // Initial T of the zone
    let t_start = thermal_model.zones[0]
        .reference_space
        .dry_bulb_temperature(&state)
        .ok_or("No dry bulb temperature found")?;

    let t_out: Float = 30.0; // T of surroundings

    let mut weather = SyntheticWeather::default();
    weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
    weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
    weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

    let dt = main_dt;

    let mut date = Date {
        day: 1,
        hour: 0.0,
        month: 1,
    };

    // test model
    let tester = SingleZoneTestModel {
        surface_area: surface_height * surface_width, // the window is a hole on the wall... does not add area
        zone_volume,
        facade_r: r,
        temp_out: t_out,
        temp_start: t_start,
        ..SingleZoneTestModel::default()
    };
    let exp_fn = tester.get_closed_solution();

    // March:
    let n = 80;
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        let time = (i as Float) * dt;
        date.add_seconds(time);

        let found_v = thermal_model.zones[0]
            .reference_space
            .dry_bulb_temperature(&state)
            .ok_or("No dry bulb temperature found")?;

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Get exact solution.
        let exp_v = exp_fn(time);
        exp.push(exp_v);
        found.push(found_v);
    }
    Ok((exp, found))
}

fn very_simple_march() -> Result<(Vec<Float>, Vec<Float>), String> {
    let zone_volume = 40.;
    let surface_width = 2.;
    let surface_height = 2.;
    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            construction: vec![TestMat::Polyurethane(0.02)],
            emissivity: 0.0,
            ..Default::default()
        },
    )?;

    let n: usize = 60;
    let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;

    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut memory = thermal_model.allocate_memory(&state)?;

    let hs_front = 10.;
    let hs_back = 10.;
    thermal_model.surfaces[0].front_hs = Some(hs_front);
    thermal_model.surfaces[0].back_hs = Some(hs_back);

    let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

    // Initial T of the zone
    let t_start = thermal_model.zones[0]
        .reference_space
        .dry_bulb_temperature(&state)
        .ok_or("No Dry Bulb temp. found")?;

    let t_out: Float = 30.0; // T of surroundings

    // test model
    let tester = SingleZoneTestModel {
        zone_volume,
        surface_area: surface_height * surface_width,
        facade_r: r,
        temp_out: t_out,
        temp_start: t_start,
        ..SingleZoneTestModel::default()
    };
    let exp_fn = tester.get_closed_solution();

    let mut weather = SyntheticWeather::default();
    weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
    weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
    weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

    let mut date = Date {
        day: 1,
        hour: 0.0,
        month: 1,
    };

    let n = 1000;
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..1000 {
        let time = (i as Float) * main_dt;
        date.add_seconds(time);

        let found_v = thermal_model.zones[0]
            .reference_space
            .dry_bulb_temperature(&state)
            .ok_or("No Dry Bulb temp. found")?;

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Get exact solution.
        let exp_v = exp_fn(time);

        exp.push(exp_v);
        found.push(found_v);
    }

    Ok((exp, found))
}

fn march_with_window_and_luminaire() -> Result<(Vec<Float>, Vec<Float>), String> {
    let surface_width = 2.;
    let surface_height = 2.;
    let zone_volume = 40.;
    let lighting_power = 100.;

    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            lighting_power,
            construction: vec![TestMat::Polyurethane(0.02)],
            emissivity: 0.0,
            ..Default::default()
        },
    )?;

    // Finished model the Model

    let n: usize = 20;
    let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;

    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut memory = thermal_model.allocate_memory(&state)?;

    // turn the lights on
    let lum_state_i = simple_model.luminaires[0]
        .power_consumption_index()
        .ok_or("Could not get power_consumption_index")?;
    state[lum_state_i] = lighting_power;

    // START TESTING.

    let hs_front = 10.;
    let hs_back = 10.;
    thermal_model.surfaces[0].front_hs = Some(hs_front);
    thermal_model.surfaces[0].back_hs = Some(hs_back);
    let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

    // Initial T of the zone
    let t_start = 22.;

    thermal_model.zones[0]
        .reference_space
        .set_dry_bulb_temperature(&mut state, t_start)?;

    let t_out: Float = 30.0; // T of surroundings

    // test model
    let tester = SingleZoneTestModel {
        zone_volume,
        surface_area: surface_height * surface_width, // the window is a hole on the wall... does not add area
        lighting_power,
        facade_r: r,
        temp_out: t_out,
        temp_start: t_start,
        ..SingleZoneTestModel::default()
    };
    let exp_fn = tester.get_closed_solution();

    let mut weather = SyntheticWeather::default();
    weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
    weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
    weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

    let dt = main_dt; // / model.dt_subdivisions() as Float;

    let mut date = Date {
        day: 1,
        hour: 0.0,
        month: 1,
    };

    // March:
    let n = 800;
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        let time = (i as Float) * dt;
        date.add_seconds(time);

        let found_v = thermal_model.zones[0]
            .reference_space
            .dry_bulb_temperature(&state)
            .ok_or("Could not get dry bulb temperature")?;

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Get exact solution.
        let exp_v = exp_fn(time);

        exp.push(exp_v);
        found.push(found_v);
    }

    Ok((exp, found))
}

fn march_with_window_and_heater() -> Result<(Vec<Float>, Vec<Float>), String> {
    let surface_height = 2.;
    let surface_width = 2.;
    let zone_volume = 40.;
    let heating_power = 100.;

    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            heating_power,
            construction: vec![TestMat::Polyurethane(0.02)],
            emissivity: 0.0,
            ..Default::default()
        },
    )?;

    // Finished model the Model

    let n: usize = 20;
    let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;
    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut memory = thermal_model.allocate_memory(&state)?;

    // turn the heater on
    if let HVAC::ElectricHeater(heater) = &simple_model.hvacs[0] {
        let hvac_state_i = heater
            .heating_cooling_consumption_index()
            .ok_or("No heating consumption index found")?;
        state[hvac_state_i] = heating_power;
    }

    // START TESTING.
    // assert!(!model.surfaces[0].is_massive());

    let hs_front = 10.;
    let hs_back = 10.;
    thermal_model.surfaces[0].front_hs = Some(hs_front);
    thermal_model.surfaces[0].back_hs = Some(hs_back);

    let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

    // Initial T of the zone
    let t_start = thermal_model.zones[0]
        .reference_space
        .dry_bulb_temperature(&state)
        .ok_or("No Dry Bulb temp. found")?;
    let t_out: Float = 30.0; // T of surroundings

    // test model
    let tester = SingleZoneTestModel {
        zone_volume,
        surface_area: surface_height * surface_width, // the window is a hole on the wall... does not add area
        heating_power,
        facade_r: r,
        temp_out: t_out,
        temp_start: t_start,
        ..SingleZoneTestModel::default()
    };
    let exp_fn = tester.get_closed_solution();

    let mut weather = SyntheticWeather::default();
    weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
    weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
    weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

    let dt = main_dt; // / model.dt_subdivisions() as Float;

    let mut date = Date {
        day: 1,
        hour: 0.0,
        month: 1,
    };

    // March:
    let n = 800;
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        let time = (i as Float) * dt;
        date.add_seconds(time);

        let found_v = thermal_model.zones[0]
            .reference_space
            .dry_bulb_temperature(&state)
            .ok_or("No Dry Bulb temp. found")?;

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Get exact solution.
        let exp_v = exp_fn(time);

        exp.push(exp_v);
        found.push(found_v);
    }
    Ok((exp, found))
}

fn march_with_window_heater_and_infiltration() -> Result<(Vec<Float>, Vec<Float>), String> {
    let surface_width = 2.;
    let surface_height = 2.;
    let zone_volume = 40.;
    let heating_power = 10.;
    let infiltration_rate = 0.1;
    let t_out: Float = 30.0; // T of surroundings

    let (simple_model, mut state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            heating_power,
            infiltration_rate,
            emissivity: 0.0,
            construction: vec![TestMat::Polyurethane(0.02)],
            ..Default::default()
        },
    )?;

    // Finished model the Model

    let n: usize = 20;
    let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;

    // Set infiltration
    let inf_vol_index = state_header.push(
        SimulationStateElement::SpaceInfiltrationVolume(0),
        infiltration_rate,
    )?;
    simple_model.spaces[0].set_infiltration_volume_index(inf_vol_index)?;
    let inf_temp_index = state_header.push(
        SimulationStateElement::SpaceInfiltrationTemperature(0),
        t_out,
    )?;
    simple_model.spaces[0].set_infiltration_temperature_index(inf_temp_index)?;

    // MAP THE STATE

    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut memory = thermal_model.allocate_memory(&state)?;

    // turn the heater on
    if let HVAC::ElectricHeater(heater) = &simple_model.hvacs[0] {
        let hvac_state_i = heater
            .heating_cooling_consumption_index()
            .ok_or("No heating/cooling consumption index found")?;
        state[hvac_state_i] = heating_power;
    }

    // START TESTING.

    let hs_front = 10.;
    let hs_back = 10.;
    thermal_model.surfaces[0].front_hs = Some(hs_front);
    thermal_model.surfaces[0].back_hs = Some(hs_back);

    let r = thermal_model.surfaces[0].discretization.r_value() + 1. / hs_front + 1. / hs_back;

    // Initial T of the zone
    let t_start = thermal_model.zones[0]
        .reference_space
        .dry_bulb_temperature(&state)
        .ok_or("No Dry Bulb temp. found")?;

    // test model
    let tester = SingleZoneTestModel {
        zone_volume,
        surface_area: surface_height * surface_width, // the window is a hole on the wall... does not add area
        heating_power,
        facade_r: r,
        temp_out: t_out,
        temp_start: t_start,
        infiltration_rate,
        ..SingleZoneTestModel::default()
    };
    let exp_fn = tester.get_closed_solution();

    let mut weather = SyntheticWeather::default();
    weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
    weather.wind_direction = Box::new(ScheduleConstant::new(0.0));
    weather.wind_speed = Box::new(ScheduleConstant::new(0.0));

    let dt = main_dt; // / model.dt_subdivisions() as Float;

    let mut date = Date {
        day: 1,
        hour: 0.0,
        month: 1,
    };

    // March:
    let n = 22;
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        let time = (i as Float) * dt;
        date.add_seconds(time);

        let found_v = thermal_model.zones[0]
            .reference_space
            .dry_bulb_temperature(&state)
            .ok_or("No Dry Bulb temp. found")?;

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Get exact solution.
        let exp_v = exp_fn(time);

        exp.push(exp_v);
        found.push(found_v);
    }
    Ok((exp, found))
}

fn march_model(
    dir: &'static str,
    simple_model: Model,
    mut state_header: SimulationStateHeader,
    emissivity: Float,
    surface_area: Float,
) -> Result<(Vec<Float>, Vec<Float>), String> {
    // Finished model the Model

    let n: usize = 20;
    // let main_dt = 60. * 60. / n as Float;
    let mut thermal_model =
        ThermalModel::new(&META_OPTIONS, (), &simple_model, &mut state_header, n)?;
        // in model like these—i.e., a single surface—EnergyPlus assumes Zero IR radation
        thermal_model.surfaces[0].back_emissivity = 0.0;
        
        let mut state = state_header.take_values().ok_or("Could not find state")?;
        let mut memory = thermal_model.allocate_memory(&state)?;

    let path_string = format!("./tests/{}/eplusout.csv", dir);
    let path = path_string.as_str();
    let cols = validate::from_csv(path, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

    //     0	Date/Time
    let site_wind_speed = &cols[0]; // 1	Environment:Site Wind Speed [m/s](TimeStep)
    let site_wind_direction = &cols[1]; // 2	Environment:Site Wind Direction [deg](TimeStep)
    let incident_solar_radiation = &cols[2]; // 3	WALL EXTERIOR:Surface Outside Face Incident Solar Radiation Rate per Area [W/m2](TimeStep)
                                             // let inside_surface_temp = &cols[3];                 // 4	WALL EXTERIOR:Surface Inside Face Temperature [C](TimeStep)
                                             // let outside_surface_temp = &cols[4];                // 5	WALL EXTERIOR:Surface Outside Face Temperature [C](TimeStep)
                                             // let exp_hs_in = &cols[5];                           // 6	WALL EXTERIOR:Surface Inside Face Convection Heat Transfer Coefficient [W/m2-K](TimeStep)
                                             // let indoor_thermal_heat_gain = &cols[6]; // 7	WALL EXTERIOR:Surface Inside Face Net Surface Thermal Radiation Heat Gain Rate [W](TimeStep)
    let outdoor_temp = &cols[7]; // 8	WALL EXTERIOR:Surface Outside Face Outdoor Air Drybulb Temperature [C](TimeStep)
                                 // let surface_wind_speed = &cols[8];          // 9	WALL EXTERIOR:Surface Outside Face Outdoor Air Wind Speed [m/s](TimeStep)
                                 // let exp_hs_out = &cols[9];                          // 10	WALL EXTERIOR:Surface Outside Face Convection Heat Transfer Coefficient [W/m2-K](TimeStep)
    let outdoor_thermal_heat_gain = &cols[10]; // 11	WALL EXTERIOR:Surface Outside Face Net Thermal Radiation Heat Gain Rate [W](TimeStep)
    let exp_zone_air_temp = &cols[11]; // 12	INTERIOR SPACE:Zone Mean Air Temperature [C](TimeStep)

    // Set initial temperature
    simple_model.spaces[0].set_dry_bulb_temperature(&mut state, exp_zone_air_temp[0])?;

    let mut date = Date {
        month: 1,
        day: 1,
        hour: 0.0,
    };
    let n = outdoor_temp.len();
    let mut exp = Vec::with_capacity(n);
    let mut found = Vec::with_capacity(n);
    for i in 0..n {
        // Get zone's temp
        let found_temp = simple_model.spaces[0]
            .dry_bulb_temperature(&state)
            .ok_or("No Dry Bulb temp. found")?;
        let exp_temp = exp_zone_air_temp[i];
        if i > 5000 {
            // skip warmup
            exp.push(exp_temp);
            found.push(found_temp);
        }

        // Set outdoor temp
        let mut weather = SyntheticWeather::default();
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(outdoor_temp[i]));
        weather.wind_direction = Box::new(ScheduleConstant::new(site_wind_direction[i]));
        weather.wind_speed = Box::new(ScheduleConstant::new(site_wind_speed[i]));

        let surface = &simple_model.surfaces[0];

        // Set Solar Radiation
        surface.set_front_incident_solar_irradiance(&mut state, incident_solar_radiation[i])?;

        // Set Long Wave radiation
        if emissivity > 1e-3 {
            // let v = indoor_thermal_heat_gain[i] / surface_area / emissivity
            // + heat::SIGMA * (ts + 273.15).powi(4);
            // surface.set_back_ir_irradiance(&mut state, v);

            let ts = surface
                .first_node_temperature(&state)
                .ok_or("No first-node temperature")?;
            let v = outdoor_thermal_heat_gain[i] / surface_area / emissivity
                + heat::SIGMA * (ts + 273.15).powi(4);
            surface.set_front_ir_irradiance(&mut state, v)?;
        }

        // March

        thermal_model.march(date, &weather, &simple_model, &mut state, &mut memory)?;

        // Advance
        date.add_hours(1. / n as Float);
    }
    Ok((exp, found))
}

fn march_test_model(
    dir: &'static str,
    emissivity: Float,
    solar_abs: Float,
    construction: Vec<TestMat>,
) -> Result<(Vec<Float>, Vec<Float>), String> {
    let surface_height = 3.;
    let surface_width = 20.;
    let zone_volume = 600.;
    let surface_area = surface_height * surface_width;

    let (simple_model, state_header) = get_single_zone_test_building(
        // &mut state,
        &SingleZoneTestBuildingOptions {
            zone_volume,
            surface_height,
            surface_width,
            construction,
            emissivity,
            solar_absorbtance: solar_abs,
            ..Default::default()
        },
    )?;

    march_model(dir, simple_model, state_header, emissivity, surface_area)
}

fn march_simple_model(
    dir: &'static str,
    filename: &'static str,
    emissivity: Float,
    surface_area: Float,
) -> Result<(Vec<Float>, Vec<Float>), String> {
    let filename = format!("./tests/{dir}/{filename}.spl");
    let (simple_model, state_header) = Model::from_file(filename)?;

    march_model(dir, simple_model, state_header, emissivity, surface_area)
}

fn theoretical(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "Theoretical Solution";

    #[valid("Nomass Wall - Walls only")]
    fn nomass_wallonly() -> Result<ValidFunc, String> {
        let (expected, found) = very_simple_march()?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    #[valid("Nomass Wall - Walls and Fenestration")]
    fn nomass_wall_and_window() -> Result<ValidFunc, String> {
        let (expected, found) = march_with_window()?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Nomass Wall - Walls and Fenestration, with Luminaire on")]
    fn window_and_luminaire() -> Result<ValidFunc, String> {
        let (expected, found) = march_with_window_and_luminaire()?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Nomass Wall - Walls and Window and heater")]
    fn nomass_wall_and_window_and_heater() -> Result<ValidFunc, String> {
        let (expected, found) = march_with_window_and_heater()?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Nomass Wall - Walls and Fenestration, with heater on and infiltration")]
    fn window_heater_and_infiltration() -> Result<ValidFunc, String> {
        let (expected, found) = march_with_window_heater_and_infiltration()?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    validations.push(nomass_wallonly()?);
    validations.push(nomass_wall_and_window()?);
    validations.push(window_and_luminaire()?);
    validations.push(nomass_wall_and_window_and_heater()?);
    validations.push(window_heater_and_infiltration()?);

    Ok(())
}

fn tilted(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    /// This test intends to test non-vertical convection coefficients and their correct placement
    #[valid("Massive and Tilted Wall, with the Space at its front")]
    fn wall1() -> Result<ValidFunc, String> {
        let (expected, found) = march_simple_model("tilted", "back", 0.9, 60.)?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    validations.push(wall1()?);

    Ok(())
}

fn horizontal(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    /// This test intends to test non-vertical convection coefficients and their correct placement
    #[valid("Massive Horizontal Wall, with Solar and Long Wave Radiation")]
    fn wall1() -> Result<ValidFunc, String> {
        let (expected, found) = march_simple_model("horizontal", "back", 0.9, 60.)?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }
    validations.push(wall1()?);

    Ok(())
}

fn massive(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid("Massive Wall, with Solar and Long Wave Radiation")]
    fn wall1() -> Result<ValidFunc, String> {
        let (expected, found) =
            march_test_model("massive_full", 0.9, 0.7, vec![TestMat::Concrete(0.2)])?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    #[valid("Massive Wall, with no Solar or Long Wave Radiation")]
    fn wall2() -> Result<ValidFunc, String> {
        // Massive, NO solar and NO Long Wave
        let (expected, found) = march_test_model(
            "massive_no_ir_no_solar",
            0.0,
            0.0,
            vec![TestMat::Concrete(0.2)],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Massive Wall, with Solar Radiation but not Long Wave Radiation")]
    fn wall3() -> Result<ValidFunc, String> {
        // Massive, WITH  solar and NO Long Wave
        let (expected, found) = march_test_model(
            "massive_no_ir_yes_solar",
            0.0,
            0.7,
            vec![TestMat::Concrete(0.2)],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Massive Wall, with Long Wave Radiation but not Solar Radiation")]
    fn wall4() -> Result<ValidFunc, String> {
        // Massive, No  solar and WITH Long Wave
        let (expected, found) = march_test_model(
            "massive_yes_ir_no_solar",
            0.9,
            0.0,
            vec![TestMat::Concrete(0.2)],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    validations.push(wall1()?);
    validations.push(wall2()?);
    validations.push(wall3()?);
    validations.push(wall4()?);

    Ok(())
}

fn mixed(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid("Mixed Mass Wall, with Solar Radiation and Long Wave Radiation")]
    fn wall1() -> Result<ValidFunc, String> {
        // Mixed Mass, With solar Radiation and Long Wave
        let (expected, found) = march_test_model(
            "mixed_full",
            0.9,
            0.7,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Mixed Mass Wall, without Solar or Long Wave Radiation")]
    fn wall2() -> Result<ValidFunc, String> {
        // Mixed Mass, NO solar and NO Long Wave
        let (expected, found) = march_test_model(
            "mixed_no_ir_no_solar",
            0.0,
            0.0,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    #[valid("Mixed Mass Wall, with Solar Radiation but no Long Wave Radiation")]
    fn wall3() -> Result<ValidFunc, String> {
        // Mixed Mass, WITH  solar and NO Long Wave
        let (expected, found) = march_test_model(
            "mixed_no_ir_yes_solar",
            0.0,
            0.7,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("Mixed Mass Wall, with Long Wave Radiation but no Solar Radiation")]
    fn wall4() -> Result<ValidFunc, String> {
        // Mixed Mass, No  solar and WITH Long Wave
        let (expected, found) = march_test_model(
            "mixed_yes_ir_no_solar",
            0.9,
            0.0,
            vec![
                TestMat::Polyurethane(0.02),
                TestMat::Concrete(0.2),
                TestMat::Polyurethane(0.02),
            ],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    validations.push(wall1()?);
    validations.push(wall2()?);
    validations.push(wall3()?);
    validations.push(wall4()?);
    Ok(())
}

fn nomass(validations: &mut Validator) -> Result<(), String> {
    const EXPECTED_LEGEND: &'static str = "EnergyPlus";

    #[valid("No Mass Wall, with Solar Radiation and Long Wave Radiation")]
    fn wall1() -> Result<ValidFunc, String> {
        // No Mass, With solar Radiation and Long Wave
        let (expected, found) =
            march_test_model("nomass_full", 0.9, 0.7, vec![TestMat::Polyurethane(0.02)])?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("No Mass Wall, without Solar or Long Wave Radiation")]
    fn wall2() -> Result<ValidFunc, String> {
        // No Mass, NO solar and NO Long Wave
        let (expected, found) = march_test_model(
            "nomass_no_ir_no_solar",
            0.0,
            0.0,
            vec![TestMat::Polyurethane(0.02)],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    #[valid("No Mass Wall, with Solar Radiation but no Long Wave Radiation")]
    fn wall3() -> Result<ValidFunc, String> {
        // No Mass, WITH  solar and NO Long Wave
        let (expected, found) = march_test_model(
            "nomass_no_ir_yes_solar",
            0.0,
            0.7,
            vec![TestMat::Polyurethane(0.02)],
        )?;
        let v = get_validator(expected, found, EXPECTED_LEGEND);
        Ok(v)
    }

    #[valid("No Mass Wall, with Long Wave Radiation but no Solar Radiation")]
    fn wall4() -> Result<ValidFunc, String> {
        // No Mass, No  solar and WITH Long Wave
        let (expected, found) = march_test_model(
            "nomass_yes_ir_no_solar",
            0.9,
            0.0,
            vec![TestMat::Polyurethane(0.02)],
        )?;

        let v = get_validator(expected, found, EXPECTED_LEGEND);

        Ok(v)
    }

    validations.push(wall1()?);
    validations.push(wall2()?);
    validations.push(wall3()?);
    validations.push(wall4()?);
    Ok(())
}

// fn march_trombe_wall(
//     dir: &'static str,
//     emissivity: Float,
//     solar_abs: Float,
//     construction: Vec<TestMat>,
// ) -> Result<(Vec<Float>, Vec<Float>),String> {
//     let surface_area = 20. * 3.;
//     let zone_volume = 600.;

//     let (simple_model, mut state_header) = get_single_zone_test_building(
//         // &mut state,
//         &SingleZoneTestBuildingOptions {
//             zone_volume,
//             surface_area,
//             construction,
//             emissivity,
//             solar_absorbtance: solar_abs,
//             ..Default::default()
//         },
//     );

//     // Finished model the Model

//     let n: usize = 20;
//     // let main_dt = 60. * 60. / n as Float;
//     let thermal_model = ThermalModel::new(&simple_model, &mut state_header, n)?;

//     let mut state = state_header.take_values().ok_or("Could not take state")?;

//     let path_string = format!("./tests/{}/eplusout.csv", dir);
//     let path = path_string.as_str();
//     let cols = validate::from_csv(path, &[3, 15, 17, 18, 24]);
//     let incident_solar_radiation = &cols[0]; //3
//     let indoor_thermal_heat_gain = &cols[1]; //15
//     let outdoor_temp = &cols[2]; //17
//     let outdoor_thermal_heat_gain = &cols[3]; //18
//     let exp_zone_air_temp = &cols[4]; //24

//     // Set initial temperature
//     simple_model.spaces[0].set_dry_bulb_temperature(&mut state, exp_zone_air_temp[0]);

//     let mut date = Date {
//         month: 1,
//         day: 1,
//         hour: 0.0,
//     };
//     let n = outdoor_temp.len();
//     let mut exp = Vec::with_capacity(n);
//     let mut found = Vec::with_capacity(n);
//     for i in 0..n {
//         // Get zone's temp
//         let found_temp = simple_model.spaces[0].dry_bulb_temperature(&state).ok_or("No Dry Bulb temp. found")?;
//         let exp_temp = exp_zone_air_temp[i];
//         if i > 000 {
//             // skip warmup
//             exp.push(exp_temp);
//             found.push(found_temp);
//         }

//         // Set outdoor temp
//         let mut weather = SyntheticWeather::default();
//         weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(outdoor_temp[i]));
//            weather.wind_direction = Box::new(ScheduleConstant::new(0.0));

//         let surface = &simple_model.surfaces[0];

//         // Set Solar Radiation
//         surface.set_back_incident_solar_irradiance(&mut state, incident_solar_radiation[i]);

//         // Set Long Wave radiation
//         if emissivity > 1e-3 {
//             let ts = surface.last_node_temperature(&state)?;
//             let v = outdoor_thermal_heat_gain[i] / surface_area / emissivity
//                 + heat::SIGMA * (ts + 273.15).powi(4);
//             surface.set_back_ir_irradiance(&mut state, v);

//             let ts = surface.first_node_temperature(&state)?;
//             let v = indoor_thermal_heat_gain[i] / surface_area / emissivity
//                 + heat::SIGMA * (ts + 273.15).powi(4);
//             surface.set_front_ir_irradiance(&mut state, v);
//         }

//         // March
//         thermal_model
//             .march(date, &weather, &simple_model, &mut state)?;

//         // Advance
//         date.add_hours(1. / n as Float);
//     }
//     (exp, found)
// }

// fn trombe_wall(validations: &mut Validator) {
//     // No Mass, With solar Radiation and Long Wave
//     let (expected, found) = march_trombe_wall(
//         "trombe_wall_full",
//         0.9,
//         0.08,
//         vec![
//             TestMat::Concrete(0.2),
//             TestMat::Air(0.05),
//             TestMat::Glass(0.03, 0.82),
//         ],
//     );
//     let v = validate::SeriesValidator {
//         title: "Trombe Wall, with Solar Radiation and Long Wave Radiation",
//         x_label: Some("time step"),
//         y_label: Some("Zone Temperature"),
//         y_units: Some("C"),
//         found_name: "Simple",
//         expected_name: "EnergyPlus",

//         expected,
//         found,

//         ..validate::SeriesValidator::default()
//     };
//     validations.push(Box::new(v));
// }

#[test]
fn validate() -> Result<(), String> {
    // cargo test --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture
    let p = "../docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }

    let target_file = format!("{}/walls.html", p);
    let mut validations = Validator::new(
        "SIMPLE Heat - Wall Heat Transfer Validation Report",
        &target_file,
    );

    theoretical(&mut validations)?;
    massive(&mut validations)?;
    mixed(&mut validations)?;
    nomass(&mut validations)?;
    tilted(&mut validations)?;
    horizontal(&mut validations)?;

    // trombe_wall(&mut validations);
    validations.validate()
}
//
