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

//! A Library for Generic Matrix operations.
//!
//! It is built generically (i.e., `GenericMatrix<T: TReq>` where `TReq` is a
//! basic numeric Trait) so that the same library can be used for defining Matrices
//! over `usize`, `i32`, `f32` and even structures (e.g., `Colour{red:f32, green: f32, blue: f32}`)
//! to which numeric operations apply

/// The kind of Ting point number used in the
/// library... the `"T"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

/// The kind of Ting point number used in the
/// library... the `"T"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
type Float = f64;

/// A macro indicating how many non-zero elements exist on each side of
/// an n-diagonal matrix
macro_rules! one_sided_n {
    ( $n : expr ) => {{
        ($n - 1) / 2
    }};
}

/// Contains a structure that allows performing mathematical operations over
/// matrices that do not contain numbers, but also other structures.
pub mod generic_matrix;
/// Traits used to make the `GenericMatrix` generic.
pub mod traits;
pub use generic_matrix::GenericMatrix;

/// A `generic_matrix` that contains a `Float`
pub mod matrix;
pub use crate::matrix::Matrix;
