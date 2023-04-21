
![build badge](https://github.com/SIMPLE-BuildingSimulation/polynomial/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/polynomial/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/polynomial/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/polynomial/branch/main/graph/badge.svg?token=VOITQZN77J)](https://codecov.io/gh/SIMPLE-BuildingSimulation/polynomial)

A light-weight representation of a polynomial. It contains a maximum of 12 coefficients.

## Quickstart

```rust
    // p = 0. + 1.0 * x^2 + 2.0*x^3 + 3.0*x^4
    let p = poly![0.0, 1.0, 2.0, 3.0];

    // 0 + 1 + 2 + 3
    assert_eq!(p.eval(1.0), 6.0);

    // 0 + 1*2^1 + 2*2^2 + 3*2^3 = 
    // 0 + 2     + 8     + 24    = 34
    assert_eq!(p.eval(2.0), 34.0);

    // It can also be used as a constant
    const P : Polynomial = poly![1., 2., 3., 4.];

```

## `f32` or `f64`?

By default, this crate works with `f64`. Use the feature `float` to use `f32`.
