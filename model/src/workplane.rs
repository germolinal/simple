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

/// A horizontal plane over the floor of a [`Space`](crate::Space), covered
/// by a grid of upward-facing sensors that measure the daylight illuminance
/// (e.g., to report how many hours of the year each point receives more
/// than 300 lux).
///
/// Workplanes are only for reporting: they consider daylight only, and they
/// are not part of the simulation state (i.e., they cannot be used for
/// control).
///
/// ## Examples
///
/// ##### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/workplane.spl}}
/// ```
///
/// ##### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/workplane.json}}
/// ```
#[derive(Debug, ObjectIO, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workplane {
    /// The name of the workplane
    pub name: String,

    /// The name of the space over whose floor the workplane is placed
    pub space: String,

    /// The height of the workplane over the floor, in m (`0.7` by default)
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<Float>,

    /// The distance between the sensors, in m (`0.5` by default). The
    /// sensors are placed on a regular grid, aligned with the X and Y axes.
    ///
    /// Smaller values mean more sensors, and so more detailed results; but
    /// also a longer time for creating the model and for each timestep.
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing: Option<Float>,

    /// The number of rays sent from each sensor when calculating its
    /// daylight coefficients (`300` by default). More rays mean less noise,
    /// but a longer time for creating the model (the time required for
    /// each timestep is not affected).
    #[serde(skip_serializing_if = "Option::is_none")]
    n_ambient_samples: Option<usize>,
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
        let mut hardcoded_ref = Workplane::new("Desks", "Office");
        hardcoded_ref.set_height(0.8).set_spacing(0.25);

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Workplane = json5::from_str(
            "{
            name: 'Desks',
            space: 'Office',
            height: 0.8,
            spacing: 0.25,
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let json_data =
            fs::read_to_string("./tests/scanner/workplane.json").map_err(|e| e.to_string())?;
        let from_json_file: Workplane =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Workplane =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // Defaults
        let defaults = Workplane::new("Desks", "Office");
        assert!(defaults.height().is_err());
        assert_eq!(defaults.height_or("test", 0.7), 0.7);
        Ok(())
    }

    #[test]
    fn in_model() -> Result<(), String> {
        let spl = format!(
            "Space {{ name: \"Office\" }}\nWorkplane {}",
            std::fs::read_to_string("./tests/scanner/workplane.spl")
                .map_err(|e| e.to_string())?
                .trim_start_matches("Workplane")
        );
        let (model, header) = Model::from_bytes(spl.as_bytes())?;
        assert_eq!(model.workplanes.len(), 1);
        assert_eq!(model.workplanes[0].space, "Office");
        assert_eq!(header.len(), 0); // Workplanes do not add to the state

        // Round trips
        let (from_spl, _) = Model::from_bytes(format!("{model}").as_bytes())?;
        assert_eq!(from_spl.workplanes.len(), 1);
        let json = serde_json::to_string(&model).map_err(|e| e.to_string())?;
        let (from_json, _) = Model::from_json(&json)?;
        assert_eq!(
            format!("{:?}", from_json.workplanes[0]),
            format!("{:?}", model.workplanes[0])
        );

        // Names are unique
        let spl = "Space { name: \"Office\" }
            Workplane { name: \"Desks\", space: \"Office\" }
            Workplane { name: \"Desks\", space: \"Office\" }";
        assert!(Model::from_bytes(spl.as_bytes()).is_err());
        Ok(())
    }
}
