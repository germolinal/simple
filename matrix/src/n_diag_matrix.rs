use crate::Float;
use crate::n_diag_generic_matrix::NDiagGenericMatrix;

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
    /// It works on the compact storage, in O(n) time and without allocating. It
    /// performs the same floating point operations, in the same order, as
    /// [`crate::Matrix::mut_n_diag_gaussian`], so both return exactly the same results.
    ///
    /// # Note
    /// It mutates both `self` and `b`.
    pub fn mut_gaussian(&mut self, b: &mut [Float]) -> Result<(), String> {
        const TINY: Float = 1e-26;
        let n = self.ncols;
        if self.nrows != n || b.len() != n {
            return Err(format!(
                "Size mismatch in Gaussian elimination: matrix is {}x{}, vector has {} elements",
                self.nrows,
                self.ncols,
                b.len()
            ));
        }
        let s = one_sided_n!(N);

        // First, go down, making self an upper-triangular matrix. Without
        // row-swapping, everything stays within the n-diag.
        for c in 0..n {
            let pivot_i = self.band_index(c, c);
            let pivot = self.data[pivot_i];
            if pivot.abs() < TINY {
                return Err(format!(
                    "Found a (nearly) zero element in the diagonal: {}. Maybe the matrix is not invertible...?",
                    pivot
                ));
            }

            // Make the pivot equals to 1. Row c is stored contiguously from
            // column c to column `end`.
            let end = (c + s + 1).min(n);
            let v = 1. / pivot;
            for x in &mut self.data[pivot_i..pivot_i + end - c] {
                *x *= v;
            }
            b[c] *= v;

            // Eliminate values below
            for r in c + 1..end {
                let i = self.band_index(r, c);
                let other = self.data[i];
                // if it is Zero already, just skip
                if other.abs() > TINY {
                    let scale = -other;
                    for k in 0..end - c {
                        let aux = self.data[pivot_i + k];
                        self.data[i + k] += scale * aux;
                    }
                    b[r] += scale * b[c];
                }
            }
        }

        // Now, run substitution upwards. Only the elements above the diagonal
        // of each column are needed, and they are not modified by this loop.
        for c in (1..n).rev() {
            for r in (c.saturating_sub(s)..c).rev() {
                let other = self.data[self.band_index(r, c)];
                // if it is Zero already, just skip
                if other.abs() > TINY {
                    b[r] += -other * b[c];
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Matrix;

    /// Deterministic pseudo-random numbers in [-1, 1)
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> Float {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2. - 1.) as Float
        }
    }

    /// The compact solver must return exactly (bit by bit) what the dense
    /// one returns.
    fn check_same_as_dense<const N: usize>(diag_boost: Float) {
        let mut rng = Lcg(42);
        let s = (N - 1) / 2;
        for n in 1..30 {
            let mut compact = NDiagMatrix::<N>::new(0.0, n, n);
            let mut dense = Matrix::new(0.0, n, n);
            for r in 0..n {
                for c in r.saturating_sub(s)..(r + s + 1).min(n) {
                    let mut v = rng.next();
                    if r == c {
                        v += diag_boost;
                    }
                    compact.set(r, c, v).unwrap();
                    dense.set(r, c, v).unwrap();
                }
            }
            let b: Vec<Float> = (0..n).map(|_| rng.next() * 100.).collect();

            let mut x_compact = b.clone();
            let compact_res = compact.mut_gaussian(&mut x_compact);
            let mut x_dense = Matrix::from_data(n, 1, b);
            let dense_res = dense.mut_n_diag_gaussian(&mut x_dense, N);

            assert_eq!(compact_res, dense_res, "N={N}, n={n}");
            if compact_res.is_ok() {
                for (a, b) in x_compact.iter().zip(x_dense.as_slice()) {
                    assert_eq!(a.to_bits(), b.to_bits(), "N={N}, n={n}: {a} != {b}");
                }
            }
        }
    }

    #[test]
    fn test_mut_gaussian_same_as_dense() {
        check_same_as_dense::<3>(4.0); // diagonally dominant, like heat transfer
        check_same_as_dense::<3>(0.0);
        check_same_as_dense::<5>(6.0);
        check_same_as_dense::<5>(0.0);
    }

    #[test]
    fn test_mut_gaussian_zero_pivot() {
        let mut k = NDiagMatrix::<3>::new(0.0, 3, 3);
        k.set(0, 0, 0.0).unwrap();
        let mut b = vec![1.0; 3];
        assert!(k.mut_gaussian(&mut b).is_err());
    }
}
