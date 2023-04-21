# SIMPLE's Solar calculations

![build badge](https://github.com/SIMPLE-BuildingSimulation/light/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/light/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/light/actions/workflows/tests.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/light/actions/workflows/style.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/light/branch/main/graph/badge.svg?token=E1H9Q763J0)](https://codecov.io/gh/SIMPLE-BuildingSimulation/light)

This is [SIMPLE's](https://www.simplesim.tools) solar calculation module. It is responsible for:

* **Calculating Incident Solar Radiation in each surface**: Contrary to EnergyPlus (and probably other tools I am less familiar with), this module uses Daylight Coefficients for performing this simulation. This method was stolen from the 
daylighting simulation world, and has the advantage of being extremely robust, and therefore capable of handling complex geometries. Perhaps the main drawback is that—because the concept of Thermal Zone does not fit within Lighting calculations (it is quite artificial for radiation purposes, actually)—reporting the "Solar Heat Gains" in a zone needs significant post-processing.
* **Calculating view factors for Infrared calculations**: This is in development... for now, it does nothing.
* **Daylighting Calculations**: Because this module is based on ray-tracing, it can perform daylight calculations. It is unclear, however, which climate based daylight metrics to include and how... if you have any idea, let me know.

# Documentation and Validation

The main supporting documentation is [HERE](https://simple-buildingsimulation.github.io/light/). It contains:

* [Validation report](https://simple-buildingsimulation.github.io/light/validation/incident_solar_radiation.html)
* [Rust API documentation](https://simple-buildingsimulation.github.io/light/rustdoc/doc/light/index.html)

