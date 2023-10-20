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

use clap::Parser;
use simple::control_trait::SimpleControl;
use simple::run_simulation::*;
use simple::void_control::VoidControl;
use simple::OccupantBehaviour;
use simple::{Model, SimulationStateHeader};

fn run_sim<C>(
    model: &Model,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
    controller: C,
) -> Result<(), String>
where
    C: SimpleControl,
{
    match &options.output {
        Some(v) => {
            let out = std::fs::File::create(v).map_err(|e| format!("{}", e))?;
            run(model, state_header, options, out, controller)
        }
        None => run(
            model,
            state_header,
            options,
            std::io::stdout().lock(),
            controller,
        ),
    }
}

fn choose_controller(
    model: Model,
    state_header: &mut SimulationStateHeader,
    options: &SimOptions,
) -> Result<(), String> {
    match &options.control_file {
        None => {
            let controller = VoidControl {};
            run_sim::<VoidControl>(&model, state_header, options, controller)
        }
        Some(v) => match v.as_str() {
            "people" => {
                let controller = OccupantBehaviour::new(&model)?;
                run_sim::<OccupantBehaviour>(&model, state_header, options, controller)
            }
            _ => {
                if let Some(control_file) = &options.control_file {
                    match &options.output {
                        Some(v) => {
                            let out = std::fs::File::create(v).map_err(|e| format!("{}", e))?;
                            run_rhai(model, state_header, options, control_file, out)
                        }
                        None => run_rhai(
                            model,
                            state_header,
                            options,
                            control_file,
                            std::io::stdout().lock(),
                        ),
                    }
                } else {
                    unreachable!()
                }
            }
        },
    }
}

fn main() {
    // cargo instruments --release --template Allocations -- -i tests/cold_apartment/cold.spl -w tests/wellington.epw -n 1 -o check.csv
    // cargo instruments --release --template 'CPU Profiler' --package simple --bin simple -- -i tests/cold_apartment/cold.spl -w tests/wellington.epw -n 1 -o check.csv
    // time cargo run --release --package simple --bin simple -- -i tests/cold_apartment/cold.spl -w tests/wellington.epw -n 1 -o check.csv
    
    // time cargo run --release --package simple --bin simple -- -i tests/neighbours/neighbours.json -w tests/wellington.epw -n 1 -o check.csv

    let options = SimOptions::parse();
    let filename = options.input_file.to_string();

    let (model, mut state_header) = if filename.ends_with(".spl") {
        match Model::from_file(filename) {
            Ok(o) => o,
            Err(e) => {
                simple::error_msgs::print_error("", e);
                std::process::exit(1);
            }
        }
    } else if filename.ends_with(".json") {        
        match Model::from_json_file(filename) {
            Ok(o) => o,
            Err(e) => {
                simple::error_msgs::print_error("", e);
                std::process::exit(1);
            }
        }
    } else {
        simple::error_msgs::print_error(
            "Unkown kind of file '{}'... expecting .json or .spl",
            filename,
        );
        // I am not sure what this number should be/
        std::process::exit(1);
    };

    if let Err(e) = choose_controller(model, &mut state_header, &options) {
        simple::error_msgs::print_error("", e);
        // I am not sure what this number should be/
        std::process::exit(1);
    }
}
