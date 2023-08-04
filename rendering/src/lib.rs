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

// #![deny(missing_docs)]

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
pub type Float = f32;
#[cfg(feature = "float")]
pub const PI: Float = std::f32::consts::PI;

#[cfg(not(feature = "float"))]
pub type Float = f64;
#[cfg(not(feature = "float"))]
pub const PI: Float = std::f64::consts::PI;

/// The number of values that represent a colour.
/// RGB is Three... you can change this for spectral
/// rendering
pub const N_CHANNELS: usize = 3;

// Core
pub mod bvh;
pub mod camera;
mod colour;
pub use colour::Spectrum;
pub mod colourmap;
pub mod image;
pub mod interaction;
pub mod material;

pub mod primitive;

pub mod primitive_samplers;
pub mod rand;
mod ray;
pub use ray::Ray;
pub mod samplers;
mod scene;
pub use scene::{Scene, Wavelengths};

pub mod triangle;

// Climate Based Daylight Model
pub mod colour_matrix;
pub use colour_matrix::ColourMatrix;

pub mod daylight_coefficients;
pub use daylight_coefficients::DCFactory;
// Readers
pub mod from_obj;
pub mod from_radiance;
pub mod from_simple_model;
pub use from_simple_model::{SimpleModelReader, SceneElement};

// Ray-tracer
mod ray_tracer;
pub use ray_tracer::{RayTracer, RayTracerHelper};


// mod backward_metropolis;
// pub use crate::backward_metropolis::{
//     mutation::RestartRay, BackwardMetropolis, Mutation, MutationSet,
// };
