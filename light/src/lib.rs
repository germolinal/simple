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

//! This is [SIMPLE's](https://www.simplesim.tools) Light and Solar calculation module. It is responsible for:
//!
//! * **Calculating Incident Solar Radiation in each surface**: Contrary to EnergyPlus (and probably other tools I am less familiar with), this module uses Daylight Coefficients for performing this simulation. This method was stolen from the
//! daylighting simulation world, and has the advantage of being extremely robust, and therefore capable of handling complex geometries. Perhaps the main drawback is that—because the concept of Thermal Zone does not fit within Lighting calculations (it is quite artificial for radiation purposes, actually)—reporting the "Solar Heat Gains" in a zone needs significant post-processing.
//! * **Calculating view factors for Infrared calculations**: This is in development... for now, it does nothing.
//! * **Daylighting Calculations**: Because this module is based on ray-tracing, it can perform daylight calculations. It is unclear, however, which climate based daylight metrics to include and how... if you have any idea, let me know.

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
pub type Float = f32;
/// Well, Pi.
#[cfg(feature = "float")]
pub const PI: Float = std::f32::consts::PI;

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
pub type Float = f64;

/// Well, Pi.
#[cfg(not(feature = "float"))]
pub const PI: Float = std::f64::consts::PI;

/// The main export of this module: A Simulation Model for
/// calculating solar and lighting factors.
pub mod solar_model;
pub use solar_model::SolarModel;
mod optical_info;
mod solar_surface;
pub use optical_info::{IRViewFactorSet, OpticalInfo};
