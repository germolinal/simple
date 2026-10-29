/*
MIT License
Copyright (c)  Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use crate::Float;
use crate::RhaiControlScript;
use crate::control_trait::SimpleControl;
use calendar::{Date, DayOfWeek};
use clap::Parser;
use communication::{MetaOptions, SimulationModel};
use model::{Model, SimulationStateHeader};
use serde_json;
use std::borrow::Borrow;
use std::io::{BufWriter, Write};

use crate::multiphysics_model::MultiphysicsModel;
use std::fs::{self};
use weather::{EPWWeather, Weather};

/// SIMPLE aims to be a modern building simulation system. (c) Wise House | Germán Molina
#[derive(Debug, Parser, Default)]
#[clap(author, version, about, long_about = None)]
pub struct SimOptions {
    /// Does not simulate, just reads the model to see whether
    /// it can be parsed appropriately
    #[arg(long)]
    pub check: bool,

    /// Translates the model from .JSON to .SPL, or the other way around.
    #[arg(short, long)]
    pub translate: Option<String>,

    /// The input simple file
    #[arg(short, long)]
    pub input_file: String,

    /// The EPW weather file
    #[arg(short, long)]
    pub weather_file: Option<String>,

    /// The control: a Rhai script, or `people` for the built-in controller
    /// that switches heaters/coolers on and off based on their setpoints.
    /// With no control, nothing is ever turned on.
    #[arg(short, long)]
    pub control_file: Option<String>,

    /// Specifies the path to which to write the results.
    /// If none is given, STDOUT is used
    #[arg(short, long)]
    pub output: Option<String>,

    /// Enable research mode, allowing control scripts to do things that are
    /// impossible in real life but useful for research (e.g., setting the
    /// temperature of a space). Without it, scripts can only operate the
    /// building (e.g., turn heaters on and off, open windows)
    #[arg(short, long)]
    pub research_mode: bool,

    // /// The starting date
    // #[clap(short = 's')]
    // pub start: Date,

    // /// The final date
    // #[clap(short = 'e')]
    // pub end: Date,
    /// The seed for the random numbers of the control script (i.e., `random()`),
    /// for repeatable simulations. Without it, every simulation gets
    /// different random numbers
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// The number of timesteps per hour in the simulation (1 to 60). The
    /// control script runs, and results are reported, every timestep
    #[arg(short, long, default_value_t = 1)]
    pub n: usize,
}

/// The timesteps of a simulation, as the [`Date`] at the end of each of them.
///
/// Weather records (e.g., in EPW files) describe the interval that
/// ends at their timestamp, so the simulation starts one record-interval
/// before the first record, and ends at the last one. For example, an
/// hourly EPW file simulated with `n = 4` gives the timesteps ending at
/// 00:15, 00:30, ..., 24:00 of every day (i.e., `8760 * 4` timesteps),
/// just like EnergyPlus.
#[derive(Clone, Copy, Debug)]
pub struct SimulationPeriod {
    /// The date of the first record in the weather
    first_record: Date,
    /// The hours between the end of the first timestep and the
    /// `first_record` (i.e., zero or negative)
    first_offset: Float,
    /// The timestep, in hours
    dt: Float,
    /// The number of timesteps
    n_steps: usize,
    /// The next timestep to return
    current: usize,
    /// The day of the week of the `first_record`, if known
    start_day_of_week: Option<DayOfWeek>,
}

impl SimulationPeriod {
    /// Builds the timesteps covering a [`Weather`] (which must have data)
    /// with `n` timesteps per hour
    fn new(weather: &Weather, n: usize) -> Self {
        let first_record = weather.data[0].date;
        let last_record = weather.data[weather.data.len() - 1].date;

        // hours between two dates, going forward (i.e., through new year, if needed)
        let hours_between = |from: Date, to: Date| -> Float {
            let mut days = to - from;
            if days < 0. {
                days += 365.;
            }
            days * 24.
        };

        // The interval described by each record
        let record_interval = if weather.data.len() > 1 {
            hours_between(first_record, weather.data[1].date)
        } else {
            1.0
        };

        let dt = 1. / n as Float;
        // The first timestep ends `dt` after the start of the first record's
        // interval... but do not go back past midnight of the first day
        let first_offset = (dt - record_interval).max(-first_record.hour);
        // ... and the last one ends at the last record
        let last_offset = hours_between(first_record, last_record);

        Self {
            first_record,
            first_offset,
            dt,
            n_steps: ((last_offset - first_offset) / dt).round() as usize + 1,
            current: 0,
            start_day_of_week: weather.start_day_of_week,
        }
    }

    /// The day of the week of a `date` within this period, if the weather
    /// says what day of the week it starts on
    pub fn day_of_week(&self, date: Date) -> Option<DayOfWeek> {
        let mut days = date.day_of_year().floor() - self.first_record.day_of_year().floor();
        if days < 0. {
            days += 365.;
        }
        self.start_day_of_week.map(|d| d.add_days(days as usize))
    }
}

impl Iterator for SimulationPeriod {
    type Item = Date;

    fn next(&mut self) -> Option<Date> {
        if self.current >= self.n_steps {
            return None;
        }
        let hours = self.first_offset + self.current as Float * self.dt;
        self.current += 1;

        let mut date = self.first_record;
        if hours < 0. {
            // Before the first record, but on the same day (see `first_offset`)
            date.hour += hours;
        } else {
            date.add_hours(hours);
        }
        Some(date)
    }
}

/// Everything needed to march a [`Model`] through a weather file, as
/// built by [`prepare`].
pub struct PreProcessData {
    /// The timesteps to simulate (i.e., the whole weather file)
    pub sim_period: SimulationPeriod,
    /// The indexes (within the [`SimulationState`](model::SimulationState))
    /// of the elements to report
    pub report_indexes: Vec<usize>,
    /// The name of every element in the
    /// [`SimulationState`](model::SimulationState)
    pub full_header: Vec<String>,
    /// The physics model
    pub model: MultiphysicsModel,
    /// The weather driving the simulation
    pub weather: Weather,
    /// The location of the simulation (from the weather)
    pub meta_options: MetaOptions,
}

fn pre_process(
    model: &Model,
    options: &SimOptions,
    state_header: &mut SimulationStateHeader,
) -> Result<PreProcessData, String> {
    // Load weather
    let weather: Weather = match &options.weather_file {
        None => {
            return Err("No weather file specified".to_string());
        }
        Some(file) => {
            if file.ends_with(".epw") {
                EPWWeather::from_file(file.to_string())?.into()
            } else if file.ends_with(".sw") {
                let s = match fs::read_to_string(file) {
                    Ok(v) => v,
                    Err(_) => return Err(format!("Could not read JSON file '{}'", file)),
                };
                serde_json::from_str(&s).map_err(|e| format!("{}", e))?
            } else {
                return Err(format!("Unsupported weather format in file '{}'", file));
            }
        }
    };

    prepare(model, weather, options.n, state_header)
}

/// Builds the physics model (with `n` timesteps per hour) and works out the
/// simulation period and the reported variables, given an already loaded
/// [`Weather`]
pub fn prepare(
    model: &Model,
    mut weather: Weather,
    n: usize,
    state_header: &mut SimulationStateHeader,
) -> Result<PreProcessData, String> {
    const MAX_N: usize = 60;
    let n = if n > MAX_N {
        eprintln!(
            "The maximum allowed value for -n param is {}... n has been automatically limited to that value",
            MAX_N
        );
        MAX_N
    } else if n == 0 {
        return Err("Parameter 'n' should be larger than 0".to_string());
    } else {
        n
    };

    if weather.data.is_empty() {
        return Err("The weather has no data".to_string());
    }

    // (before sorting, so the first record is the first one in time)
    let sim_period = SimulationPeriod::new(&weather, n);

    weather.sort_data();

    let meta_options = MetaOptions {
        latitude: weather.location.latitude,
        longitude: weather.location.longitude,
        standard_meridian: (weather.location.timezone as Float * 15.).to_radians(),
        elevation: weather.location.elevation,
    };

    // Create physics model
    let physics_model = MultiphysicsModel::new(&meta_options, (), model, state_header, n)?;

    // Collect variables we need to report
    let full_header: Vec<String> = state_header
        .elements
        .iter()
        .map(|x| x.stringify(model))
        .collect();

    // Report everything, unless specific outputs were requested
    let outputs = &model.borrow().outputs;
    let report_indexes: Vec<usize> = if outputs.is_empty() {
        (0..full_header.len()).collect()
    } else {
        outputs
            .iter()
            .map(|item| {
                let item = serde_json::to_string(item).map_err(|e| e.to_string())?;
                full_header.iter().position(|x| x == &item).ok_or_else(|| {
                    format!("Output {item} does not match anything that is being simulated")
                })
            })
            .collect::<Result<_, String>>()?
    };

    Ok(PreProcessData {
        sim_period,
        report_indexes,
        full_header,
        weather,
        model: physics_model,
        meta_options,
    })
}

/// Writes the header of the results: `Date`, followed by the name of each
/// reported element.
fn write_header<W: Write>(
    out: &mut W,
    full_header: &[String],
    report_indexes: &[usize],
) -> std::io::Result<()> {
    out.write_all(b"Date,")?;
    for (index, i) in report_indexes.iter().enumerate() {
        let s = &full_header[*i];
        if index < report_indexes.len() - 1 {
            write!(out, "{s},")?;
        } else {
            write!(out, "{s}")?;
        }
    }
    out.write_all(b"\n")
}

/// Writes one row of results: the date, followed by the reported values.
fn write_row<W: Write>(
    out: &mut W,
    date: Date,
    state: &[Float],
    report_indexes: &[usize],
) -> std::io::Result<()> {
    write!(out, "{date},")?;
    for (index, i) in report_indexes.iter().enumerate() {
        if index < report_indexes.len() - 1 {
            write!(out, "{:.3},", state[*i])?;
        } else {
            write!(out, "{:.3}", state[*i])?;
        }
    }
    out.write_all(b"\n")
}

/// This function drives the simulation, after having parsed and built
/// the Building, State and People.
pub fn run<T, C, M>(
    model: M,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
    out: T,
    controller: C,
) -> Result<(), String>
where
    T: std::io::Write,
    C: SimpleControl,
    M: Borrow<Model>,
{
    let mut out = BufWriter::new(out);
    let pre_process_data = pre_process(model.borrow(), options, state_header)?;

    let mut state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;

    let mut memory = pre_process_data.model.allocate_memory(&state)?;

    // Write header
    write_header(
        &mut out,
        &pre_process_data.full_header,
        &pre_process_data.report_indexes,
    )
    .map_err(|e| format!("Could not write to output file (header): {e}"))?;

    /* ************************************ */
    /* SIMULATE THE WHOLE SIMULATION PERIOD */
    /* ************************************ */
    let mut last_reported_month: u8 = u8::MAX;
    for date in pre_process_data.sim_period {
        if date.month != last_reported_month {
            last_reported_month = date.month;
            eprintln!("  ... Simulating month {}", last_reported_month);
        }

        controller.control(model.borrow(), &pre_process_data.model, &mut state)?;

        // Physics
        pre_process_data.model.march(
            date,
            &pre_process_data.weather,
            model.borrow(),
            &mut state,
            &mut memory,
        )?;

        // Print all the values in the state
        write_row(&mut out, date, &state, &pre_process_data.report_indexes)
            .map_err(|e| format!("Could not write to output file (Date '{date}'): {e}"))?;
    }

    out.flush()
        .map_err(|e| format!("Could not write to output file: {e}"))
}

