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

//! Daylight coefficient matrices, prepared for fast multiplication.

use crate::Float;
use matrix::Matrix;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Rows with fewer than this fraction of non-zero values are stored sparse.
///
/// Benchmarks (`light/benches/dc_matrix.rs`) show that sparse rows are faster
/// than dense ones up to ~95% of non-zero values.
const MAX_SPARSE_DENSITY: Float = 0.9;

/// With the `parallel` feature, products with at least this many
/// multiplications are calculated in parallel (one row per task).
#[cfg(feature = "parallel")]
const MIN_PARALLEL_WORK: usize = 32_768;

/// One row of a [`DCMatrix`]
#[derive(Debug, Clone)]
enum Row {
    /// All values are zero
    Zero,
    /// The values of every column
    Dense(Vec<Float>),
    /// The non-zero values, and their columns (in ascending order)
    Sparse {
        columns: Vec<usize>,
        values: Vec<Float>,
    },
}

impl Row {
    /// Multiplies this row by `sky`, adding the terms in ascending column order
    fn times(&self, sky: &[Float]) -> Float {
        match self {
            Row::Zero => 0.0,
            Row::Dense(values) => values
                .iter()
                .zip(sky)
                .fold(0.0, |acc, (v, s)| acc + *v * *s),
            Row::Sparse { columns, values } => columns
                .iter()
                .zip(values)
                .fold(0.0, |acc, (c, v)| acc + *v * sky[*c]),
        }
    }

    /// The number of multiplications done by `times()`
    fn work(&self) -> usize {
        match self {
            Row::Zero => 0,
            Row::Dense(values) => values.len(),
            Row::Sparse { values, .. } => values.len(),
        }
    }
}

/// A daylight coefficient matrix (one row per surface, one column per sky
/// bin), prepared for being multiplied by sky vectors in every timestep.
///
/// Surfaces that see only part of the sky (e.g., walls) have many zero
/// coefficients, so their rows are stored sparse. Rows that are mostly
/// non-zero (e.g., roofs, or when solar radiation bounces between surfaces)
/// are stored dense, so no case is slower than a dense product.
///
/// Results are exactly the same as those of multiplying the original
/// [`Matrix`]: non-zero terms are added in the same order, and the terms
/// skipped are exactly zero, so they would not change the sums. With the
/// `parallel` feature, large products are split by rows, and each row is
/// still added up in order, so results do not change either.
#[derive(Debug, Clone)]
pub struct DCMatrix {
    ncols: usize,
    rows: Vec<Row>,
    /// The number of multiplications in each product
    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    work: usize,
}

impl From<&Matrix> for DCMatrix {
    fn from(m: &Matrix) -> Self {
        let (nrows, ncols) = m.size();
        if nrows == 0 || ncols == 0 {
            return Self {
                ncols,
                rows: Vec::new(),
                work: 0,
            };
        }
        let rows = m
            .as_slice()
            .chunks_exact(ncols)
            .map(|row| {
                let non_zero = row.iter().filter(|v| **v != 0.0).count();
                if non_zero == 0 {
                    Row::Zero
                } else if (non_zero as Float) < MAX_SPARSE_DENSITY * ncols as Float {
                    let (columns, values) = row
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| **v != 0.0)
                        .map(|(c, v)| (c, *v))
                        .unzip();
                    Row::Sparse { columns, values }
                } else {
                    Row::Dense(row.to_vec())
                }
            })
            .collect::<Vec<Row>>();
        let work = rows.iter().map(Row::work).sum();
        Self { ncols, rows, work }
    }
}

impl DCMatrix {
    /// The number of rows (i.e., surfaces)
    pub fn nrows(&self) -> usize {
        self.rows.len()
    }

    /// Checks whether the matrix has no rows
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Multiplies `self` by the column vector `sky`, putting the results in
    /// `into` (one value per row).
    pub fn mul_vec_into(&self, sky: &[Float], into: &mut [Float]) -> Result<(), String> {
        if sky.len() != self.ncols {
            return Err(format!(
                "Size mismatch: daylight coefficient matrix has {} columns, sky vector has {} elements",
                self.ncols,
                sky.len()
            ));
        }
        if into.len() != self.rows.len() {
            return Err(format!(
                "Size mismatch: daylight coefficient matrix has {} rows, result has {} elements",
                self.rows.len(),
                into.len()
            ));
        }

        #[cfg(feature = "parallel")]
        if self.work >= MIN_PARALLEL_WORK {
            self.rows
                .par_iter()
                .zip(into.par_iter_mut())
                .for_each(|(row, res)| *res = row.times(sky));
            return Ok(());
        }

        for (row, res) in self.rows.iter().zip(into.iter_mut()) {
            *res = row.times(sky);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-random numbers in [0, 1)
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> Float {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 11) as f64 / (1u64 << 53) as f64) as Float
        }
    }

    /// Builds a matrix in which each row has a different fraction of
    /// non-zero values, from all zeros to all non-zero.
    fn mixed_matrix(nrows: usize, ncols: usize, rng: &mut Lcg) -> Matrix {
        let mut m = Matrix::new(0.0, nrows, ncols);
        for r in 0..nrows {
            let density = r as Float / (nrows - 1) as Float;
            for c in 0..ncols {
                if rng.next() < density {
                    m.set(r, c, rng.next() * 10. - 2.).unwrap();
                }
            }
        }
        m
    }

    #[test]
    fn test_rows_kinds() {
        let mut rng = Lcg(1);
        let dc = DCMatrix::from(&mixed_matrix(11, 145, &mut rng));
        assert!(matches!(dc.rows[0], Row::Zero));
        assert!(matches!(dc.rows[1], Row::Sparse { .. }));
        assert!(matches!(dc.rows[10], Row::Dense(_)));
    }

    #[test]
    fn test_same_as_matrix_product() {
        let mut rng = Lcg(7);
        for (nrows, ncols, dense) in [
            (1, 1, true),
            (2, 3, false),
            (11, 145, false),
            (30, 2305, false),
            (60, 2305, true), // with `parallel`, this one is split by rows
        ] {
            let m = if dense {
                let data = (0..nrows * ncols).map(|_| rng.next()).collect();
                Matrix::from_data(nrows, ncols, data)
            } else {
                mixed_matrix(nrows, ncols, &mut rng)
            };
            let sky: Vec<Float> = (0..ncols).map(|_| rng.next() * 100. - 10.).collect();

            let expected = &m * &Matrix::from_data(ncols, 1, sky.clone());
            let mut found = vec![0.0; nrows];
            DCMatrix::from(&m).mul_vec_into(&sky, &mut found).unwrap();

            for (a, b) in found.iter().zip(expected.as_slice()) {
                assert_eq!(a.to_bits(), b.to_bits(), "{nrows}x{ncols}: {a} != {b}");
            }
        }
    }

    #[test]
    fn test_empty() {
        let dc = DCMatrix::from(&Matrix::empty());
        assert!(dc.is_empty());
        assert!(dc.mul_vec_into(&[], &mut []).is_ok());
    }

    #[test]
    fn test_size_mismatch() {
        let dc = DCMatrix::from(&Matrix::new(1.0, 2, 3));
        assert!(dc.mul_vec_into(&[1.0; 2], &mut [0.0; 2]).is_err());
        assert!(dc.mul_vec_into(&[1.0; 3], &mut [0.0; 3]).is_err());
    }
}
