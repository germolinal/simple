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

use derive::{ObjectAPI, ObjectIO};

use geometry::{Loop3D, Polygon3D};
use serde::{Deserialize, Serialize};

use crate::boundary::Boundary;
use crate::model::Model;
use crate::simulation_state_element::StateElementField;

/// Defines whether the Fenestration is fixed or openable.
///
/// ## Example
///
/// #### `.json`
/// ```json
/// {{#include ../../../tests/scanner/fenestration_position.json}}
/// ```
///
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model,
/// as it is always embeded on a `Fenestration` object
///
#[derive(Debug, Copy, Clone, ObjectIO, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum FenestrationPosition {
    /// It is fixed at a certain open fraction
    Fixed {
        /// The fraction at which this `Fenestration` is open (or closed)
        fraction: Option<Float>,
    },
    /// It can be position at any position, from fully opened to fully closed
    Continuous {
        /// The maximum fraction this `Fenestration` can be opened.
        max: Option<Float>,

        /// The maximum fraction this `Fenestration` can be opened.
        min: Option<Float>,
    },
    /// It can only be opened or closed, no in-between
    Binary {
        /// The open-fraction when this `Fenestration` is open
        open: Option<Float>,

        /// The open-fraction when this `Fenestration` is closed
        closed: Option<Float>,
    },
}

/// Defines whether the fenestration is a Door or a Window.
///
/// At present, this option has no effect whatsoever
///
/// ## Example
///
/// #### `.json`
/// ```json
/// {{#include ../../../tests/scanner/fenestration_type.json}}
/// ```
///
/// > **Note**: This object cannot be declared by itself in a `SIMPLE` model,
/// as it is always embeded on a `Fenestration` object
///
#[derive(Debug, Copy, Clone, Eq, PartialEq, ObjectIO, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum FenestrationType {
    /// This is a Window
    Window,
    /// This is a Door
    Door,
}

/// A surface that can potentially be opened and closed.
/// It can be of any Construction and it does not need to be
/// a hole in another surface.
///
/// ## Examples
///
/// #### `.spl`
/// ```json
/// {{#include ../../../tests/scanner/fenestration.spl}}
/// ```
///
/// #### `.json`
/// ```json
/// {{#include ../../../tests/scanner/fenestration.json}}
/// ```
#[derive(Debug, ObjectIO, ObjectAPI, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fenestration {
    /// The name of the sub surface
    pub name: String,

    /// An array of Numbers representing the vertices of the
    /// surface. The length of this array must be divisible by 3.
    pub vertices: Polygon3D,

    /// The name of the Construction object in the
    /// constructions property of the Model object    
    pub construction: String,

    /// Defines whether a `Fenestration` is a Window, Door, or other.
    /// If none is given, the assumed behaviour is that it is a Window.
    pub category: Option<FenestrationType>,

    /// The opportunity for operating the Fenestration.
    /// If none is given, the window is assumed to be Fixed
    /// at Closed position.
    operation: Option<FenestrationPosition>,

    /// The front Boundary. No boundary means it leads to the
    /// exterior
    #[serde(default)]
    pub front_boundary: Boundary,

    /// The back Boundary. No boundary means it leads to the
    /// exterior
    #[serde(default)]
    pub back_boundary: Boundary,

    /// The front convection coefficient, in `W/m2K`
    /// 
    /// This value fixes the value, so the automatic calculations
    /// in SIMPLE have no effect.
    precalculated_front_convection_coef: Option<Float>,

    /// The back convection coefficient, in `W/m2K`
    /// 
    /// This value fixes the value, so the automatic calculations
    /// in SIMPLE have no effect.
    precalculated_back_convection_coef: Option<Float>,

    /// The name of the surface containing this `Fenestration`,
    /// if any. A hole will be made in the parent surface in order
    /// to accomodate
    ///
    /// Note that a `Fenestration` can be self contained (i.e., it can have
    /// no parent surface), which allows representing situation where
    /// the fenestration is very large and therefore there would be no area
    /// for wall.
    ///
    /// > Note: This field will not be serialized. This is beacuse there is
    /// no quarantee that the `surfaces` will be there before the `fenestrations`
    /// and thus there would be errors when deserializing. In any case, it is assumed that
    /// JSON models are read/written by machines—e.g., either by
    /// serializing a Model or by another kind of machine—and therefore
    /// the convenience of not having to write down the vertices around
    /// holes is not much needed.
    #[serde(skip_serializing)]
    parent_surface: Option<String>,

    #[physical("front_temperature")]
    #[serde(skip)]
    first_node_temperature: StateElementField,

    #[physical("back_temperature")]
    #[serde(skip)]
    last_node_temperature: StateElementField,

    /// Index of the SimulationStateElement representing
    /// the fraction open in the SimulationState
    #[operational]
    #[serde(skip)]
    open_fraction: StateElementField,

    #[physical]
    #[serde(skip)]
    front_convection_coefficient: StateElementField,

    #[physical]
    #[serde(skip)]
    back_convection_coefficient: StateElementField,

    #[physical]
    #[serde(skip)]
    front_convective_heat_flow: StateElementField,

    #[physical]
    #[serde(skip)]
    back_convective_heat_flow: StateElementField,

    #[physical]
    #[serde(skip)]
    front_incident_solar_irradiance: StateElementField,

    #[physical]
    #[serde(skip)]
    back_incident_solar_irradiance: StateElementField,

    #[physical]
    #[serde(skip)]
    front_ir_irradiance: StateElementField,

    #[physical]
    #[serde(skip)]
    back_ir_irradiance: StateElementField,
}

