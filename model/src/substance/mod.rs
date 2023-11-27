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

/// Contains the gases
pub mod gas;
/// Module describing solid materials, transparent or not
pub mod normal;

pub use crate::substance::gas::Gas;
pub use crate::substance::normal::Normal;

use crate::model::Model;
use derive::{GroupAPI, GroupIO};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A physical substance with physical—i.e., optical, thermal—properties.
///
/// Note that, contrary to EnergyPlus' `Materials`, `Substances` do not
/// contain information about the thickness, which in Simple is given when
/// creating a `Material`. The idea is to enable multiple materials of different
/// thicknesses to reference the same material.
///
/// > Note: Glazing substances are `Normal` substances with `solar_transmitance`
/// and `visible_transmittance`. However, contrary to all other properties, this property
/// does depend on the thickness of the substance. So, in order
/// to build a coherent Glazing, you'll need to match this Substance
/// with an appropriate Material
///
/// ## Examples
///
/// A normal substance
///
/// ```json
/// {{#include ../../../model/tests/scanner/substance_normal.spl}}
/// ```
///
/// A gas
/// ```json
/// {{#include ../../../model/tests/scanner/substance_gas.spl}}
/// ```
///
#[derive(Debug, Clone, GroupAPI, GroupIO, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Substance {
    /// A normal (i.e., solid, homogeneous) substance such as glass,
    /// timber or concrete.    
    Normal(Arc<Normal>),

    /// A gas, as understood by the standard ISO-15099(2003).
    Gas(Arc<Gas>),
}

impl std::fmt::Display for Substance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {        
        let j = serde_json::to_string_pretty(&self).unwrap();
        write!(f, "{}\n\n", j)
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {

    use super::{gas::GasSpecification, *};

    #[test]
    fn serde_gas() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut xenon = Gas::new("Some Gas");
        xenon.set_gas(GasSpecification::Xenon);
        let rust_xenon = xenon.wrap();

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: Substance = json5::from_str(
            "{    
            type: 'Gas',
            name: 'Some Gas',     
            gas: 'Xenon'
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_xenon), format!("{:?}", json5_heater));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/substance_gas";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json: Substance = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_xenon), format!("{:?}", from_json));

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&rust_xenon).map_err(|e| e.to_string())?;
        println!("{}", &rust_json);
        let rust_again: Substance = serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_xenon), format!("{:?}", rust_again));

        // test simple
        let (model, ..) = Model::from_file("./tests/scanner/substance_gas.spl")?;
        assert_eq!(1, model.substances.len());

        if let Substance::Gas(g) = &model.substances[0] {
            assert_eq!("Some Gas", g.name());
            if let Ok(GasSpecification::Xenon) = g.gas() {
                assert!(true)
            } else {
                assert!(false, "Wrong gas")
            }
        } else {
            assert!(false, "Wrong substance")
        }

        Ok(())
    }

    #[test]
    fn serde_normal() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut normal = Normal::new("the substance");
        normal.set_thermal_conductivity(12.);
        let rust_normal = normal.wrap();

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: Substance = json5::from_str(
            "{    
            type: 'Normal',
            name: 'the substance',     
            thermal_conductivity: 12,
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_normal), format!("{:?}", json5_heater));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/substance_normal";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json: Substance = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_normal), format!("{:?}", from_json));

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&rust_normal).map_err(|e| e.to_string())?;
        println!("{}", &rust_json);
        let rust_again: Substance = serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", rust_normal), format!("{:?}", rust_again));

        // Check spl
        let (model, ..) = Model::from_file("./tests/scanner/substance_normal.spl")?;
        assert_eq!(1, model.substances.len());

        if let Substance::Normal(g) = &model.substances[0] {
            assert_eq!("the substance", g.name());
            assert!((12. - g.thermal_conductivity()?).abs() < 1e-9);
        } else {
            assert!(false, "Wrong substance")
        }

        Ok(())
    }

    #[test]
    fn substance_from_file() -> Result<(), String> {
        let (model, ..) = Model::from_file("./tests/box.spl")?;
        assert_eq!(model.substances.len(), 2);

        let in_any = |name: &String| -> bool {
            let in_first = name == model.substances[0].name();
            let in_second = name == model.substances[1].name();
            in_first || in_second
        };

        assert!(in_any(&"the substance".to_string()));
        assert!(in_any(&"the gas substance".to_string()));

        Ok(())
    }
}