/// This function drives the simulation, after having parsed and built
/// the Building, State and People.
pub fn run_rhai<T>(
    model: Model,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
    control_file: &String,
    out: T,
) -> Result<(), String>
where
    T: std::io::Write,
{
    let mut out = BufWriter::new(out);
    let model = std::sync::Arc::new(model);

    let pre_process_data = pre_process(model.borrow(), options, state_header)?;

    let state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;
    let mut memory = pre_process_data.model.allocate_memory(&state)?;

    let (mut controller, state) =
        RhaiControlScript::from_file(&model, state, control_file, options.research_mode)?;
    if let Some(seed) = options.seed {
        controller.set_seed(seed);
    }

    // Write header
    write_header(
        &mut out,
        &pre_process_data.full_header,
        &pre_process_data.report_indexes,
    )
    .map_err(|e| format!("Could not write to output file (header): {e}"))?;

    /* ************************************ */
    /* SIMULATE THE WHOLE SIMULATION PERIOD */
    /* ************************************ */
    let mut last_reported_month: u8 = 99;

    let sim_period = pre_process_data.sim_period;
    for date in sim_period {
        if date.month != last_reported_month {
            last_reported_month = date.month;
            eprintln!("  ... Simulating month {}", last_reported_month);
        }

        controller.control(date, sim_period.day_of_week(date))?;

        let mut state_lock = (*state).lock().unwrap();
        // Physics
        // let model = model.as_ref();
        // let mut state = (*state).borrow_mut();
        pre_process_data.model.march(
            date,
            &pre_process_data.weather,
            model.borrow(),
            &mut state_lock,
            &mut memory,
        )?;

        // Print all the values in the state
        write_row(
            &mut out,
            date,
            &state_lock,
            &pre_process_data.report_indexes,
        )
        .map_err(|e| format!("Could not write to output file (Date '{date}'): {e}"))?;
        std::mem::drop(state_lock); // drop mutex
    }

    out.flush()
        .map_err(|e| format!("Could not write to output file: {e}"))
}

/***********/
/* TESTING */
/***********/
