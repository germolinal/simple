use crate::n_diag_generic_matrix::NDiagGenericMatrix;
use crate::Float;
use crate::Matrix;

/// A compact version of an N-diagonal matrix, for floating point numbers
pub type NDiagMatrix<const N: usize> = NDiagGenericMatrix<N, Float>;

impl<const N: usize> NDiagMatrix<N> {
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
    pub fn mut_gaussian(&mut self, b: &mut Vec<Float>) -> Result<(), String> {
        let mut b_aux = Matrix::from_data(b.len(), 1, b.clone());
        let mut self_aux = Matrix::new(0.0, self.ncols, self.nrows);
        for r in 0..self.nrows {
            for c in 0..self.ncols {
                self_aux.set(r, c, self.get(r, c)?)?;
            }
        }

        self_aux.mut_n_diag_gaussian(&mut b_aux, N)?;

        b.copy_from_slice(b_aux.as_slice());
        Ok(())
    }
}
