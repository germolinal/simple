use crate::traits::Numberish;
use crate::Float;
use serde::{Deserialize, Serialize};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// The main Structure in this library
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GenericMatrix<T: Numberish> {
    pub(crate) ncols: usize,
    pub(crate) nrows: usize,

    // Contains the data ordered by row,
    // Going left to right, and up and down.
    pub(crate) data: Vec<T>,
}

impl<T: Numberish> std::fmt::Display for GenericMatrix<T> {
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

impl<T: Numberish> GenericMatrix<T> {
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

        for (nrow, value) in data.iter().enumerate() {
            let i = nrow * (n_rows + 1);
            // v[i] = data[nrow];
            v[i] = *value;
        }

        GenericMatrix::from_data(n_rows, n_rows, v)
    }

    /// Creates an Identity matrix of size NxN
    #[must_use]
    pub fn eye(n: usize) -> Self {
        GenericMatrix {
            nrows: n,
            ncols: n,
            data: (0..(n * n))
                .map(|i| {
                    let col = i % n;
                    let row = (i - col) / n;
                    if row == col {
                        T::one()
                    } else {
                        T::zero()
                    }
                })
                .collect(),
        }
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

    /// Copies the data from a `GenericMatrix` into `self`.
    ///
    /// # Panics
    /// Panics if the matrices are of different sizes
    pub fn copy_from(&mut self, other: &GenericMatrix<T>) {
        assert_eq!(self.nrows, other.nrows);
        assert_eq!(self.ncols, other.ncols);
        self.data.copy_from_slice(&other.data)
    }

    /// Returns a tuple with number of rows and columns    
    pub fn size(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Gets the index of an element within the `data` array of the Matrix
    pub(crate) fn index(&self, nrow: usize, ncol: usize) -> usize {
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

        // Benchmarks showed that making this parallel was not worth
        // it unless the matrices were very big.
        std::iter::zip(
            std::iter::zip(self.data.iter(), other.data.iter()),
            into.data.iter_mut(),
        )
        .for_each(|((x, y), res)| *res = *x + *y);
        

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

        // Benchmarks showed that making this parallel was not worth
        // it unless the matrices were very big.
        std::iter::zip(
            std::iter::zip(self.data.iter(), other.data.iter()),
            into.data.iter_mut(),
        )
        .for_each(|((x, y), res)| *res = *x - *y);

        Ok(())
    }

    /// Scales a matrix by `s` and puts the result in `into`     
    pub fn scale_into(&self, s: Float, into: &mut GenericMatrix<T>) -> Result<(), String> {
        if self.ncols != into.ncols || self.nrows != into.nrows {
            return Err("Result matrix when scaling is not of appropriate size".to_string());
        }

        std::iter::zip(into.data.iter_mut(), self.data.iter())
            .for_each(|(to, from)| *to = *from * s);

        Ok(())
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

        // Clear
        *into *= T::zero();

        // Multiply.
        let self_rows: Vec<&[T]> = self.data.chunks_exact(self.ncols).collect();
        #[cfg(not(feature = "parallel"))]
        let self_rows = self_rows
            .into_iter()
            .zip(into.data.chunks_exact_mut(other.ncols));
        #[cfg(feature = "parallel")]
        let self_rows = self_rows
            .into_par_iter()
            .zip(into.data.par_chunks_exact_mut(other.ncols));

        self_rows.for_each(|(row_data, into_data)| {
            for (col, item) in into_data.iter_mut().enumerate().take(other.ncols) {
                let coldata = other.data.iter().skip(col).step_by(other.ncols);
                let row_data = row_data.iter();
                let aux = row_data
                    .zip(coldata)
                    .map(|(a, b)| *a * *b)
                    .fold(T::zero(), |acc, val| acc + val);
                *item = aux;
            }
        });

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

        // Clear
        *into *= T::zero();

        // Multiply.
        let i = self.data.chunks_exact(self.ncols);
        let i = i.into_iter().zip(into.data.chunks_exact_mut(other.ncols));

        let one_sided_n = one_sided_n!(n);
        i.enumerate().for_each(|(diag_i, (row_data, into_data))| {
            // for c in 0..other.ncols {
            for (c, item) in into_data.iter_mut().enumerate().take(other.ncols) {
                // dbg!(diag_i, c);
                let ini = if diag_i >= one_sided_n {
                    diag_i - one_sided_n
                } else {
                    0
                };
                let fin = if diag_i + one_sided_n + 1 > self.ncols {
                    self.ncols
                } else {
                    diag_i + one_sided_n + 1
                };

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
        let mut ret: GenericMatrix<T> = GenericMatrix::new(T::zero(), self.nrows, other.ncols);
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

impl<T: Numberish> std::ops::Add<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn add(self, other: &GenericMatrix<T>) -> Self::Output {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }

        GenericMatrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(x, y)| *x + *y)
                .collect(),
        }
    }
}

impl<T: Numberish> std::ops::AddAssign<&GenericMatrix<T>> for GenericMatrix<T> {
    fn add_assign(&mut self, other: &GenericMatrix<T>) {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }

        self.data
            .iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a += *b);
    }
}

impl<T: Numberish> std::ops::Sub<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn sub(self, other: &GenericMatrix<T>) -> Self::Output {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being added are of different sizes");
        }
        GenericMatrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(x, y)| *x - *y)
                .collect(),
        }
    }
}

