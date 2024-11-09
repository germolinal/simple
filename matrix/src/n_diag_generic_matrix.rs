use crate::traits::Numberish;
use crate::Float;
use serde::{Deserialize, Serialize};

/// A compact version of an N-Diagonal matrix
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NDiagGenericMatrix<const N: usize, T: Numberish> {
    pub(crate) ncols: usize,
    pub(crate) nrows: usize,

    // Contains the data ordered by row,
    // Going left to right, and up and down.
    pub(crate) data: Vec<T>,
}

impl<const N: usize, T: Numberish> std::fmt::Display for NDiagGenericMatrix<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.nrows {
            write!(f, "\n\t")?;
            for j in 0..self.ncols {
                write!(f, "{}, ", self.get(i, j).unwrap())?;
            }
        }
        Ok(())
    }
}

impl<const N: usize, T: Numberish> NDiagGenericMatrix<N, T> {
    const fn validate_n() {
        // Assert at runtime that N is non-zero and even
        assert!(N > 0, "N in N-Diagonal matrix must be non-zero.");
        assert!(N % 2 != 0, "N in N-Diagonal matrix must be an odd number.");
    }

    /// Creates a new `NDiagGenericMatrix` of `nrows` and `ncols` full of values `v`
    #[must_use]
    pub fn new(v: T, nrows: usize, ncols: usize) -> Self {
        Self::validate_n();

        // There is a central set of rows in every n-diag matrix that
        // has enough room to contain the whole n-diag. However, there
        // is also a top and bottom padding, where the n-diag does not fit
        // fully
        //
        // E.g.,
        // | 1, 1, 0, 0, 0 | --> Top padding
        // | 1, 1, 1, 0, 0 | --> Full on
        // | 0, 1, 1, 1, 0 | --> Full on
        // | 0, 0, 1, 1, 1 | --> Full on
        // | 0, 0, 0, 1, 1 | --> Bottom padding
        //
        // Let's count the non-zero elements in this thing, slowly.
        // This is not very efficient... but let's optimize it
        // when some profile states we have to.
        let n = Self::acc_n_in_row(nrows, ncols);
        let data = vec![v; n];
        let data = data
            .iter()
            .enumerate()
            .map(|(i, _)| T::one() * i as Float)
            .collect();

        Self { nrows, ncols, data }
    }

    /// Copies the data from a `GenericMatrix` into `self`.
    ///
    /// # Panics
    /// Panics if the matrices are of different sizes
    pub fn copy_from(&mut self, other: &Self) {
        assert_eq!(self.nrows, other.nrows);
        assert_eq!(self.ncols, other.ncols);
        self.data.copy_from_slice(&other.data)
    }

    fn acc_n_in_row(row: usize, ncols: usize) -> usize {
        let mut n = 0;
        (0..row).for_each(|r| {
            n += Self::n_in_row(r, ncols);
        });
        n
    }

    const fn zeroes_before(row: usize) -> usize {
        let s = one_sided_n!(N);
        if row < s {
            // Top pading... no Zeroes
            0
        } else {
            // this can be infinite, potentially. Handle this value
            // with care
            row - s
        }
    }

    const fn n_in_row(row: usize, ncols: usize) -> usize {
        let s = one_sided_n!(N);
        let n_zeroes_before = Self::zeroes_before(row);
        if row < s {
            // top padding
            s + 1 + row
        } else if n_zeroes_before + N <= ncols {
            // somewhere passed the top padding.
            N
        } else {
            // Bottom padding
            if n_zeroes_before < ncols {
                // there are some values
                ncols - n_zeroes_before
            } else {
                // just zeroes here.
                0
            }
        }
    }

    /// Gets the index of an element within the `data` array of the Matrix
    fn index(&self, nrow: usize, ncol: usize) -> Result<usize, String> {
        if Self::is_diag_value(nrow, ncol) {
            let n = Self::acc_n_in_row(nrow, self.ncols) + ncol - Self::zeroes_before(nrow);
            Ok(n)
        } else {
            Err(format!(
                "Trying to access element ({},{}), which is off diagonal of {}-diagonal matrix",
                nrow, ncol, N
            ))
        }
    }

