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

#![deny(missing_docs)]

//! This crate was developed with the intent of helping you to validate scientific
//! tools; for exmple, for comparing the results of the temperature calculated
//! by an algorithm and those measured in an experiment. It is supposed to be embedded
//! the unit or integration testing.
//!
//! Different Validation tasks can be created by implementing the [`Validate`] trait.  
//! These tasks will be stored into a [`Validator`] object which will write a report
//! into an HTML File.
//!
//! # Example
//!
//! ```
//! use validate::{valid, Validator, Validate, ValidationResult};
//!
//!
//! // This checks that two numbers are equal
//! struct CustomValidator {
//!     expected: u8,
//!     found: u8,
//!     title: &'static str,
//! }
//!
//! impl Validate for CustomValidator {
//!     fn validate(&self)->ValidationResult{
//!         let valid = self.expected == self.found;
//!         let mut ret = String::new();
//!         let mut err = String::new();
//!         
//!         
//!         // We return an error if the validation is not succesful, but we still
//!         // write something into the report. Even if this particular Validation
//!         // fails, the Validations object will run all the validations and print
//!         // the error messages into the STDERR
//!         if valid {
//!             ret = format!("## {}\n\n * Passed! {} and {} are equal", self.title, self.expected, self.found);
//!         }else{
//!             err = format!("{}\n{} and {} aren't equal", err, self.expected, self.found);
//!             ret = format!("{}\n\n# {}\n\n * Failed... {} and {} aren't equal",ret, self.title, self.expected, self.found );
//!         };
//!         if err.len()>0{
//!             ValidationResult::Err(err, ret)
//!         }else{
//!             ValidationResult::Ok(ret)
//!         }
//!     }
//! }
//!
//! /// Some explanation about the validation
//! ///
//! /// It is always important to know what is it that we are validating
//! #[valid("Some Validation")]
//! fn check_if_equal()->Box<dyn Validate>{
//!     let v = CustomValidator{
//!         expected: 2,
//!         found: 2,
//!         title: "Check that 2 and 3 are equal"
//!     };
//!     Box::new(v) // We need a Box.
//! }
//!
//! // Write a test
//! #[test]
//! fn test_custon_validator(){
//!     let mut validator = Validator::new("Test Validation", "report.html");
//!     validator.push(check_if_equal());
//!     validator.validate().unwrap()
//! }
//!
//! ```

use numberish::Numberish;
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::{fs::File, io::Write};

pub use derive::valid;

/// A wrapper that contains an object that implements [`Validate`]
mod validator_wrapper;
pub use validator_wrapper::ValidatorWrapper;

/// A trait defining some numerical-ish trait.
mod numberish;

/// A Validator that plots two time series and calculates—if required—the
/// Root Mean Squared Error and Mean Bias Error between them.
///
/// # Example
///
/// ```
/// use validate::{Validator, SeriesValidator};
///
/// let expected = vec![1., 2., 3.];
/// let found = vec![5., 6., 6.];
///
/// let mut validator = Validator::new("Validate Time series", "report.html");    
/// // Note that we are not defining a maximum allowed error
/// let v = validate::SeriesValidator {
///     x_label: Some("time step"),
///     y_label: Some("Zone Temperature"),
///     y_units: Some("C"),
///     expected,
///     found,
///     ..SeriesValidator::default()
/// };
/// validator.push(Box::new(v));
///
/// // This will not fail because we did not set a maximum allowed
/// // Root Mean Square Error or Mean Bias Error... if we did, it would return
/// // an error and the unwrap woul
/// validator.validate().unwrap();
/// ```
mod time_series;
pub use time_series::SeriesValidator;

/// A Validator that creates a scatter plot from two datasets, indicating
/// the R-value and the linear equation fitting
///
/// # Example
///
/// ```
/// use validate::{Validator, ScatterValidator};
///
/// let expected = vec![1., 2., 3.];
/// let found = vec![2., 4., 6.];
///
/// let mut validator = Validator::new("Validate Scatter", "report.html");    
/// // Note that we are not defining a maximum allowed error
/// let v = validate::ScatterValidator {
///     units: Some("C"),
///     expected,
///     found,
///     ..ScatterValidator::default()
/// };
/// validator.push(Box::new(v));
///
/// // This will not fail because we did not set a maximum allowed
/// // Root Mean Square Error or Mean Bias Error... if we did, it would return
/// // an error and the unwrap woul
/// validator.validate().unwrap();
/// ```
mod scatter;
pub use scatter::ScatterValidator;

