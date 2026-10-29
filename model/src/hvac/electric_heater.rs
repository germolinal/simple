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
use crate::model::Model;
use crate::simulation_state_element::StateElementField;
use serde::{Deserialize, Serialize};

use derive::{GroupMemberAPI, ObjectIO};

/// A simple model of an Electric Heater. It can only heat
/// and has a COP of 1.
///
/// Like every device in `SIMPLE`, it does not turn itself on: something
/// (e.g., a control script) has to set its `power_consumption`, which
/// is delivered to the air of its `target_space`. The thermostat that
/// controls it—if any—is assumed to be in the `target_space`
///
/// > **Note:** The physics deliver whatever `power_consumption` is set to
/// > (i.e., they do not limit it to `max_heating_power`). This limit—and
/// > the setpoint—are used by the controllers (e.g., the one used with
/// > `simple -c people` uses the heater as On/Off, at full power).
///
/// ## Examples
///
/// #### `.spl`
///
/// ```json
/// {{#include ../../../model/tests/scanner/hvac_electric_heater.spl}}
/// ```
///
/// #### `.json`
///
/// ```json
/// {{#include ../../../model/tests/scanner/hvac_electric_heater.json}}
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, ObjectIO, GroupMemberAPI)]
#[serde(deny_unknown_fields)]
pub struct ElectricHeater {
    /// The name of the system
    pub name: String,

    /// The `Space` that this [`ElectricHeater`] heats
    target_space: Option<String>,

    /// Max heating power, in W
    max_heating_power: Option<Float>,

    /// The heating setpoint, in C: the heater should heat when the dry bulb
    /// temperature of the `target_space` is below this value.
    ///
    /// > **Note:** Like every device in `SIMPLE`, this heater does nothing
    /// > unless something turns it on (see the "Controlling the building"
    /// > chapter of the book). This setpoint is not used by the
    /// > physics: it is used by the controllers that turn the device
    /// > on and off (e.g., the one used with `simple -c people`), which
    /// > run once every timestep.
    heating_setpoint: Option<Float>,

    /// The power consumption, in W. This is what turns the heater on
    /// (values above zero) and off (zero).
    #[operational("power_consumption")]
    #[serde(skip)]
    heating_cooling_consumption: StateElementField,
}

impl ElectricHeater {
    /// Wraps the `ElectricHeater` in an `HVAC` enum
    pub fn wrap(self) -> crate::hvac::HVAC {
        crate::hvac::HVAC::ElectricHeater(std::sync::Arc::new(self))
    }
}

impl super::SmallHVAC for ElectricHeater {
    fn heating_setpoint(&self) -> Result<Float, String> {
        match self.heating_setpoint() {
            Ok(v) => Ok(*v),
            Err(e) => Err(e),
        }
    }

    fn cooling_setpoint(&self) -> Result<Float, String> {
        Err("Electric Heaters do not cool".into())
    }

    fn target_space(&self) -> Result<&String, String> {
        self.target_space()
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
        let mut hardcoded_ref = ElectricHeater::new("Bedrooms heater");
        hardcoded_ref.set_target_space("Bedroom");

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: ElectricHeater = json5::from_str(
            "{            
            name: \"Bedrooms heater\",
            target_space: 'Bedroom',    
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/electric_heater";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: ElectricHeater =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: ElectricHeater =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        // (the heater needs its `target_space`)
        let spl = fs::read_to_string("./tests/scanner/hvac_electric_heater.spl")
            .map_err(|e| e.to_string())?;
        let spl = format!("Space {{ name: \"Bedroom\" }}\n{spl}");
        let (model, ..) = Model::from_bytes(spl.as_bytes())?;
        assert_eq!(model.hvacs.len(), 1);
        if let HVAC::ElectricHeater(hvac) = &model.hvacs[0] {
            assert_eq!("Bedrooms heater", hvac.name());
            assert_eq!("Bedroom", hvac.target_space()?);
        } else {
            panic!("Incorrect heater!")
        }

        Ok(())
    }
}
