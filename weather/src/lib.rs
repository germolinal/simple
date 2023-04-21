/*
MIT License
Copyright (c) 2021 Germán Molina
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

//! This library is both an [EPW file parser](https://energyplus.net/weather) and
//! a trait that allows getting weather data for simulations

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

/// Data associated to a specific Location
pub mod location;
pub use crate::location::Location;

/// Data associated to the specific weather conditions at a particular moment
pub mod current_weather;
pub use crate::current_weather::CurrentWeather;

/// Data for representing the ground temperature according to EPW data
pub mod epw_ground_temperature;

/// A scanner for reading EPW files
mod epw_scanner;

/// Like [`CurrentWeather`] but including all the data available in EPW files
pub mod epw_weather_line;

/// A representation of an EPW file
pub mod epw_weather;
pub use crate::epw_weather::EPWWeather;

/// Allows creating weathers that can be used for highly-specific
/// simulation. E.g., Having a sinusoidal exterior temperature with no
/// sun.
pub mod synthetic_weather;
pub use crate::synthetic_weather::SyntheticWeather;
pub use calendar::Date;

/// The basic trait defining a Weather that can be used in
/// Building Simulation
pub trait Weather: Sync {
    /// Retreives a [`CurrentWeather`] object based on the date.
    fn get_weather_data(&self, date: Date) -> CurrentWeather;
}
