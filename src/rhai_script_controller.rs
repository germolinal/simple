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

use model::rhai_api::register_control_api;
use model::{Model, SimulationState};
use rhai::{Engine, AST};
use std::fs::{self};
use std::sync::{Arc, Mutex};

/// A controller that adapts the state of the building based on a user-defined
/// script written in [Rhai](https://rhai.rs) programming language.
///
/// This is quite a powerful feature as it allows the user to specify quite complex
/// control algorythms.
pub struct RhaiControlScript {
    ast: AST,
    engine: Engine,
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

        let state = Arc::new(Mutex::new(state));
        let model = Arc::new(model);
        register_control_api(&mut engine, &model, &state, research_mode);
        let ast = match engine.compile(control_script) {
            Ok(v) => v,
            Err(e) => return Err(format!("Rhai {}", e)),
        };

        Ok((Self { ast, engine }, state))
    }

    /// Runs a control script
    pub fn control(&self) -> Result<(), String> {
        // Control
        if let Err(e) = self.engine.eval_ast::<()>(&self.ast) {
            return Err(format!("Rhai {}", e));
        }
        Ok(())
    }
}
