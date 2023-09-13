use calendar::Date;
use communication::{MetaOptions, SimulationModel};
use light::{Float, SolarModel};
use model::SolarOptions;
use schedule::ScheduleConstant;
use test_models::*;
use validate::{valid, ScatterValidator, ValidFunc, Validator};
use weather::SyntheticWeather;
const SIGMA: Float = 5.670374419e-8;
fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<ScatterValidator<Float>> {
    Box::new(ScatterValidator {
        units: Some("W/m2"),
        expected_legend: Some("EnergyPlus"),
        found_legend: Some("SIMPLE"),
        allowed_intersect_delta: Some(0.7),
        allowed_r2: Some(0.98),
        allowed_slope_delta: Some(0.01),
        expected,
        found,
        ..validate::ScatterValidator::default()
    })
}

fn get_simple_results(
    city: &str,
    orientation_str: &str,
) -> Result<(Vec<Float>, Vec<Float>), String> {
    let path = format!("./tests/{city}_{orientation_str}/eplusout.csv");
    let cols = validate::from_csv(&path, &[1, 2, 3, 4, 10, 11, 13, 14]);

    let horizontal_ir = cols[0].clone(); //1
    let diffuse_horizontal_rad = &cols[1]; //2
    let direct_normal_rad = &cols[2]; //3
    let _incident_solar_radiation = &cols[3]; //4

    let _inside_surface_temp = cols[4].clone(); //10
    let outside_surface_temp = cols[5].clone(); //11

    let dry_bulb_temp = cols[6].clone(); //13
    let outside_ir_gain = cols[7].clone(); //14

    let orientation = match orientation_str {
        "east" => -90.,
        "south" => 0.,
        "west" => 90.,
        "north" => 180.,
        _ => unreachable!(),
    };

    let (lat, lon, std_mer): (Float, Float, Float) = match city.as_bytes() {
        b"wellington" => (-41.3, -174.78, -180.),
        b"barcelona" => (41.28, -2.07, -15.), // ??? GMT + 1
        _ => panic!("Unsupported city '{}'", city),
    };

    let meta_options = MetaOptions {
        latitude: lat.to_radians(),
        longitude: lon.to_radians(),
        standard_meridian: std_mer.to_radians(),
        elevation: 0.0,
    };

    let zone_volume = 600.;

    let (model, mut state_header) = get_single_zone_test_building(&SingleZoneTestBuildingOptions {
        zone_volume,
        surface_width: 20.,
        surface_height: 3.,
        construction: vec![TestMat::Concrete(0.2)],
        orientation,
        ..Default::default()
    });

    // Finished model the Model
    let mut options = SolarOptions::new();
    options
        .set_n_solar_irradiance_points(100)
        .set_solar_ambient_divitions(3000)
        .set_solar_sky_discretization(1);

    let n: usize = 20;
    let solar_model = SolarModel::new(&meta_options, options, &model, &mut state_header, n)?;
    let mut state = state_header.take_values().ok_or("Could not take state")?;
    let mut date = Date {
        month: 1,
        day: 1,
        hour: 0.5,
    };
    let mut found = Vec::with_capacity(horizontal_ir.len());
    let mut expected = Vec::with_capacity(horizontal_ir.len());

    let surface_area = 60.0;
    let emmisivity = 0.9;
    for index in 0..horizontal_ir.len() {
        let gain = outside_ir_gain[index];
        let ts = outside_surface_temp[index];

        // gain = area * emissivity*(incident  - sigma  * ts^4)
        // --> gain/area/emissivity = incident - sigma * ts^4
        // --> gain/area/emissivity  + sigma * ts^4 = incident
        let expected_v: Float =
            gain / surface_area / emmisivity + SIGMA * (ts as Float + 273.15).powi(4);

        // Set outdoor temp
        let mut weather = SyntheticWeather::default();
        weather.dew_point_temperature = Box::new(ScheduleConstant::new(11.)); //11C is what Radiance uses by default.
        weather.horizontal_infrared_radiation_intensity =
            Box::new(ScheduleConstant::new(horizontal_ir[index]));
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(dry_bulb_temp[index]));
        weather.direct_normal_radiation = Box::new(ScheduleConstant::new(direct_normal_rad[index]));
        weather.diffuse_horizontal_radiation =
            Box::new(ScheduleConstant::new(diffuse_horizontal_rad[index]));

        let surface = &model.surfaces[0];

        // March
        solar_model.march(date, &weather, &model, &mut state, &mut ())?;

        let front_radiation = surface
            .front_ir_irradiance(&state)
            .ok_or("Could not get front IR irradiance")?;
        found.push(front_radiation);
        expected.push(expected_v);

        // Advance
        date.add_hours(1. / n as Float);
        // assert!(false)
    }
    Ok((expected, found))
}

fn barcelona(validator: &mut Validator) -> Result<(), String> {
    const CITY: &'static str = "barcelona";

    #[valid("Exterior Incident Long Wave Radiation - Barcelona, South")]
    fn validate_barcelona_south() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "south")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Barcelona, North")]
    fn validate_barcelona_north() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "north")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Barcelona, West")]
    fn validate_barcelona_west() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "west")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Barcelona, East")]
    fn validate_barcelona_east() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "east")?;
        Ok(get_validator(expected, found))
    }

    validator.push(validate_barcelona_south()?);
    validator.push(validate_barcelona_north()?);
    validator.push(validate_barcelona_west()?);
    validator.push(validate_barcelona_east()?);
    Ok(())
}

fn wellington(validator: &mut Validator) -> Result<(), String> {
    const CITY: &'static str = "wellington";

    #[valid("Exterior Incident Long Wave Radiation - Wellington, South")]
    fn validate_wellington_south() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "south")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Wellington, North")]
    fn validate_wellington_north() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "north")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Wellington, West")]
    fn validate_wellington_west() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "west")?;
        Ok(get_validator(expected, found))
    }

    #[valid("Exterior Incident Long Wave Radiation - Wellington, East")]
    fn validate_wellington_east() -> Result<ValidFunc, String> {
        let (expected, found) = get_simple_results(CITY, "east")?;
        Ok(get_validator(expected, found))
    }

    validator.push(validate_wellington_south()?);
    validator.push(validate_wellington_north()?);
    validator.push(validate_wellington_west()?);
    validator.push(validate_wellington_east()?);
    Ok(())
}

#[test]
fn validate_ir_radiation() -> Result<(), String> {
    // cargo test --package light --test validate_ir_radiation -- validate_ir_radiation --exact --nocapture
    let mut validator = Validator::new(
        "Validate Longwave (i.e., IR) Radiation",
        "../docs/validation/incident_ir_radiation.html",
    );

    barcelona(&mut validator)?;
    wellington(&mut validator)?;

    validator.validate()
}