/// Asserts whether two numbers are close enough
/// by comparing the first argument with the second, and
/// the threshold being the third.
///
/// # Examples
/// ```
/// use validate::assert_close;
/// assert_close!(1., 1.01, 0.1);
/// assert_close!(1., 1.0000001); // This assumes a threshold of 1e-6
/// ```
#[macro_export]
macro_rules! assert_close {
    ($left:expr, $right:expr, $allowed_diff: expr ) => {
        match ($left, $right, $allowed_diff) {
            (left_val, right_val, allowed_diff) => {
                let diff = (left_val as f64 - right_val as f64).abs();
                if (diff > allowed_diff) {
                    panic!(
                        "{} and {} are not close enough (allowed difference was {}... found {})",
                        left_val, right_val, allowed_diff, diff
                    );
                }
            }
        }
    };
    ($left:expr, $right:expr ) => {
        match ($left, $right) {
            (left_val, right_val) => {
                let allowed_diff: f64 = 1e-6;
                let diff = (left_val as f64 - right_val as f64).abs();
                if (diff > allowed_diff) {
                    panic!(
                        "{} and {} are not close enough (allowed difference was {}... found {})",
                        left_val, right_val, allowed_diff, diff
                    );
                }
            }
        }
    };
}

/// Asserts whether two numbers are close enough
///
/// # Examples
/// ```
/// use validate::assert_not_close;
/// assert_not_close!(1., 10., 0.2);
/// assert_not_close!(1., 10.); // This assumes a threshold of 1e-6
/// ```
#[macro_export]
macro_rules! assert_not_close {
    ($left:expr, $right:expr, $allowed_diff: expr ) => {
        match ($left, $right, $allowed_diff) {
            (left_val, right_val, allowed_diff) => {
                let diff = (left_val as f64 - right_val as f64).abs();
                if (diff < allowed_diff) {
                    panic!(
                        "{} and {} are too close (minimum difference was {}... found {})",
                        left_val, right_val, allowed_diff, diff
                    );
                }
            }
        }
    };
    ($left:expr, $right:expr ) => {
        match ($left, $right) {
            (left_val, right_val) => {
                let allowed_diff: f64 = 1e-6;
                let diff = (left_val as f64 - right_val as f64).abs();
                if (diff < allowed_diff) {
                    panic!(
                        "{} and {} are too close (minimum difference was {}... found {})",
                        left_val, right_val, allowed_diff, diff
                    );
                }
            }
        }
    };
}

/// The type that represents the output of a valid
/// validation function
pub use crate::validator_wrapper::ValidFunc;

/// Implements a validation error, where
/// `Ok` returns just the text to write in the report,
/// but `Err` returns not only that but also an error message
pub enum ValidationResult {
    /// Returns an error, containing
    /// something to write in the report (1st param) and also an error message
    /// to show in the terminal (2nd param)    
    Err(String, String),

    /// Returns a message to write on the report
    Ok(String),
}

impl std::convert::From<String> for ValidationResult {
    fn from(value: String) -> Self {
        Self::Err(value.clone(), value.clone())
    }
}

impl ValidationResult {
    /// Panics if this `ValidationResult` is of type `Err`.
    ///
    /// # Note
    ///
    /// Calling this function discards the correct results from the stdout
    pub fn unwrap(&self) {
        if let ValidationResult::Err(_, err) = self {
            panic!("{}", err)
        }
    }

    /// Checks if the result is an error
    ///
    /// ```
    /// use validate::ValidationResult;
    ///
    /// assert!(ValidationResult::Err("a".into(), "b".into()).is_err());
    /// assert!(!ValidationResult::Ok("a".into()).is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        matches!(self, ValidationResult::Err(_, _))
    }

    /// Checks if the result is not an error
    ///
    /// ```
    /// use validate::ValidationResult;
    ///
    /// assert!(!ValidationResult::Err("a".into(), "b".into()).is_ok());
    /// assert!(ValidationResult::Ok("a".into()).is_ok());
    /// ```
    pub fn is_ok(&self) -> bool {
        !matches!(self, ValidationResult::Err(_, _))
    }
}

