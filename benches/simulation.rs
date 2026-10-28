//! End-to-end benchmarks: one week of a whole-building simulation, and the
//! daylight-coefficient pre-processing done before a simulation starts.
//!
//! Run with `cargo bench --bench simulation` (add `--features parallel` to
//! compare). Models and weather are the ones used by the validation tests.
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use simple::{
    EPWWeather, MetaOptions, Model, MultiphysicsModel, OpticalInfo, Period, SimulationModel,
    SimulationStateHeader, Weather,
};

const WEATHER: &str = "./tests/wellington.epw";
/// Timesteps per hour, as in the validation tests
const N: usize = 4;

fn load_weather() -> Weather {
    let mut weather: Weather = EPWWeather::from_file(WEATHER.to_string())
        .expect("Could not read weather")
        .into();
    weather.sort_data();
    weather
}

fn meta_options(weather: &Weather) -> MetaOptions {
    MetaOptions {
        latitude: weather.location.latitude,
        longitude: weather.location.longitude,
        standard_meridian: (weather.location.timezone as simple::Float * 15.).to_radians(),
        elevation: weather.location.elevation,
    }
}

/// Marches `model_file` through the first week of the weather file. Optical
/// data is read from the cached file referenced in the model, so this only
/// measures the time-marching.
fn march_one_week(c: &mut Criterion, name: &str, model_file: &str) {
    let weather = load_weather();
    let meta = meta_options(&weather);
    let dt = 60. * 60. / N as simple::Float;
    let start = weather.data[0].date;
    let mut end = start;
    end.add_hours(7. * 24.);

    let mut group = c.benchmark_group("march_one_week");
    group.sample_size(10);
    group.bench_function(name, |b| {
        b.iter_batched(
            || {
                let (model, mut header) = Model::from_file(model_file).unwrap();
                let physics = MultiphysicsModel::new(&meta, (), &model, &mut header, N).unwrap();
                let state = header.take_values().unwrap();
                let memory = physics.allocate_memory(&state).unwrap();
                (model, physics, state, memory)
            },
            |(model, physics, mut state, mut memory)| {
                for date in Period::new(start, end, dt) {
                    physics
                        .march(date, &weather, &model, &mut state, &mut memory)
                        .unwrap();
                }
                state
            },
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

pub fn march_box(c: &mut Criterion) {
    march_one_week(c, "box", "./tests/box/box.spl");
}

pub fn march_cold_apartment(c: &mut Criterion) {
    march_one_week(c, "cold_apartment", "./tests/cold_apartment/cold.spl");
}

/// Calculates daylight coefficients and view factors for the box model, with
/// fewer rays than the validation tests use, so it runs in reasonable time.
pub fn optical_info_box(c: &mut Criterion) {
    let mut group = c.benchmark_group("optical_info");
    group.sample_size(10);
    group.bench_function("box_10pts_300samples", |b| {
        b.iter_batched(
            || {
                let (mut model, header) = Model::from_file("./tests/box/box.spl").unwrap();
                let mut options = model.solar_options.clone().unwrap();
                options
                    .set_n_solar_irradiance_points(10)
                    .set_solar_ambient_divitions(300)
                    .set_solar_sky_discretization(1);
                model.solar_options = Some(options.clone());
                (model, header, options)
            },
            |(model, mut header, options): (Model, SimulationStateHeader, _)| {
                OpticalInfo::new(&options, &model, &mut header).unwrap()
            },
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

criterion_group!(benches, march_box, march_cold_apartment, optical_info_box);
criterion_main!(benches);
