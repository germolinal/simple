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

#![deny(missing_docs)]

//! WebAssembly bindings for `SIMPLE`.
//!
//! There is no file system in the browser, so everything (the model, the
//! weather and the control script) is passed in as text. The
//! [`SimulationState`] is a plain vector of numbers, so it is exposed both as
//! a copy ([`Simulation::state`]) and as a pointer into the WebAssembly
//! memory ([`Simulation::state_ptr`]), which JavaScript can wrap in a
//! `Float64Array` (`Float32Array` with the `float` feature) without copying.

use simple::multiphysics_model::MultiphysicsModelMemory;
use simple::run_simulation::{SimulationPeriod, prepare};
use simple::{
    Date, EPWWeather, Float, IlluminanceRecord, Model, MultiphysicsModel, RhaiControlScript,
    SimulationModel, SimulationState, SimulationStateHeader, Weather, WorkplaneModel,
    WorkplaneModelMemory, WorkplaneSensors,
};
use std::sync::{Arc, Mutex, MutexGuard};
use wasm_bindgen::prelude::*;

/// Parses a model, written either in SIMPLE's own format (`.spl`) or in JSON
/// (anything starting with `{`).
fn parse_model(model: &str) -> Result<(Model, SimulationStateHeader), String> {
    let (mut model, header) = if model.trim_start().starts_with('{') {
        Model::from_json(model)?
    } else {
        Model::from_bytes(model.as_bytes())?
    };

    // Optical data cannot be cached in a file when there is no file system,
    // so it is always calculated
    if let Some(options) = &model.solar_options {
        let mut value = serde_json::to_value(options).map_err(|e| e.to_string())?;
        if let Some(obj) = value.as_object_mut() {
            obj.remove("optical_data_path");
        }
        model.solar_options = Some(serde_json::from_value(value).map_err(|e| e.to_string())?);
    }

    Ok((model, header))
}

/// Parses a weather file, either an EPW or a SIMPLE weather (`.sw`, i.e.,
/// JSON, starting with `{`)
fn parse_weather(weather: &str) -> Result<Weather, String> {
    if weather.trim_start().starts_with('{') {
        serde_json::from_str(weather).map_err(|e| e.to_string())
    } else {
        Ok(EPWWeather::from_bytes(weather.as_bytes())?.into())
    }
}

/// A simulation that can be advanced one timestep at a time, or run to the
/// end of the weather file.
#[wasm_bindgen]
pub struct Simulation {
    model: Arc<Model>,
    physics: MultiphysicsModel,
    memory: MultiphysicsModelMemory,
    weather: Weather,
    period: SimulationPeriod,
    state: Arc<Mutex<SimulationState>>,
    controller: Option<RhaiControlScript>,
    full_header: Vec<String>,
    report_indexes: Vec<usize>,
    date: Option<Date>,
    workplanes: WorkplaneModel,
    workplane_memory: WorkplaneModelMemory,
    illuminance: Vec<IlluminanceRecord>,
}

#[wasm_bindgen]
impl Simulation {
    /// Builds a simulation.
    ///
    /// * `model`: the contents of a `.spl` or JSON model
    /// * `weather`: the contents of an EPW or `.sw` weather file
    /// * `n`: the number of timesteps per hour
    /// * `control_script`: an optional Rhai script, run before every timestep
    /// * `research_mode`: allows the control script to set physical properties
    ///   (e.g., the temperature of a space), which cannot be set in real life
    ///
    /// Building the simulation includes calculating the daylight
    /// coefficients of the model's workplanes (if any), which can take a
    /// while.
    #[wasm_bindgen(constructor)]
    pub fn new(
        model: &str,
        weather: &str,
        n: usize,
        control_script: Option<String>,
        research_mode: bool,
    ) -> Result<Simulation, JsError> {
        let (model, mut state_header) = parse_model(model).map_err(|e| JsError::new(&e))?;
        let weather = parse_weather(weather).map_err(|e| JsError::new(&e))?;

        let data = prepare(&model, weather, n, &mut state_header).map_err(|e| JsError::new(&e))?;

        let state = state_header
            .take_values()
            .ok_or_else(|| JsError::new("Could not take values from SimulationStateHeader"))?;
        let memory = data
            .model
            .allocate_memory(&state)
            .map_err(|e| JsError::new(&e))?;

        let workplanes =
            WorkplaneModel::new(&data.meta_options, &model, WorkplaneModel::DEFAULT_MF)
                .map_err(|e| JsError::new(&e))?;
        let workplane_memory = workplanes.allocate_memory();
        let dt_hours = 1. / data.model.dt_subdivisions as Float;
        let illuminance = workplanes
            .workplanes()
            .iter()
            .map(|w| IlluminanceRecord::new(w.sensors.len(), dt_hours))
            .collect();

        let model = Arc::new(model);
        let (controller, state) = match control_script {
            Some(script) => {
                let (controller, state) =
                    RhaiControlScript::new(&model, state, script, research_mode)
                        .map_err(|e| JsError::new(&e))?;
                (Some(controller), state)
            }
            None => (None, Arc::new(Mutex::new(state))),
        };

        Ok(Self {
            model,
            physics: data.model,
            memory,
            weather: data.weather,
            period: data.sim_period,
            state,
            controller,
            full_header: data.full_header,
            report_indexes: data.report_indexes,
            date: None,
            workplanes,
            workplane_memory,
            illuminance,
        })
    }

