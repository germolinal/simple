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
use crate::substance::Substance;
use crate::Float;
use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Represents a physical material
/// with common physical properties (e.g.,
/// timber, concrete, brick, glass).  In other words,
/// it cannot change its thermal/optical properties based
/// on its internal state (e.g., it cannot change its specific heat
/// based on temperature, like Gas or Phase Change Materials).
///
/// ## Examples
///
/// #### `.spl`
///
/// ```json
/// {{#include ../../../model/tests/scanner/substance_normal.spl}}
/// ```
///
/// #### `.json`
///
/// ```json
/// {{#include ../../../model/tests/scanner/substance_normal.json}}
/// ```
#[derive(Default, Clone, Debug, ObjectIO, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Normal {
    /// The name of the Substance. Should be unique for each
    /// Substance in the Model object    
    pub name: String,

    /* THERMAL PROPERTIES */
    /// The thermal conductivity of the substance in W/m.K    
    #[serde(skip_serializing_if = "Option::is_none")]
    thermal_conductivity: Option<Float>,

    /// The specific heat capacity of the substance in J/kg.K
    #[serde(skip_serializing_if = "Option::is_none")]
    specific_heat_capacity: Option<Float>,

    /// The density of the substance in kg/m3
    #[serde(skip_serializing_if = "Option::is_none")]
    density: Option<Float>,

    /* SOLAR RADIATION PROPERTIES */
    /// Solar absorbtance (from 0 to 1) at the front side
    /// (Front being the side closer to the first material in a construction)
    ///
    /// Absorbtance is used instead of reflectance (which is used in visible radiation
    /// properties) to maintain coherence with EnergyPlus
    #[serde(skip_serializing_if = "Option::is_none")]
    front_solar_absorbtance: Option<Float>,

    /// Solar absorbtance (from 0 to 1) at the front side
    /// (Back being the side closer to the last material in a construction)
    ///
    /// Absorbtance is used instead of reflectance (which is used in visible radiation
    /// properties) to maintain coherence with EnergyPlus... because in Thermal calculation
    /// we mostly care about how much is absorbed; in lighting we care mainly about how much
    /// is reflected.
    #[serde(skip_serializing_if = "Option::is_none")]
    back_solar_absorbtance: Option<Float>,

    /// The front solar transmittance at normal incidence (from 0 to 1)    
    ///
    /// Please note that, contrary to all other properties, this property
    /// does depend on the thickness of the substance. So, in order
    /// to build a coherent Glazing, you'll need to match this Substance
    /// with an appropriate `Material`
    ///
    /// Transmittance is used instead of transmissivity (which is used in visible radiation
    /// properties) to maintain coherence with EnergyPlus... because in Thermal calculation
    /// we mostly care about how much is absorbed; in lighting we care mainly about how much
    /// is reflected.
    #[serde(skip_serializing_if = "Option::is_none")]
    solar_transmittance: Option<Float>,

    /* VISIBLE RADIATION PROPERTIES */
    /// Solar absorbtance (from 0 to 1) at the front side
    /// (Front being the side closer to the first material in a construction)
    ///
    /// Reflectance is used instead of Absorbtance (which is used in solar radiation
    /// properties) to maintain coherence with Radiance... because in Lighting
    /// we really care about how much is reflected; in thermal we care mainly about how much
    /// is absorbed.
    #[serde(skip_serializing_if = "Option::is_none")]
    front_visible_reflectance: Option<Float>,

    /// Solar absorbtance (from 0 to 1) at the front side
    /// (Back being the side closer to the last material in a construction)
    ///
    /// Reflectance is used instead of Absorbtance (which is used in solar radiation
    /// properties) to maintain coherence with Radiance... because in Lighting
    /// we really care about how much is reflected; in thermal we care mainly about how much
    /// is absorbed.
    #[serde(skip_serializing_if = "Option::is_none")]
    back_visible_reflectance: Option<Float>,

    /// The front solar transmittance at normal incidence (from 0 to 1)    
    ///
    /// Please note that, contrary to all other properties, this property
    /// does depend on the thickness of the substance. So, in order
    /// to build a coherent Glazing, you'll need to match this Substance
    /// with an appropriate `Material`
    ///
    /// Transmissivity is used instead of Transmittance (which is used in solar radiation
    /// properties) to maintain coherence with Radiance
    #[serde(skip_serializing_if = "Option::is_none")]
    visible_transmissivity: Option<Float>,

    /* INFRARED RADIATION PROPERTIES */
    /// Front thermal absorbtance (i.e., emissitivy; from 0 to 1)
    /// (Front being the side closer to the first material in a construction)
    #[serde(skip_serializing_if = "Option::is_none")]
    front_thermal_absorbtance: Option<Float>,

    /// Back thermal absorbtance (i.e., emissitivy; from 0 to 1)
    /// (Back being the side closer to the last material in a construction)
    #[serde(skip_serializing_if = "Option::is_none")]
    back_thermal_absorbtance: Option<Float>,
}

impl Normal {
    /// Calculates the thermal diffusivity of the
    /// Normal
    pub fn thermal_diffusivity(&self) -> Result<Float, String> {
        let thermal_conductivity = self.thermal_conductivity()?;
        let density = self.density()?;
        let specific_heat_capacity = self.specific_heat_capacity()?;
        Ok(thermal_conductivity / (density * specific_heat_capacity))
    }

    /// Wraps a `Normal` in a [`Substance`] enum
    pub fn wrap(self) -> Substance {
        crate::substance::Substance::Normal(std::sync::Arc::new(self))
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;
    use crate::model::Model;

    #[test]
    fn serde() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let normal = Normal {
            name: "the substance".into(),
            thermal_conductivity: Some(12.),
            ..Normal::default()
        };

        // Deserialize from hardcoded string and check they are the same
        let json5_heater: Normal = json5::from_str(
            "{    
            name: 'the substance',     
            thermal_conductivity: 12,            
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", normal), format!("{:?}", json5_heater));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/normal";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json: Normal = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", normal), format!("{:?}", from_json));

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&normal).map_err(|e| e.to_string())?;
        println!("{}", &rust_json);
        let rust_again: Normal = serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(format!("{:?}", normal), format!("{:?}", rust_again));

        // Check spl
        let (model, ..) = Model::from_file("./tests/scanner/substance_normal.spl")?;
        assert_eq!(1, model.substances.len());

        if let Substance::Normal(g) = &model.substances[0] {
            assert_eq!("the substance", g.name());
            assert!((12. - g.thermal_conductivity.ok_or("No thermal conductivity")?).abs() < 1e-9);
        } else {
            assert!(false, "Wrong substance")
        }

        Ok(())
    }

    #[test]
    fn test_substance_basic() -> Result<(), String> {
        let s_name = "The Normal".to_string();
        let mut s = Normal::new(s_name.clone());
        assert_eq!(s_name, s.name);
        assert!(s.thermal_conductivity().is_err());
        assert!(s.specific_heat_capacity().is_err());
        assert!(s.density().is_err());

        // Fill with properties
        let lambda = 1.23123;
        let rho = 1.2312312555;
        let c = 9.123128;
        s.set_thermal_conductivity(lambda)
            .set_specific_heat_capacity(c)
            .set_density(rho);

        assert_eq!(s.thermal_diffusivity()?, lambda / rho / c);
        assert_eq!(*s.density()?, rho);
        assert_eq!(*s.specific_heat_capacity()?, c);
        assert_eq!(*s.thermal_conductivity()?, lambda);

        Ok(())
    }
}
