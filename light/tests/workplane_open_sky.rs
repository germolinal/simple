// A workplane under an open sky (i.e., over a floor with nothing else around)
// should receive the global horizontal illuminance reported in the weather
// file.

use calendar::Date;
use communication::MetaOptions;
use light::{Float, WorkplaneModel};
use model::Model;
use weather::{EPWWeather, Weather};

#[test]
fn open_sky_illuminance() -> Result<(), String> {
    let epw = EPWWeather::from_file("../tests/wellington.epw")?;
    // EPW values are the averages of the hour before... compare them with
    // the middle of that hour
    let reference: Vec<(Date, Float)> = epw
        .data
        .iter()
        .map(|l| {
            let mut date = l.date();
            date.hour -= 0.5;
            (date, l.global_horizontal_illuminance)
        })
        .collect();
    let mut weather: Weather = epw.into();
    let location = weather.location.clone();
    weather.sort_data();

    let model = Model::from_bytes(
        br#"
        Space { name: "Patio" }
        Substance { type: "Normal", name: "s", thermal_conductivity: 1, density: 1, specific_heat_capacity: 1 }
        Material { name: "m", substance: "s", thickness: 0.1 }
        Construction { name: "c", materials: ["m"] }
        Surface {
            name: "floor",
            construction: "c",
            front_boundary: { type: "Space", space: "Patio" },
            vertices: [0,0,0,  4,0,0,  4,4,0,  0,4,0]
        }
        Workplane {
            name: "Patio",
            space: "Patio",
            spacing: 2, // 4 sensors
            n_ambient_samples: 3000,
        }
        "#,
    )?
    .0;

    let meta = MetaOptions {
        latitude: location.latitude,
        longitude: location.longitude,
        standard_meridian: (location.timezone as Float * 15.).to_radians(),
        elevation: location.elevation,
    };
    let workplanes = WorkplaneModel::new(&meta, &model, WorkplaneModel::DEFAULT_MF)?;
    assert_eq!(workplanes.workplanes()[0].sensors.len(), 4);
    let mut memory = workplanes.allocate_memory();

    let (mut found_total, mut expected_total) = (0.0, 0.0);
    let mut n = 0;
    for (date, expected) in reference.iter().copied() {
        workplanes.march(date, &weather, &mut memory)?;
        let lux = memory.illuminance(0).ok_or("no workplane")?;
        let found = lux.iter().sum::<Float>() / lux.len() as Float;
        found_total += found;
        expected_total += expected;
        n += 1;
    }
    assert_eq!(n, 8760);
    let ratio = found_total / expected_total;
    eprintln!("Annual open-sky illuminance: found/expected = {ratio:.3}");
    assert!((ratio - 1.).abs() < 0.05, "found/expected = {ratio}");
    Ok(())
}
