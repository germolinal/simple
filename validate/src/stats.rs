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

use crate::numberish::Numberish;

/// Attempts transform a `usize` into a generic parameter `T`.
/// Panics if the usize is too large
pub(crate) fn try_into_t(n_usize: usize) -> f64 {
    if n_usize > i32::MAX as usize {
        panic!("Too many samples... the limit is {}", i32::MAX)
    }
    (n_usize as i32).into()
}

/// Calculates the maximum and minimum in a series.
///
/// # Example
/// ```
/// use validate::assert_close;
/// use validate::stats::min_max;
///
/// let x = vec![1., 2., 3., 4., 5.];
/// let (min, max) = min_max(&x);
/// assert_close!(min, 1.);
/// assert_close!(max, 5.);
/// ```
///
/// # Panics
///
/// * if the dataset is empty
/// * If there are any `NaN` in the dataset
pub fn min_max<T: Numberish>(x: &[T]) -> (T, T) {
    assert_ne!(
        x.len(),
        0,
        "Trying to calculate Max and Min of empty dataset"
    );

    let mut max: T = f32::MIN.into();
    let mut min: T = f32::MAX.into();
    for v in x {
        assert!(
            !v.is_it_nan(),
            "Found NaN when calculating min and max of dataset"
        );
        if *v < min {
            min = *v;
        }
        if *v > max {
            max = *v;
        }
    }
    (min, max)
}

/// Calculates the mean of a dataset
///
/// # The math
/// ```math
/// \bar{x} = \frac{\sum_{i=0}^{n}x_i}{n}
/// ```
/// # Example
///
/// ```
/// use validate::stats::mean;
/// use validate::assert_close;
///
/// let x = vec![0.; 10];
///
/// assert_close!(0., mean(&x));
///
/// let x = vec![-1., -1., 1., 1.];
/// assert_close!(0., mean(&x));
///
/// ```
/// # Panics
///
/// * If the dataset is empty
pub fn mean<T: Numberish>(x: &[T]) -> f64 {
    assert_ne!(x.len(), 0, "Trying to calculate mean of empty dataset");
    let n = try_into_t(x.len());

    let s: f64 = x.iter().fold(0.0, |acc, item| acc + (*item).into());
    s / n
}

/// Calculates the coefficients $`a`$ and $`b`$ that best fit the model $`y = a + b\times x`$.
/// Returns $`a`$, $`b`$ and the $`R^2`$ of the fit.
///
/// # Example
/// ```
/// use validate::assert_close;
/// use validate::stats::linear_coefficients;
///
/// // Perfect fit
/// let x = vec![1., 2., 3., 4.];
/// let (a, b, rsquared) = linear_coefficients(&x, &x);
/// assert_close!(a, 0.);
/// assert_close!(b, 1.);
/// assert_close!(rsquared, 1.);
///
/// // Not so perfect fit
/// let x = vec![1., 2., 3., 4.];
/// let y = vec![6., 2., 1., 0.];
/// let (a, b, rsquared) = linear_coefficients(&x, &y);
/// assert_close!(a, 7.);
/// assert_close!(b, -1.9);
/// assert_close!(rsquared, 0.8699, 1e-3);
/// ```
/// # Panics
///
/// * Panics if the datasets `x` and `y` are of different lengths
/// * If the datasets are empty
pub fn linear_coefficients<T: Numberish>(x: &[T], y: &[T]) -> (f64, f64, f64) {
    assert_eq!(x.len(), y.len(), "Calculating linear coefficients of two datasets of different length. x.len() = {}, y.len = {}", x.len(), y.len());
    assert_ne!(
        x.len(),
        0,
        "Trying to calculate linear coefficients of empty datasets"
    );

    let n = try_into_t(x.len());

    let ss_x: f64 = x.iter().fold(0.0, |acc, &item| acc + item.into());
    let ss_xx: f64 = x
        .iter()
        .map(|x| *x * *x)
        .fold(0.0, |acc, item| acc + item.into());

    let ss_y: f64 = y.iter().fold(0.0, |acc, &item| acc + item.into());

    let ss_yy: f64 = y
        .iter()
        .map(|y| *y * *y)
        .fold(0.0, |acc, item| acc + item.into());

    let ss_xy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(x, y)| *x * *y)
        .fold(0.0, |acc, item| acc + item.into());

    let b = (ss_xy - ss_x * ss_y / n) / (ss_xx - ss_x * ss_x / n);
    let a = (ss_y - b * ss_x) / n;
    let rsquared = (n * ss_xy - ss_x * ss_y) * (n * ss_xy - ss_x * ss_y)
        / ((n * ss_xx - ss_x * ss_x) * (n * ss_yy - ss_y * ss_y));

    // let rsquared = 1.;
    (a, b, rsquared)
}

