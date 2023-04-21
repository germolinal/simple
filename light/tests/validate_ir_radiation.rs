use calendar::Date;
use communication::{MetaOptions, SimulationModel};
use light::{Float, SolarModel};
use schedule::ScheduleConstant;
use model::SolarOptions;
use test_models::*;
use validate::{valid, ScatterValidator, Validate, Validator};
use weather::SyntheticWeather;
const SIGMA: Float = 5.670374419e-8;
fn get_validator(expected: Vec<f64>, found: Vec<f64>) -> Box<ScatterValidator> {
    // Box::new(SeriesValidator {
    //     x_label: Some("Timestep".into()),
    //     y_label: Some("Longwave Rad.".into()),
    //     y_units: Some("W/m2"),
    //     expected_legend: Some("EnergyPlus"),
    //     found_legend: Some("SIMPLE"),
    //     expected,
    //     found,
    //     ..validate::SeriesValidator::default()
    // })
    Box::new(ScatterValidator {
        // expected_label: Some("Timestep".into()),
        // expected: Some("Longwave Rad.".into()),
        units: Some("W/m2"),
        expected_legend: Some("EnergyPlus"),
        found_legend: Some("SIMPLE"),
        expected,
        found,
        ..validate::ScatterValidator::default()
    })
}

fn get_simple_results(city: &str, orientation_str: &str) -> (Vec<f64>, Vec<f64>) {
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
        b"wellington" => (-41.3, 174.78, 180.),
        b"barcelona" => (41.28, 2.07, 15.), // ??? GMT + 1
        _ => panic!("Unsupported city '{}'", city),
    };

    let meta_options = MetaOptions {
        latitude: lat.to_radians(),
        longitude: lon.to_radians(),
        standard_meridian: std_mer.to_radians(),
        elevation: 0.0,
    };

    let zone_volume = 600.;

    let (model, mut state_header) =
        get_single_zone_test_building(&SingleZoneTestBuildingOptions {
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
        .set_n_solar_irradiance_points(10)
        .set_solar_ambient_divitions(3000)
        .set_solar_sky_discretization(1);

    let n: usize = 20;
    let solar_model =
        SolarModel::new(&meta_options, options, &model, &mut state_header, n).unwrap();
    let mut state = state_header.take_values().unwrap();
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
        let expected_v = gain / surface_area / emmisivity + SIGMA * (ts + 273.15).powi(4);

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
        solar_model
            .march(date, &weather, &model, &mut state,&mut ())
            .unwrap();

        let front_radiation = surface.front_ir_irradiance(&state).unwrap();
        found.push(front_radiation);
        expected.push(expected_v);

        // Advance
        date.add_hours(1. / n as f64);
        // assert!(false)
    }
    (expected, found)
}

fn barcelona(validator: &mut Validator) {
    const CITY: &'static str = "barcelona";

    #[valid(Exterior Incident Long Wave Radiation - Barcelona, South)]
    fn validate_barcelona_south() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "south");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Barcelona, North)]
    fn validate_barcelona_north() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "north");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Barcelona, West)]
    fn validate_barcelona_west() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "west");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Barcelona, East)]
    fn validate_barcelona_east() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "east");
        get_validator(expected, found)
    }

    validator.push(validate_barcelona_south());
    validator.push(validate_barcelona_north());
    validator.push(validate_barcelona_west());
    validator.push(validate_barcelona_east());
}

fn wellington(validator: &mut Validator) {
    const CITY: &'static str = "wellington";

    #[valid(Exterior Incident Long Wave Radiation - Wellington, South)]
    fn validate_wellington_south() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "south");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Wellington, North)]
    fn validate_wellington_north() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "north");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Wellington, West)]
    fn validate_wellington_west() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "west");
        get_validator(expected, found)
    }

    #[valid(Exterior Incident Long Wave Radiation - Wellington, East)]
    fn validate_wellington_east() -> Box<dyn Validate> {
        let (expected, found) = get_simple_results(CITY, "east");
        get_validator(expected, found)
    }

    validator.push(validate_wellington_south());
    validator.push(validate_wellington_north());
    validator.push(validate_wellington_west());
    validator.push(validate_wellington_east());
}

#[test]
fn validate_ir_radiation() {
    // cargo test --package light --test validate_ir_radiation -- validate_ir_radiation --exact --nocapture
    let mut validator = Validator::new(
        "Validate Longwave (i.e., IR) Radiation",
        "./docs/validation/incident_ir_radiation.html",
    );

    barcelona(&mut validator);
    wellington(&mut validator);

    validator.validate().unwrap();
}