impl<T: Numberish> std::ops::SubAssign<&GenericMatrix<T>> for GenericMatrix<T> {
    fn sub_assign(&mut self, other: &GenericMatrix<T>) {
        if self.ncols != other.ncols || self.nrows != other.nrows {
            panic!("Matrices being substracted are of different sizes");
        }

        self.data
            .iter_mut()
            .zip(&other.data)
            .for_each(|(a, b)| *a -= *b);
    }
}

impl<T: Numberish> std::ops::Mul<T> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;

    fn mul(self, s: T) -> Self::Output {
        GenericMatrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data: self.data.iter().map(|a| *a * s).collect(),
        }
    }
}

impl<T: Numberish> std::ops::Mul<&GenericMatrix<T>> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;
    fn mul(self, other: &GenericMatrix<T>) -> Self::Output {
        let mut ret = GenericMatrix::new(T::zero(), self.nrows, other.ncols);
        self.prod_into(other, &mut ret).unwrap();
        ret
    }
}

impl<T: Numberish> std::ops::MulAssign<T> for GenericMatrix<T> {
    fn mul_assign(&mut self, s: T) {
        self.data.iter_mut().for_each(|a| *a *= s);
    }
}

impl<T: Numberish> std::ops::Div<T> for &GenericMatrix<T> {
    type Output = GenericMatrix<T>;
    fn div(self, s: T) -> Self::Output {
        GenericMatrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data: self.data.iter().map(|a| *a / s).collect(),
        }
    }
}

impl<T: Numberish> std::ops::DivAssign<T> for GenericMatrix<T> {
    fn div_assign(&mut self, s: T) {
        self.data.iter_mut().for_each(|a| *a /= s);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Float, GenericMatrix, Matrix};

    #[test]
    fn test_serde() -> Result<(), String> {
        // use serde::{Deserialize, Serialize};

        let m = Matrix::from_data(2, 2, vec![1., 2., 3., 4.]);
        let json = serde_json::to_string(&m).map_err(|e| e.to_string())?;
        println!("{}", json);

        let m2: Matrix = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        println!("{}", &m2);

        Ok(())
    }

    #[test]
    fn test_default() {
        let m = Matrix::default();

        assert_eq!(m.ncols, 0);
        assert_eq!(m.nrows, 0);
        assert_eq!(m.data.len(), 0);
    }

    #[test]
    fn test_display() {
        let t = Matrix::eye(5);

        println!("I = {}", t);
    }

    #[test]
    fn test_from_data() {
        let data = vec![0.; 6];
        let _ = GenericMatrix::from_data(3, 2, data.clone());

        let _ = GenericMatrix::from_data(2, 3, data);
    }

    #[test]
    #[should_panic]
    fn test_from_data_fail() {
        let data = vec![0.; 2];
        let _ = GenericMatrix::from_data(1, 1, data);
    }

