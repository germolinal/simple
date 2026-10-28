# A Rust Validation library

![build badge](https://github.com/SIMPLE-BuildingSimulation/validate/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/validate/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/validate/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/validate/branch/main/graph/badge.svg?token=7KD1MASSHJ)](https://codecov.io/gh/SIMPLE-BuildingSimulation/validate)
![style badge](https://github.com/SIMPLE-BuildingSimulation/validate/actions/workflows/style.yaml/badge.svg)


This crate was developed with the intent of helping you to validate scientific
tools; for exmple, for comparing the results of the temperature calculated
by an algorithm and those measured in an experiment. It is supposed to be embedded
the unit or integration testing.

Different Validation tasks can be created by implementing the [`Validate`] trait.  
These tasks will be stored into a [`Validations`] object which will write a report 
into a Markdown File.

[CHECK THE DOCS](https://simple-buildingsimulation.github.io/validate/validate/index.html)

# Example 1: Built-in Time Series validator

Validate has a built-in time series validator that plots two time series and calculates—if required—the 
Root Mean Squared Error (RSME) and Mean Bias Error (MBE) between them. Setting a maximum RMSE or MBE
is optional and it will make it return an error if these values are exceded

```rs
use validate::{valid, SeriesValidator, Validate, Validator};

/// This test serves as an example of how to use the Validation crate.
/// 
/// You can add explanation using `Markdown`, and these will be translated into
/// the `HTML` report.
#[valid(Time Series Test)]
fn time_series_test()->Box<dyn Validate>{
    let expected = vec![1., 2., 3.];
    let found = vec![5., 6., 6.];
    let v = SeriesValidator {
        x_label: Some("time step".into()),
        y_label: Some("Zone Temperature".into()),
        y_units: Some("C"),        
        expected,
        found,
        ..validate::SeriesValidator::default()
    };

    Box::new(v)
}

// Write a test
#[test]
fn test_series_validator() {
    let mut validator = Validator::new("Validate Time series", "report.html");
    
    let v = time_series_test();    
    validator.push(v);

    // This will not fail because we did not set a maximum allowed
    // Root Mean Square Error or Mean Bias Error... if we did, it would return
    // an error and the unwrap woul
    validator.validate().unwrap();
}
```
This produces a result as follows

![chart](./readme_img/example.png)


# Example 2: Write custom validator

```rs

 use validate::{valid, Validator, Validate, ValidationResult};


 // This checks that two numbers are equal
 struct CustomValidator {
     expected: u8,
     found: u8,
     title: &'static str,
 }

 impl Validate for CustomValidator {
     fn validate(&self)->ValidationResult{
         let valid = self.expected == self.found;
         let mut ret = String::new();
         let mut err = String::new();
         
         
         // We return an error if the validation is not succesful, but we still
         // write something into the report. Even if this particular Validation
         // fails, the Validations object will run all the validations and print
         // the error messages into the STDERR
         if valid {
             ret = format!("## {}\n\n * Passed! {} and {} are equal", self.title, self.expected, self.found);
         }else{
             err = format!("{}\n{} and {} aren't equal", err, self.expected, self.found);
             ret = format!("{}\n\n# {}\n\n * Failed... {} and {} aren't equal",ret, self.title, self.expected, self.found );
         };
         if err.len()>0{
             ValidationResult::Err(err, ret)
         }else{
             ValidationResult::Ok(ret)
         }
     }
 }

 /// Some explanation about the validation
 ///
 /// It is always important to know what is it that we are validating
 #[valid(Some Validation)]
 fn check_if_equal()->Box<dyn Validate>{
     let v = CustomValidator{
         expected: 2,
         found: 2,
         title: "Check that 2 and 3 are equal"
     };
     Box::new(v) // We need a Box.
 }

 #[test]
 fn test_custon_validator(){

     let mut validator = Validator::new("Test Validation", "report.html");
     validator.push(check_if_equal());
     validator.validate().unwrap()
 }



```
