/*
MIT License
Copyright (c)  Germán Molina
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
use model::{SimulationStateElement, SimulationStateHeader, Space};
use std::sync::Arc;

/// A thermal representation of a [`Space`]
pub struct ThermalZone {
    /// The `Space` that this [`Thermal Zone`] represents
    pub reference_space: Arc<Space>,

    /// volume of the zone
    volume: Float,
}

impl ThermalZone {
    /// This function creates a new ThermalZone from a Space.
    /// It will copy the index of the space, so it should be used
    /// by iterating the spaces in a model (so there is no mismatch).
    pub fn from_space(
        space: &Arc<Space>,
        state: &mut SimulationStateHeader,
        space_index: usize,
    ) -> Result<Self, String> {
        let volume = *space.volume()?;
        // Add Space Temperature state
        let state_index = state.push(
            // start, by default, at 22.0 C
            SimulationStateElement::SpaceDryBulbTemperature(space_index),
            22.0,
        )?;
        space.set_dry_bulb_temperature_index(state_index)?;

        Ok(ThermalZone {
            reference_space: Arc::clone(space),
            volume,
        })
    }

    /// Retrieves the heat capacity of the ThermalZone's air
    pub fn mcp(&self, temp: Float) -> Float {
        let air = crate::gas::AIR;
        let air_density = air.density(temp + 273.15); //kg/m3
        let air_specific_heat = air.heat_capacity(temp + 273.15); //J/kg.K

        self.volume * air_density * air_specific_heat / 1.
    }
}
