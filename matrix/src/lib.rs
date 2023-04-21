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

//! A Library for Generic Matrix operations.
//!
//! It is built generically (i.e., `GenericMatrix<T: TReq>` where `TReq` is a
//! basic numeric Trait) so that the same library can be used for defining Matrices
//! over `usize`, `i32`, `f32` and even structures (e.g., `Colour{red:f32, green: f32, blue: f32}`)
//! to which numeric operations apply

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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

/// A shorthand for `GenericMatrix<Float>`; i.e., a normal
/// matrix. Note that `Float` is defined as `f32` if the feature `float`
/// is utilized; otherwise, it defauts to `f64`.
pub type Matrix = GenericMatrix<Float>;

/// A macro indicating how many non-zero elements exist on each side of
/// an n-diagonal matrix
macro_rules! one_sided_n {
    ( $n : expr ) => {{
        ($n - 1) / 2
    }};
}

impl Matrix {
    /// Solves an $`A \times x=b`$ problem using the  [Gaussian Elimination](https://en.wikipedia.org/wiki/Gaussian_elimination)
    /// algorithm. It assumes that $`A`$ is `n`-diagonal (e.g., [tri-diaglnal](https://en.wikipedia.org/wiki/Tridiagonal_matrix)).
    /// It puts the results into `x` (whose data is completely replaced).
    ///
    /// Returns an error if an element in the diagonal of the matrix is Zero (row-swapping is not
    /// yet supported)
    ///
    /// # Note
    /// This method will clone both `self` and `b`, meaning that it won't be very performant. Check `mut_n_diag_gaussian()`
    /// for what might be a more conservative approach.
    pub fn n_diag_gaussian(&self, b: &Matrix, n: usize) -> Result<Matrix, String> {
        // Clone Self and B; then solve... put results on X
        let a = self.clone();
        let b = b.clone();
        a.mut_n_diag_gaussian(b, n)
    }
    /// Solves an $`A \times x=b`$ problem using the  [Gaussian Elimination](https://en.wikipedia.org/wiki/Gaussian_elimination)
    /// algorithm. It assumes that $`A`$ is `n`-diagonal (e.g., [tri-diaglnal](https://en.wikipedia.org/wiki/Tridiagonal_matrix)).
    ///
    /// Both `self` and `b` are completely consumed, as they are modified in the process.
    ///
    /// Returns an error if an element in the diagonal of the matrix is Zero (row-swapping is not
    /// yet supported)
    ///
    /// # Note
    /// It mutates both `self` and `b`. If this is not what you want, call `n_diag_gaussian()` instead
    pub fn mut_n_diag_gaussian(mut self, mut b: Matrix, n: usize) -> Result<Matrix, String> {
        const TINY: Float = 1e-26;
        let one_sided_n = one_sided_n!(n);

        // Scale row operation.
        fn aux_scale(matrix: &mut Matrix, row: usize, col_start: usize, col_end: usize, v: Float) {
            for col in col_start..col_end {
                if col == matrix.ncols {
                    break;
                }
                let i = matrix.index(row, col);
                matrix.data[i] *= v;
            }
        }
        let scale_row = |a: &mut Matrix, b: &mut Matrix, row: usize, v: Float| {
            aux_scale(a, row, row, row + one_sided_n + 1, v);
            aux_scale(b, row, 0, b.ncols, v);
        };

        // Add rows operation
        fn aux_add(
            matrix: &mut Matrix,
            row_from: usize,
            row_into: usize,
            scale: Float,
            col_start: usize,
            col_end: usize,
        ) {
            for col in col_start..col_end {
                if col == matrix.ncols {
                    break;
                }
                let aux = matrix.data[matrix.index(row_from, col)];
                let i = matrix.index(row_into, col);
                matrix.data[i] += scale * aux;
            }
        }
        let add_rows =
            |a: &mut Matrix, b: &mut Matrix, row_from: usize, row_into: usize, scale: Float| {
                aux_add(
                    a,
                    row_from,
                    row_into,
                    scale,
                    row_from,
                    row_from + one_sided_n + 1,
                );
                aux_add(b, row_from, row_into, scale, 0, b.ncols);
            };

        // First, go down, making self an upper-triangular matrix
        for c in 0..self.ncols {
            // Get value, and check that it has content
            let pivot = self.data[self.index(c, c)];
            if pivot.abs() < TINY {
                return Err(format!("Found a (nearly) zero element in the diagonal: {}. Maybe the matrix is not invertible...?", pivot));
            }

            // Make the pivot equals to 1.
            scale_row(&mut self, &mut b, c, 1. / pivot);

            // We do not need to pivot any more
            if c == self.ncols - 1 {
                break;
            }

            // Iterate, eliminating values below
            let ini = c + 1;
            let fin = c + one_sided_n + 1;
            for r in ini..fin {
                if let Ok(other) = self.get(r, c) {
                    // if it is Zero already, just skip
                    if other.abs() > TINY {
                        add_rows(&mut self, &mut b, c, r, -other);
                    }
                } else {
                    break;
                }
            }
        } // end of going down.

        // Now, run substitution upwards (We don't need to iterate the first row)
        for c in (1..self.ncols).into_iter().rev() {
            let pivot = self.data[self.index(c, c)];
            debug_assert!((1. - pivot).abs() < 1e-15, "pivot is {}", pivot);

            let ini = if c < one_sided_n { 0 } else { c - one_sided_n };

            for r in (ini..c).into_iter().rev() {
                if let Ok(other) = self.get(r, c) {
                    // if it is Zero already, just skip
                    if other.abs() > TINY {
                        add_rows(&mut self, &mut b, c, r, -other);
                    }
                }
            }
        }

        Ok(b)
    }