    /// Seeds the random numbers of the control script (i.e., `random()`), for
    /// repeatable simulations. Without it, every simulation gets different
    /// random numbers. It does nothing if there is no control script.
    pub fn set_seed(&mut self, seed: u64) {
        if let Some(controller) = &mut self.controller {
            controller.set_seed(seed);
        }
    }

    /// Runs the control script (if any) and advances the simulation by one
    /// timestep. Returns `false`, without doing anything, once the end of the
    /// weather file has been reached.
    pub fn step(&mut self) -> Result<bool, JsError> {
        let Some(date) = self.period.next() else {
            return Ok(false);
        };

        if let Some(controller) = &mut self.controller {
            controller
                .control(date, self.period.day_of_week(date))
                .map_err(|e| JsError::new(&e))?;
        }

        let mut state = lock(&self.state);
        self.physics
            .march(
                date,
                &self.weather,
                self.model.as_ref(),
                &mut state,
                &mut self.memory,
            )
            .map_err(|e| JsError::new(&format!("{e} (Date '{date}')")))?;
        drop(state);

        if !self.illuminance.is_empty() {
            self.workplanes
                .march(date, &self.weather, &mut self.workplane_memory)
                .map_err(|e| JsError::new(&format!("{e} (Date '{date}')")))?;
            for (i, record) in self.illuminance.iter_mut().enumerate() {
                let lux = self.workplane_memory.illuminance(i).unwrap_or_default();
                record.push(date, lux).map_err(|e| JsError::new(&e))?;
            }
        }

        self.date = Some(date);
        Ok(true)
    }

    /// The names of the model's workplanes. Workplanes are referred to by
    /// their index in this list.
    pub fn workplane_names(&self) -> Vec<String> {
        self.workplanes
            .workplanes()
            .iter()
            .map(|w| w.name.clone())
            .collect()
    }

    /// The position of the sensors of a workplane, as `[x0, y0, z0, x1, ...]`
    pub fn workplane_sensors(&self, workplane: usize) -> Result<Vec<Float>, JsError> {
        Ok(self
            .workplane(workplane)?
            .sensors
            .iter()
            .flat_map(|p| [p.x, p.y, p.z])
            .collect())
    }

    /// The floor area (in m2) represented by each sensor of a workplane
    pub fn workplane_areas(&self, workplane: usize) -> Result<Vec<Float>, JsError> {
        Ok(self.workplane(workplane)?.areas.clone())
    }

    /// The illuminance (in lux) at each sensor of a workplane in the last
    /// timestep simulated
    pub fn illuminance(&self, workplane: usize) -> Result<Vec<Float>, JsError> {
        self.workplane(workplane)?;
        Ok(self
            .workplane_memory
            .illuminance(workplane)
            .map(|v| v.to_vec())
            .unwrap_or_default())
    }

    /// The illuminance (in lux) at each sensor of a workplane in every
    /// timestep simulated so far, one timestep after the other
    pub fn illuminance_history(&self, workplane: usize) -> Result<Vec<Float>, JsError> {
        Ok(self.record(workplane)?.values().to_vec())
    }

    /// The number of hours each sensor of a workplane received at least
    /// `min` lux. If `from` and `to` are given, only the hours of the day
    /// within that range count (e.g., from 8 to 18).
    pub fn hours_above(
        &self,
        workplane: usize,
        min: Float,
        from: Option<Float>,
        to: Option<Float>,
    ) -> Result<Vec<Float>, JsError> {
        Ok(self
            .record(workplane)?
            .hours_above(min, occupied(from, to)?))
    }

    /// The number of hours each sensor of a workplane received between `min`
    /// and `max` lux (both included). If `from` and `to` are given, only the
    /// hours of the day within that range count (e.g., from 8 to 18).
    pub fn hours_within(
        &self,
        workplane: usize,
        min: Float,
        max: Float,
        from: Option<Float>,
        to: Option<Float>,
    ) -> Result<Vec<Float>, JsError> {
        Ok(self
            .record(workplane)?
            .hours_within(min, max, occupied(from, to)?))
    }

    /// The number of hours simulated so far that are within `from` and `to`
    /// (or all of them, if not given)
    pub fn occupied_hours(
        &self,
        workplane: usize,
        from: Option<Float>,
        to: Option<Float>,
    ) -> Result<Float, JsError> {
        Ok(self.record(workplane)?.occupied_hours(occupied(from, to)?))
    }

