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

//! A Finite Difference-based Thermal simulation module.
//!
//! It uses finite differences for marching forward in time and also
//! for calculating the heat transfer through walls.

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

/// Well, $`\pi`$
#[cfg(feature = "float")]
pub const PI: Float = std::f32::consts::PI;
/// Well, $`\pi`$
#[cfg(not(feature = "float"))]
pub const PI: Float = std::f64::consts::PI;

/// The [Stefan–Boltzmann](https://en.wikipedia.org/wiki/Stefan–Boltzmann_constant) constant (in $`W m^{-2} K^4`$),
/// necessary for Radiation calculations
pub const SIGMA: Float = 5.670374419e-8;

/// The main thermal model, which manages the whole time-marching
/// solution
pub mod heat_model;

/// For dealing with Cavities, i.e., layers made out of a gas
/// enclosed within two materials of different emissivities.
pub mod cavity;

/// For creating thermal networks for heat-transfering surfaces.
pub mod discretization;

/// For calculating convection coefficients under different
/// surface conditions.
pub mod convection;

/// Definitions for the thermal properties of gases.
pub mod gas;

/// Glazing layer abstracted to only their optical properties.
pub mod glazing;

/// For HVAC-related calculations.
pub mod heating_cooling;

/// For calculating heat transfer through all kinds of surfaces.
pub mod surface;

/// For calculating the temperatures within Spaces.
pub mod zone;

mod luminaire;
