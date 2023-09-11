use geometry::Vector3D;
use heat::convection::ConvectionParams;
use heat::surface::is_windward;
use heat::Float;
use validate::*;

fn get_validator(expected: Vec<Float>, found: Vec<Float>) -> Box<dyn Validate> {
    Box::new(SeriesValidator {
        x_label: Some("time step"),
        y_label: Some("Convection Coefficient"),
        y_units: Some("W/m2.K"),

        expected_legend: Some("EnergyPlus (TARP)"),
        found_legend: Some("SIMPLE"),
        expected,
        found,

        allowed_mean_bias_error: Some(0.03),
        allowed_root_mean_squared_error: Some(0.01),
        ..validate::SeriesValidator::default()
    })

    // Box::new(ScatterValidator {
    //     // x_label: Some("time step"),
    //     // y_label: Some("Convection Coefficient"),
    //     units: Some("W/m2.K"),

    //     expected_legend: Some("EnergyPlus (TARP)"),
    //     found_legend: Some("SIMPLE"),
    //     expected,
    //     found,
    //     ..validate::ScatterValidator::default()
    // })
}

fn calc_convection(
    dir: &'static str,
    area: Float,
    perimeter: Float,
    normal: Vector3D,
) -> (Vec<Float>, Vec<Float>, Vec<Float>, Vec<Float>) {
    let path_string = format!("./tests/{}/eplusout.csv", dir);
    let path = path_string.as_str();
    let cols = validate::from_csv::<Float>(path, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

    // let site_wind_speed = &cols[0];             // 1	Environment:Site Wind Speed [m/s](TimeStep)
    let site_wind_direction = &cols[1]; // 2	Environment:Site Wind Direction [deg](TimeStep)
                                        // let incident_solar_radiation = &cols[2];    // 3	WALL EXTERIOR:Surface Outside Face Incident Solar Radiation Rate per Area [W/m2](TimeStep)
    let inside_surface_temp = &cols[3]; // 4	WALL EXTERIOR:Surface Inside Face Temperature [C](TimeStep)
    let outside_surface_temp = &cols[4]; // 5	WALL EXTERIOR:Surface Outside Face Temperature [C](TimeStep)
    let exp_hs_in = &cols[5]; // 6	WALL EXTERIOR:Surface Inside Face Convection Heat Transfer Coefficient [W/m2-K](TimeStep)
                              // let indoor_thermal_heat_gain = &cols[6];    // 7	WALL EXTERIOR:Surface Inside Face Net Surface Thermal Radiation Heat Gain Rate [W](TimeStep)
    let outdoor_temp = &cols[7]; // 8	WALL EXTERIOR:Surface Outside Face Outdoor Air Drybulb Temperature [C](TimeStep)
    let surface_wind_speed = &cols[8]; // 9	WALL EXTERIOR:Surface Outside Face Outdoor Air Wind Speed [m/s](TimeStep)
    let exp_hs_out = &cols[9]; // 10	WALL EXTERIOR:Surface Outside Face Convection Heat Transfer Coefficient [W/m2-K](TimeStep)
                               // let outdoor_thermal_heat_gain = &cols[10];   // 11	WALL EXTERIOR:Surface Outside Face Net Thermal Radiation Heat Gain Rate [W](TimeStep)
    let zone_air_temp = &cols[11]; // 12	INTERIOR SPACE:Zone Mean Air Temperature [C](TimeStep)

    let cos_tilt = normal * Vector3D::new(0., 0., 1.);
    let n = outdoor_temp.len();
    let mut found_hs_in = Vec::with_capacity(n);
    let mut found_hs_out = Vec::with_capacity(n);
    for i in 0..n {
        let env_in = ConvectionParams {
            air_temperature: zone_air_temp[i],
            air_speed: surface_wind_speed[i], //0.,
            rad_temperature: 0.0,             // not used for convection purposes
            surface_temperature: inside_surface_temp[i],
            roughness_index: 1,
            cos_surface_tilt: cos_tilt,
        };
        let env_out = ConvectionParams {
            air_temperature: outdoor_temp[i],
            air_speed: surface_wind_speed[i],
            rad_temperature: 0.0, // not used for convection purposes
            surface_temperature: outside_surface_temp[i],
            roughness_index: 1,
            cos_surface_tilt: -cos_tilt,
        };

        let windward = is_windward(site_wind_direction[i].to_radians(), cos_tilt, normal);

        found_hs_in.push(env_in.get_tarp_natural_convection_coefficient());
        found_hs_out.push(env_out.get_tarp_convection_coefficient(area, perimeter, windward))
    }

    (
        exp_hs_in.clone(),
        found_hs_in,
        exp_hs_out.clone(),
        found_hs_out,
    )
}

const AREA: Float = 20. * 3.;
const PERIMETER: Float = (20. + 3.) * 2.; //30.9838667697;
fn vertical(validations: &mut Validator) {
    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Vertical Wall - Natural (i.e., Interior) Convection Coefficient )]
    fn natural() -> Box<dyn Validate> {
        let (expected_in, found_in, ..) = calc_convection(
            "massive_full",
            AREA,
            PERIMETER,
            Vector3D::new(0., -1., 0.), // South
        );
        get_validator(expected_in, found_in)
    }

    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Vertical Wall - Forced (i.e., Exterior) Convection Coefficient )]
    fn forced() -> Box<dyn Validate> {
        let (.., expected_out, found_out) = calc_convection(
            "massive_full",
            AREA,
            PERIMETER,
            Vector3D::new(0., -1., 0.), // South
        );
        get_validator(expected_out, found_out)
    }

    validations.push(natural());
    validations.push(forced());
}

