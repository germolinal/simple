# Welcome to SIMPLE's Thermal Simulation Module

![build badge](https://github.com/SIMPLE-BuildingSimulation/heat/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/heat/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/heat/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/heat/branch/main/graph/badge.svg?token=X6RV5WE0UL)](https://codecov.io/gh/SIMPLE-BuildingSimulation/heat)
[![Clippy check](https://github.com/SIMPLE-BuildingSimulation/heat/actions/workflows/style.yaml/badge.svg)](https://github.com/SIMPLE-BuildingSimulation/heat/actions/workflows/style.yaml)


This module reads a [SIMPLE model](https://github.com/SIMPLE-BuildingSimulation/model) and estimates thermodynamic properties. **It is still under development**. Check the documentation [Here](https://simple-buildingsimulation.github.io/heat/rustdoc/doc/heat/index.html)

# Is it accurate?

Check the automatic Validation report [HERE](https://simple-buildingsimulation.github.io/heat/validation/walls.html)

## Some features

* Walls are modelled through Finite Difference method, and the solution is found through a [Runge-Kutta method](https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods)
* Materials are described as they are, the module will choose which ones should be considered no-mass.
* Fenestration and Walls are all treated equally. (Photons don't care whether they are reaching a window or a wall or a door; they just bounce and heat stuff up)
* The temperature of heat zones are updated through [an analytical equation](https://simple-buildingsimulation.github.io/heat/rustdoc/doc/heat/model/struct.ThermalModel.html#method.calculate_zones_abc)

## Get involved!

* **Want to add new features?** Check the docs [HERE](https://simple-buildingsimulation.github.io/heat/rustdoc/doc/heat/index.html) and feel free to create new issues.
* More testing and validation would be great. Check the validation report [HERE](https://simple-buildingsimulation.github.io/heat/)



