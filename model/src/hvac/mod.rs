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

use serde::{Deserialize, Serialize};
mod electric_heater;
mod ideal_heater_cooler;
pub use crate::hvac::electric_heater::ElectricHeater;
pub use crate::hvac::ideal_heater_cooler::IdealHeaterCooler;
use crate::model::Model;
use derive::{GroupAPI, GroupIO};
use std::sync::Arc;

/// A collection of elements heating and cooling systems
///
/// ## Example `.spl`
///
/// ```rs
/// {{ #include ../../../tests/scanner/hvac_electric_heater.spl }}
///
/// ```
///
/// ## Example `.json`
///
/// ```rs
/// {{ #include ../../../tests//scanner/hvac_electric_heater.json }}
///
/// ```
///
#[derive(Clone, Debug, Serialize, Deserialize, GroupAPI, GroupIO)]
#[serde(tag = "type")]
pub enum HVAC {
    /// An ideal heating/cooling device.
    /// Heats and Cools with an efficiency of
    /// 1, and nothing effects its COP or efficiency    
    IdealHeaterCooler(Arc<IdealHeaterCooler>),

    /// An electric heater, it can only
    /// heat.
    ElectricHeater(Arc<ElectricHeater>),
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn serde_ideal_heater_cooler() {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut rust_reference = IdealHeaterCooler::new("Bedrooms heater");
        rust_reference.target_spaces.push("Bedroom".into());
        let rust_reference = rust_reference.wrap();

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: HVAC = json5::from_str(
            "{            
            type: 'IdealHeaterCooler',
            name: \"Bedrooms heater\",
            target_spaces: ['Bedroom'],    
        }",
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json5_heater)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/hvac_ideal_heater_cooler";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let json_heater: HVAC = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json_heater)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&rust_reference).unwrap();
        println!("{}", &rust_json);
        let rust_heter_2: HVAC = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", rust_heter_2)
        );

        // Check simple
        let (model, ..) =
            Model::from_file("./tests/scanner/hvac_ideal_heater_cooler.spl").unwrap();
        assert_eq!(model.hvacs.len(), 1);

        if let HVAC::IdealHeaterCooler(hvac) = &model.hvacs[0] {
            assert_eq!("Bedrooms heater", hvac.name());
            assert_eq!(1, hvac.target_spaces.len());
            assert_eq!("Bedroom", hvac.target_spaces[0]);
        } else {
            assert!(false, "Wrong space!")
        }
    }

    #[test]
    fn serde_electric_heater() {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut rust_reference = ElectricHeater::new("Bedrooms heater");
        rust_reference.set_target_space("Bedroom");
        let rust_reference = rust_reference.wrap();

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: HVAC = json5::from_str(
            "{            
            type: 'ElectricHeater',
            name: \"Bedrooms heater\",
            target_space: 'Bedroom',    
        }",
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json5_heater)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/hvac_electric_heater";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let json_heater: HVAC = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", json_heater)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&rust_reference).unwrap();
        println!("{}", &rust_json);
        let rust_heter_2: HVAC = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", rust_reference),
            format!("{:?}", rust_heter_2)
        );

        // Check simple
        let (model, ..) =
            Model::from_file("./tests/scanner/hvac_electric_heater.spl").unwrap();
        assert_eq!(model.hvacs.len(), 1);

        if let HVAC::ElectricHeater(hvac) = &model.hvacs[0] {
            assert_eq!("Bedrooms heater", hvac.name());
            assert_eq!("Bedroom", hvac.target_space().unwrap());
        } else {
            assert!(false, "Wrong space!")
        }
    }
}
