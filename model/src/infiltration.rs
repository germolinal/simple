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

/// Infiltration is the unintended air exchange between an interior
/// space and the exterior. In `SIMPLE`, infiltrations are added to the
/// intended ventilation.
///
/// This group of objects allow defining infiltration
/// on different ways. Which variety is better will depend on the
/// information available and the kind of building.
///
/// ## Examples
///
/// #### `.spl`
/// ```json
/// // Note that Infiltration is always attached to a Space
/// {{#include ../../../model/tests/box.spl:bedroom}}
/// ```
///
/// #### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/infiltration_design_flow_rate.json}}
/// ```
///
/// ```json
/// {{#include ../../../model/tests/scanner/infiltration_constant.json}}
/// ```
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model,
/// as it is always embeded on a `Space`
#[derive(Debug, PartialEq, Clone, ObjectIO, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Infiltration {
    /// A contant infiltration, specified in `m3/s`
    Constant {
        /// Flow
        flow: Float,
    },

    /// It is the same as the `DesignFlowRate` infiltration object,
    /// but specifying the  default values from BLAST, as described
    /// in the EnergyPlus' Input Output reference
    Blast {
        /// Flow
        flow: Float,
    },

    /// It is the same as the `DesignFlowRate` infiltration object,
    /// but specifying the default values from DOE-2 as described
    /// in the EnergyPlus' Input Output reference
    Doe2 {
        /// Flow
        flow: Float,
    },

    /// Sets the infiltration to the `DesignFlowRate` values using an
    /// arbitrary set of values. This option is based on EnergyPlus'
    /// object of the same name.
    ///
    ///
    /// The flow $\phi$ (in $m^3/s$) is calculated from the parameters
    /// $A$, $B$, $C$, $D$ and $\phi_{design}$ as follows:
    ///
    /// $$ \phi = \phi_{design} (A + B|T_{space} - T_{outside}| + C\times W_{speed} + D\times W^2_{speed})$$
    ///
    /// The inputs to this object are $A$, $B$, $C$, $D$, $\phi_{design}$ .
    DesignFlowRate {
        /// Factor A in equation above
        a: Float,
        /// Factor B in equation above
        b: Float,
        /// Factor C in equation above
        c: Float,
        /// Factor D in equation above
        d: Float,
        /// Factor Phi_design in equation above
        phi: Float,
    },

    /// Sets the infiltration based on `EffectiveLeakageArea` as
    /// described in the EnergyPlus' Input Output reference. This
    /// variant of infiltration requires the space to be part of a
    /// `Building`
    ///     
    /// The infiltration rate—in $m^3/s$—is calculated based on the
    /// following equation:
    ///
    /// $$ \phi = \frac{A_L}{1000} \sqrt{C_s \Delta T + C_w W^2_{speed}}$$
    ///
    /// where:
    /// * $A_L$ is the effecctive air leakage in $cm^2$ @ 4Pa
    /// * $C_s$ is the coefficient for stack induced infiltration
    /// * $C_w$ is the coefficient for wind induced infiltration
    ///
    /// **The only input to this object is the effecctive air leakage, $A_L$, in $cm^2$ @ 4Pa**.
    /// The other parameters—$C_s$ and $C_w$—are derived based
    /// on the required `Building` object associated with the `Space` that owns
    /// this `Infiltration`. For this to work, the associated `Building` needs
    /// to have been assigned the fields `n_storeys` and a `shelter_class`
    /// (which allow calculating $C_s$ and $C_w$) OR the properties of
    /// `stack_coefficient` (i.e., $C_s$) and `wind_coefficient` (i.e., $C_w$).
    ///
    /// > **Note:** The `EffectiveAirLeakageArea` object is appropriate for buildings
    /// > of 3 storeys or less.
    ///
    /// ## Example
    ///
    /// ```json
    /// {{#include ../../../model/tests/cold_wellington_apartment.spl:building}}
    ///
    /// {{#include ../../../model/tests/cold_wellington_apartment.spl:kids_bedroom}}
    /// ```
    ///
    /// ### Values for $A_L$
    ///
    /// *(source of this example: Sherman and Grimsrud (1980) Infiltration-Pressurization correlation: Simplified physical modeling). Conference of American Society of Heating, Refrigeration and Air Conditioning Engineers*
    ///
    /// Ideally, the value from $A_L$ should come from a blower-door test. When doing one of these,
    /// you can get a table like the following:
    ///
    /// | Q ($m^3/s$) | $\Delta P$ ($Pa$) |
    /// ------|------------------------------
    /// | 10 | 0.222 |
    /// | 20 | 0.338 |
    /// | 30 | 0.433 |
    /// | 40 | 0.513 |
    /// | 50 | 0.586 |
    ///
    /// This data can be fitted in a model with the shape:
    ///

    /// $$ Q = K {\Delta P}^b$$

    ///
    /// In this case, the values for $K$ and $b$ are $0.0555$ and $0.6032$, respectively.
    ///
    /// Evaluating this model at $4 Pa$, we get an air flow $Q$ of $0.128 m^3/s$.
    ///
    /// To calculate $A_L$ we now need to use this data in the following equation, which relates
    /// air flows through openings of size $A_L$:
    ///
    /// $$ Q = A_L \sqrt{\frac{2 \Delta P}{\rho}} $$
    ///
    /// Where $\rho$ is the air density of $1.2 kg/m^3$. So, the estimated $A_L$ would be $0.0496 m^2$
    EffectiveAirLeakageArea {
        /// The Effective Air Leakage Area (in m2 ... ? check)
        area: Float,
    },
    // FlowCoefficient...?
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    use crate::Model;

    #[test]
    fn serde_constant() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = Infiltration::Constant { flow: 1.2 };

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Infiltration = json5::from_str(
            "{            
            type: 'Constant',
            flow: 1.2,
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/infiltration_constant";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Infiltration =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Infiltration =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/box.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.spaces.len(), 1);
        if let Ok(Infiltration::Doe2 { flow }) = model.spaces[0].infiltration() {
            assert!((flow - 0.24).abs() < 1e-9);
        } else {
            assert!(false, "Incorrect infiltration!")
        }

        Ok(())
    }

    #[test]
    fn serde_design_flow_rate() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = Infiltration::DesignFlowRate {
            a: 1.,
            b: 0.,
            c: 0.04,
            d: 0.,
            phi: 1.2,
        };

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Infiltration = json5::from_str(
            "{
            type: 'DesignFlowRate',
            a: 1.0,
            b: 0.0,
            c: 0.04,
            d: 0.0,
            phi: 1.2
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/infiltration_design_flow_rate";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Infiltration =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Infiltration =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        // check simple
        let (model, ..) =
            Model::from_file("./tests/box_with_window.spl").map_err(|e| e.to_string())?;
        assert_eq!(model.spaces.len(), 1);
        if let Ok(Infiltration::DesignFlowRate { a, b, c, d, phi }) = model.spaces[0].infiltration()
        {
            assert!((phi - 0.24).abs() < 1e-9);
            assert!((a - 0.1).abs() < 1e-9);
            assert!((b - 2.1).abs() < 1e-9);
            assert!((c - 0.0).abs() < 1e-9);
            assert!((d - 1.2).abs() < 1e-9);
        } else {
            panic!("Incorrect infiltration!")
        }

        Ok(())
    }

    #[test]
    fn infiltration_from_file() -> Result<(), String> {
        let (model, ..) = Model::from_file("./tests/box.spl")?;
        assert_eq!(model.spaces.len(), 1);
        assert!(&"Bedroom".to_string() == model.spaces[0].name());

        if let Ok(inf) = model.spaces[0].infiltration() {
            assert_eq!(*inf, Infiltration::Doe2 { flow: 0.24 })
        } else {
            assert!(false)
        }

        Ok(())
    }
}