/// Calculates the Root Mean Squared Error between to datasets, indicating
/// the average absolute difference between them.
///
/// Contrary to the [`mean_bias_error`], this indicator does not compensate positive errors
/// with negative ones. In other words, the Root Mean Squared Error is an indicator of the
/// **absolute** difference between two datasets.
/// For example, if $`y`$ is above $`x`$ by $`1.2`$ units for half of the
/// time and below it by the same magnitude during the rest of the time, then the Root Mean Squared Error
/// will be positive while the [`mean_bias_error`] will be $`0`$.
///
/// # The math
/// ```math
/// RMSE = \sqrt{ \frac{\sum_{i=0}^{n}(y_i - x_i)^2}{n} }
/// ```
/// # Example
///
/// ```
/// use validate::stats::root_mean_squared_error;
/// use validate::assert_close;
///
/// // The absolute difference between x and y is consistently 1 so the
/// // RMSE is 1.
/// let x = vec![0.; 10];
/// let y = vec![1.; 10];
///
/// assert_close!(1., root_mean_squared_error(&x, &y));
///
/// // Even if the halpf of the errors are positive and the other
/// // are negatives, the absolute error is still the same and thus RMSE
/// // is still 1.
/// let x = vec![0., 0., 0., 0.];
/// let y = vec![-1., -1., 1., 1.];
/// assert_close!(1., root_mean_squared_error(&x, &y));
///
/// // Errors of 3 and 4 units: √((9 + 16) / 2) ≈ 3.54 (it used to return the
/// // mean squared error, 12.5)
/// let x = vec![0., 0.];
/// let y = vec![3., -4.];
/// assert_close!(12.5_f64.sqrt(), root_mean_squared_error(&x, &y));
///
/// ```
///
/// # Panics
///
/// * Panics if the datasets `x` and `y` are of different lengths
/// * If the datasets are empty
pub fn root_mean_squared_error<T: Numberish>(x: &[T], y: &[T]) -> f64 {
    assert_eq!(x.len(), y.len(), "Calculating Root Mean Squared Error of two datasets of different length. x.len() = {}, y.len = {}", x.len(), y.len());
    assert_ne!(
        x.len(),
        0,
        "Trying to calculate Root Mean Squared Error of empty datasets"
    );
    let n = try_into_t(x.len());

    let squared_error: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(x, y)| (*y - *x) * (*y - *x))
        .fold(0.0, |acc, item| acc + item.into());
    (squared_error / n).sqrt()
}

/// Calculates the Mean Bias Error between to datasets, indicating whether
/// one of them is consistently larger or smaller than the other.
///
/// Contrary to the [`root_mean_squared_error`], this indicator compensates positive errors
/// with negative ones. For example, if $`y`$ is above $`x`$ by $`1.2`$ units for half of the
/// time and below it by the same magnitude during the rest of the time, then the Mean Bias Error
/// will be $`0`$ while the [`root_mean_squared_error`] will be positive.
///
/// If the $`y`$ is consistently greater than $`x`$, then
/// this value will be positive. Otherwise, it will be negative.
///
/// # The math
/// ```math
/// MBE = \frac{\sum_{i=0}^{n}(y_i - x_i)}{n}
/// ```
///
/// # Example
///
/// ```
/// use validate::stats::mean_bias_error;
/// use validate::assert_close;
///
/// // y is consistently 1 unit grater than x... hence the result
/// let x = vec![0.; 10];
/// let y = vec![1.; 10];
///
/// assert_close!(1., mean_bias_error(&x, &y));
///
/// // y is not consistently greater than y... the errors cancel out.
/// let x = vec![0., 0., 0., 0.];
/// let y = vec![-1., -1., 1., 1.];
/// assert_close!(0., mean_bias_error(&x, &y));
/// ```
///
/// # Panics
///
/// * Panics if the datasets `x` and `y` are of different lengths
/// * If the datasets are empty
pub fn mean_bias_error<T: Numberish>(x: &[T], y: &[T]) -> f64 {
    assert_eq!(
        x.len(),
        y.len(),
        "Calculating Mean Bias Error of two datasets of different length. x.len() = {}, y.len = {}",
        x.len(),
        y.len()
    );
    assert_ne!(
        x.len(),
        0,
        "Trying to calculate Mean Bias Error of empty datasets"
    );

    let n = try_into_t(x.len());
    let bias_error: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(x, y)| *y - *x)
        .fold(0.0, |acc, item| acc + item.into());
    bias_error / n
}
