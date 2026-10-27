use crate::Float;
use crate::traits::Numberish;
use serde::{Deserialize, Serialize};

/// A compact version of an N-Diagonal matrix.
///
/// The elements are stored in "band storage": each row keeps the `N` elements
/// around its diagonal, from column `row - s` to column `row + s` (where
/// `s = (N - 1) / 2`), so element `(row, col)` is at
/// `row * N + (col + s - row)`. At the top and bottom corners some of those
/// positions fall outside the matrix; they are kept as padding, and never read.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawNDiagGenericMatrix<T>")]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct NDiagGenericMatrix<const N: usize, T: Numberish> {
    pub(crate) ncols: usize,
    pub(crate) nrows: usize,

    // Contains N elements per row (see above)
    pub(crate) data: Vec<T>,
}

/// What a [`NDiagGenericMatrix`] is deserialized from, before checking that
/// its size is consistent.
#[derive(Deserialize)]
struct RawNDiagGenericMatrix<T> {
    ncols: usize,
    nrows: usize,
    data: Vec<T>,
}

impl<const N: usize, T: Numberish> TryFrom<RawNDiagGenericMatrix<T>> for NDiagGenericMatrix<N, T> {
    type Error = String;
    fn try_from(raw: RawNDiagGenericMatrix<T>) -> Result<Self, String> {
        let expected = raw.nrows * N;
        if raw.data.len() != expected {
            return Err(format!(
                "{}-diagonal matrix has {} rows and {} columns, but {} elements (expected {})",
                N,
                raw.nrows,
                raw.ncols,
                raw.data.len(),
                expected
            ));
        }
        Ok(Self {
            ncols: raw.ncols,
            nrows: raw.nrows,
            data: raw.data,
        })
    }
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
    /// The number of diagonals at each side of the main one
    const S: usize = one_sided_n!(N);

    const fn validate_n() {
        // Assert at runtime that N is non-zero and even
        assert!(N > 0, "N in N-Diagonal matrix must be non-zero.");
        assert!(!N.is_multiple_of(2), "N in N-Diagonal matrix must be an odd number.");
    }

    /// Creates a new `NDiagGenericMatrix` of `nrows` and `ncols`, with value `v`
    /// in every element of the n-diag.
    #[must_use]
    pub fn new(v: T, nrows: usize, ncols: usize) -> Self {
        Self::validate_n();
        Self {
            nrows,
            ncols,
            data: vec![v; nrows * N],
        }
    }