    /// Solves an $A \times x=b$ problem using the  [Gauss-Seidel](https://en.wikipedia.org/wiki/Gauss–Seidel_method)
    /// algorithm
    pub fn gauss_seidel(
        &self,
        b: &Matrix,
        x: &mut Matrix,
        max_iter: usize,
        conv_check: Float,
    ) -> Result<(), String> {
        // Check dimensions
        if self.ncols != self.nrows {
            return Err(format!("Gauss-Seidel algorithm (for solving Ax=b) only works for squared matrices A... found A to be {} by {}", self.nrows, self.ncols));
        }
        if self.ncols != b.nrows {
            return Err(format!("Gauss-Seidel algorithm (for solving Ax=b) requires A to have the same number of columns as b has rows... found {} and {}, respectively", self.ncols, b.nrows));
        }
        if self.ncols != x.nrows {
            return Err(format!("Gauss-Seidel algorithm (for solving Ax=b) requires A to have the same number of columns as b has rows... found {} and {}, respectively", self.ncols, x.nrows));
        }

        let (n, ..) = self.size();
        let zeroes = vec![0.0; n];
        let mut new_x = x.clone();
        for _nit in 0..max_iter {
            new_x.data.copy_from_slice(&zeroes);

            for i in 0..n {
                let mut s1 = 0.0;
                for j in 0..i {
                    let index = self.index(i, j);
                    s1 += self.data[index] * new_x.data[j];
                }
                let mut s2 = 0.0;
                if i < n {
                    // let ini = i+1;
                    for j in i + 1..n {
                        let index = self.index(i, j);
                        s2 += self.data[index] * x.data[j];
                    }
                }
                let index = self.index(i, i);
                new_x.data[i] = (b.data[i] - s1 - s2) / self.data[index];
            }

            // Check convergence
            let mut max_err = -Float::MAX;
            for i in 0..n {
                let err = (x.data[i] - new_x.data[i]).abs();
                if err > max_err {
                    max_err = err;
                }
            }

            if max_err < conv_check {
                return Ok(());
            }
            x.data.copy_from_slice(&new_x.data);
        }

        return Err(format!(
            "Gauss-Seidel algorithm did not converge after {} iterations",
            max_iter
        ));
    }
}

/// A simple trait required for initializing some matrices (e.g., the
/// identity matrix)
pub trait OneZero {
    /// Returns an element considered to be 0.
    fn zero() -> Self;

    /// Returns an element considered to be 1.
    fn one() -> Self;
}

impl OneZero for Float {
    fn zero() -> Self {
        0.
    }
    fn one() -> Self {
        1.
    }
}

use serde::{Deserialize, Serialize};

