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
use model::hvac::{ElectricHeater, IdealHeaterCooler, HVAC};
use model::{Model, SimulationState};

/// An HVAC element from the point of view of the thermal
/// model.
pub enum ThermalHVAC {
    /// An ideal heater cooler
    IdealHeaterCooler {
        /// A clone of the parent HVAC
        parent: IdealHeaterCooler,

        /// The space this HVAC is heating/cooling
        target_space_index: usize,        
    },

    /// Electric heater.
    ElectricHeater {
        /// A clone of the parent HVAC
        parent: ElectricHeater,

        /// The space this heater is heating
        target_space_index: usize,
    },
}

impl ThermalHVAC {
    /// Builds a new [`ThermalHVAC`] from an HVAC and its location
    pub fn from(hvac: &HVAC, model: &Model) -> Result<Self, String> {
        match hvac {
            HVAC::ElectricHeater(e) => {
                let parent = (**e).clone();
                for (i, s) in model.spaces.iter().enumerate() {
                    if s.name() == parent.target_space()? {
                        return Ok(Self::ElectricHeater {
                            parent,
                            target_space_index: i,
                        });
                    }
                }
                Err(format!("ElectricHeater is supposed to be in a space called '{}'... but it was not found", parent.target_space()?))
            }
            HVAC::IdealHeaterCooler(e) => {                
                let parent = (**e).clone();
                for (i, s) in model.spaces.iter().enumerate() {
                    if s.name() == parent.target_space()? {
                        return Ok(Self::IdealHeaterCooler {
                            parent,
                            target_space_index: i,
                        });
                    }
                }
                Err(format!("IdealHeaterCooler is supposed to be in a space called '{}'... but it was not found", parent.target_space()?))
                
            }
        }
    }

    /// Retrieves a `Vec<(usize, Float)>` containing the amount of heat (the `Float` in W) going into
    /// each space (of index `usize`)
    pub fn calc_cooling_heating_power(
        &self,
        state: &SimulationState,
    ) -> Result<Vec<(usize, Float)>, String> {
        match self {
            Self::IdealHeaterCooler {                
                parent,
                target_space_index,
            } => {
                let mut ret = Vec::with_capacity(1);
                if let Ok(_space) = parent.target_space() {
                    let consumption_power =
                        match parent.heating_cooling_consumption(state) {
                            Some(v) => v,
                            None => return Err(format!(
                                "Could not get Heating consumption if IdealHeaterCooler called '{}'",
                                parent.name()
                            )),
                        };
                    ret.push((*target_space_index, consumption_power))
                }
                Ok(ret)
            }
            Self::ElectricHeater {
                parent,
                target_space_index,
            } => {
                let mut ret = Vec::with_capacity(1);
                if let Ok(_space) = parent.target_space() {
                    let consumption_power =
                        match parent.heating_cooling_consumption(state) {
                            Some(v) => v,
                            None => return Err(format!(
                                "Could not get Heating consumption of ElectricHeater called '{}'",
                                parent.name()
                            )),
                        };
                    ret.push((*target_space_index, consumption_power))
                }
                Ok(ret)
            }
        }
    }
}
