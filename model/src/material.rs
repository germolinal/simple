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

use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// The representation of a physical layer-Material.
/// That is to say, a layer of a certain thickness
/// made of a certain Substance
///
/// ## Examples
///
/// ##### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/material.spl}}
/// ```
///
/// ##### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/material.json}}
/// ```
#[derive(Debug, ObjectIO, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    /// The name of the material object
    pub name: String,

    /// The name of the `Substance` of which this
    /// [`Material`] is made of    
    pub substance: String,

    /// The thickness of the [`Material`]
    pub thickness: Float,
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;
    use crate::Model;

    #[test]
    fn serde() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = Material::new("Fancy Material", "Fancy Substance", 0.2);

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Material = json5::from_str(
            "{
            name:'Fancy Material',
            substance: 'Fancy Substance',
            thickness: 0.2
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/material";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Material =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Material =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/box.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.materials.len(), 2);

        let in_any = |name: &String| -> bool {
            let in_first = name == model.materials[0].name();
            let in_second = name == model.materials[1].name();
            in_first || in_second
        };

        assert!(in_any(&"the material".to_string()));
        assert!(in_any(&"another material".to_string()));

        Ok(())
    }

    #[test]
    fn test_material_basic() -> Result<(), String> {
        // We need a substance
        let substance = "sub_name".to_string();

        // And a name
        let mat_name = "The material".to_string();

        // And a thickness
        let thickness = 123123.123;

        let s = Material::new(mat_name.clone(), substance.clone(), thickness);
        assert_eq!(mat_name, s.name);
        assert_eq!(substance, s.substance);
        assert_eq!(thickness, s.thickness);

        Ok(())
    }
}
