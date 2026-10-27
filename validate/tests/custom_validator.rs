use validate::{valid, Validate, ValidationResult, Validator, ValidFunc};

// This checks that two numbers are equal
struct CustomValidator {
    expected: u8,
    found: u8,
    title: &'static str,
}

impl Validate for CustomValidator {
    fn validate(&self) -> ValidationResult {
        let valid = self.expected == self.found;
        let mut ret = String::new();
        let mut err = String::new();

        // We return an error if the validation is not succesful, but we still
        // write something into the report. Even if this particular Validation
        // fails, the Validations object will run all the validations and print
        // the error messages into the STDERR
        if valid {
            ret = format!(
                "## {}\n\n * Passed! {} and {} are equal",
                self.title, self.expected, self.found
            );
        } else {
            err = format!("{}\n{} and {} aren't equal", err, self.expected, self.found);
            ret = format!(
                "{}\n\n# {}\n\n * Failed... {} and {} aren't equal",
                ret, self.title, self.expected, self.found
            );
        };
        if err.len() > 0 {
            ValidationResult::Err(err, ret)
        } else {
            ValidationResult::Ok(ret)
        }
    }
}

/// Some explanation about the validation
///
/// It is always important to know what is it that we are validating
#[valid("Some Validation")]
fn check_if_equal() -> ValidFunc {
    let v = CustomValidator {
        expected: 2,
        found: 2,
        title: "Check that 2 and 3 are equal",
    };
    Box::new(v) // We need a Box.
}

#[test]
fn test_custom_validator() {
    let mut validator = Validator::new("Test Validation", "report.html");
    validator.push(check_if_equal());
    validator.validate().unwrap()
}
