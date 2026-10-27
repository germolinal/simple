use validate::{valid, SeriesValidator, Validate};

/// Some docs
///
/// These are quite important
#[valid(Wonderful Test)]
fn aux() -> Box<dyn Validate> {
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
    Box::new(v)
}