    /// Sets all the elements within the n-diag to `v`
    pub fn fill(&mut self, v: T) {
        self.data.fill(v);
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

    /// The columns of `row` that are within the n-diag (and the matrix)
    fn columns_in_row(&self, row: usize) -> std::ops::Range<usize> {
        let ini = row.saturating_sub(Self::S);
        let fin = (row + Self::S + 1).min(self.ncols);
        ini..fin.max(ini)
    }

    /// Gets the index of an element within the `data` array of the Matrix
    fn index(&self, nrow: usize, ncol: usize) -> Result<usize, String> {
        if Self::is_diag_value(nrow, ncol) {
            Ok(self.band_index(nrow, ncol))
        } else {
            Err(format!(
                "Trying to access element ({},{}), which is off diagonal of {}-diagonal matrix",
                nrow, ncol, N
            ))
        }
    }

    /// Like `index()`, for elements known to be within the n-diag.
    pub(crate) fn band_index(&self, nrow: usize, ncol: usize) -> usize {
        debug_assert!(Self::is_diag_value(nrow, ncol));
        nrow * N + ncol + Self::S - nrow
    }

    /// States whether the element should contain a valid number
    fn is_diag_value(nrow: usize, ncol: usize) -> bool {
        ncol + Self::S >= nrow && ncol <= nrow + Self::S
    }

    /// Returns a tuple with number of rows and columns
    pub fn size(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Gets an element from the matrix
    pub fn get(&self, nrow: usize, ncol: usize) -> Result<T, String> {
        if nrow < self.nrows && ncol < self.ncols {
            if Self::is_diag_value(nrow, ncol) {
                Ok(self.data[self.band_index(nrow, ncol)])
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
            let i = self.index(nrow, ncol)?;
            self.data[i] = v;
            Ok(v)
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Adds `v` to the element in position `nrow,ncol`.
    pub fn add_to_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<(), String> {
        if nrow < self.nrows && ncol < self.ncols {
            let i = self.index(nrow, ncol)?;
            self.data[i] += v;
            Ok(())
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Multiplies the element in position `nrow,ncol` by `v`.
    pub fn scale_element(&mut self, nrow: usize, ncol: usize, v: T) -> Result<(), String> {
        if nrow < self.nrows && ncol < self.ncols {
            let i = self.index(nrow, ncol)?;
            self.data[i] *= v;
            Ok(())
        } else {
            Err("Row or Column out of bounds.".to_string())
        }
    }

    /// Multiplies this column with a column vector.
    pub fn column_prod_into(&self, vec: &[T], into: &mut [T]) -> Result<(), String> {
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

        for (row, item) in into.iter_mut().enumerate() {
            *item = T::zero();
            if row >= self.nrows {
                continue;
            }
            let cols = self.columns_in_row(row);
            let first = if cols.is_empty() {
                0
            } else {
                self.band_index(row, cols.start)
            };
            let this_data = &self.data[first..first + cols.len()];
            for (a, b) in this_data.iter().zip(&vec[cols]) {
                *item += *a * *b;
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

/// Accesses an element within the n-diag.
///
/// # Panics
/// Panics if the element is out of bounds, or outside of the n-diag (use
/// `get()` for reading those as zeroes).
impl<const N: usize, T: Numberish> std::ops::Index<(usize, usize)> for NDiagGenericMatrix<N, T> {
    type Output = T;
    fn index(&self, (nrow, ncol): (usize, usize)) -> &T {
        assert!(
            nrow < self.nrows && ncol < self.ncols && Self::is_diag_value(nrow, ncol),
            "Element ({nrow},{ncol}) is out of bounds or outside the n-diag of a {}x{} {N}-diagonal matrix",
            self.nrows,
            self.ncols
        );
        &self.data[self.band_index(nrow, ncol)]
    }
}

/// Accesses an element within the n-diag.
///
/// # Panics
/// Panics if the element is out of bounds, or outside of the n-diag.
impl<const N: usize, T: Numberish> std::ops::IndexMut<(usize, usize)> for NDiagGenericMatrix<N, T> {
    fn index_mut(&mut self, (nrow, ncol): (usize, usize)) -> &mut T {
        assert!(
            nrow < self.nrows && ncol < self.ncols && Self::is_diag_value(nrow, ncol),
            "Element ({nrow},{ncol}) is out of bounds or outside the n-diag of a {}x{} {N}-diagonal matrix",
            self.nrows,
            self.ncols
        );
        let i = self.band_index(nrow, ncol);
        &mut self.data[i]
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

    /// Every element within the n-diag gets its own position in `data`, and
    /// elements outside it read as zero.
    fn check_index_mapping<const N: usize>() {
        for (nrows, ncols) in [(1, 1), (2, 2), (5, 5), (8, 5), (5, 8), (12, 12)] {
            let mut m = NDiagGenericMatrix::<N, Float>::new(0.0, nrows, ncols);
            let s = (N - 1) / 2;
            let in_band = |r: usize, c: usize| r.abs_diff(c) <= s;
            let value = |r: usize, c: usize| (1 + r * 100 + c) as Float;
            for r in 0..nrows {
                for c in 0..ncols {
                    if in_band(r, c) {
                        m.set(r, c, value(r, c)).unwrap();
                        assert_eq!(m[(r, c)], value(r, c));
                    } else {
                        assert!(m.set(r, c, 1.0).is_err());
                    }
                }
            }
            for r in 0..nrows {
                for c in 0..ncols {
                    let expected = if in_band(r, c) { value(r, c) } else { 0.0 };
                    assert_eq!(
                        m.get(r, c).unwrap(),
                        expected,
                        "N={N} {nrows}x{ncols} ({r},{c})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_index_mapping() {
        check_index_mapping::<1>();
        check_index_mapping::<3>();
        check_index_mapping::<5>();
    }

    #[test]
    fn test_column_prod_into() {
        // [2 1 0; 1 2 1; 0 1 2] * [1 2 3]' = [4 8 8]'
        let mut m = NDiagGenericMatrix::<3, Float>::new(1.0, 3, 3);
        for i in 0..3 {
            m[(i, i)] = 2.0;
        }
        let mut into = vec![Float::NAN; 3];
        m.column_prod_into(&[1., 2., 3.], &mut into).unwrap();
        assert_eq!(into, vec![4., 8., 8.]);
    }

    #[test]
    #[should_panic]
    fn test_index_outside_band() {
        let m = NDiagGenericMatrix::<3, Float>::new(1.0, 5, 5);
        let _ = m[(0, 3)];
    }

    #[test]
    fn test_new_fills_with_value() {
        let m = NDiagGenericMatrix::<3, Float>::new(2.5, 6, 6);
        for r in 0..6_usize {
            for c in 0..6_usize {
                let expected = if r.abs_diff(c) <= 1 { 2.5 } else { 0.0 };
                assert_eq!(m.get(r, c).unwrap(), expected, "({r},{c})");
            }
        }
    }

    #[test]
    fn test_fill() {
        let mut m = NDiagGenericMatrix::<3, Float>::new(Float::NAN, 4, 4);
        m.fill(0.0);
        assert!(m.data.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn test_serde() {
        let m = NDiagGenericMatrix::<3, Float>::new(1.0, 4, 4);
        let s = serde_json::to_string(&m).unwrap();
        let back: NDiagGenericMatrix<3, Float> = serde_json::from_str(&s).unwrap();
        assert_eq!(back.data, m.data);

        let r: Result<NDiagGenericMatrix<3, Float>, _> =
            serde_json::from_str(r#"{"ncols":4,"nrows":4,"data":[1.0,2.0]}"#);
        assert!(r.is_err());
    }
}
