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

use crate::Float;

use super::*;

#[test]
fn test_serde() {
    // use serde::{Deserialize, Serialize};

    let m = Matrix::from_data(2, 2, vec![1., 2., 3., 4.]);
    let json = serde_json::to_string(&m).unwrap();
    println!("{}", json);

    let m2: Matrix = serde_json::from_str(&json).unwrap();
    println!("{}", &m2);
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

/***********/
/*   NEW   */
/***********/
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
fn test_diag() {
    let v = vec![1., 2., 3., 4.];
    let m = Matrix::diag(v.clone());
    assert_eq!(m.nrows, v.len());
    assert_eq!(m.ncols, v.len());

    let n = v.len();

    for c in 0..n {
        for r in 0..n {
            if r == c {
                assert_eq!(m.get(c, r).unwrap(), v[c])
            } else {
                assert_eq!(m.get(c, r).unwrap(), 0.0)
            }
        }
    }
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

/*******/
/* GET */
/*******/
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
#[should_panic]
fn test_get_fail_1() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.get(nrows + 1, ncols + 1).unwrap();
}

#[test]
#[should_panic]
fn test_get_fail_2() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.get(nrows, ncols).unwrap();
}

#[test]
#[should_panic]
fn test_get_fail_3() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.get(nrows - 1, ncols).unwrap();
}

#[test]
#[should_panic]
fn test_get_fail_4() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.get(nrows, ncols - 1).unwrap();
}

/*******/
/* SET */
/*******/
#[test]
fn test_set() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let mut a = Matrix::new(0.0, nrows, ncols);

    let mut count: Float = 0.0;
    for r in 0..nrows {
        for c in 0..ncols {
            let result = a.set(r, c, count);
            match result {
                Err(_e) => {
                    assert!(false)
                }
                Ok(_v) => {
                    assert!(true)
                }
            }
            count += 1.0;
        }
    }

    //check values
    for i in 0..nrows * ncols {
        a.data[i] = i as Float;
    }
}

#[test]
#[should_panic]
fn test_set_fail_1() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let mut a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.set(nrows + 1, ncols + 1, 12.3).unwrap();
}

#[test]
#[should_panic]
fn test_set_fail_2() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let mut a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.set(nrows, ncols, 12.3).unwrap();
}

#[test]
#[should_panic]
fn test_set_fail_3() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let mut a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.set(nrows - 1, ncols, 12.3).unwrap();
}

#[test]
#[should_panic]
fn test_set_fail_4() {
    let nrows: usize = 3;
    let ncols: usize = 4;
    let mut a = Matrix::new(0.0, nrows, ncols);

    // Should fail
    let _ = a.set(nrows + 1, ncols - 1, 12.3).unwrap();
}

#[test]
fn test_add_to_element() {
    let mut a = Matrix::new(0.0, 5, 5);
    a.add_to_element(0, 0, 2.).unwrap();
    assert!((a.get(0, 0).unwrap() - 2.).abs() < 1e-29);
}

#[test]
fn test_scale_element() {
    let mut a = Matrix::new(3.0, 5, 5);
    a.scale_element(0, 0, 2.).unwrap();
    assert!((a.get(0, 0).unwrap() - 6.).abs() < 1e-29);
}

#[test]
fn test_copy_from() {
    let mut a = Matrix::new(0.0, 2, 4);

    assert!(!a.data.iter().any(|x| *x != 0.0));

    let b = Matrix::new(1.2, 2, 4);

    a.copy_from(&b);
    assert!(!a.data.iter().any(|x| *x != 1.2));
}

