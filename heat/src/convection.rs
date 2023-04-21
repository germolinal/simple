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

use crate::Float;
const MIN_H: Float = 0.1;

/// Represents a border condition of between a Surface
/// and a Zone or the exterior
#[derive(Debug, Clone, Copy)]
pub struct ConvectionParams {
    /// The dry bulb temperature of the air, in $`C`$
    pub air_temperature: Float,

    /// The wind speed, in m/2
    pub air_speed: Float,

    /// The incident Infrared Irradiance, in $`W/m^2`$
    pub rad_temperature: Float,

    // /// The incident Solar Irradiance, in $`W/m^2`$
    // pub solar_radiation: Float,

    // /// The environmental emissivity, used for calculating incident IR
    // /// irradiance, if needed
    // pub env_emissivity: Float,
    /// The surface temperature in $`C`$
    pub surface_temperature: Float,

    /// The roughness index, between 1 (Very Rough) and 6 (Very smooth)
    pub roughness_index: usize,

    /// The cosine of the surface tilt. Zero is 90 degrees; >0 means
    /// facing up; <0 means facing down
    pub cos_surface_tilt: Float,
}

impl ConvectionParams {
    /// Calculates the indoor (i.e., natural) convection coefficient according to the TARP
    /// model, based on the description given in EnergyPlus' Engineering     
    ///
    ///
    /// # The math
    ///
    /// > *This whole section comes from EnergyPlus' Engineering Reference*
    ///
    /// The Engineering Reference presents three cases, depending on the difference in temperature
    /// $`\Delta T = T_{air} - T_{surface}`$ and the angle of the surface (facing up or down). Then,
    /// it defines three cases: when either of them is zero, when both have the same sign, or when both
    /// have different signs. This function summarizes this through a variable called `aux`, which is the
    /// product between $`\Delta T`$ and $`cos(\theta)`$ ($`\theta`$ being the angle of the surface). So,
    /// the three cases before can be sumarized as `aux == 0`; `aux > 0` and `aux < 0`.
    ///
    /// ## `aux == 0`
    ///
    /// ```math
    /// h = 1.31|\Delta T|^{1/3}
    /// ```
    ///
    /// ## `aux < 0.0`
    ///
    /// ```math
    /// h = \frac{9.482 |\Delta T|^{1/3}}{7.238 - |cos(\theta)|}
    /// ```
    ///
    /// ## `aux > 0.0`    
    ///
    /// ```math
    /// h = \frac{1.81 |\Delta T|^{1/3}}{1.382 + |cos(\theta)|}
    /// ```
    pub fn get_tarp_natural_convection_coefficient(&self) -> Float {
        let delta_t = self.air_temperature - self.surface_temperature;
        let abs_delta_t = delta_t.abs();

        let h = if delta_t.abs() < 1e-3 || self.cos_surface_tilt.abs() < 1e-3 {
            1.31 * abs_delta_t.powf(1. / 3.)
        } else if (delta_t < 0. && self.cos_surface_tilt < 0.)
            || (delta_t > 0. && self.cos_surface_tilt > 0.)
        {
            9.482 * abs_delta_t.powf(1. / 3.) / (7.238 - self.cos_surface_tilt.abs())
        } else if (delta_t > 0. && self.cos_surface_tilt < 0.)
            || (delta_t < 0. && self.cos_surface_tilt > 0.)
        {
            1.81 * abs_delta_t.powf(1. / 3.) / (1.382 + self.cos_surface_tilt.abs())
        } else {
            unreachable!()
        };

        if h < MIN_H {
            MIN_H
        } else {
            h
        }
    }

    /// Calculates the exterior convection coefficient according to the TARP
    /// model, based on the description given in EnergyPlus' Engineering     
    ///
    ///
    /// # The math
    ///
    /// > *This whole section comes from EnergyPlus' Engineering Reference*
    ///
    /// The total convection
    /// coefficient $`h_c`$ is based on two components: the natural convection $`h_n`$
    /// and the forced convection $`h_f`$.
    ///
    /// ```math
    /// h_c = h_f + h_n
    /// ```
    ///
    /// The forced component is calculated as follows:
    ///
    /// ```math
    /// h_f = 2.537 W_f R_f \left( \frac{P \times V_z}{A}\right)^{0.5}
    /// ```
    ///
    /// Where $`A`$ is the area of the surface; $`V_z`$, the local wind speed; $`P`$,
    /// the perimeter of the surface; and  $`W_f`$ is $`1.0`$ for surfaces facing the
    /// wind and $`0.5`$ for others    
    ///
    /// The value of $`R_f`$, on its part, is based on the roughness of the material.
    ///
    ///
    /// | Roughness index | $`R_f`$ | Example Material |
    /// |-----------------|---------|------------------|
    /// | 1. Very Rough   | 2.17    | Stucco           |
    /// | 2. Rough        | 1.67    | Brick            |
    /// | 3. Medium Rough | 1.52    | Concrete         |
    /// | 4. Medoum Smooth| 1.13    | Clear pine       |
    /// | 5. Smooth       | 1.11    | Smooth plaster   |
    /// | 6. Very Smooth  | 1.0     | Glass            |
    ///
    /// The natural convection is the same one used for indoor convection
    pub fn get_tarp_convection_coefficient(
        &self,
        area: Float,
        perimeter: Float,
        windward: bool,
    ) -> Float {
        const COEFFICIENTS: [Float; 6] = [2.17, 1.67, 1.52, 1.13, 1.11, 1.];

        let rf = COEFFICIENTS[self.roughness_index];

        let wf = if windward { 1.0 } else { 0.5 };

        let forced = 2.537 * wf * rf * (perimeter * self.air_speed / area).sqrt();

        let natural = self.get_tarp_natural_convection_coefficient();

        forced + natural // this will never be less than MIN_HS because natural is already limited
    }
}

// #[cfg(test)]
// mod testing {
//     #[test]
//     fn test_indoor_coefficient(){
//         assert!(false)
//     }

//     #[test]
//     fn test_outdoor_coefficient(){
//         assert!(false)
//     }
// }
