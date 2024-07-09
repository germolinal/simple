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

//! The main `SIMPLE` Simulation crate, combining pretty much every other
//! development.
//!
//! This crate provides a high-level simulation framework, hidding all the physics
//! behind the different modules (thermal, light, etc.)

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
pub type Float = f32;

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
pub type Float = f64;

/// The model that puts different physics domains together.
pub mod multiphysics_model;
pub use multiphysics_model::MultiphysicsModel;

/// Allows passing a "control"
pub mod control_trait;

/// A controled that does nothing
pub mod void_control;
pub use void_control::VoidControl;

/// Structure that controls
pub mod rhai_script_controller;
pub use rhai_script_controller::RhaiControlScript;

/// Some default control routines, used when no
/// Rhai control script is given.
pub mod occupant_behaviour;
pub use occupant_behaviour::OccupantBehaviour;

/// A module with some useful functions to run a simulation
pub mod run_simulation;

// Re-exports
pub use calendar::{Date, Period};
pub use communication::{MetaOptions, SimulationModel};
pub use geometry;
pub use light::OpticalInfo;
pub use matrix::Matrix;
pub use model;
pub use model::{Model, SimulationState, SimulationStateElement, SimulationStateHeader, *};
pub use polynomial::*;
pub use rendering::{
    rand::get_rng, rand::RandGen, rand::Rng, samplers, Scene, SceneElement, SimpleModelReader,
    Wavelengths,
};
pub use schedule::*;
pub use weather::{
    CurrentWeather, EPWWeather, EPWWeatherLine, Location, PerezSky, SkyUnits, Solar,
    SyntheticWeather, Time, Weather, WeatherTrait,
};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn display() -> Result<(), String> {
        let meta_options = MetaOptions::default();

        let (model, mut state) = Model::from_file("./tests/box/box.spl")?;
        let _ = MultiphysicsModel::new(&meta_options, (), &model, &mut state, 2)?;

        let string = format!("{}", model);
        let (model, mut state) = Model::from_bytes(string.as_bytes())?;
        let _ = MultiphysicsModel::new(&meta_options, (), &model, &mut state, 2)?;

        Ok(())
    }
}
