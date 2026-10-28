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
use poloto::build;

/// Validates a time series based on Mean Bias Error and Root Mean Squared Error
#[derive(Default, Clone)]
pub struct SeriesValidator<T: Numberish> {
    /// The maximum allowed Mean Bias Error
    pub allowed_mean_bias_error: Option<f64>,

    /// The maximum allowed Root Mean Squared Error
    pub allowed_root_mean_squared_error: Option<f64>,

    /// The units in the y axis of the chart
    pub y_units: Option<&'static str>,

    /// The label in the y axis of the chart
    pub y_label: Option<&'static str>,

    /// The units in the x axis of the chart
    pub x_units: Option<&'static str>,

    /// The label in the x axis of the chart
    pub x_label: Option<&'static str>,

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
}

impl<T: Numberish> Validate for SeriesValidator<T> {
    fn validate(&self) -> ValidationResult {
        let mut err_msg = String::new();
        let mut file_msg = String::new();

        if self.expected.len() != self.found.len() {
            err_msg = format!(
                "Series to compare have different lengths. expected.len() = {}, found.len() = {}",
                self.expected.len(),
                self.found.len()
            );
            return ValidationResult::Err(err_msg.clone(), err_msg);
        }

        let num = self.expected.len();

        let mean_bias_error = crate::stats::mean_bias_error(&self.expected, &self.found);
        file_msg = format!("{}\n * Mean Bias Error: {:.4}", file_msg, mean_bias_error);

        // Process Root Mean Squared Error
        let root_mean_squared_error =
            crate::stats::root_mean_squared_error(&self.expected, &self.found);
        file_msg = format!(
            "{}\n * Root Mean Squared Error: {:.4}",
            file_msg, root_mean_squared_error
        );

        let mut nchecks = 0;

        // Check compliance
        if let Some(allowed_mean_bias_error) = self.allowed_mean_bias_error {
            nchecks += 1;
            if mean_bias_error.abs() > allowed_mean_bias_error {
                err_msg = format!(
                    "{} * Mean Bias Error is {:.4}, which is greater than the allowed value of {:.4}",
                    err_msg,
                    mean_bias_error.abs(),
                    allowed_mean_bias_error
                );
            }
        }
        if let Some(allowed_root_mean_squared_error) = self.allowed_root_mean_squared_error {
            nchecks += 1;
            // this is always positive... but just in case
            if root_mean_squared_error.abs() > allowed_root_mean_squared_error {
                err_msg = format!(
                    "{}\n * Root Mean Squared Error is {:.4}, which is greater than the allowed value of {:.4}",
                    err_msg,  root_mean_squared_error, allowed_root_mean_squared_error
                );
            }
        }

        let exp_legend = self.expected_legend.unwrap_or("Expected");
        let line_expected = build::plot(exp_legend)
            .line((0..num).map(|i| -> [f64; 2] { [i as f64, self.expected[i].into()] }));
        let found_legend = self.found_legend.unwrap_or("Found");
        let line_found = build::plot(found_legend)
            .line((0..num).map(|i| -> [f64; 2] { [i as f64, self.found[i].into()] }));

        let mut x_label: String = self.x_label.unwrap_or("x").into();
        if let Some(units) = self.x_units {
            x_label = format!("{} ({})", x_label, units);
        }
        let mut y_label: String = self.y_label.unwrap_or("y").into();
        if let Some(units) = self.y_units {
            y_label = format!("{} ({})", y_label, units);
        }
        let chart_title = self.chart_title.unwrap_or("");
        let chart = poloto::frame_build()
            .data(poloto::plots!(line_expected, line_found, build::origin()))
            .build_and_label((chart_title, x_label, y_label))
            .append_to(poloto::header().light_theme())
            .render_string()
            .unwrap_or_else(|e| format!("Could not draw the chart: {e}"))
            // On one line, or Markdown breaks it apart (e.g., at blank lines)
            .replace(['\n', '\t'], " ");

        let show_err = if nchecks == 0 {
            "No checks performed..."
        } else if err_msg.is_empty() {
            "No errors found"
        } else {
            &err_msg
        };

        let file = format!(
            "{}\n#### Errors:\n {}\n#### Data:\n\n{}",
            file_msg, show_err, chart
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
    fn test_series_perfect() {
        use crate::Validator;

        let mut validator = Validator::new("Time series test", "./tests/series.html");

        let expected = vec![1., 2., 3., 4.];
        let found = expected.clone();

        let scatter = SeriesValidator {
            expected: expected.clone(),
            found: found.clone(),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        let scatter = SeriesValidator {
            expected: expected.clone(),
            found: found.clone(),
            allowed_mean_bias_error: Some(0.1),
            allowed_root_mean_squared_error: Some(0.1),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        validator.validate().unwrap()
    }

    #[test]
    fn test_series_perfect_fail() {
        use crate::Validator;

        let mut validator = Validator::new("Time series test", "./tests/series.html");

        let expected = vec![1., 2., 3., 4.];
        let found = expected.clone();

        let scatter = SeriesValidator {
            expected,
            found,
            allowed_mean_bias_error: Some(0.1),
            allowed_root_mean_squared_error: Some(-0.1),
            ..Default::default()
        };

        validator.push(Box::new(scatter));

        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_series_not_correlated_fail() {
        use crate::Validator;

        let mut validator = Validator::new("Time series test", "./tests/series.html");

        let expected = vec![1., 2., 3., 4.];
        let found = vec![1., 6., -1., 4.];

        let scatter = SeriesValidator {
            expected,
            found,

            allowed_root_mean_squared_error: Some(0.1),
            allowed_mean_bias_error: Some(0.1),

            ..Default::default()
        };

        validator.push(Box::new(scatter));

        assert!(validator.validate().is_err());
    }
}