/// Define the basic algebraic requirements for T
pub trait TReq:
    Copy
    + Clone
    + OneZero
    + PartialEq
    + Sized
    + std::fmt::Display
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::Mul<Float, Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::MulAssign
    + std::ops::Div<Float, Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::DivAssign
    + Sync
    + Send
    + Serialize
    + core::fmt::Debug
{
}
impl<
        T: OneZero
            + Copy
            + Clone
            + PartialEq
            + Sized
            + std::fmt::Display
            + std::fmt::Debug
            + std::ops::Add<Output = Self>
            + std::ops::Sub<Output = Self>
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Mul<Float, Output = Self>
            + std::ops::Mul<Output = Self>
            + std::ops::MulAssign
            + std::ops::Div<Float, Output = Self>
            + std::ops::Div<Output = Self>
            + std::ops::DivAssign
            + Sync
            + Send
            + Serialize
            + core::fmt::Debug
    > TReq for T
{
}

/// The main Structure in this library
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GenericMatrix<T: TReq> {
    ncols: usize,
    nrows: usize,

    // Contains the data ordered by row,
    // Going left to right, and up and down.
    data: Vec<T>,
}

impl<T: TReq> std::fmt::Display for GenericMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.nrows {
            write!(f, "\n\t")?;
            for j in 0..self.ncols {
                write!(f, "{}, ", self.get(i, j).unwrap())?;
            }
            //write!(f,"\n")?;
        }
        Ok(())
    }
}

impl<T: TReq> GenericMatrix<T> {
    /// Copies the data from a `GenericMatrix` into `self`.
    ///
    /// # Panics
    /// Panics if the matrices are of different sizes
    pub fn copy_from(&mut self, other: &GenericMatrix<T>) {
        assert_eq!(self.nrows, other.nrows);
        assert_eq!(self.ncols, other.ncols);
        self.data.copy_from_slice(&other.data)
    }

    /// Creates a `GenericMatrix` from a vector containing the elements of the matrix
    #[must_use]
    pub fn from_data(nrows: usize, ncols: usize, data: Vec<T>) -> Self {
        if nrows * ncols != data.len() {
            panic!("When creating Matrix: Number of rows (nrows = {}) and cols (ncols = {}) does not match length of data (data.len() = {})... (nrows * ncols = {})", nrows, ncols, data.len(), nrows*ncols)
        }
        // return
        Self { nrows, ncols, data }
    }

    /// Creates a `GenericMatrix` of `nrows` and `ncols` full of values `v`
    #[must_use]
    pub fn new(v: T, nrows: usize, ncols: usize) -> Self {
        GenericMatrix {
            nrows,
            ncols,
            data: vec![v; nrows * ncols],
        }
    }

    /// Creates a squared matrix with the elements of `data`
    /// in the diagonal
    #[must_use]
    pub fn diag(data: Vec<T>) -> Self {
        let n_rows = data.len();
        let n_elements = n_rows * n_rows;
        let mut v = vec![T::zero(); n_elements];

        for nrow in 0..n_rows {
            let i = nrow * (n_rows + 1);
            v[i] = data[nrow];
        }

        GenericMatrix::from_data(n_rows, n_rows, v)
    }

    /// Creates an empty Matrix (i.e., size 0x0)
    #[must_use]
    pub fn empty() -> Self {
        GenericMatrix {
            nrows: 0,
            ncols: 0,
            data: Vec::with_capacity(0),
        }
    }

