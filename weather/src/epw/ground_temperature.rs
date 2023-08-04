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
use serde::{Deserialize, Serialize};

/// Ground Temperature information gotten from from an EPW file
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct EPWGroundTemperature {
    /// The depth of the measurement in m
    pub depth: Float, // m

    /// the conductivity of the soil in W/m-K
    pub soil_conductivity: Option<Float>,

    /// The density of the soil in kg/m3
    pub soil_density: Option<Float>,

    /// The specific heat of the soil in J/kg-K
    pub soil_specific_heat: Option<Float>,

    /// The averate temperature of the soil at this specific `depth`
    pub average_monthly_temperature: [Float; 12],
}