impl Fenestration {
    /// Clones the outer [`Loop3D`] of the [`Fenestration`]
    pub fn clone_loop(&self) -> Loop3D {
        self.vertices.outer().clone()
    }

    /// Gets the area, based on the [`Polygon3D`] that represents
    /// this [`Fenestration`]
    pub fn area(&self) -> Float {
        self.vertices.area()
    }

    /// Can the fenestration be operated?
    pub fn is_operable(&self) -> bool {
        if let Some(o) = &self.operation {
            match o {
                FenestrationPosition::Fixed { .. } => false,
                FenestrationPosition::Continuous { .. } => true,
                FenestrationPosition::Binary { .. } => true,
            }
        } else {
            false
        }
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn serde_fenestration_positions() {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = FenestrationPosition::Continuous {
            max: Some(1.0),
            min: Some(0.0),
        };

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: FenestrationPosition = json5::from_str(
            "{
            type : 'Continuous', 
            max : 1.0,
            min : 0.0
        }",
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/fenestration_position";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: FenestrationPosition = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: FenestrationPosition = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        // ... no need, it can't be defined in SIMPLE standalone
    }

    #[test]
    fn serde_fenestration_type() {
        use json5;
        use std::fs;

        // Hardcode a reference
        let hardcoded_ref = FenestrationType::Window;

        // Deserialize from hardcoded string and check they are the same
        let from_hardcoded_json: FenestrationType = json5::from_str(
            "{
            type : 'Window',             
        }",
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_hardcoded_json)
        );

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/fenestration_type";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: FenestrationType = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: FenestrationType = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        // ... no need, it can't be defined in SIMPLE standalone
    }

    #[test]
    fn serde_fenestration() {
        use json5;
        use std::fs;

        // Hardcode a reference
        // this is too verbose... we will start from the next one.
        // let hardcoded_ref = Fenestration::new();

        // Deserialize from hardcoded string and check they are the same
        let hardcoded_ref: Fenestration = json5::from_str(
            "{
            name: 'Window 1',
            construction: 'Double Clear Glass',
            back_boundary: {
                type: 'Space',
                space: 'Space 1',
            },
            operation: {
                type: 'Fixed',
            },
            category: {
                type: 'Window',
            },
            vertices: [
                0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ]
        }",
        )
        .unwrap();
        assert_eq!(&hardcoded_ref.name, "Window 1");
        assert_eq!(&hardcoded_ref.construction, "Double Clear Glass");        
        assert!( matches!(&hardcoded_ref.front_boundary, Boundary::Outdoor ) );
        if let Boundary::Space { space } = &hardcoded_ref.back_boundary {
            assert_eq!(space, "Space 1")
        } else {
            assert!(false, "Incorrect fenestration operat")
        }

        if let Some(FenestrationPosition::Fixed { fraction }) = &hardcoded_ref.operation {
            assert!(fraction.is_none())
        } else {
            assert!(false, "Incorrect fenestration operat")
        }

        if let Some(c) = &hardcoded_ref.category {
            assert_eq!(&FenestrationType::Window, c);
        } else {
            assert!(false, "Incorrect fenestration cagetory")
        }

        assert_eq!(&hardcoded_ref.vertices.outer().len(), &(4 as usize));

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/fenestration";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: Fenestration = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: Fenestration = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // check simple
        let (model, ..) = Model::from_file("./tests/box_with_window.spl").unwrap();
        assert_eq!(model.fenestrations.len(), 1);
        assert_eq!("Zn001:Wall001:Win001", model.fenestrations[0].name());
    }
}
