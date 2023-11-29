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

/// Represents the boundary of a `Surface`
///
/// By default (i.e., if no boundary is assigned to a `Surface`),
/// the boundary will be assumed to be outside.
///
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model. It is always
/// embedded on a `Surface`
///
/// ## Examples
///
/// #### A `Space` boundary (in `.json`)
/// ```json
/// {{#include ../../../model/tests/scanner/boundary_space.json}}
/// ```
/// #### A `Ground` boundary (in `.json`)
/// ```json
/// {{#include ../../../model/tests/scanner/boundary_ground.json}}
/// ```
#[derive(Clone, Debug, ObjectIO, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Boundary {
    /// Leads Outdoors. This is also the default (i.e., when no
    /// Boundary is set)
    #[default]
    Outdoor,

    /// The Surface is in contact with the Ground
    Ground,

    /// The Surface leads to another space whose temperature
    /// and other properties are part of the simulation
    ///
    /// Border conditions:
    /// * **Solar Radiation**: As calculated by the Solar module
    /// * **Net Shortwave (IR) Radiation**: Zero, for now at least (this is a good assumption if the surfaces inside the `Space` are at similar temperatures)
    /// * **Convection Coefficient**: Indoor
    /// * **Wind**: No
    Space {
        /// The space to which this boundary leads to
        space: String,
    },

    /// The surface leads to an environment with no wind or sun, and with a fixed
    /// mean-radiant and dry bulb temperature
    ///
    /// This object is useful for defining surfaces that lead to spaces that
    /// we one is not interested in modelling; for example, a wall that separates
    /// an apartment's room with the hall of the building. In that case, we don't
    /// need to simulate the temperature of the hall... but we can assume a certain
    /// temperature.
    ///
    /// Border conditions:
    /// * **Solar Radiation**: None
    /// * **Net Shortwave (IR) Radiation**: Calculated based on the set ambient temperature and the surface temperature
    /// * **Convection Coefficient**: Indoor
    /// * **Wind**: No
    AmbientTemperature {
        /// The temperature in the evironment
        temperature: Float,
    },

    /// The temperature of the other side of the wall will be
    /// the same as the one measured at the interior of the space
    Adiabatic,
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    use crate::model::Model;

    #[test]
    fn serde_ground() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = Boundary::Ground;
        println!(
            "{}",
            json5::to_string(&hardcoded_ref).map_err(|e| e.to_string())?
        );

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Boundary = json5::from_str(
            "{            
            type: 'Ground',
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/boundary_ground";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Boundary =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Boundary =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        Ok(())
    }

    #[test]
    fn serde_space() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = Boundary::Space {
            space: "Some Room".into(),
        };
        println!(
            "{}",
            json5::to_string(&hardcoded_ref).map_err(|e| e.to_string())?
        );

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Boundary = json5::from_str(
            "{            
            type: 'Space',
            space: 'Some Room',
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/boundary_space";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Boundary =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Boundary =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) =
            Model::from_file("./tests/scanner/boundary.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.surfaces.len(), 1);
        assert!("the surface" == model.surfaces[0].name());

        if let Boundary::Space { space } = &model.surfaces[0].front_boundary {
            assert_eq!(space, "Some Space")
        } else {
            assert!(false, "Wrong space!")
        }

        Ok(())
    }
}