    /// States whether the element should contain a valid number
    fn is_diag_value(nrow: usize, ncol: usize) -> bool {
        let s = one_sided_n!(N);
        ncol + s >= nrow && ncol <= nrow + s
    }

    /// Returns a tuple with number of rows and columns
    pub fn size(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Gets an element from the matrix
    pub fn get(&self, nrow: usize, ncol: usize) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            if Self::is_diag_value(nrow, ncol) {
                let i = self.index(nrow, ncol)?;
                Ok(self.data[i])
            } else {
                Ok(T::zero())
            }
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Sets an element from the matrix
    pub fn set(&mut self, nrow: usize, ncol: usize, v: T) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            if Self::is_diag_value(nrow, ncol) {
                let i = self.index(nrow, ncol)?;
                self.data[i] = v;
                Ok(v)
            } else {
                Err(format!(
                    "Trying to set element ({},{}), which is off diagonal of {}-diagonal matrix",
                    nrow, ncol, N
                ))
            }
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Sets an element from the matrix
    pub fn add_to_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            if Self::is_diag_value(nrow, ncol) {
                let i = self.index(nrow, ncol)?;
                self.data[i] += v;
                Ok(v)
            } else {
                Err(format!(
                    "Trying to set element ({},{}), which is off diagonal of {}-diagonal matrix",
                    nrow, ncol, N
                ))
            }
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Sets an element from the matrix
    pub fn scale_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            if Self::is_diag_value(nrow, ncol) {
                let i = self.index(nrow, ncol)?;
                self.data[i] *= v;
                Ok(v)
            } else {
                Err(format!(
                    "Trying to set element ({},{}), which is off diagonal of {}-diagonal matrix",
                    nrow, ncol, N
                ))
            }
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Multiplies this column with a column vector.
    pub fn column_prod_into(&self, vec: &[T], into: &mut Vec<T>) -> Result<(), String> {
        if vec.len() != self.ncols {
            return Err(format!(
                "Size mismatch. Column vector has {} elements, matrix has {} columns",
                vec.len(),
                self.ncols
            ));
        }
        if vec.len() != into.len() {
            return Err(format!(
                "Size mismatch. Column vector has {} elements, destination vector has {} elements",
                vec.len(),
                into.len()
            ));
        }

        let mut acc = 0;
        for row in 0..vec.len() {
            let non_zeroes = Self::n_in_row(row, self.ncols);
            into[row] = T::zero();
            if non_zeroes > 0 {
                let this_data = &self.data[acc..acc + non_zeroes];
                acc += non_zeroes;

                let zeroes_before = Self::zeroes_before(row);
                let ini = zeroes_before;
                let fin = ini + non_zeroes;
                let other_data = &vec[ini..fin];

                for i in 0..non_zeroes {
                    into[row] += this_data[i] * other_data[i];
                }
            }
        }

        Ok(())
    }

    /// Scales a matrix by `s` and puts the result in `into`
    pub fn scale_into(&self, s: Float, into: &mut Self) -> Result<(), String> {
        if self.ncols != into.ncols || self.nrows != into.nrows {
            return Err("Result matrix when scaling is not of appropriate size".to_string());
        }

        std::iter::zip(into.data.iter_mut(), self.data.iter())
            .for_each(|(to, from)| *to = *from * s);

        Ok(())
    }
}

impl<const N: usize, T: Numberish> std::ops::MulAssign<T> for NDiagGenericMatrix<N, T> {
    fn mul_assign(&mut self, s: T) {
        self.data.iter_mut().for_each(|a| *a *= s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let aux = NDiagGenericMatrix::<3, Float>::new(1.0, 8, 5);
        print!("{aux}")
    }
}
