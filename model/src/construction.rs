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

use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// An object representing an array of
/// Materials, ordered from front to back
///
/// Front and back can be indoor, outdoor, or they can both be
/// indoor (it depends exclusively on the `Boundary` set to the
/// surface)
///
///
///  ## Examples
///
/// #### `.spl`
/// ```rs
/// {{#include ../../../model/tests/scanner/construction.spl}}
/// ```
/// #### `.json`
/// ```rs
/// {{#include ../../../model/tests/scanner/construction.json}}
/// ```
#[derive(Default, Debug, ObjectIO, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Construction {
    /// The name of the Construction object.
    /// Must be unique within the model
    pub name: String,

    /// The indices of the Material objects in the
    /// materials property of the Model object
    pub materials: Vec<String>,
    // front finishing
    // back finishing
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;
    use crate::material::Material;
    use crate::Model;
    use std::sync::Arc;

    #[test]
    fn serde() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut hardcoded_ref = Construction::new("The Construction");
        hardcoded_ref.materials.push("layer 1".into());
        hardcoded_ref.materials.push("layer 2".into());

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Construction = json5::from_str(
            "{
            name: 'The Construction',
            materials: [
                'layer 1',
                'layer 2',
            ]
        }",
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/construction";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Construction =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Construction =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/box.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.constructions.len(), 1);
        assert!(&"the construction".to_string() == model.constructions[0].name());

        Ok(())
    }

    #[test]
    fn test_construction_basic() -> Result<(), String> {
        let c_name = "The construction".to_string();

        let mut c = Construction::new(c_name.clone());
        assert_eq!(0, c.materials.len());
        assert_eq!(c_name, c.name);

        // Create substance
        let sub = "the_sub".to_string();

        // Create a Material
        let mat_1 = "mat_1".to_string();

        c.materials.push(mat_1.clone());
        assert_eq!(1, c.materials.len());
        assert_eq!(mat_1, c.materials[0]);

        let mat_2_name = "mat_2".to_string();
        let mat_2_thickness = 1.12312;
        let mat_2 = Arc::new(Material::new(
            mat_2_name.clone(),
            sub.clone(),
            mat_2_thickness,
        ));

        c.materials.push(mat_2.name().clone());
        assert_eq!(2, c.materials.len());
        assert_eq!(mat_2_name, c.materials[1]);

        Ok(())
    }
}