    /// Checks whether a Matrix has Zero columns and Zero rows
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nrows == 0 && self.ncols == 0
    }

    /// Creates an Identity matrix of size NxN
    #[must_use]
    pub fn eye(n: usize) -> Self {
        let mut ret = GenericMatrix {
            nrows: n,
            ncols: n,
            data: vec![T::zero(); n * n],
        };

        for i in 0..n {
            ret.set(i, i, T::one()).unwrap();
        }

        // return
        ret
    }

    /// Returns a tuple with number of rows and columns    
    pub fn size(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Gets the index of an element within the `data` array of the Matrix
    fn index(&self, nrow: usize, ncol: usize) -> usize {
        self.ncols * nrow + ncol
    }

    /// Gets an element from the matrix    
    pub fn get(&self, nrow: usize, ncol: usize) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            let i: usize = self.index(nrow, ncol);
            Ok(self.data[i])
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Adds `v` to the element in position `nrow,ncol`.
    pub fn add_to_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<(), String> {
        if nrow < self.nrows && ncol < self.ncols {
            let i: usize = self.index(nrow, ncol);
            self.data[i] += v;
            Ok(())
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Multiplies the element in position `nrow,ncol` by `v`.
    pub fn scale_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<(), String> {
        if nrow < self.nrows && ncol < self.ncols {
            let i: usize = self.index(nrow, ncol);
            self.data[i] *= v;
            Ok(())
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Sets an element into the matrix    
    pub fn set(&mut self, nrow: usize, ncol: usize, v: T) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            let mut i: usize = self.ncols * nrow;
            i += ncol;
            self.data[i] = v;
            Ok(v)
        } else {
            Err("Row or Column out of bounds".to_string())
        }
    }

    /* ARITHMETIC OPERATION */

    /// Adds `self` with `other`, puting the result in `into`    
    pub fn add_into(
        &self,
        other: &GenericMatrix<T>,
        into: &mut GenericMatrix<T>,
    ) -> Result<(), String> {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            return Err("Matrices being added are of different sizes".to_string());
        }

        #[cfg(not(feature = "parallel"))]
        {
            into.data = std::iter::zip(self.data.iter(), other.data.iter())
                .map(|(x, y)| *x + *y)
                .collect();
        }

        #[cfg(feature = "parallel")]
        {
            into.data = self
                .data
                .par_iter()
                .zip(&other.data)
                .map(|(x, y)| *x + *y)
                .collect();
        }

        // return
        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Please use the function `add_into`... it does the same, but the name fits better with the names of other functions"]
    pub fn add(&self, other: &GenericMatrix<T>, ret: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            return Err("Matrices being added are of different sizes".to_string());
        }

        // Add
        for i in 0..self.data.len() {
            ret.data[i] = self.data[i] + other.data[i]
        }

        // return
        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Replace code `let result = a.from_add(&b);` by `let result = &a + &b;`"]
    #[allow(deprecated)]
    pub fn from_add(&self, other: &GenericMatrix<T>) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, self.ncols);

        match self.add(other, &mut ret) {
            Ok(_) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    #[allow(missing_docs)]
    #[deprecated = "Please replace `a.add_to_this(&b)` by `a += &b;`"]
    pub fn add_to_this(&mut self, other: &GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            return Err("Matrices being added are of different sizes".to_string());
        }

        // Add
        for i in 0..self.data.len() {
            self.data[i] += other.data[i]
        }

        // return
        Ok(())
    }

    /// Substracts `other` from `self`, puting the result in `into`    
    pub fn sub_into(
        &self,
        other: &GenericMatrix<T>,
        into: &mut GenericMatrix<T>,
    ) -> Result<(), String> {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            return Err("Matrices being substracted are not the same size".to_string());
        }

        #[cfg(not(feature = "parallel"))]
        {
            into.data = std::iter::zip(self.data.iter(), other.data.iter())
                .map(|(x, y)| *x - *y)
                .collect();
        }

        #[cfg(feature = "parallel")]
        {
            into.data = self
                .data
                .par_iter()
                .zip(&other.data)
                .map(|(x, y)| *x - *y)
                .collect();
        }

        // return
        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Use function `sub_into` instead"]
    pub fn sub(&self, other: &GenericMatrix<T>, ret: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            return Err("Matrices being substracted are not the same size".to_string());
        }

        // Subtract
        for i in 0..self.data.len() {
            ret.data[i] = self.data[i] - other.data[i]
        }

        // return
        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Replace code `let result = a.from_sub(&b);` by `let result = &a - &b;`"]
    #[allow(deprecated)]
    pub fn from_sub(&self, other: &GenericMatrix<T>) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, self.ncols);

        match self.sub(other, &mut ret) {
            Ok(_) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    /// Scales a matrix by `s` and puts the result in `into`     
    pub fn scale_into(&self, s: Float, into: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != into.ncols || self.nrows != into.nrows {
            return Err("Result matrix when scaling is not of appropriate size".to_string());
        }

        #[cfg(not(feature = "parallel"))]
        {
            into.data = self.data.iter().map(|x| *x * s).collect();
        }

        #[cfg(feature = "parallel")]
        {
            into.data = self.data.par_iter().map(|x| *x * s).collect();
        }

        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Please use function `scale_into()` instead... they do the same but the latter fits better within the names of other functions"]
    pub fn scale(&self, s: Float, ret: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != ret.ncols || self.nrows != ret.nrows {
            return Err("Result matrix when scaling is not of appropriate size".to_string());
        }

        // Add
        for i in 0..self.data.len() {
            ret.data[i] = self.data[i] * s;
        }

        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Please replace `let result = a.from_scale(s)` by `let result = &a*s`"]
    #[allow(deprecated)]
    pub fn from_scale(&self, s: Float) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, self.ncols);

        // Add
        self.scale(s, &mut ret)?;

        // return
        Ok(ret)
    }

    #[allow(missing_docs)]
    #[deprecated = "Please replace `let result = a.from_div(s)` by `let result = &a/s`"]
    pub fn from_div(&self, s: T) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, self.ncols);

        // Add
        for i in 0..self.data.len() {
            ret.data[i] = self.data[i] / s;
        }

        // return
        Ok(ret)
    }

    #[allow(missing_docs)]
    #[deprecated = "Please replace `let result = a.from_prod(b)` by `let result = &a*&b`"]
    #[allow(deprecated)]
    pub fn from_prod(&self, other: &GenericMatrix<T>) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, other.ncols);

        match self.prod(other, &mut ret) {
            Ok(_) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    #[allow(missing_docs)]
    #[deprecated = "Please change the code from `a.scale_this(s)` to `a *= s`"]
    pub fn scale_this(&mut self, s: T) {
        for i in 0..self.data.len() {
            self.data[i] *= s;
        }
    }

    #[allow(missing_docs)]
    #[deprecated = "Please use function `div_into()` instead... they do the same but the latter fits better within the names of other functions"]
    pub fn div(&mut self, s: T) {
        for i in 0..self.data.len() {
            self.data[i] /= s;
        }
    }

    /// Multiplies a matrix by `other`, putting the result into `into`    
    #[allow(clippy::needless_collect)]
    pub fn prod_into(
        &self,
        other: &GenericMatrix<T>,
        into: &mut GenericMatrix<T>,
    ) -> Result<(), String> {
        if self.ncols != other.nrows {
            return Err("Size mismatch for GenericMatrix multiplication".to_string());
        }

        if into.nrows != self.nrows || into.ncols != other.ncols {
            return Err(
                "Result matrix size mismatch for GenericMatrix n-diag multiplication".to_string(),
            );
        }

        // Multiply.
        let i: Vec<&[T]> = self.data.chunks_exact(self.ncols).collect();
        #[cfg(not(feature = "parallel"))]
        let i = i.into_iter().zip(into.data.chunks_exact_mut(other.ncols));

        #[cfg(feature = "parallel")]
        let i = i
            .into_par_iter()
            .zip(into.data.par_chunks_exact_mut(other.ncols));

        let _ = i.for_each(|(row_data, into_data)| {
            for (c, item) in into_data.iter_mut().enumerate().take(other.ncols) {
                // Add the numbers
                for (i, a) in row_data.iter().enumerate().take(other.nrows) {
                    // let a = *inner_item; //row_data[i];
                    let other_i = other.index(i, c);

                    let b = other.data[other_i];
                    *item += *a * b;
                }
            }
        });

        // return
        Ok(())
    }

    #[allow(missing_docs)]
    #[deprecated = "Please use the better-named prod_into function... they do the same"]
    pub fn prod(&self, other: &GenericMatrix<T>, ret: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != other.nrows {
            return Err("Size mismatch for GenericMatrix multiplication".to_string());
        }

        if ret.nrows != self.nrows || ret.ncols != other.ncols {
            return Err(
                "Result matrix size mismatch for GenericMatrix n-diag multiplication".to_string(),
            );
        }

        // Multiply.
        for r in 0..self.nrows {
            for c in 0..other.ncols {
                // (r,c) is the position in the resulting matrix.
                let mut v = T::zero();

                // Add the numbers
                for i in 0..other.nrows {
                    let a_i = self.index(r, i);
                    let a = self.data[a_i];
                    let b_i = other.index(i, c);
                    let b = other.data[b_i];
                    v += a * b;
                }

                // Set the value
                match ret.set(r, c, v) {
                    Ok(_v) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        // return
        Ok(())
    }

    /// Multiplies a tri-diagonal matrix `self` by another matrix `other`, puting the results into `into`. When matrices are large
    /// this can be much faster than just multiplying, as we acknowledge that most of the Matrix
    /// will be Zeroes.    
    pub fn prod_tri_diag_into(
        &self,
        other: &GenericMatrix<T>,
        into: &mut GenericMatrix<T>,
    ) -> Result<(), String> {
        self.prod_n_diag_into(other, 3, into)
    }

    /// Multiplies an `n`-diagonal matrix `self` by another matrix `other`. When matrices are large
    /// this can be much faster than just multiplying, as we acknowledge that most of the Matrix
    /// will be Zeroes.
    pub fn prod_n_diag_into(
        &self,
        other: &GenericMatrix<T>,
        n: usize,
        into: &mut GenericMatrix<T>,
    ) -> Result<(), String> {
        if self.ncols != other.nrows {
            return Err("Size mismatch for GenericMatrix n-diag multiplication".to_string());
        }

        if into.nrows != self.nrows || into.ncols != other.ncols {
            return Err(
                "Result matrix size mismatch for GenericMatrix n-diag multiplication".to_string(),
            );
        }

        // Multiply.
        let i: Vec<&[T]> = self.data.chunks_exact(self.ncols).collect();
        #[cfg(not(feature = "parallel"))]
        let i = i.into_iter().zip(into.data.chunks_exact_mut(other.ncols));

        #[cfg(feature = "parallel")]
        let i = i
            .into_par_iter()
            .zip(into.data.par_chunks_exact_mut(other.ncols));

        let one_sided_n = one_sided_n!(n);
        let _ = i.enumerate().for_each(|(r, (row_data, into_data))| {
            // for c in 0..other.ncols {
            for (c, item) in into_data.iter_mut().enumerate().take(other.ncols) {
                // Add before the diagonal

                let ini = if r >= one_sided_n { r - one_sided_n } else { 0 };

                let fin = r;
                // for i in ini..fin {
                for (i, a) in row_data.iter().enumerate().take(fin).skip(ini) {
                    let other_i = other.index(i, c);
                    let b = other.data[other_i];
                    *item += *a * b;
                }

                // Add the diagonal
                let a = row_data[r];
                let other_i = other.index(r, c);
                let b = other.data[other_i];
                *item += a * b;

                // Add after the diagonal

                let ini = if r < self.ncols { r + 1 } else { self.ncols };

                let fin = if r + one_sided_n + 1 > self.ncols {
                    self.ncols
                } else {
                    r + one_sided_n + 1
                };

                // for i in ini..fin {
                for (i, a) in row_data.iter().enumerate().take(fin).skip(ini) {
                    let other_i = other.index(i, c);
                    let b = other.data[other_i];
                    *item += *a * b;
                }
            }
        });

        // return
        Ok(())
    } // end of function

    /// Multiplies an `n`-diagonal matrix `self` by another `other` and returns an `Result<Matrix>`
    pub fn from_prod_n_diag(
        &self,
        other: &GenericMatrix<T>,
        n: usize,
    ) -> Result<GenericMatrix<T>, String> {
        // Initialize matrix full of zeroes
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, other.ncols);
        self.prod_n_diag_into(other, n, &mut ret)?;
        Ok(ret)
    }

    /// Checks if two matrices are exactly the same (as in `element == other_element`... beware Floats).
    pub fn compare(&self, other: &GenericMatrix<T>) -> bool {
        if self.ncols != other.ncols {
            return false;
        }
        if self.nrows != other.nrows {
            return false;
        }
        for i in 0..self.data.len() {
            if self.data[i] != other.data[i] {
                return false;
            }
        }
        // return
        true
    }

    /// Concatenates the rows in `other` below the rows of `self`    
    pub fn concat_rows(&mut self, other: &GenericMatrix<T>) -> Result<(), String> {
        let (other_rows, other_cols) = other.size();
        if self.ncols != other_cols {
            return Err(
                "Mismatch in the rows of `self` and `other` when concatenating rows".to_string(),
            );
        }
        // Resize self
        let len_self = self.data.len();
        let len_other = other.data.len();
        let final_length = len_self + len_other;
        self.nrows += other_rows;
        self.data.resize(final_length, T::zero());

        // Copy values
        self.data[len_self..final_length].copy_from_slice(&other.data);

        Ok(())
    }
}

impl<T: TReq> std::ops::Add<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn add(self, other: &GenericMatrix<T>) -> Self::Output {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }

        let ret_data = {
            #[cfg(not(feature = "parallel"))]
            {
                self.data
                    .iter()
                    .zip(&other.data)
                    .map(|(x, y)| *x + *y)
                    .collect()
            }

            #[cfg(feature = "parallel")]
            {
                self.data
                    .par_iter()
                    .zip(&other.data)
                    .map(|(x, y)| *x + *y)
                    .collect()
            }
        };
        // let z = self.data.iter().zip(other.data.iter());
        // let ret_data : Vec<T> = z.map(|(a, b)| *a + *b).collect();

        GenericMatrix::from_data(self.nrows, self.ncols, ret_data)
    }
}

