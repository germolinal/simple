use crate::generic_matrix::GenericMatrix;
use crate::Float;

/// A shorthand for `GenericMatrix<Float>`; i.e., a normal
/// matrix. Note that `Float` is defined as `f32` if the feature `float`
/// is utilized; otherwise, it defauts to `f64`.
pub type Matrix = GenericMatrix<Float>;

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
        let mut a = self.clone();
        let mut b = b.clone();
        a.mut_n_diag_gaussian(&mut b, n)?;
        Ok(b)
    }

    /// Solves an $`A \times x=b`$ problem using the  [Gaussian Elimination](https://en.wikipedia.org/wiki/Gaussian_elimination)
    /// algorithm. It assumes that $`A`$ is `n`-diagonal (e.g., [tri-diaglnal](https://en.wikipedia.org/wiki/Tridiagonal_matrix)).
    ///
    /// Both `self` and `b` are modified in the process, `b` becomes the answer.
    ///
    /// Returns an error if an element in the diagonal of the matrix is Zero (row-swapping is not
    /// yet supported)
    ///
    /// # Note
    /// It mutates both `self` and `b`. If this is not what you want, call `n_diag_gaussian()` instead
    pub fn mut_n_diag_gaussian(&mut self, b: &mut Matrix, n: usize) -> Result<(), String> {
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
            scale_row(self, b, c, 1. / pivot);

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
                        add_rows(self, b, c, r, -other);
                    }
                } else {
                    break;
                }
            }
        } // end of going down.

        // Now, run substitution upwards (We don't need to iterate the first row)
        for c in (1..self.ncols).rev() {
            let pivot = self.data[self.index(c, c)];
            debug_assert!((1. - pivot).abs() < 1e-15, "pivot is {}", pivot);

            let ini = if c < one_sided_n { 0 } else { c - one_sided_n };

            for r in (ini..c).rev() {
                if let Ok(other) = self.get(r, c) {
                    // if it is Zero already, just skip
                    if other.abs() > TINY {
                        add_rows(self, b, c, r, -other);
                    }
                }
            }
        }
        Ok(())
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

        Err(format!(
            "Gauss-Seidel algorithm did not converge after {} iterations",
            max_iter
        ))
    }
}
