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


//! A light-weight representation of a polynomial. 
//! It contains a maximum of 12 coefficients.
//!
//! ## Quickstart
//!
//!```rust
//!     use polynomial::{poly, Polynomial};
//!    // p = 0. + 1.0 * x^2 + 2.0*x^3 + 3.0*x^4
//!    let p = poly![0.0, 1.0, 2.0, 3.0];
//!
//!    // 0 + 1 + 2 + 3
//!    assert_eq!(p.eval(1.0), 6.0);
//!
//!    // 0 + 1*2^1 + 2*2^2 + 3*2^3 = 
//!    // 0 + 2     + 8     + 24    = 34
//!    assert_eq!(p.eval(2.0), 34.0);
//!
//!    // It can also be used as a constant
//!    const P : Polynomial = poly![1., 2., 3., 4.];
//!
//!```
//!
//! ## `f32` or `f64`?
//!
//! By default, this crate works with `f64`. Use the feature `float` to use `f32`.



/// The floating point type to use. Defaults to `f64`... enable 
/// `f32` by using the `float` feature
#[cfg(not(feature = "float"))]
type Float = f64;

/// The floating point type to use. Defaults to `f64`... enable 
/// `f32` by using the `float` feature
#[cfg(feature = "float")]
type Float = f32;


/// A simple polynomial structure in the form A0 + A1*x + A2*x^2....
///
/// It contains the coefficients and
/// can be evaluated for any Float value
/// 
/// # Example
/// 
/// ```
///     use polynomial::*;
///     // p = 0. + 1.0 * x^2 + 2.0*x^3 + 3.0*x^4
///     let p = poly![0.0, 1.0, 2.0, 3.0];
///
///     // 0 + 1 + 2 + 3
///     assert_eq!(p.eval(1.0), 6.0);
/// ```
#[derive(Debug, Copy, Clone, Default)]
pub struct Polynomial{
    /// The coefficients of the polynomial, starting from the 
    /// constant and increasing with the power.
    pub coefficients: [Float; 12],

    /// The number of valid coefficients (any further 
    /// than this will be ignored during evaluation)
    pub len: usize,
}

impl Polynomial{
    /// Creates a new empty Polynomial where all coefficients are Zero
    /// and the len is Zero
    pub const fn new() -> Self {
        Self {
            len: 0,
            coefficients: [0.0; 12],
        }
    }
}



/// A convenient way of defining a `Polynomial`.
/// 
/// # Examples
/// 
/// ```
///     use polynomial::{poly, Polynomial};
///     let p = poly![0.0, 1.0, 2.0, 3.0];
/// ```
/// 
/// It can also be used for constants
/// 
/// ```
///     use polynomial::{poly, Polynomial};
///     const P : Polynomial = poly![0.0, 1.0, 2.0, 3.0];
/// ```
#[macro_export]
macro_rules! poly {
    () => {
        Polynomial::new()
    };
    ( $($e : expr),+ ) => {{
        let mut coefficients = [0.0;12];
        // let mut p = poly![];
        // $(p.push($e);)*
        // p
        let given_coefs = [$($e,)*];
        let len = given_coefs.len();
        if len > 12 {
            panic!("too many coefficients passed to a Polynomial. Maximum is 12.")
        }

        // for i in 0..len {
        let mut i = 0;
        while i < len {
            coefficients[i] = given_coefs[i];
            i+=1;
        }
        Polynomial {
            len,
            coefficients
        }
    }};

}

impl Polynomial{
    /// Adds a new coefficient to the polynomial
    pub fn push(&mut self, v: Float) {
        if self.len >= self.coefficients.len() {
            panic!(
                "The number of coefficients in a Polynomial is limited to {}. Cannot add a new one",
                self.coefficients.len()
            )
        }
        self.coefficients[self.len] = v;
        self.len += 1;
    }

    /// Returns the number of coefficients
    pub fn len(&self) -> usize {
        self.len
    }

    /// Evaluates the polynomial with a certain input.
    pub fn eval(&self, x: Float) -> Float {
        let mut y = 0.0;
        for i in 0..self.len {
            y += self.coefficients[i] * x.powi(i as i32);
        }

        y
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    const _POL: Polynomial = Polynomial::new();
    const _POL2: Polynomial = poly![1., 2., 3.];

    #[test]
    fn test_macro_empty() {
        // Empty polynomial
        let p = poly![];
        assert_eq!(p.len(), 0);
        assert_eq!(p.len, 0);
        for i in 0..p.coefficients.len() {
            assert_eq!(0.0, p.coefficients[i]);
        }
    }

    #[test]
    fn test_macro_single() {
        let p = poly![1.0];
        assert_eq!(p.len(), 1);
        assert_eq!(p.len, 1);

        assert_eq!(p.coefficients[0], 1.0);
    }

    #[test]
    fn test_macro_two() {
        // Polynomial with two values
        let p = poly![6.0, 3.0];
        assert_eq!(p.len(), 2);
        assert_eq!(p.len, 2);

        assert_eq!(p.coefficients[0], 6.0);
        assert_eq!(p.coefficients[1], 3.0);
        for i in 2..p.coefficients.len() {
            assert_eq!(0.0, p.coefficients[i]);
        }

        // 6 + 3 * 1^1 = 9
        assert_eq!(p.eval(1.0), 9.0);

        // 6 + 3 * 2 = 12
        assert_eq!(p.eval(2.0), 12.0);
    }

    #[test]
    fn test_macro_four() {
        // Polynomial with four values
        let p = poly![0.0, 1.0, 2.0, 3.0];
        assert_eq!(p.len(), 4);
        assert_eq!(p.len, 4);

        assert_eq!(p.coefficients[0], 0.0);
        assert_eq!(p.coefficients[1], 1.0);
        assert_eq!(p.coefficients[2], 2.0);
        assert_eq!(p.coefficients[3], 3.0);
        for i in 4..p.coefficients.len() {
            assert_eq!(0.0, p.coefficients[i]);
        }

        // 0 + 1 + 2 + 3
        assert_eq!(p.eval(1.0), 6.0);

        // 0 + 1*2^1 + 2*2^2 + 3*2^3 = ??
        // 0 + 2     + 8     + 24    = 34
        assert_eq!(p.eval(2.0), 34.0);
    }

    #[test]
    fn test_default() {
        let p = Polynomial::default();
        assert_eq!(p.len(), 0);
        for v in p.coefficients {
            assert_eq!(v, 0.0);
        }
    }
}