impl<T: TReq> std::ops::AddAssign<&GenericMatrix<T>> for GenericMatrix<T> {
    fn add_assign(&mut self, other: &GenericMatrix<T>) {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }

        #[cfg(not(feature = "parallel"))]
        self.data
            .iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a += *b);

        #[cfg(feature = "parallel")]
        self.data
            .par_iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a += *b);
    }
}

impl<T: TReq> std::ops::Sub<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn sub(self, other: &GenericMatrix<T>) -> Self::Output {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }
        let ret_data = {
            #[cfg(not(feature = "parallel"))]
            {
                self.data
                    .iter()
                    .zip(&other.data)
                    .map(|(x, y)| *x - *y)
                    .collect()
            }

            #[cfg(feature = "parallel")]
            {
                self.data
                    .par_iter()
                    .zip(&other.data)
                    .map(|(x, y)| *x - *y)
                    .collect()
            }
        };
        GenericMatrix::from_data(self.nrows, self.ncols, ret_data)
    }
}

impl<T: TReq> std::ops::SubAssign<&GenericMatrix<T>> for GenericMatrix<T> {
    fn sub_assign(&mut self, other: &GenericMatrix<T>) {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being substracted are of different sizes");
        }
        #[cfg(not(feature = "parallel"))]
        self.data
            .iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a -= *b);

        #[cfg(feature = "parallel")]
        self.data
            .par_iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a -= *b);
    }
}

