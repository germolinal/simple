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
    altitude: Option<Float>,

    /// The kind of terrain
    terrain: Option<TerrainClass>,

    /// In degrees. South is negative and North is positive
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    latitude: Option<Float>,

    /// In degrees. West is negative, east is positive.
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    longitude: Option<Float>,

    /// In degrees. This is 15*GMT Time Zone
    ///
    /// This value is irrelevant if a simulation is ran based
    /// on an EPW weather file (in which case it is extracted
    /// such file). However, it can be useful when producing
    /// synthetic weathers or HVAC sizing, or other applications.
    standard_meridian: Option<Float>,
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    use crate::Model;

    #[test]
    fn serde_site_details() {
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
        .unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/site_details";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: SiteDetails = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: SiteDetails = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/scanner/site_details.spl").unwrap();
        if let Some(d) = &model.site_details {
            assert!((123. - d.altitude.unwrap()).abs() < 1e-8);

            if let Some(terrain) = d.terrain {
                assert_eq!(terrain, TerrainClass::Urban);
            } else {
                assert!(false, "No terrain!")
            }
        } else {
            assert!(false, "No site details!")
        }
    }

    #[test]
    fn site_details_from_file() {
        let (model, ..) = Model::from_file("./tests/box_with_window.spl").unwrap();

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
    }
}
