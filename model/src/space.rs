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

use core::fmt;

use crate::infiltration::Infiltration;
use crate::model::Model;
use crate::simulation_state_element::StateElementField;
use crate::Float;
use derive::{ObjectAPI, ObjectIO};
use serde::{Deserialize, Serialize};

/// The category of a space.
#[derive(Debug, Default, ObjectIO, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SpacePurpose {
    /// Bathroom, toilette, shower, etc.    
    Bathroom,
    /// Bedroom
    Bedroom,
    /// Dining room
    DiningRoom,
    /// Kitchen
    Kitchen,
    /// Living room
    LivingRoom,
    /// Office
    Office,
    /// Garage
    Garage,
    /// Hallway
    Hallway,
    /// Other
    #[default]
    Other,
}

impl std::fmt::Display for SpacePurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SpacePurpose::Bathroom => "Bathroom",
            SpacePurpose::Bedroom => "Bedroom",
            SpacePurpose::DiningRoom => "Dining Room",
            SpacePurpose::Kitchen => "Kitchen",
            SpacePurpose::LivingRoom => "Living Room",
            SpacePurpose::Office => "Office",
            SpacePurpose::Garage => "Garage",
            SpacePurpose::Hallway => "Hallway",
            SpacePurpose::Other => "Other",
        };
        write!(f, "{}", s)
    }
}

/// Represents a space with homogeneous temperature within a building. It is often actual room enclosed by walls, but it can also
/// be more than one room. In this latter case, there will be walls
/// within the Space, meaning that there are walls whose Front and Back
/// boundary is this space.
///
/// ## Examples
///
/// #### `.spl`
/// ```rs
/// {{#include ../../../model/tests/scanner/space.spl}}
/// ```
/// #### `.json`
/// ```rs
/// {{#include ../../../model/tests/scanner/space.json}}
/// ```
#[derive(Debug, ObjectIO, ObjectAPI, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Space {
    /// The name of the space
    pub name: String,

    /// Volume of the space
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<Float>,

    /// The infiltration in the space
    #[serde(skip_serializing_if = "Option::is_none")]
    infiltration: Option<Infiltration>,

    // The importance of this space over time
    // importance : Option<Box<dyn Schedule<Float>>>,
    /// The building in which this `Space` is inserted
    #[serde(skip_serializing_if = "Option::is_none")]
    building: Option<String>,

    /// The storey in which the space is located,
    /// indexing from 0 (i.e., ground floor is 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    storey: Option<usize>,

    /// The purposes in a room. It can have multiple
    /// purposes (e.g., a Living/Dining/Kithen space)
    #[serde(skip_serializing_if = "Vec::is_empty")]    
    #[serde(default)]
    pub purposes: Vec<SpacePurpose>,

    #[physical]
    #[serde(skip)]
    dry_bulb_temperature: StateElementField,

    #[physical]
    #[serde(skip)]
    brightness: StateElementField,

    #[physical]
    #[serde(skip)]
    loudness: StateElementField,

    #[physical]
    #[serde(skip)]
    infiltration_volume: StateElementField,

    #[physical]
    #[serde(skip)]
    infiltration_temperature: StateElementField,

    #[physical]
    #[serde(skip)]
    ventilation_volume: StateElementField,

    #[physical]
    #[serde(skip)]
    ventilation_temperature: StateElementField,
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
        let mut hardcoded_ref = Space::new("Walrus Enclosure");
        hardcoded_ref.set_volume(249.);
        hardcoded_ref.set_building("Wonderful Zoo");

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: Space = json5::from_str(
            "{
            name: 'Walrus Enclosure',
            volume: 249,
            building: 'Wonderful Zoo'
        }",
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/space";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).map_err(|e| e.to_string())?;
        let from_json_file: Space = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).map_err(|e| e.to_string())?;
        let from_serialized: Space = serde_json::from_str(&rust_json).map_err(|e| e.to_string())?;
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/scanner/space.spl")?;
        assert_eq!(model.spaces.len(), 1);
        assert!("Walrus Enclosure" == model.spaces[0].name());

        Ok(())
    }

    #[test]
    fn test_new() -> Result<(), String> {
        let space_name = "the_space".to_string();

        let mut space = Space::new(space_name.clone());
        assert_eq!(space.name, space_name);
        assert!(space.volume().is_err());

        let vol = 987.12312;
        space.set_volume(vol);
        assert_eq!(*space.volume().unwrap(), vol);

        let i = 91;
        assert!(space
            .dry_bulb_temperature
            .lock()
            .map_err(|e| e.to_string())?
            .is_none());
        assert!(space.dry_bulb_temperature_index().is_none());
        space
            .set_dry_bulb_temperature_index(i)
            .map_err(|e| e.to_string())?;
        assert!(space
            .dry_bulb_temperature
            .lock()
            .map_err(|e| e.to_string())?
            .is_some());
        assert_eq!(
            space
                .dry_bulb_temperature_index()
                .ok_or("no dry bulb temperature")?,
            i
        );

        let i = 191;
        assert!(space
            .brightness
            .lock()
            .map_err(|e| e.to_string())?
            .is_none());
        assert!(space.brightness_index().is_none());
        space.set_brightness_index(i).map_err(|e| e.to_string())?;
        assert!(space
            .brightness
            .lock()
            .map_err(|e| e.to_string())?
            .is_some());
        assert_eq!(space.brightness_index().ok_or("no brightness")?, i);

        let i = 111;
        assert!(space.loudness.lock().map_err(|e| e.to_string())?.is_none());
        assert!(space.loudness_index().is_none());
        space.set_loudness_index(i).map_err(|e| e.to_string())?;
        assert!(space.loudness.lock().map_err(|e| e.to_string())?.is_some());
        assert_eq!(space.loudness_index().ok_or("no loudness")?, i);

        Ok(())
    }
}