impl<T: TReq> std::ops::Mul<T> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn mul(self, s: T) -> Self::Output {
        let ret_data = {
            #[cfg(not(feature = "parallel"))]
            {
                self.data.iter().map(|a| *a * s).collect()
            }
            #[cfg(feature = "parallel")]
            {
                self.data.par_iter().map(|a| *a * s).collect()
            }
        };

        GenericMatrix::from_data(self.nrows, self.ncols, ret_data)
    }
}

impl<T: TReq> std::ops::Mul<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;
    fn mul(self, other: &GenericMatrix<T>) -> Self::Output {
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, other.ncols);
        self.prod_into(other, &mut ret).unwrap();
        ret
    }
}

impl<T: TReq> std::ops::MulAssign<T> for GenericMatrix<T> {
    fn mul_assign(&mut self, s: T) {
        // self.data.iter_mut().for_each(|a| *a *= s);
        #[cfg(not(feature = "parallel"))]
        self.data.iter_mut().for_each(|a| *a *= s);

        #[cfg(feature = "parallel")]
        self.data.par_iter_mut().for_each(|a| *a *= s);
    }
}

impl<T: TReq> std::ops::Div<T> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;
    fn div(self, s: T) -> Self::Output {
        let ret_data = {
            #[cfg(not(feature = "parallel"))]
            {
                self.data.iter().map(|a| *a / s).collect()
            }
            #[cfg(feature = "parallel")]
            {
                self.data.par_iter().map(|a| *a / s).collect()
            }
        };

        GenericMatrix::from_data(self.nrows, self.ncols, ret_data)
    }
}

impl<T: TReq> std::ops::DivAssign<T> for GenericMatrix<T> {
    fn div_assign(&mut self, s: T) {
        #[cfg(not(feature = "parallel"))]
        self.data.iter_mut().for_each(|a| *a /= s);

        #[cfg(feature = "parallel")]
        self.data.par_iter_mut().for_each(|a| *a /= s);
    }
}

#[cfg(test)]
mod test;
