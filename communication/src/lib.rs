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

//! This library contains some standards that SIMPLE uses for communicating different
//! simulation modules. For now, it is only really useful for ensuring good communication
//! between modules at compile time.

#[cfg(feature = "float")]
type Float = f32;
#[cfg(not(feature = "float"))]
type Float = f64;

/// A set of options that affect the whole simulation but aren't part of
/// the model itself (e.g., location)
#[derive(Debug, Default)]
pub struct MetaOptions {
    /// The Latitude in Radians.
    ///
    /// South is negative, North is Positive.
    pub latitude: Float,

    /// The Longitude in Radians.
    ///
    /// West is Negative, East is Positive    
    pub longitude: Float,

    /// The Standard Meridian, in Radians. This is `15*GMT_time`
    pub standard_meridian: Float,

    /// The elevation of the site, in meters
    pub elevation: Float,
}

/// Helps communicating issues to user
pub trait ErrorHandling {
    /// Indicates a module name that will be used
    /// for reporting errors
    fn module_name() -> &'static str;

    /// Returns a user error
    fn user_error<T>(errmsg: String) -> Result<T, String> {
        let name: &'static str = <Self as ErrorHandling>::module_name();
        Err(format!("User Error in module '{}' : {}", name, errmsg))
    }

    /// Returns an Internal error
    fn internal_error<T>(errmsg: String) -> Result<T, String> {
        let name: &'static str = <Self as ErrorHandling>::module_name();
        Err(format!("Internal Error in module '{}' : {}", name, errmsg))
    }
}

use calendar::Date;
use model::{Model, SimulationState, SimulationStateHeader};
use std::borrow::Borrow;
use weather::WeatherTrait;

/// Protocols for SimulationModels
pub trait SimulationModel: ErrorHandling {
    /// This is Self
    type OutputType;

    /// The structure containing the options for creating the model
    type OptionType;

    /// A structure for keeping mutable data that is not considered a result
    ///
    /// For example, the Daylight Coefficients calculation—in the Light module—requires
    /// creating a Sky vector in each timestep. With this, we can avoid re-allocating constantly by
    /// creating the vector once and then passing it to the `march` method.
    type AllocType;

    /// Creates a new Simulation Model
    /// # Arguments;
    /// * A model model
    /// * A model state model (data important to the Sim Engine will be added)
    /// * The number of timesteps per hour for the main simulation... internally, each simulation model will choose its own sub-subdivition (e.g. half or one third of what is asked for)
    fn new<M: Borrow<Model>>(
        meta_option: &MetaOptions,
        options: Self::OptionType,
        model: M,
        state: &mut SimulationStateHeader,
        n: usize,
    ) -> Result<Self::OutputType, String>;

    /// Marchs forward in the simulation.
    /// # Arguments
    /// * The model
    /// * The model state (will be modified)
    /// * A piece of weather data
    fn march<W: WeatherTrait, M: Borrow<Model>>(
        &self,
        date: Date,
        weather: &W,
        model: M,
        state: &mut SimulationState,
        alloc: &mut Self::AllocType,
    ) -> Result<(), String>;

    /// Allocates the memory needed to run the simulation.
    ///
    /// The purpose of this is to allocate memory once and thus save time
    /// during the simulation
    fn allocate_memory(&self, state: &SimulationState) -> Result<Self::AllocType, String>;
}
