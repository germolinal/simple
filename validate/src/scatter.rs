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
use crate::Validate;
use crate::ValidationResult;
use poloto::prelude::*;

/// Validates a time series based on Mean Bias Error and Root Mean Squared Error
#[derive(Default, Clone)]
pub struct ScatterValidator<T> {
    /// The units in the x and y axis of the chart (they are supposed to be the same)
    pub units: Option<&'static str>,

    /// The name of the series caled `expected`
    pub expected_legend: Option<&'static str>,

    /// The time series containing the expected values
    pub expected: Vec<T>,

    /// The name of the `found` time series
    pub found_legend: Option<&'static str>,

    /// The time series containing the found values
    pub found: Vec<T>,

    /// the title of the chart
    pub chart_title: Option<&'static str>,

    /// The minimum allowed R2 for the regression coefficient.
    pub allowed_r2: Option<T>,

    /// The maximum allowd difference between the found intersect
    /// and the expected one. Defaults to 0.05
    pub allowed_intersect_delta: Option<T>,

    /// The expected intersect to find in the regression coefficients    
    ///
    /// Defaults to 0.0. It is only checked if the `allowed_slope_delta`
    /// is not None.
    pub expected_intersect: Option<T>,

    /// The maximum allowd difference between the found slope
    /// and the expected one. Defaults to 0.05
    pub allowed_slope_delta: Option<T>,

    /// The expected slope to find in the regression coefficients    
    ///
    /// Defaults to 1.0. It is only checked if the `allowed_slope_delta`
    /// is not None.
    pub expected_slope: Option<T>,
}

