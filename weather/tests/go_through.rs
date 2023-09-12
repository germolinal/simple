use calendar::{Date, Period};
use validate::*;
use weather::{EPWWeather, Float, Weather};

#[test]
fn test_go_through() -> Result<(), String> {
    // cargo test --release --package weather --test go_through -- test_go_through --exact --nocapture

    /// Checks whether `SIMPLE`'s EPW module is interpolating properly
    #[valid("Simple's EPW module vs EnergyPlus - accessing data")]
    fn drybulb() -> Box<dyn Validate> {
        let n = 199999000;

        let cols = validate::from_csv::<Float>("./tests/eplusout.csv", &[1]);
        let exp_dry_bulb = &cols[0];

        let start = Date {
            day: 1,
            month: 1,
            hour: 0.0,
        };

        let end = Date {
            day: 31,
            month: 12,
            hour: 23.99999999,
        };

        let dt = 60. * 60. / 20.;
        let sim_period = Period::new(start, end, dt);

        let weather: Weather = EPWWeather::from_file("./tests/wellington.epw")
            .unwrap()
            .into();

        let mut expected = Vec::with_capacity(n);
        let mut found = Vec::with_capacity(n);
        for (date, exp) in sim_period.into_iter().zip(exp_dry_bulb).skip(20)
        // We skip 20 beacause EnergyPlus seems to be making up data between midningt and 1am?
        {
            expected.push(*exp as Float);
            let data = weather.find_weather_line(date);
            let found_temp = data.dry_bulb_temperature;
            found.push(found_temp);
            // println!("{},{}", date, found_temp);
        }

        Box::new(ScatterValidator {
            // label: Some("time step"),
            units: Some("C"),

            expected_legend: Some("EnergyPlus"),
            found_legend: Some("SIMPLE"),
            allowed_intersect_delta: Some(0.05),
            allowed_slope_delta: Some(0.01),
            allowed_r2: Some(0.99),
            expected,
            found,
            ..validate::ScatterValidator::default()
        })
    }

    let p = "../docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).map_err(|e| e.to_string())?;
    }

    let target_file = format!("{}/weather.html", p);
    let mut validations = Validator::new("SIMPLE EPW File Parser", &target_file);

    validations.push(drybulb());

    validations.validate()?;

    Ok(())
}
