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

/// This class modifies the wind speed in the site
#[derive(Debug, Eq, PartialEq, Clone, Copy, ObjectIO, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TerrainClass {
    /// Describes a Flat, Open Country
    Country,

    /// Describes a Rough, Wooded Country or Suburb
    Suburbs,

    /// Describes Towns, City Outskirts, and centers of large cities
    City,

    /// Describes sites next to the Ocean or Bayou Flat
    Ocean,

    /// Describes Urban, Industrual or Forest terrain
    Urban,
}

impl std::default::Default for TerrainClass {
    fn default() -> Self {
        Self::Suburbs
    }
}

impl TerrainClass {
    /// Calculates the value by which
    /// the weather file wind speed needs to be multiplied in order to estimate the wind
    /// speed at a certain height.
    ///
    /// This is a rip off from EnergyPlus' Engineering Reference, where they explain that
    /// the corrected wind speed ($`V_z`$ in $`m/s`$) at a certain altitude $`z`$ in $`m`$
    /// (e.g., the height of the window's centroid) can be estimated through an equation that
    /// relates the measurements at the meteorological station and those in the site.
    ///
    /// Specifically, this equation depends on the altitude
    /// at which the wind speed was measured at the meteorological station ($`z_{met}`$,
    /// assumed to be $`10m`$), the so-called "wind speed profile boundary layer" at the
    /// weather station ($`\delta_{met}`$, assumed to be $`240m`$) and the "wind speed profile
    /// exponent" at the meteorological station $`\alpha_{met}`$. Also, it depends on the
    /// "wind speed profile boundary layer" at the site ($`\delta`$) and the "wind speed profile
    /// exponent" $`\alpha`$.
    ///
    /// ```math
    /// V_z = \left(\frac{\delta_{met}}{z_{met}}\right)^{\alpha_{met}}\left(\frac{z}{\delta} \right)^{\alpha}
    /// ```
    /// The values for $`\alpha`$ and $`\delta`$ depend on the kind of terrain.
    ///
    /// | Terrain Class | $`\alpha`$ | $`\delta`$ |
    /// |---------------|------------|------------|
    /// | Country       | 0.14       | 270        |
    /// | Suburbs       | 0.22       | 370        |
    /// | City          | 0.33       | 460        |
    /// | Ocean         | 0.10       | 210        |
    /// | Urban         | 0.22       | 370        |
    ///
    /// > Note: if height is Zero, then we assume the wind speed to be Zero
    pub fn wind_speed_modifier(&self, height: Float) -> Float {
        // Surface touching the ground... no wind
        if height < 1e-5 {
            return 0.0;
        }

        let (alpha, delta) = match self {
            Self::Country => (0.14, 270.),
            Self::Suburbs => (0.22, 370.),
            Self::City => (0.33, 460.),
            Self::Ocean => (0.10, 210.),
            Self::Urban => (0.22, 370.),
        };

        (270. / 10. as Float).powf(0.14) * (height / delta).powf(alpha)
    }
}

/// Some information about the site in which the building(s) are located
///
/// # Examples
///
/// #### `.spl`
/// ```json
/// {{#include ../../../model/tests/scanner/site_details.spl }}
/// ```
///  #### `.json`
/// ```rs
/// {{#include ../../../model/tests/scanner/site_details.spl }}
/// ```
#[derive(Clone, Debug, ObjectIO, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteDetails {
    /// The altitude of the site.
    #[serde(skip_serializing_if = "Option::is_none")]
    altitude: Option<Float>,

    /// The kind of terrain
    #[serde(skip_serializing_if = "Option::is_none")]
    terrain: Option<TerrainClass>,

    /// In degrees. South is negative and North is positive
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<Float>,

    /// In degrees. West is negative, east is positive.
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<Float>,

    /// In degrees. This is 15*GMT Time Zone
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    #[serde(skip_serializing_if = "Option::is_none")]
    standard_meridian: Option<Float>,
}

impl SiteDetails {
    /// Calculates the value by which
    /// the weather file wind speed needs to be multiplied in order to estimate the wind
    /// speed at a certain height.
    ///
    /// This is a rip off from EnergyPlus' Engineering Reference, where they explain that
    /// the corrected wind speed ($`V_z`$ in $`m/s`$) at a certain altitude $`z`$ in $`m`$
    /// (e.g., the height of the window's centroid) can be estimated through an equation that
    /// relates the measurements at the meteorological station and those in the site.
    ///
    /// Specifically, this equation depends on the altitude
    /// at which the wind speed was measured at the meteorological station ($`z_{met}`$,
    /// assumed to be $`10m`$), the so-called "wind speed profile boundary layer" at the
    /// weather station ($`\delta_{met}`$, assumed to be $`240m`$) and the "wind speed profile
    /// exponent" at the meteorological station $`\alpha_{met}`$. Also, it depends on the
    /// "wind speed profile boundary layer" at the site ($`\delta`$) and the "wind speed profile
    /// exponent" $`\alpha`$.
    ///
    /// ```math
    /// V_z = \left(\frac{\delta_{met}}{z_{met}}\right)^{\alpha_{met}}\left(\frac{z}{\delta} \right)^{\alpha}
    /// ```
    /// The values for $`\alpha`$ and $`\delta`$ depend on the kind of terrain.
    ///
    /// | Terrain Class | $`\alpha`$ | $`\delta`$ |
    /// |---------------|------------|------------|
    /// | Country       | 0.14       | 270        |
    /// | Suburbs       | 0.22       | 370        |
    /// | City          | 0.33       | 460        |
    /// | Ocean         | 0.10       | 210        |
    /// | Urban         | 0.22       | 370        |
    ///
    /// > Note: if height is Zero, then we assume the wind speed to be Zero
    pub fn wind_speed_modifier(&self, height: Float) -> Float {
        if let Some(terrain) = self.terrain {
            terrain.wind_speed_modifier(height)
        } else {
            // default to default
            TerrainClass::default().wind_speed_modifier(height)
        }
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    use crate::Model;

    #[test]
    fn serde_site_details() -> Result<(), String> {
        use json5;
        use std::fs;

        // Hardcode a reference
        let mut hardcoded_ref = SiteDetails::new();
        hardcoded_ref.set_altitude(123.);
        hardcoded_ref.set_terrain(TerrainClass::Urban);

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: SiteDetails = json5::from_str(
            "{            
            altitude: 123.0,
            terrain: {
                type: 'Urban'
            }
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/site_details";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: SiteDetails =
            serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: SiteDetails =
            serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/scanner/site_details.spl")?;
        if let Some(d) = &model.site_details {
            assert!((123. - d.altitude.ok_or("Could not get altitude")?).abs() < 1e-8);

            if let Some(terrain) = d.terrain {
                assert_eq!(terrain, TerrainClass::Urban);
            } else {
                assert!(false, "No terrain!")
            }
        } else {
            assert!(false, "No site details!")
        }
        Ok(())
    }

    #[test]
    fn site_details_from_file() -> Result<(), String> {
        let (model, ..) = Model::from_file("./tests/box_with_window.spl")?;

        if let Some(opt) = model.site_details {
            if let Some(altitude) = opt.altitude {
                assert!((altitude - 10.).abs() < 1e-6);
            } else {
                unreachable!()
            }
            if let Some(terrain) = opt.terrain {
                assert_eq!(terrain, TerrainClass::Urban);
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
        Ok(())
    }
}
