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
use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Indicates how sheltered is the building. This affects its infiltration
///  
/// ## Examples
///
/// #### `.json`
///
/// ```json
/// {{#include ../../../model/tests/scanner/shelter_class.json}}
/// ```
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model,
/// as it is always embeded on a `Building` object
#[derive(Copy, Clone, ObjectIO, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[inline_enum]
pub enum ShelterClass {
    /// No obstructions or local shielding
    NoObstructions,

    /// Typical shelter for an isolated rural house
    IsolatedRural,

    /// Typical shelter caused by other buildings across the street
    #[default]
    Urban,

    /// Typical shelter for urban buildings on larger lots
    LargeLotUrban,

    /// Typical shelter produced by buildings that are immediately adjacent.
    SmallLotUrban,
}

/// This object is utilized to group `Space` objects together for
/// metering and/or shared values. For example, the number of storeys
/// and the `ShelterClass` will help defining the `Infiltrations`
///  
/// ## Examples
///
/// #### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/building.spl}}
/// ```
/// #### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/building.json}}
/// ```
///
#[derive(Default, Debug, ObjectIO, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    /// The name of the Building
    pub name: String,

    /// The number of storeys of this building.
    ///
    /// This value use used by the `AirFlow` module when a `Space` associated
    /// to this `Building` has been assigned an `EffectiveAirLeakageArea`
    /// infiltration. This value is required for calculating the Stack
    /// Coefficient ($C_s$) and the Wind Coefficient ($C_w$) of the
    /// `EffectiveAirLeakageArea` infiltration. $C_s$ and $C_w$ can be inputed
    /// directly by assigning values to the `stack_coefficient` and
    /// `wind_coefficient` fields, in which case the `n_storeys` field will
    /// be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    n_storeys: Option<usize>,

    /// The `ShelterClass` of this building.
    ///
    /// This value use used by the `AirFlow` module when a `Space` associated
    /// to this `Building` has been assigned an `EffectiveAirLeakageArea`
    /// infiltration. This value is required for calculating the Wind
    /// Coefficient ($C_s$) of the
    /// `EffectiveAirLeakageArea` infiltration.  $C_w$ can be inputed
    /// directly by assigning values to the `wind_coefficient` field, in
    /// which case the `shelter_class` field will be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    shelter_class: Option<ShelterClass>,

    /// The stack coefficient of this building, used for
    /// calculating infiltrations in `Spaces` that utilize the `EffectiveAirLeakageArea`
    /// infiltration option.
    ///
    /// If not given, the number of storeys will be used for getting
    /// this values (based on EnergyPlus' Engineering Reference).
    ///
    /// > **Note:** The `EffectiveAirLeakageArea` object is appropriate for buildings
    /// > of 3 storeys or less.
    #[serde(skip_serializing_if = "Option::is_none")]
    stack_coefficient: Option<Float>,

    /// The wind coefficient of this building, used for
    /// calculating infiltrations in `Spaces` that utilize the `EffectiveAirLeakageArea`
    /// infiltration option.
    ///
    /// If not given, the number of storeys will be used for getting
    /// this values (based on EnergyPlus' Engineering Reference).
    ///
    /// > **Note:** The `EffectiveAirLeakageArea` object is appropriate for buildings
    /// > of 3 storeys or less.
    #[serde(skip_serializing_if = "Option::is_none")]
    wind_coefficient: Option<Float>,
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn serde_shelter_class() -> Result<(), String> {
        use crate::Model;
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = ShelterClass::Urban;
        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: ShelterClass =
            json5::from_str("'Urban'").map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/shelter_class";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: ShelterClass =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: ShelterClass =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // Check simple
        let (model, ..) =
            Model::from_file("./tests/scanner/building.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.buildings.len(), 1);

        assert_eq!(model.buildings[0].name, "Main Building");
        if let Some(ShelterClass::Urban) = model.buildings[0].shelter_class {
            assert!(true)
        } else {
            assert!(false, "Wrong shelter class")
        }

        Ok(())
    }

    #[test]
    fn serde_building() -> Result<(), String> {
        use crate::Model;
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut hardcoded_ref = Building::new("Main Building");
        hardcoded_ref.set_shelter_class(ShelterClass::Urban);
        // Deserialize from hardcoded string and check they are the same
        let from_harcoded_json: Building = json5::from_str(
            "{            
            name: 'Main Building',
            shelter_class: 'Urban'
        }",
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_harcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/building";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Building =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Building =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // Check simple
        let (model, ..) =
            Model::from_file("./tests/scanner/building.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.buildings.len(), 1);

        assert_eq!(model.buildings[0].name, "Main Building");
        if let Some(ShelterClass::Urban) = model.buildings[0].shelter_class {
            assert!(true)
        } else {
            assert!(false, "Wrong shelter class")
        }

        Ok(())
    }
}