/*******/
/* ADD */
/*******/
#[test]
fn test_add_correct() {
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
    a.add_into(&b, &mut result).unwrap();
    for i in 0..result.data.len() {
        assert_eq!(result.data[i], a_val + b_val);
    }

    // Try add_into
    let mut result = Matrix::new(0.0, nrows, ncols);
    a.add_into(&b, &mut result).unwrap();
    for i in 0..result.data.len() {
        assert_eq!(result.data[i], a_val + b_val);
    }

    // try add_assign
    a += &b;
    for i in 0..a.data.len() {
        assert_eq!(a.data[i], a_val + b_val);
    }
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
#[should_panic]
fn test_add_fail_2() {
    let nrows: usize = 2;
    let ncols: usize = 2;
    let a_val: Float = 2.0;
    let a = Matrix::new(a_val, nrows, ncols);

    let b_val: Float = 12.0;
    let b = Matrix::new(b_val, nrows, 2 * ncols);
    let mut c = Matrix::new(0.0, nrows, ncols);
    let _ = a.add_into(&b, &mut c).unwrap();
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

/*******/
/* SUB */
/*******/
#[test]
fn test_sub_correct() {
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
    a.sub_into(&b, &mut result).unwrap();
    for i in 0..result.data.len() {
        assert_eq!(result.data[i], a_val - b_val);
    }

    // Try sub_into
    let mut result = Matrix::new(0.0, nrows, ncols);
    a.sub_into(&b, &mut result).unwrap();
    for i in 0..result.data.len() {
        assert_eq!(result.data[i], a_val - b_val);
    }

    // try sub_assign
    // Try add_into
    a -= &b;
    for i in 0..a.data.len() {
        assert_eq!(a.data[i], a_val - b_val);
    }
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
#[should_panic]
fn test_sub_fail_2() {
    let nrows: usize = 2;
    let ncols: usize = 2;
    let a_val: Float = 2.0;
    let a = Matrix::new(a_val, nrows, ncols);

    let b_val: Float = 12.0;
    let b = Matrix::new(b_val, nrows, 2 * ncols);
    let mut c = Matrix::new(0.0, nrows, ncols);
    let _ = a.sub_into(&b, &mut c).unwrap();
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
fn test_scale() {
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
    a.scale_into(s, &mut aprime).unwrap();
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
    a.scale_into(1. / s, &mut aprime).unwrap();
    for i in 0..aprime.data.len() {
        assert_eq!(aprime.data[i], a_val / s);
    }
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
fn test_prod_n_diag() {
    // Check that we are adding the correct amount of rows/cols
    let n = 3;
    let n_rows: usize = 23;

    let a_val: Float = 2.0;
    let mut a = Matrix::new(0.0, n_rows, n_rows);
    for r in 0..n_rows {
        for c in 0..n_rows {
            if r == c || r + 1 == c || r == c + 1 {
                a.set(r, c, a_val).unwrap();
            }
        }
    }

    let b = Matrix::new(1.0, n_rows, 1);

    let c = a.from_prod_n_diag(&b, n).unwrap();

    for i in 0..n_rows {
        if i == 0 || i == n_rows - 1 {
            assert_eq!(2.0 * a_val, c.get(i, 0).unwrap());
        } else {
            assert_eq!(3.0 * a_val, c.get(i, 0).unwrap());
        }
    }

    let other_c = &a * &b;

    assert!(c.compare(&other_c));
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
fn test_concat() {
    let mut a = Matrix::new(1.0, 10, 10);
    let b = Matrix::new(5.0, 2, 10);

    a.concat_rows(&b).unwrap();
    assert_eq!(a.size(), (12, 10));
    for row in 0..12 {
        for col in 0..10 {
            if row < 10 {
                assert_eq!(1.0, a.get(row, col).unwrap());
            } else {
                assert_eq!(5.0, a.get(row, col).unwrap());
            }
        }
    }
}

#[test]
#[should_panic]
fn test_gauss_seidel_exp_fail_non_squared() {
    let a = Matrix::from_data(2, 1, vec![16., 3.]);
    let b = Matrix::from_data(2, 1, vec![11., 13.]);
    let mut x = b.clone();
    a.gauss_seidel(&b, &mut x, 1, 1.).unwrap();
}

#[test]
#[should_panic]
fn test_gauss_seidel_exp_fail_wrong_ncols() {
    let a = Matrix::from_data(2, 2, vec![16., 3., 2.1, 50.]);
    let b = Matrix::from_data(3, 1, vec![11., 13., 0.2]);
    let mut x = b.clone();
    a.gauss_seidel(&b, &mut x, 1, 1.).unwrap();
}

#[test]
#[should_panic]
fn test_gauss_seidel_exp_fail_wrong_ncols_2() {
    let a = Matrix::from_data(2, 2, vec![16., 3., 2.1, 50.]);
    let b = Matrix::from_data(2, 1, vec![11., 13.]);
    let mut x = Matrix::from_data(3, 1, vec![11., 13., 0.2]);
    a.gauss_seidel(&b, &mut x, 1, 1.).unwrap();
}

#[test]
fn test_gauss_seidel() {
    // Example 1
    let a = Matrix::from_data(2, 2, vec![16., 3., 7., -11.]);
    let b = Matrix::from_data(2, 1, vec![11., 13.]);

    let mut x = Matrix::new(1.0, 2, 1);
    let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);

    a.gauss_seidel(&b, &mut x, 20, 0.001).unwrap();
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

    a.gauss_seidel(&b, &mut x, 20, 0.0001).unwrap();
    println!("{}", &x - &exp_x);
}

#[test]
#[should_panic]
fn test_gauss_seidel_non_converge() {
    // Example 1

    let b = Matrix::from_data(2, 1, vec![11., 13.]);
    let a = Matrix::from_data(2, 2, vec![2., 3., 5., 7.]);

    let mut x = Matrix::new(1.0, 2, 1);
    let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);

    a.gauss_seidel(&b, &mut x, 20, 0.001).unwrap();
    println!("{}", &x - &exp_x);
}

#[test]
fn test_n_diag_gaussian_elimination() {
    #[cfg(not(feature = "float"))]
    const TINY: Float = 1e-8;
    #[cfg(feature = "float")]
    const TINY: Float = 1e-4;

    // Example 1
    println!("\n====\nExample 1");
    let a = Matrix::from_data(2, 2, vec![2., 3., 5., 7.]);
    let exp_x = Matrix::from_data(2, 1, vec![-38., 29.]);
    let b = &a * &exp_x;

    let x = a.clone().mut_n_diag_gaussian(b.clone(), 3).unwrap();
    println!("delta = {}", &x - &exp_x);
    println!("b = {}", &b);
    println!("x = {}", &x);
    assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));

    let other_x = a.n_diag_gaussian(&b, 3).unwrap();
    assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

    // Example 2
    println!("\n====\nExample 2");
    let a = Matrix::from_data(2, 2, vec![16., 3., 7., -11.]);
    let exp_x = Matrix::from_data(2, 1, vec![0.8122, -0.6650]);
    let b = &a * &exp_x;
    let x = a.clone().mut_n_diag_gaussian(b.clone(), 4).unwrap();
    println!("delta = {}", &x - &exp_x);
    println!("b = {}", &b);
    println!("x = {}", &x);
    assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
    let other_x = a.n_diag_gaussian(&b, 3).unwrap();
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
    let x = a.clone().mut_n_diag_gaussian(b.clone(), 3).unwrap();
    println!("delta = {}", &x - &exp_x);
    println!("b = {}", &b);
    println!("x = {}", &x);
    assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
    let other_x = a.n_diag_gaussian(&b, 3).unwrap();
    assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

    // Example 4
    println!("\n====\nExample 4");
    let a = Matrix::from_data(
        5,
        5,
        vec![
            1., 23., -2., 0., 0., -1., -35., 23., -1., 0., -52., -1., 56., 9., -23., 0., -4., 2.,
            -7., -23., 0., 0., -9., 1., 2.,
        ],
    );
    let exp_x = Matrix::from_data(5, 1, vec![1., 2., 3., 4., 5.]);
    let b = &a * &exp_x;
    let x = a.clone().mut_n_diag_gaussian(b.clone(), 5).unwrap();
    println!("delta = {}", &x - &exp_x);
    println!("x = {}", &x);
    println!("b = {}", &b);
    assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
    let other_x = a.n_diag_gaussian(&b, 5).unwrap();
    println!("other_x -- {}", &other_x);
    println!("other -- {}", &other_x - &exp_x);
    assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));

    // Example 5
    println!("\n====\nExample 5");
    let a = Matrix::from_data(3, 3, vec![-8.4, 8.4, 0., 8.4, -16.8, 8.4, 0., 8.4, -18.6]);
    let exp_x = Matrix::from_data(3, 1, vec![120., 0., 120.]);
    let b = &a * &exp_x;
    let x = a.clone().mut_n_diag_gaussian(b.clone(), 3).unwrap();
    println!("delta = {}", &x - &exp_x);
    println!("b = {}", &b);
    println!("x = {}", &x);
    println!("x - exp_x = {}", &x - &exp_x);
    assert!(!(&x - &exp_x).data.iter().any(|x| x.abs() > TINY));
    let other_x = a.n_diag_gaussian(&b, 3).unwrap();
    assert!(!(&other_x - &exp_x).data.iter().any(|x| x.abs() > TINY));
}
