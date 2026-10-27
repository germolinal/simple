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

use crate::{Validate, ValidationResult};

/// The type that represents the output of a valid 
/// validation function
pub type ValidFunc = Box<dyn Validate + 'static>;

type ValidationFn = fn() -> Result<ValidFunc, String>;

/// A wrapper that contains an object that implements [`Validate`]
pub struct ValidatorWrapper {
    /// The title of the test
    pub title: String,

    /// A description
    pub description: String,

    /// The Validator
    pub val: ValidationFn,
}

impl ValidatorWrapper {
    /// Format the description of a Validator
    fn format_description(&self, txt: String) -> String {
        format!(
            "## {}\n\n{}\n\n #### Indicators \n\n{}\n",
            self.title, self.description, txt
        )
    }
}

impl Validate for ValidatorWrapper {
    /// Validates a Wrapper
    fn validate(&self) -> ValidationResult {
        let v = &self.val;
        let validator = match v() {
            Ok(v) => v,
            Err(e) => return ValidationResult::Err(e.clone(), e),
        };

        match validator.validate() {
            ValidationResult::Ok(txt) => {
                let ret = self.format_description(txt);
                ValidationResult::Ok(ret)
            }
            ValidationResult::Err(txt, err) => {
                let ret = self.format_description(txt);
                ValidationResult::Err(ret, err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SeriesValidator;

    #[test]
    fn test_wrapper() -> Result<(), String> {
        fn aux() -> Result<Box<dyn Validate>, String> {
            let expected = vec![1., 2., 3.];
            let found = vec![5., 6., 6.];

            let v = SeriesValidator {
                x_label: Some("time step".into()),
                y_label: Some("Zone Temperature".into()),
                y_units: Some("C"),
                expected,
                found,
                ..SeriesValidator::default()
            };
            Ok(Box::new(v))
        }

        let t = aux;

        let wrapper = ValidatorWrapper {
            title: "Some Title".into(),
            description: "The Description".into(),
            val: t,
        };

        match wrapper.validate() {
            ValidationResult::Ok(txt) => {
                println!("{}", txt)
            }
            ValidationResult::Err(_, err) => {
                panic!("{}", err)
            }
        };

        Ok(())
    }
}
