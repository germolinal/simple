# GenericMatrix library

[![Build](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/build.yaml/badge.svg)](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/build.yaml)
[![Clippy check](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/style.yaml/badge.svg)](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/style.yaml)
[![Docs](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/docs.yaml/badge.svg)](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/docs.yaml)
[![Tests](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/tests.yaml/badge.svg)](https://github.com/SIMPLE-BuildingSimulation/matrix/actions/workflows/tests.yaml)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/matrix/branch/master/graph/badge.svg?token=YDZTGGZ1AQ)](https://codecov.io/gh/SIMPLE-BuildingSimulation/matrix)

A Library for Generic Matrix operations.

It is built generically (i.e., `GenericMatrix<T: TReq>` where `TReq` is a 
basic numeric Trait) so that the same library can be used for defining Matrices
over `usize`, `i32`, `f32` and even structures (e.g., 
`Colour{red:f32, green: f32, blue: f32}`) to which numeric operations apply