impl<T: Numberish> Validate for ScatterValidator<T> {
    fn validate(&self) -> ValidationResult {
        let mut err_msg = String::new();

        if self.expected.len() != self.found.len() {
            err_msg = format!(
                "Series to compare have different lengths. expected.len() = {}, found.len() = {}",
                self.expected.len(),
                self.found.len()
            );
            return ValidationResult::Err(err_msg.clone(), err_msg);
        }

        let (intersect, slope, r2) =
            crate::stats::linear_coefficients(self.expected.as_slice(), self.found.as_slice());
        // Errors, in the units of the data
        let mean_bias_error =
            crate::stats::mean_bias_error(self.expected.as_slice(), self.found.as_slice());
        let root_mean_squared_error =
            crate::stats::root_mean_squared_error(self.expected.as_slice(), self.found.as_slice());
        let max_abs_error = self
            .expected
            .iter()
            .zip(self.found.iter())
            .map(|(e, f)| {
                let (e, f): (f64, f64) = ((*e).into(), (*f).into());
                (f - e).abs()
            })
            .fold(0.0, f64::max);
        let units = self.units.map(|u| format!(" {u}")).unwrap_or_default();
        let fit_msg = format!(
            " * Fit: {:.4} + {:.4}x \n * R2 = {:.4}\n * Mean Bias Error: {:.4}{units}\n * Root Mean Squared Error: {:.4}{units}\n * Maximum absolute error: {:.4}{units}",
            intersect, slope, r2, mean_bias_error, root_mean_squared_error, max_abs_error
        );

        let mut nchecks = 0;
        // Check compliance
        if let Some(allowed_r2) = self.allowed_r2 {
            nchecks +=1;
            if r2 < allowed_r2.into() {
                err_msg = format!(
                    "{}\n *  R2 is {:.4}, which is lower than the allowed value of {:.4}",
                    err_msg, r2, allowed_r2
                );
                // err_msg = format!("{} \n **Failed!** {}",err_msg, err_msg)
            }
        }

        if let Some(allowed_intersect_delta) = self.allowed_intersect_delta {
            nchecks +=1;
            let expected_intersect: f64 = match self.expected_intersect {
                Some(v) => v.into(),
                None => 0.0,
            };
            let delta = (intersect - expected_intersect).abs();
            if delta > allowed_intersect_delta.into() {
                err_msg = format!(
                    "{}\n *  Intersect is {:.4} when expecting {:.4}... difference ({:.4}) is higher than the allowed value of {:.4}",
                    err_msg,
                    intersect,
                    expected_intersect,
                    delta,
                    allowed_intersect_delta
                );
            }
        }

        if let Some(allowed_slope_delta) = self.allowed_slope_delta {
            nchecks +=1;
            let expected_slope: f64 = match self.expected_slope {
                Some(v) => v.into(),
                None => 1.0,
            };
            let delta = (slope - expected_slope).abs();
            if delta > allowed_slope_delta.into() {
                err_msg = format!(
                    "{}\n *  Slope is {:.4} when expecting {:.4}... difference ({:.4}) is higher than the allowed value of {:.4}",
                    err_msg,
                    slope,
                    expected_slope,
                    delta,
                    allowed_slope_delta
                );
            }
        }

        let n = self.expected.len();
        let data = |i: usize| [self.expected[i].into(), self.found[i].into()];

        let exp_legend = self.expected_legend.unwrap_or("Expected");
        let found_legend = self.found_legend.unwrap_or("Found");
        let origin = poloto::build::origin();

        let (.., max_x) = crate::stats::min_max(&self.expected);
        let fit = |i: usize| {
            if i == 0 {
                [0., intersect]
            } else if i == 1 {
                [max_x.into(), (intersect + max_x.into() * slope)]
            } else {
                unreachable!();
            }
        };
        let range = (0..2).map(|x| x as usize);
        let fit = range.clone().map(fit).buffered_plot().line("fit");

        let exp_fit = |i: usize| {
            if i == 0 {
                [0.0, 0.0]
            } else if i == 1 {
                [max_x.into(), max_x.into()]
            } else {
                unreachable!();
            }
        };
        let exp_fit = range.map(exp_fit).buffered_plot().line("expected_fit");
        let range = 0..n;
        let scatter = range.map(data).buffered_plot().scatter("some name");

        let chart_title = self.chart_title.unwrap_or("");
        let p = quick_fmt!(
            chart_title,
            &exp_legend,
            &found_legend,
            scatter,
            fit,
            exp_fit,
            origin
        );

        let show_err = if nchecks == 0 {
            "No checks performed..."
        }else if err_msg.is_empty() { 
            "No errors found" 
        } else { &err_msg };
        let file = format!(
            "{}\n#### Errors:\n {}\n\n#### Data:\n{}",
            fit_msg,
            show_err,
            poloto::disp(|w| p.simple_theme(w))
        );

        if !err_msg.is_empty() {
            ValidationResult::Err(file, err_msg)
        } else {
            ValidationResult::Ok(file)
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_scatter_perfect() {
        use crate::Validator;

        let mut validator = Validator::new("Scatter test", "./tests/scatter.html");

        let expected = vec![1., 2., 3., 4.];
        let found = expected.clone();

        let scatter = ScatterValidator {
            expected: expected.clone(),
            found: found.clone(),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        let scatter = ScatterValidator {
            expected: expected.clone(),
            found: found.clone(),
            allowed_intersect_delta: Some(0.1),
            allowed_slope_delta: Some(0.1),
            allowed_r2: Some(1.0),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        validator.validate().unwrap()
    }

    #[test]
    fn test_scatter_perfect_fail() {
        use crate::Validator;

        let mut validator = Validator::new("Scatter test", "./tests/scatter.html");

        let expected = vec![1., 2., 3., 4.];
        let found = expected.clone();

        let scatter = ScatterValidator {
            expected,
            found,
            allowed_intersect_delta: Some(0.1),
            allowed_slope_delta: Some(-0.1),
            allowed_r2: Some(1.2),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_scatter_not_correlated_fail() {
        use crate::Validator;

        let mut validator = Validator::new("Scatter test", "./tests/scatter.html");

        let expected = vec![1., 2., 3., 4.];
        let found = vec![1., 6., -1., 4.];

        let scatter = ScatterValidator {
            expected,
            found,
            expected_intersect: Some(3.9),
            expected_slope: Some(13.9),
            allowed_intersect_delta: Some(0.1),
            allowed_slope_delta: Some(0.1),
            allowed_r2: Some(0.82),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        assert!(validator.validate().is_err());
    }
}