    #[test]
    fn test_new() {
        let nrows: usize = 3;
        let ncols: usize = 12;
        let a_val: Float = 2.0;

        let a = Matrix::new(a_val, nrows, ncols);

        assert_eq!(nrows, a.nrows);
        assert_eq!(ncols, a.ncols);

        // Check content
        for i in 0..a.data.len() {
            assert_eq!(a.data[i], a_val);
        }

        // IDENTITY
        let eye = Matrix::eye(ncols);
        // Check content
        for r in 0..ncols {
            for c in 0..ncols {
                let result = eye.get(r, c);
                match result {
                    Ok(v) => {
                        if r == c {
                            assert_eq!(v, 1.0);
                        } else {
                            assert_eq!(v, 0.0);
                        }
                    }
                    Err(_e) => {
                        assert!(false)
                    }
                }
            }
        }

        // Emtpy
        let e = Matrix::empty();
        assert_eq!(e.nrows, 0);
        assert_eq!(e.ncols, 0);
        assert_eq!(e.data.len(), 0);
        assert!(e.is_empty());
    }

    #[test]
    fn test_diag() -> Result<(), String> {
        let v = vec![1., 2., 3., 4.];
        let m = Matrix::diag(v.clone());
        assert_eq!(m.nrows, v.len());
        assert_eq!(m.ncols, v.len());

        let n = v.len();

        for c in 0..n {
            for r in 0..n {
                if r == c {
                    assert_eq!(m.get(c, r)?, v[c])
                } else {
                    assert_eq!(m.get(c, r)?, 0.0)
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_eye() -> Result<(), String> {
        let n = 3;
        let t = Matrix::eye(n);
        assert_eq!(t.nrows, n);
        assert_eq!(t.ncols, n);

        println!("{}", t);

        for row in 0..n {
            for col in 0..n {
                if row == col {
                    assert_eq!(t.get(row, col)?, 1.);
                } else {
                    assert_eq!(t.get(row, col)?, 0.);
                }
            }
        }

        Ok(())
    }
    #[test]
    fn test_empty() {
        let e = Matrix::empty();
        assert!(e.is_empty());
        let e = Matrix::eye(2);
        assert!(!e.is_empty());
    }

    #[test]
    fn test_copy_from() {
        let mut a = Matrix::new(0.0, 2, 4);

        assert!(!a.data.iter().any(|x| *x != 0.0));

        let b = Matrix::new(1.2, 2, 4);

        a.copy_from(&b);
        assert!(!a.data.iter().any(|x| *x != 1.2));
    }

    #[test]
    fn test_get() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        //check values
        for i in 0..nrows * ncols {
            a.data[i] = i as Float;
        }

        let mut count: Float = 0.0;
        for r in 0..nrows {
            for c in 0..ncols {
                let result = a.get(r, c);
                match result {
                    Ok(v) => {
                        assert_eq!(v, count);
                        count += 1.0;
                    }
                    Err(_e) => {
                        assert!(false)
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_fail_1() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.get(nrows + 1, ncols + 1);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_fail_2() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.get(nrows, ncols);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_fail_3() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.get(nrows - 1, ncols);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_fail_4() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.get(nrows, ncols - 1);

        assert!(res.is_err());
    }

    #[test]
    fn test_set() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        let mut count: Float = 0.0;
        for r in 0..nrows {
            for c in 0..ncols {
                let result = a.set(r, c, count);
                assert!(result.is_ok());
                count += 1.0;
            }
        }

        //check values
        for i in 0..nrows * ncols {
            a.data[i] = i as Float;
        }
    }

    #[test]
    fn test_set_fail_1() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.set(nrows + 1, ncols + 1, 12.3);
        assert!(res.is_err());
    }

    #[test]
    fn test_set_fail_2() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.set(nrows, ncols, 12.3);
        assert!(res.is_err());
    }

    #[test]
    fn test_set_fail_3() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.set(nrows - 1, ncols, 12.3);
        assert!(res.is_err());
    }

    #[test]
    fn test_set_fail_4() {
        let nrows: usize = 3;
        let ncols: usize = 4;
        let mut a = Matrix::new(0.0, nrows, ncols);

        // Should fail
        let res = a.set(nrows + 1, ncols - 1, 12.3);
        assert!(res.is_err());
    }

    #[test]
    fn test_add_to_element() -> Result<(), String> {
        let mut a = Matrix::new(0.0, 5, 5);
        a.add_to_element(0, 0, 2.)?;
        assert!((a.get(0, 0)? - 2.).abs() < 1e-29);
        Ok(())
    }

    #[test]
    fn test_scale_element() -> Result<(), String> {
        let mut a = Matrix::new(3.0, 5, 5);
        a.scale_element(0, 0, 2.)?;
        assert!((a.get(0, 0)? - 6.).abs() < 1e-29);

        Ok(())
    }

    #[test]
    fn test_add_correct() -> Result<(), String> {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let mut a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, ncols);

        // Try the ugly way
        let result = &a + &b;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val + b_val);
        }

        // Try the Pretty operator
        let result = &a + &b;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val + b_val);
        }

        // Try ugly add
        let mut result = Matrix::new(0.0, nrows, ncols);
        a.add_into(&b, &mut result)?;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val + b_val);
        }

        // Try add_into
        let mut result = Matrix::new(0.0, nrows, ncols);
        a.add_into(&b, &mut result)?;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val + b_val);
        }

        // try add_assign
        a += &b;
        for i in 0..a.data.len() {
            assert_eq!(a.data[i], a_val + b_val);
        }

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_add_fail_1() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);

        let _ = &a + &b;
    }

    #[test]
    fn test_add_fail_2() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);
        let mut c = Matrix::new(0.0, nrows, ncols);
        let res = a.add_into(&b, &mut c);

        assert!(res.is_err());
    }

    #[test]
    #[should_panic]
    fn test_add_fail_3() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);
        let _ = &a + &b;
    }

    #[test]
    fn test_sub_correct() -> Result<(), String> {
        // Ok addition.
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let mut a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, ncols);

        // Ugly way
        let result = &a - &b;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val - b_val);
        }

        // Try the pretty operator
        let result = &a - &b;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val - b_val);
        }

        // Try the ugly sub
        let mut result = Matrix::new(0.0, nrows, ncols);
        a.sub_into(&b, &mut result)?;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val - b_val);
        }

        // Try sub_into
        let mut result = Matrix::new(0.0, nrows, ncols);
        a.sub_into(&b, &mut result)?;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val - b_val);
        }

        // try sub_assign
        // Try add_into
        a -= &b;
        for i in 0..a.data.len() {
            assert_eq!(a.data[i], a_val - b_val);
        }

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_sub_fail_1() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);

        let _ = &a - &b;
    }

    #[test]
    fn test_sub_fail_2() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);
        let mut c = Matrix::new(0.0, nrows, ncols);
        let res = a.sub_into(&b, &mut c);
        assert!(res.is_err());
    }

    #[test]
    #[should_panic]
    fn test_sub_fail_3() {
        let nrows: usize = 2;
        let ncols: usize = 2;
        let a_val: Float = 2.0;
        let a = Matrix::new(a_val, nrows, ncols);

        let b_val: Float = 12.0;
        let b = Matrix::new(b_val, nrows, 2 * ncols);
        let _ = &a - &b;
    }

    /*******/
    /* SCALE */
    /*******/
    #[test]
    fn test_scale() -> Result<(), String> {
        let a_val: Float = 2.0;
        let s: Float = 32.2;
        let a = Matrix::new(a_val, 23, 56);

        // Test from scale
        let result = &a * s;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val * s);
        }

        // Test scale this
        let mut aprime = a.clone();
        aprime *= s;
        for i in 0..aprime.data.len() {
            assert_eq!(aprime.data[i], a_val * s);
        }

        // scale_into
        let mut aprime = a.clone();
        a.scale_into(s, &mut aprime)?;
        for i in 0..aprime.data.len() {
            assert_eq!(aprime.data[i], a_val * s);
        }

        // Test from div
        let result = &a / s;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val / s);
        }

        // Test div this
        let mut aprime = a.clone();
        aprime /= s;
        for i in 0..aprime.data.len() {
            assert_eq!(aprime.data[i], a_val / s);
        }

        // div_into
        let mut aprime = a.clone();
        a.scale_into(1. / s, &mut aprime)?;
        for i in 0..aprime.data.len() {
            assert_eq!(aprime.data[i], a_val / s);
        }

        Ok(())
    }

    #[test]
    fn test_scale_this() {
        let a_val: Float = 2.0;
        let s: Float = 32.2;

        let mut a = Matrix::new(a_val, 23, 56);

        a *= s;

        for i in 0..a.data.len() {
            assert_eq!(a.data[i], a_val * s);
        }
    }

    #[test]
    fn test_div() {
        let a_val: Float = 2.0;
        let s: Float = 32.2;

        let a = Matrix::new(a_val, 23, 56);
        let result = &a / s;
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], a_val / s);
        }
    }

    #[test]
    fn test_div_this() {
        let a_val: Float = 2.0;
        let s: Float = 32.2;

        let mut a = Matrix::new(a_val, 23, 56);
        a /= s;

        for i in 0..a.data.len() {
            assert_eq!(a.data[i], a_val / s);
        }
    }

    /********/
    /* MULT */
    /********/
    #[test]
    fn test_prod_into() -> Result<(), String> {
        let a_rows: usize = 16;
        let a_cols: usize = a_rows;
        let a_val: Float = 12.0;
        let a = Matrix::new(a_val, a_rows, a_cols);

        let b_val: Float = 12.0;
        let b_rows = a_cols;
        let b_cols = 16;
        let b = Matrix::new(b_val, b_rows, b_cols);

        // Ugly old way
        let mut value = Matrix::new(123., a_rows, b_cols);
        a.prod_into(&b, &mut value)?;
        println!("{}", value);
        assert_eq!(value.ncols, b_cols);
        assert_eq!(value.nrows, a_rows);
        for r in 0..value.nrows {
            for c in 0..value.ncols {
                let aux = value.get(r, c);
                match aux {
                    Ok(v) => {
                        assert_eq!(v, b_val * a_val * b_rows as Float);
                    }
                    Err(_e) => {
                        assert!(false)
                    }
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_prod() {
        let a_rows: usize = 16;
        let a_cols: usize = a_rows;
        let a_val: Float = 12.0;
        let a = Matrix::new(a_val, a_rows, a_cols);

        let b_val: Float = 12.0;
        let b_rows = a_cols;
        let b_cols = 16;
        let b = Matrix::new(b_val, b_rows, b_cols);

        // Ugly old way
        let value = &a * &b;
        println!("{}", value);
        assert_eq!(value.ncols, b_cols);
        assert_eq!(value.nrows, a_rows);
        for r in 0..value.nrows {
            for c in 0..value.ncols {
                let aux = value.get(r, c);
                match aux {
                    Ok(v) => {
                        assert_eq!(v, b_val * a_val * b_rows as Float);
                    }
                    Err(_e) => {
                        assert!(false)
                    }
                }
            }
        }

        // Against EYE
        let eye = Matrix::eye(a_cols);
        let v = &a * &eye;

        assert_eq!(v.ncols, eye.ncols);
        assert_eq!(v.nrows, eye.nrows);
        for i in 0..eye.data.len() {
            assert_eq!(v.data[i], a.data[i])
        }

        // pretty operator
        let value = &a * &b;
        assert_eq!(value.ncols, b_cols);
        assert_eq!(value.nrows, a_rows);
        for r in 0..value.nrows {
            for c in 0..value.ncols {
                let aux = value.get(r, c);
                match aux {
                    Ok(v) => {
                        assert_eq!(v, b_val * a_val * b_rows as Float);
                    }
                    Err(_e) => {
                        assert!(false)
                    }
                }
            }
        }

        // Against EYE
        let eye = Matrix::eye(a_cols);
        let v = &a * &eye;
        assert_eq!(v.ncols, eye.ncols);
        assert_eq!(v.nrows, eye.nrows);
        for i in 0..eye.data.len() {
            assert_eq!(v.data[i], a.data[i])
        }
    }

    #[test]
    fn test_prod_n_diag() -> Result<(), String> {
        // Check that we are adding the correct amount of rows/cols
        let n = 3;
        let n_rows: usize = 23;

        let a_val: Float = 2.0;
        let mut a = Matrix::new(0.0, n_rows, n_rows);
        for r in 0..n_rows {
            for c in 0..n_rows {
                if r == c || r + 1 == c || r == c + 1 {
                    a.set(r, c, a_val)?;
                }
            }
        }

        let b = Matrix::new(1.0, n_rows, 3);

        let c = a.from_prod_n_diag(&b, n)?;

        for i in 0..n_rows {
            if i == 0 || i == n_rows - 1 {
                assert_eq!(2.0 * a_val, c.get(i, 0)?);
            } else {
                assert_eq!(3.0 * a_val, c.get(i, 0)?);
            }
        }

        let other_c = &a * &b;

        assert!(c.compare(&other_c));

        Ok(())
    }

    /********/
    /* COMPARE */
    /********/
    #[test]
    fn test_compare() {
        let a = Matrix::new(1.0, 10, 10);
        assert!(a.compare(&a));

        let b = Matrix::new(1.0, 9, 9);
        assert!(!a.compare(&b));

        let c = Matrix::new(2.1, 10, 10);
        assert!(!a.compare(&c));
    }

    #[test]
    fn test_concat() -> Result<(), String> {
        let mut a = Matrix::new(1.0, 10, 10);
        let b = Matrix::new(5.0, 2, 10);

        a.concat_rows(&b)?;
        assert_eq!(a.size(), (12, 10));
        for row in 0..12 {
            for col in 0..10 {
                if row < 10 {
                    assert_eq!(1.0, a.get(row, col)?);
                } else {
                    assert_eq!(5.0, a.get(row, col)?);
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_gauss_seidel_exp_fail_non_squared() {
        let a = Matrix::from_data(2, 1, vec![16., 3.]);
        let b = Matrix::from_data(2, 1, vec![11., 13.]);
        let mut x = b.clone();
        let ret = a.gauss_seidel(&b, &mut x, 1, 1.);
        assert!(ret.is_err());
    }

    #[test]
    fn test_gauss_seidel_exp_fail_wrong_ncols() {
        let a = Matrix::from_data(2, 2, vec![16., 3., 2.1, 50.]);
        let b = Matrix::from_data(3, 1, vec![11., 13., 0.2]);
        let mut x = b.clone();
        let res = a.gauss_seidel(&b, &mut x, 1, 1.);
        assert!(res.is_err());
    }

    #[test]
    fn test_gauss_seidel_exp_fail_wrong_ncols_2() {
        let a = Matrix::from_data(2, 2, vec![16., 3., 2.1, 50.]);
        let b = Matrix::from_data(2, 1, vec![11., 13.]);
        let mut x = Matrix::from_data(3, 1, vec![11., 13., 0.2]);
        let res = a.gauss_seidel(&b, &mut x, 1, 1.);
        assert!(res.is_err());
    }

    #[test]
    fn test_gauss_seidel() -> Result<(), String> {
        // Example 1
        let a = Matrix::from_data(2, 2, vec![16., 3., 7., -11.]);
        let b = Matrix::from_data(2, 1, vec![11., 13.]);

        let mut x = Matrix::new(1.0, 2, 1);
        let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);

        a.gauss_seidel(&b, &mut x, 20, 0.001)?;
        println!("{}", &x - &exp_x);

        // Example 2

        let a = Matrix::from_data(
            4,
            4,
            vec![
                10., -1., 2., 0., -1., 11., -1., 3., 2., -1., 10., -1., 0., 3., -1., 8.,
            ],
        );

        let b = Matrix::from_data(4, 1, vec![6., 25., -11., 15.]);

        let mut x = Matrix::from_data(4, 1, vec![0.; 4]);
        let exp_x = Matrix::from_data(4, 1, vec![1., 2., -1., 1.]);

        a.gauss_seidel(&b, &mut x, 20, 0.0001)?;
        println!("{}", &x - &exp_x);

        Ok(())
    }

    #[test]
    fn test_gauss_seidel_non_converge() {
        // Example 1

        let b = Matrix::from_data(2, 1, vec![11., 13.]);
        let a = Matrix::from_data(2, 2, vec![2., 3., 5., 7.]);

        let mut x = Matrix::new(1.0, 2, 1);
        let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);

        let res = a.gauss_seidel(&b, &mut x, 20, 0.001);
        assert!(res.is_err());
        println!("{}", &x - &exp_x);
    }

    #[test]
    fn test_n_diag_gaussian_elimination() -> Result<(), String> {
        #[cfg(not(feature = "float"))]
        const TINY: Float = 1e-8;
        #[cfg(feature = "float")]
        const TINY: Float = 1e-4;

        // Example 1
        println!("\n====\nExample 1");
        let a = Matrix::from_data(2, 2, vec![2., 3., 5., 7.]);
        let exp_x = Matrix::from_data(2, 1, vec![-38., 29.]);
        let b = &a * &exp_x;
        let mut x = b.clone();

        a.clone().mut_n_diag_gaussian(&mut x, 3)?;
        println!("delta = {}", &x - &exp_x);
        println!("b = {}", &b);
        println!("x = {}", &x);
        assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
        let other_x = a.n_diag_gaussian(&b, 3)?;
        assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

        // Example 2
        println!("\n====\nExample 2");
        let a = Matrix::from_data(2, 2, vec![16., 3., 7., -11.]);
        let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);
        let b = &a * &exp_x;
        let mut x = b.clone();
        a.clone().mut_n_diag_gaussian(&mut x, 4)?;
        println!("delta = {}", &x - &exp_x);
        println!("b = {}", &b);
        println!("x = {}", &x);
        assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
        let other_x = a.n_diag_gaussian(&b, 3)?;
        assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

        // Example 3
        println!("\n====\nExample 3");
        let a = Matrix::from_data(
            4,
            4,
            vec![
                1., 2., 0., 0., 2., 1., 2., 0., 0., 5., 2., 3., 0., 0., 4., 2.,
            ],
        );

        let exp_x = Matrix::from_data(4, 1, vec![1., 2., 3., 4.]);
        let b = &a * &exp_x;
        let mut x = b.clone();
        a.clone().mut_n_diag_gaussian(&mut x, 3)?;
        println!("delta = {}", &x - &exp_x);
        println!("b = {}", &b);
        println!("x = {}", &x);
        assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
        let other_x = a.n_diag_gaussian(&b, 3)?;
        assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

        // Example 4
        println!("\n====\nExample 4");
        let a = Matrix::from_data(
            5,
            5,
            vec![
                1., 23., -2., 0., 0., -1., -35., 23., -1., 0., -52., -1., 56., 9., -23., 0., -4.,
                2., -7., -23., 0., 0., -9., 1., 2.,
            ],
        );
        let exp_x = Matrix::from_data(5, 1, vec![1., 2., 3., 4., 5.]);
        let b = &a * &exp_x;
        let mut x = b.clone();
        a.clone().mut_n_diag_gaussian(&mut x, 5)?;
        println!("delta = {}", &x - &exp_x);
        println!("x = {}", &x);
        println!("b = {}", &b);
        assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
        let other_x = a.n_diag_gaussian(&b, 5)?;
        println!("other_x -- {}", &other_x);
        println!("other -- {}", &other_x - &exp_x);
        assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

        // Example 5
        println!("\n====\nExample 5");
        let a = Matrix::from_data(3, 3, vec![-8.4, 8.4, 0., 8.4, -16.8, 8.4, 0., 8.4, -18.6]);
        let exp_x = Matrix::from_data(3, 1, vec![120., 0., 120.]);
        let b = &a * &exp_x;
        let mut x = b.clone();
        a.clone().mut_n_diag_gaussian(&mut x, 3)?;
        println!("delta = {}", &x - &exp_x);
        println!("b = {}", &b);
        println!("x = {}", &x);
        println!("x - exp_x = {}", &x - &exp_x);
        assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
        let other_x = a.n_diag_gaussian(&b, 3)?;
        assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

        Ok(())
    }
}
