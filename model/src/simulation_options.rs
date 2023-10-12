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

use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// The options for the solar and lighting calculations
///
/// ## Examples
/// ##### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/solar_options.spl}}
/// ```
/// ##### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/solar_options.json}}
/// ```
#[derive(Default, Debug, Clone, ObjectIO, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolarOptions {
    /// Number of points utilized to calculate the average incident solar
    /// irradiance over each fenestration or surface in W/m2.
    ///
    /// A larger number of points leads to more accurate results but, as usual,
    /// it has an impact on the time of the computation. Specifically,
    /// the time required for creating the model increases linearly with
    /// the number of points. The time required to process each timestep
    /// is not affected.        
    #[serde(skip_serializing_if = "Option::is_none")]
    n_solar_irradiance_points: Option<usize>,

    /// Number of primary rays sent from each of the `n_solar_irradiance_points`
    /// when calculating the average incident solar irradiance over
    /// each surface or fenestration.
    ///
    /// A larger number of rays leads to more accurate results but, as usual,
    /// it has an impact on the time of the computation. Specifically,
    /// the time required for creating the model increases linearly with
    /// the number of points. The time required to process each timestep
    /// is not affected.
    #[serde(skip_serializing_if = "Option::is_none")]
    solar_ambient_divitions: Option<usize>,

    /// The sky discretization scheme used for solar irradiance. A value
    /// of 1 leads to 145 sky patches + the ground.
    ///
    /// A larger number leads to more accurate results but, as usual,
    /// it has an impact on the time of the computation. Although
    /// the time required for creating the model is not greatly affected,
    /// the time required for processing each timestep can increase considerably.    
    #[serde(skip_serializing_if = "Option::is_none")]
    solar_sky_discretization: Option<usize>,

    /// A path to the file containing information about solar radiation and other
    /// optical stuff. If the file does not exist, this information will be calculated
    /// and saved in this path. If it does exist, it will be loaded and used directly,
    /// saving time    
    #[serde(skip_serializing_if = "Option::is_none")]
    optical_data_path: Option<String>,
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
        let mut hardcoded_ref = SolarOptions::new();
        hardcoded_ref.set_n_solar_irradiance_points(30);
        hardcoded_ref.set_solar_sky_discretization(2);
        hardcoded_ref.set_optical_data_path("data.json");

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: SolarOptions = json5::from_str(
            "{            
            n_solar_irradiance_points: 30,
            solar_sky_discretization: 2,
            optical_data_path: 'data.json'
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/solar_options";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: SolarOptions =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: SolarOptions =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/scanner/solar_options.spl")?;
        if let Some(ops) = model.solar_options {
            assert_eq!(30, *ops.n_solar_irradiance_points()?);
            assert_eq!(2, *ops.solar_sky_discretization()?);
            assert_eq!("data.json", ops.optical_data_path()?);
        }

        Ok(())
    }

    #[test]
    fn solar_options_from_file() -> Result<(), String> {
        let (model, ..) = Model::from_file("./tests/box_with_window.spl")?;
        if let Some(opt) = model.solar_options {
            assert_eq!(opt.n_solar_irradiance_points, Some(100));
        } else {
            unreachable!()
        }

        Ok(())
    }
}