/// This structure holds a number of validations to be ran, runs them,
/// and writes the results into an HTML report. It has a title, which is used
/// as a Header in its report.
pub struct Validator<'a> {
    /// The title of this section
    title: &'a str,

    /// The validations to run
    validations: Vec<Box<dyn Validate>>,

    /// The file in which the report will be written
    target_file: &'a str,
}

impl<'a> Validator<'a> {
    /// Creates a new `Validator` that will write a report on `target_file` and put the
    /// supporting data on `report_data_dir`
    pub fn new(title: &'a str, target_file: &'a str) -> Self {
        // Let's check that we can write into this file
        if let std::io::Result::Err(_e) = fs::File::create(target_file) {
            panic!(
                "Cannot write to file '{}'... check that such directory exists.",
                target_file
            )
        }

        Self {
            title,
            target_file,
            validations: Vec::new(),
        }
    }

    /// Adds a new validation to the `Validator`
    pub fn push(&mut self, v: Box<dyn Validate>) {
        self.validations.push(v)
    }

    /// Runs the validations, writes the report and fails the task if necessary
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();

        // Solve
        let txt: Vec<String> = self
            .validations
            .iter()
            .map(|v| {
                // md.write_all(b"\n\n").unwrap();
                match v.validate() {
                    ValidationResult::Err(txt, e) => {
                        errors.push(e);
                        txt
                    }
                    ValidationResult::Ok(txt) => txt,
                }
            })
            .collect();
        let txt = format!("# {}\n\n{}", self.title, txt.join("\n"));

        // Write
        // Set up options and parser.
        let options = Options::empty();
        // options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&txt, options);

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        let mut output = fs::File::create(self.target_file).unwrap();

        // Open HTML
        output.write_all(format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>{}</title></head><body>", self.title).as_bytes()).unwrap();

        output.write_all(html_output.as_bytes()).unwrap();
        // Close html
        output.write_all(b"</body></html>").unwrap();

        // Return
        if errors.is_empty() {
            Ok(())
        } else {
            for e in errors {
                eprintln!("{}", e);
            }
            Err("Some validations failed...".to_string())
        }
    }
}

/// The main trait of this crate. All validator modules need
/// to comply with this trait.
pub trait Validate {
    /// Runs a validation procedure, returning an error message if
    /// the validation failed.
    ///
    /// Whether it fails or not,
    /// it should write the results of the validations into `file`
    /// so a full report is written.
    fn validate(&self) -> ValidationResult;
}

/// Reads a number of columns from a CSV, transforms them into f64
pub fn from_csv<T: Numberish>(path: &str, cols: &[usize]) -> Vec<Vec<T>> {
    let reader = File::open(path).unwrap();
    let mut rdr = csv::Reader::from_reader(reader);

    let mut ret: Vec<Vec<T>> = Vec::with_capacity(cols.len());
    for _ in cols {
        ret.push(Vec::new());
    }

    for record in rdr.records() {
        let data = record.unwrap();
        // dbg!(&data);

        for (i, col) in cols.iter().enumerate() {
            // dbg!(i,col);
            // dbg!(data.get(*col));
            match data.get(*col) {
                Some(v) => {
                    // dbg!(v.to_string());
                    let v = v.trim();
                    let v: T = v.parse::<f32>().unwrap().into();
                    ret[i].push(v);
                }
                None => continue, //panic!("Index out of bounds: {}", *col)
            }
        }
    }

    ret
}

/// Module with some useful functions for calculating
/// indicators for validation (e.g., Mean Squared Error)
pub mod stats;

#[cfg(test)]
mod tests {
    use crate::from_csv;

    #[test]
    fn test_from_csv() {
        let data = from_csv::<f64>("./tests/test_data/data.csv", &vec![0, 1, 2, 3]);
        for c in 0..4 {
            let d = &data[c];
            assert_eq!(d.len(), 3);
            for (i, found) in d.iter().enumerate() {
                let exp = 10 * i + c;
                assert_close!(exp as f64, *found);
            }
        }
    }

    #[test]
    fn test_assert_close_correct() {
        assert_close!(1., 2., 2.);
        assert_not_close!(1., 21., 1.);
    }

    #[test]
    #[should_panic]
    fn test_assert_close_fail() {
        assert_close!(1., 2., 0.2);
    }

    #[test]
    #[should_panic]
    fn test_assert_not_close_fail() {
        assert_not_close!(1., 1., 0.1);
    }

    #[test]
    #[should_panic]
    fn test_assert_not_close_fail_2() {
        assert_not_close!(1., 1.);
    }
}