    /// Runs every remaining timestep, returning the reported values (see
    /// [`Simulation::report_indexes`]) of every timestep, one row after the
    /// other.
    pub fn run(&mut self) -> Result<Vec<Float>, JsError> {
        let mut results = Vec::new();
        while self.step()? {
            let state = lock(&self.state);
            results.extend(self.report_indexes.iter().map(|i| state[*i]));
        }
        Ok(results)
    }

    /// The name of every element in the state
    pub fn header(&self) -> Vec<String> {
        self.full_header.clone()
    }

    /// The indexes of the state elements requested in the model's outputs
    /// (or all of them, if no outputs were requested)
    pub fn report_indexes(&self) -> Vec<u32> {
        self.report_indexes.iter().map(|i| *i as u32).collect()
    }

    /// The date of the last timestep simulated (e.g., `"Jan 1, 00:30"`), or
    /// `undefined` if [`Simulation::step`] has not been called yet
    pub fn date(&self) -> Option<String> {
        self.date.map(|d| d.to_string())
    }

    /// The month (1-12) of the last timestep simulated
    pub fn month(&self) -> Option<u8> {
        self.date.map(|d| d.month)
    }

    /// The day of the month of the last timestep simulated
    pub fn day(&self) -> Option<u8> {
        self.date.map(|d| d.day)
    }

    /// The hour (0-23.99..., with decimals) of the last timestep simulated
    pub fn hour(&self) -> Option<Float> {
        self.date.map(|d| d.hour)
    }

    /// A copy of the whole state
    pub fn state(&self) -> Vec<Float> {
        lock(&self.state).clone()
    }

    /// Overwrites the whole state (e.g., to apply control from JavaScript)
    pub fn set_state(&mut self, values: &[Float]) -> Result<(), JsError> {
        let mut state = lock(&self.state);
        if values.len() != state.len() {
            return Err(JsError::new(&format!(
                "Expecting a state of {} elements, found {}",
                state.len(),
                values.len()
            )));
        }
        state.copy_from_slice(values);
        Ok(())
    }

    /// The value of a single element of the state
    pub fn get(&self, index: usize) -> Result<Float, JsError> {
        lock(&self.state)
            .get(index)
            .copied()
            .ok_or_else(|| JsError::new(&format!("State index {index} out of bounds")))
    }

    /// Sets the value of a single element of the state
    pub fn set(&mut self, index: usize, value: Float) -> Result<(), JsError> {
        let mut state = lock(&self.state);
        let len = state.len();
        let v = state
            .get_mut(index)
            .ok_or_else(|| JsError::new(&format!("State index {index} out of bounds ({len})")))?;
        *v = value;
        Ok(())
    }

    /// The address of the state within the WebAssembly memory. The state
    /// never moves, so a view such as
    /// `new Float64Array(wasm_memory().buffer, sim.state_ptr(), sim.state_len())`
    /// reads and writes it directly. Views are detached whenever the memory
    /// grows, though, so check `view.length` (or re-create the view) after
    /// calling into the simulation.
    pub fn state_ptr(&self) -> *const Float {
        lock(&self.state).as_ptr()
    }

    /// The number of elements in the state
    pub fn state_len(&self) -> usize {
        lock(&self.state).len()
    }
}

impl Simulation {
    fn workplane(&self, i: usize) -> Result<&WorkplaneSensors, JsError> {
        self.workplanes
            .workplanes()
            .get(i)
            .ok_or_else(|| JsError::new(&format!("There is no workplane {i}")))
    }

    fn record(&self, i: usize) -> Result<&IlluminanceRecord, JsError> {
        self.illuminance
            .get(i)
            .ok_or_else(|| JsError::new(&format!("There is no workplane {i}")))
    }
}

/// The occupied hours of the day, from JavaScript's optional `from` and `to`
fn occupied(from: Option<Float>, to: Option<Float>) -> Result<Option<(Float, Float)>, JsError> {
    match (from, to) {
        (Some(from), Some(to)) => Ok(Some((from, to))),
        (None, None) => Ok(None),
        _ => Err(JsError::new(
            "Occupied hours need both 'from' and 'to' (or neither)",
        )),
    }
}

/// The WebAssembly memory, which [`Simulation::state_ptr`] points into
#[wasm_bindgen]
pub fn wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// Locks the state. Nothing runs in parallel in WebAssembly, so the lock is
/// always free (and a poisoned lock just means a previous step failed).
fn lock(state: &Mutex<SimulationState>) -> MutexGuard<'_, SimulationState> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: &str = include_str!("../../tests/box/box.spl");
    const WEATHER: &str = include_str!("../../tests/wellington.epw");

    #[test]
    fn strips_optical_data_path() -> Result<(), String> {
        let (model, _) = parse_model(BOX)?;
        let options = model.solar_options.ok_or("no solar options")?;
        assert!(options.optical_data_path().is_err());
        assert_eq!(*options.n_solar_irradiance_points()?, 100);
        Ok(())
    }

    #[test]
    fn parses_weather() -> Result<(), String> {
        let weather = parse_weather(WEATHER)?;
        assert_eq!(weather.data.len(), 8760);
        Ok(())
    }
}
