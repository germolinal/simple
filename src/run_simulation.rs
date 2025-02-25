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
use crate::control_trait::SimpleControl;
use crate::Float;
use crate::RhaiControlScript;
use calendar::Period;
use clap::Parser;
use communication::{MetaOptions, SimulationModel};
use model::{Model, SimulationStateHeader};
use serde_json;
use std::borrow::Borrow;

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

    /// The control script
    #[arg(short, long)]
    pub control_file: Option<String>,

    /// Specifies the path to which to write the results.
    /// If none is given, STDOUT is used
    #[arg(short, long)]
    pub output: Option<String>,

    /// Enable research mode, allowing some unrealistic
    /// but very powerful functions in the API
    #[arg(short, long)]
    pub research_mode: bool,

    // /// The starting date
    // #[clap(short = 's')]
    // pub start: Date,

    // /// The final date
    // #[clap(short = 'e')]
    // pub end: Date,
    /// The number of timesteps per hour in the simulation
    #[arg(short, long, default_value_t = 1)]
    pub n: usize,
}

struct PreProcessData {
    sim_period: Period,
    report_indexes: Vec<usize>,
    full_header: Vec<String>,
    model: MultiphysicsModel,
    weather: Weather,
}

fn pre_process(
    model: &Model,
    options: &SimOptions,
    state_header: &mut SimulationStateHeader,
) -> Result<PreProcessData, String> {
    const MAX_N: usize = 60;
    let n = if options.n > MAX_N {
        eprintln!("The maximum allowed value for -n param is {}... n has been automatically limited to that value", MAX_N);
        MAX_N
    } else if options.n == 0 {
        return Err("Parameter 'n' should be larger than 0".to_string());
    } else {
        options.n
    };

    let dt = 60. * 60. / n as Float;

    // Load weather
    let mut weather: Weather = match &options.weather_file {
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

    let start = weather.data[0].date;
    let mut end = weather.data[weather.data.len() - 1].date;
    end.hour -= 0.1;

    let sim_period = Period::new(start, end, dt);

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

    let report_indexes: Vec<usize> = model
        .borrow()
        .outputs
        .iter()
        .filter_map(|item| {
            full_header.iter().position(|x| {
                x == &serde_json::to_string(item)
                    .expect("There was an error interpreting the inputs")
            })
        })
        .collect();

    let report_indexes = if report_indexes.is_empty() {
        (0..full_header.len()).collect()
    } else {
        report_indexes
    };

    Ok(PreProcessData {
        sim_period,
        report_indexes,
        full_header,
        weather,
        model: physics_model,
    })
}

/// This function drives the simulation, after having parsed and built
/// the Building, State and Peoeple.
pub fn run<T, C, M>(
    model: M,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
    mut out: T,
    controller: C,
) -> Result<(), String>
where
    T: std::io::Write,
    C: SimpleControl,
    M: Borrow<Model>,
{
    let pre_process_data = pre_process(model.borrow(), options, state_header)?;

    let mut state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;

    let report_len = if model.borrow().outputs.is_empty() {
        state_header.elements.len()
    } else {
        model.borrow().outputs.len()
    };

    let mut memory = pre_process_data.model.allocate_memory(&state)?;

    // Write header
    let _u = out
        .write(b"Date,")
        .expect("Could not write to output file (header => 'Date').");
    for (index, i) in pre_process_data.report_indexes.iter().enumerate() {
        let s: &String = &pre_process_data.full_header[*i];
        if index < report_len - 1 {
            let s = format!("{s},");
            let _u = out
                .write(s.as_bytes())
                .expect("Could not write to output file (header).");
        } else {
            let _u = out
                .write(s.as_bytes())
                .expect("Could not write to output file (header).");
        }
    }
    let _u = out
        .write(b"\n")
        .expect("Could not write to output file (header newline).");

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
        let ds = format!("{},", date);
        let _u = out
            .write(ds.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write to output file (Date '{}')", date));

        for (index, i) in pre_process_data.report_indexes.iter().enumerate() {
            let st = if index < report_len - 1 {
                format!("{:.3},", state[*i])
            } else {
                format!("{:.3}", state[*i])
            };
            let _u = out
                .write(st.as_bytes())
                .expect("Could not write to output file (data)");
        }
        let _u = out
            .write(b"\n")
            .expect("Could not write to output file (newline)");
    }

    Ok(())
}

/// This function drives the simulation, after having parsed and built
/// the Building, State and Peoeple.
pub fn run_rhai<T>(
    model: Model,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
    control_file: &String,
    mut out: T,
) -> Result<(), String>
where
    T: std::io::Write,
{
    let model = std::sync::Arc::new(model);

    let pre_process_data = pre_process(model.borrow(), options, state_header)?;

    let state = state_header
        .take_values()
        .ok_or("Could not take values from SimulationStateHeader")?;
    let mut memory = pre_process_data.model.allocate_memory(&state)?;

    let (controller, state) =
        RhaiControlScript::from_file(&model, state, control_file, options.research_mode)?;

    let report_len = if model.outputs.is_empty() {
        state_header.elements.len()
    } else {
        model.outputs.len()
    };

    // Write header
    let _u = out
        .write(b"Date,")
        .expect("Could not write to output file (header => 'Date').");
    for (index, i) in pre_process_data.report_indexes.iter().enumerate() {
        let s: &String = &pre_process_data.full_header[*i];
        if index < report_len - 1 {
            let s = format!("{s},");
            let _u = out
                .write(s.as_bytes())
                .expect("Could not write to output file (header).");
        } else {
            let _u = out
                .write(s.as_bytes())
                .expect("Could not write to output file (header).");
        }
    }
    let _u = out
        .write(b"\n")
        .expect("Could not write to output file (header newline).");

    /* ************************************ */
    /* SIMULATE THE WHOLE SIMULATION PERIOD */
    /* ************************************ */
    let mut last_reported_month: u8 = 99;

    for date in pre_process_data.sim_period {
        if date.month != last_reported_month {
            last_reported_month = date.month;
            eprintln!("  ... Simulating month {}", last_reported_month);
        }

        controller.control()?;

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
        let ds = format!("{},", date);
        let _u = out
            .write(ds.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write to output file (Date '{}')", date));

        for (index, i) in pre_process_data.report_indexes.iter().enumerate() {
            let st = if index < report_len - 1 {
                format!("{:.3},", state_lock[*i])
            } else {
                format!("{:.3}", state_lock[*i])
            };
            let _u = out
                .write(st.as_bytes())
                .expect("Could not write to output file (data)");
        }
        std::mem::drop(state_lock); // drop mutex
        let _u = out
            .write(b"\n")
            .expect("Could not write to output file (newline)");
    }

    Ok(())
}

/***********/
/* TESTING */
/***********/
