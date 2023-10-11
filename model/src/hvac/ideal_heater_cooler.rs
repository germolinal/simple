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

use crate::hvac::HVAC;
use crate::model::Model;
use crate::simulation_state_element::StateElementField;
use crate::Float;
use derive::{GroupMemberAPI, ObjectIO};
use serde::{Deserialize, Serialize};

/// An ideal Heating and Cooling device, with a COP of 1.
///
/// It only has two states: On/Off.
///
/// ## Example
///
/// #### `.spl`
///
/// ```json
/// {{#include ../../../model/tests/scanner/hvac_ideal_heater_cooler.spl}}
/// ```
///
/// #### `.json`
///
/// ```json
/// {{#include ../../../model/tests/scanner/hvac_ideal_heater_cooler.json}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, ObjectIO, GroupMemberAPI)]
#[serde(deny_unknown_fields)]
pub struct IdealHeaterCooler {
    /// The name of the system
    pub name: String,

    /// The `Space`s that this `IdealHeaterCooler` heats and/or
    /// cools
    pub target_space: Option<String>,

    /// Max heating power
    max_heating_power: Option<Float>,

    /// Max cooling power
    max_cooling_power: Option<Float>,

    /// The temperature that automatically triggers the on/off option.
    ///
    /// This tempareture is 'measured' in the `thermostat_location`. If
    /// the dry bulb tempreature in the `thermostat_location` is below
    /// this value, the system starts heating.
    ///
    /// > Note: This assumes automatic control; that is to say, this
    /// condition will be evaluated every timestep of the heat model
    /// simulation (as opposed to the occupant/people control timestep,
    /// which is the one set by the user witht the simulation options)
    heating_setpoint: Option<Float>,

    /// The temperature that triggers the on/off option.
    ///
    /// This tempareture is 'measured' in the `thermostat_location`. If
    /// the dry bulb tempreature in the `thermostat_location` is over
    /// this value, the system starts cooling.
    ///
    /// > Note: This assumes automatic control; that is to say, this
    /// condition will be evaluated every timestep of the heat model
    /// simulation (as opposed to the occupant/people control timestep,
    /// which is the one set by the user witht the simulation options)
    cooling_setpoint: Option<Float>,

    /// The heating or cooling power consumption (not delivered to the `Space`)    
    #[operational("power_consumption")]
    #[serde(skip)]
    heating_cooling_consumption: StateElementField,
}

impl IdealHeaterCooler {
    /// Wraps the `IdealHeaterCooler` in an [`HVAC`] enum
    pub fn wrap(self) -> HVAC {
        crate::hvac::HVAC::IdealHeaterCooler(std::sync::Arc::new(self))
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::HVAC;

    #[test]
    fn serde() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut rust_reference = IdealHeaterCooler::new("Bedrooms heater");
        rust_reference.set_target_space("Bedroom");

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: IdealHeaterCooler = json5::from_str(
            "{            
            name: \"Bedrooms heater\",
            target_space: 'Bedroom',    
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json5_heater)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/ideal_heater_cooler";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let json_heater: IdealHeaterCooler =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json_heater)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&rust_reference).map_err(|e| e.to_string())?;
        let rust_heter_2: IdealHeaterCooler =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", rust_heter_2)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/scanner/hvac_ideal_heater_cooler.spl")?;
        assert_eq!(model.hvacs.len(), 1);
        if let HVAC::IdealHeaterCooler(hvac) = &model.hvacs[0] {
            assert_eq!("Bedrooms heater", hvac.name());
            assert_eq!("Bedroom", hvac.target_space()?);
        } else {
            assert!(false, "Incorrect heater!")
        }

        Ok(())
    }
}
