use validate::{valid, SeriesValidator, ValidFunc, Validator};

/// This test serves as an example of how to use the Validation crate.
///
/// You can add explanation using `Markdown`, and these will be translated into
/// the `HTML` report.
#[valid("Time Series Test")]
fn time_series_test() -> Result<ValidFunc, String> {
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

    Ok(Box::new(v))
}

#[test]
fn test_series_validator() -> Result<(),String> {
    let mut validator = Validator::new("Validate Time series", "report.html");

    let v = time_series_test()?;
    validator.push(v);

    // This will not fail because we did not set a maximum allowed
    // Root Mean Square Error or Mean Bias Error... if we did, it would return
    // an error and the unwrap woul
    validator.validate()?;

    Ok(())
}