fn tilted(validations: &mut Validator) {
    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Tilted Wall - Natural (i.e., Interior) Convection Coefficient )]
    fn natural() -> Box<dyn Validate> {
        let (expected_in, found_in, ..) = calc_convection(
            "tilted",
            AREA,
            PERIMETER,
            Vector3D::new(0., -1., 1.).get_normalized(), // South, tilted
        );
        get_validator(expected_in, found_in)
    }

    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Tilted Wall - Forced (i.e., Exterior) Convection Coefficient )]
    fn forced() -> Box<dyn Validate> {
        let (.., expected_out, found_out) = calc_convection(
            "tilted",
            AREA,
            PERIMETER,
            Vector3D::new(0., -1., 1.).get_normalized(), // South, tilted
        );
        get_validator(expected_out, found_out)
    }

    validations.push(natural());
    validations.push(forced());
}

fn horizontal(validations: &mut Validator) {
    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Horizontal Wall - Natural (i.e., Interior) Convection Coefficient )]
    fn natural() -> Box<dyn Validate> {
        let (expected_in, found_in, ..) = calc_convection(
            "horizontal",
            AREA,
            PERIMETER,
            Vector3D::new(0., 0., 1.).get_normalized(), // South, tilted
        );
        get_validator(expected_in, found_in)
    }

    /// Heat Transfer Coefficients calculated in SIMPLE, compared to those calculated by the TARP model in EnergyPlus
    #[valid(Horizontal Wall - Forced (i.e., Exterior) Convection Coefficient )]
    fn forced() -> Box<dyn Validate> {
        let (.., expected_out, found_out) = calc_convection(
            "horizontal",
            AREA,
            PERIMETER,
            Vector3D::new(0., 0., 1.).get_normalized(), // South, tilted
        );
        get_validator(expected_out, found_out)
    }

    validations.push(natural());
    validations.push(forced());
}

#[test]
fn validate() -> Result<(), String> {
    // cargo test --package heat --test validate_convection -- validate --exact --nocapture
    let p = "../docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }

    let target_file = format!("{}/convection_coefficients.html", p);
    let mut validations = Validator::new(
        "SIMPLE Heat - Convection Coefficients Validation Report",
        &target_file,
    );

    vertical(&mut validations);
    tilted(&mut validations);
    horizontal(&mut validations);

    validations.validate()?;

    Ok(())
}
