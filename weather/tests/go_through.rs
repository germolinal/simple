use calendar::{Date, DateFactory};
use validate::*;
use weather::{EPWWeather, Float, Weather};

#[test]
fn test_go_through() {
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
            hour: 23.,
        };

        let dt = 60. * 60. / 20.;
        let sim_period = DateFactory::new(start, end, dt);
        
        let weather: Weather = EPWWeather::from_file("./tests/wellington.epw").unwrap().into();

        let mut i = 0;
        
        let mut expected = Vec::with_capacity(n);
        let mut found = Vec::with_capacity(n);
        for date in sim_period {
            expected.push(exp_dry_bulb[i] as Float);
            i += 1;
            let data = weather.find_weather_line(date);
            let found_temp = data.dry_bulb_temperature;
            found.push(found_temp);

            if i >= n || i >= exp_dry_bulb.len() {
                break;
            }
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
        std::fs::create_dir(p).unwrap();
    }

    let target_file = format!("{}/weather.html", p);
    let mut validations = Validator::new(
        "SIMPLE EPW File Parser",
        &target_file,
    );

    validations.push(drybulb());

    validations.validate().unwrap();
}
