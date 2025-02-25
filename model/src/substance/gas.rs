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

use crate::substance::Substance;
use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Represent a common gas, with known physical properties
///
///
/// ## Examples
///
/// #### `.json`
/// ```json
/// {{#include ../../../model/tests/scanner/gas_specification.json}}
/// ```
///
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model,
/// as it is always embeded on a `Substance` of type `Gas`
///
#[derive(Debug, Copy, Clone, ObjectIO, Serialize, Deserialize)]
#[inline_enum]
#[serde(deny_unknown_fields)]
pub enum GasSpecification {
    /// Air
    Air,

    /// Argon
    Argon,

    /// Krypton
    Krypton,

    /// Xenon
    Xenon,
}

/// Represents a Gas, as understood by the standard ISO-15099(2003).
///
/// ## Examples
///
/// #### `.spl`
///
/// ```json
/// {{#include ../../../model/tests/scanner/substance_gas.spl}}
/// ```
///
/// #### `.json`
///
/// ```json
/// {{#include ../../../model/tests/scanner/substance_gas.json}}
/// ```
#[derive(Debug, Clone, ObjectIO, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gas {
    /// The name of the Substance. Should be unique for each
    /// Substance in the Model object    
    pub name: String,

    /// A predefined gas
    #[serde(skip_serializing_if = "Option::is_none")]
    gas: Option<GasSpecification>,
}

impl Gas {
    /// Wraps the `Gas` in a [`Substance`] enum
    pub fn wrap(self) -> Substance {
        crate::substance::Substance::Gas(std::sync::Arc::new(self))
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;
    use crate::model::Model;
    use json5;

    #[test]
    fn serde_gas_spec() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let air = GasSpecification::Air;
        assert_eq!(
            "\"Air\"",
            json5::to_string(&air).map_err(|e| e.to_string())?
        );
        let argon = GasSpecification::Argon;
        assert_eq!(
            "\"Argon\"",
            json5::to_string(&argon).map_err(|e| e.to_string())?
        );
        let krypton = GasSpecification::Krypton;
        assert_eq!(
            "\"Krypton\"",
            json5::to_string(&krypton).map_err(|e| e.to_string())?
        );
        let xenon = GasSpecification::Xenon;
        assert_eq!(
            "\"Xenon\"",
            json5::to_string(&xenon).map_err(|e| e.to_string())?
        );

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: GasSpecification = json5::from_str("'Air'").map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", json5_heater));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/gas_specification";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json: GasSpecification =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", from_json));

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&air).map_err(|e| e.to_string())?;
        println!("{}", &rust_json);
        let rust_again: GasSpecification =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", rust_again));

        Ok(())
    }

    #[test]
    fn serde_gas() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let xenon = GasSpecification::Xenon;
        let air = Gas {
            name: "Xenon".into(),
            gas: Some(xenon),
        };

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: Gas = json5::from_str(
            "{    
            name: 'Xenon',     
            gas: 'Xenon'        
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", json5_heater));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/gas_xenon";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json: Gas = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", from_json));

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&air).map_err(|e| e.to_string())?;
        println!("{}", &rust_json);
        let rust_again: Gas = serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", air), format!("{:?}", rust_again));

        // test simple
        let (model, ..) = Model::from_file("./tests/scanner/substance_gas.spl")?;
        assert_eq!(1, model.substances.len());

        if let Substance::Gas(g) = &model.substances[0] {
            assert_eq!("Some Gas", g.name());
            if let Some(GasSpecification::Xenon) = g.gas {
                assert!(true)
            } else {
                assert!(false, "Wrong gas")
            }
        } else {
            assert!(false, "Wrong substance")
        }

        Ok(())
    }
}
