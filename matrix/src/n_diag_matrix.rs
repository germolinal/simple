use crate::n_diag_generic_matrix::NDiagGenericMatrix;
use crate::Float;

/// A compact version of an N-diagonal matrix, for floating point numbers
pub type NDiagMatrix<const N: usize> = NDiagGenericMatrix<N, Float>;
