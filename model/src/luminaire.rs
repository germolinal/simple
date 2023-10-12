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
use crate::model::Model;
use crate::simulation_state_element::StateElementField;
use crate::Float;
use derive::{ObjectAPI, ObjectIO};
use serde::{Deserialize, Serialize};

/// A Luminaire
///
/// ## Examples
///
/// ##### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/luminaire.spl}}
/// ```
///
/// ##### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/luminaire.json}}
/// ```
#[derive(Debug, ObjectIO, ObjectAPI, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Luminaire {
    /// The name of the Luminaire
    name: String,

    /// The maximum power consumption
    #[serde(skip_serializing_if = "Option::is_none")]
    max_power: Option<Float>,

    /// The name of the space in which the space is located
    ///
    /// While this value might not be relevant for
    /// e.g., lighting calculations, this is necessary for
    /// thermal simulations, in which the heat disipated by
    /// a luminaire will be disipated into the air of a thermal
    /// zone. So, if this is an exterior luminaire or if no thermal
    /// calculation is performed, this can be left empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    target_space: Option<String>,

    /// The state of the luminaire    
    #[operational]
    #[serde(skip)]
    power_consumption: StateElementField,
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::Model;

    #[test]
    fn serde() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut hardcoded_ref = Luminaire::new("Some Light");
        hardcoded_ref.set_max_power(30.);
        hardcoded_ref.set_target_space("A bright space!");

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Luminaire = json5::from_str(
            "{
            name: 'Some Light',
            max_power: 30,
            target_space: 'A bright space!'
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/luminaire";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Luminaire =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Luminaire =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) =
            Model::from_file("./tests/box_with_window.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.luminaires.len(), 1);
        assert_eq!(model.luminaires[0].name(), &"The Luminaire".to_string());

        Ok(())
    }
}
