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

use calendar::Date;
use model::rhai_api::register_control_api;
use model::{Model, SimulationState};
use rhai::{AST, Engine, Map, Scope};
use std::fs::{self};
use std::sync::{Arc, Mutex};

/// A controller that adapts the state of the building based on a user-defined
/// script written in [Rhai](https://rhai.rs) programming language.
///
/// This is quite a powerful feature as it allows the user to specify quite complex
/// control algorithms.
///
/// The script runs once before every timestep. Besides the API for reading
/// and operating the building (e.g., `space("Bedroom")`), it can use:
///
/// * `month`, `day` and `hour`: the date at the end of the timestep being
///   simulated (`hour` goes from 0 to 24, so midnight is `24.0` of the day
///   that ends)
/// * `memory`: a map whose contents persist from one timestep to the
///   next (all other variables are lost when the script ends)
///
/// `print()` and `debug()` write to `stderr`, so they do not get mixed with
/// the results.
pub struct RhaiControlScript {
    ast: AST,
    engine: Engine,
    /// The `memory` map, kept between runs of the script
    memory: Map,
}

impl RhaiControlScript {
    /// Creates a new Rhai-based controller from a Rhai file
    pub fn from_file(
        model: &Arc<Model>,
        state: SimulationState,
        control_file: &String,
        research_mode: bool,
    ) -> Result<(Self, Arc<Mutex<SimulationState>>), String> {
        let control_script = match fs::read_to_string(control_file) {
            Ok(v) => v,
            Err(_) => return Err(format!("Could not read Rhai script '{}'", control_file)),
        };

        Self::new(model, state, control_script, research_mode)
    }
    /// Creates a new Rhai-based controller from a Rhai script
    pub fn new(
        model: &Arc<Model>,
        state: SimulationState,
        control_script: impl AsRef<str>, //&String,
        research_mode: bool,
    ) -> Result<(Self, Arc<Mutex<SimulationState>>), String> {
        // Register API
        let mut engine = rhai::Engine::new();
        // Keep stdout for the results
        engine.on_print(|s| eprintln!("{s}"));
        engine.on_debug(|s, _source, pos| eprintln!("[debug] {pos:?}: {s}"));

        let state = Arc::new(Mutex::new(state));
        let model = Arc::new(model);
        register_control_api(&mut engine, &model, &state, research_mode);
        let ast = match engine.compile(control_script) {
            Ok(v) => v,
            Err(e) => return Err(format!("Rhai {}", e)),
        };

        Ok((
            Self {
                ast,
                engine,
                memory: Map::new(),
            },
            state,
        ))
    }

    /// Runs the control script for the timestep ending at `date`
    pub fn control(&mut self, date: Date) -> Result<(), String> {
        // Dates come from the weather, whose records are slightly earlier
        // than the full hour (e.g., 0.999999 for 1:00)
        let hour = (date.hour as rhai::FLOAT * 1e4).round() / 1e4;

        let mut scope = Scope::new();
        scope.push_constant("month", date.month as rhai::INT);
        scope.push_constant("day", date.day as rhai::INT);
        scope.push_constant("hour", hour);
        scope.push("memory", std::mem::take(&mut self.memory));

        let res = self.engine.run_ast_with_scope(&mut scope, &self.ast);

        self.memory = scope
            .remove::<Map>("memory")
            .ok_or("Rhai: the variable 'memory' must remain a map (e.g., `memory.x = 1`)")?;
        res.map_err(|e| format!("Rhai {}", e))
    }
}
