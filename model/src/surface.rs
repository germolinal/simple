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

use crate::{Boundary, Model};
use derive::{ObjectAPI, ObjectIO};
use geometry::Polygon3D;
use serde::{Deserialize, Serialize};

use crate::simulation_state_element::StateElementField;

/// The kind of surface, useful for pre- and post-processing.
/// 
/// This information does not affect the simulation outcome.
/// For example, `SIMPLE` will decide whether a wall is interior
/// or exterior based on the boundaries, and will not check these 
/// labels. However, these labels can be useful for creating 
/// measures (e.g., insulate all the exterior walls)
/// 
/// # Example
/// 
/// #### `.json`
/// ```json
/// {{#include ../../../tests/scanner/surface_type.json}}
/// ```
/// ```json
/// {{#include ../../../tests/scanner/surface_type_2.json}}
/// ```
/// 
#[derive(Debug,ObjectIO, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SurfaceType {
    /// A Wall that connects a space with the exterior.
    ExteriorWall,

    /// A Wall connecting two spaces
    InteriorWall,

    /// A useful horizontal surface that touches the ground
    GroundFloor,

    /// A floor that leads to the exterior (e.g., a floor
    /// suspended by columns)
    ExteriorFloor,
    
    /// Floor/ceiling that connects two spaces, and whose area is part of the  
    /// building or project being modelled. For instance, the top ceiling of an 
    /// apartment is often, strictly speaking, an interior floor... but it is 
    /// not part of the sellable area of the apartment itself
    InteriorFloor,
    
    /// A floor/ceiling that connect two spaces, but whose area is not part
    /// of the area of the project/building (see the `InteriorFloor` variant).
    Ceiling,

    /// A surfaces at the top of a building
    Roof,

    /// Other categories that might be useful (again, these labels are
    /// used based on conventions.)
    Custom {
        /// The name of the custom type
        name: String
    } 
}

/// A fixed (i.e., not movable) surface in the building (or surroundings). This can be of
/// any Construction, transparent or not.
///
/// ## Examples
///
/// #### `.spl`
/// ```json
/// {{#include ../../../tests/scanner/surface.spl}}
/// ```
/// #### `.json`
/// ```json
/// {{#include ../../../tests/scanner/surface.json}}
/// ```
#[derive(Debug, ObjectIO, ObjectAPI, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    /// The name of the surface
    pub name: String,

    /// An array of Numbers representing the vertices of the
    /// surface. The length of this array must be divisible by 3.
    pub vertices: Polygon3D,

    /// The name of the construction in the Model's
    /// Construction array    
    pub construction: String,

    /// The Boundary in front of the Surface
    #[serde(default)]
    pub front_boundary: Boundary,

    /// The Boundary in back of the Surface
    #[serde(default)]
    pub back_boundary: Boundary,
    
    /// The Surface Category. 
    /// 
    /// This field does not affect the simulations, as 
    /// it was designed to be used based on conventions (see
    /// [`SurfaceType`] documentation). So, if no [`SurfaceType`]
    /// is assigned, we cannot tell you what to do.
    category: Option<SurfaceType>,

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

    /* STATE */
    #[physical("front_temperature")]
    #[serde(skip)]
    first_node_temperature: StateElementField,

    #[physical("back_temperature")]
    #[serde(skip)]
    last_node_temperature: StateElementField,

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

/// A surface in the Model, separating two spaces,
/// or a space and the exterior, or exterior and exterior
impl Surface {
    /// Returns the area of the [`Surface`] (calculated
    /// based on the [`Polygon3D`] that represents it)
    pub fn area(&self) -> Float {
        self.vertices.area()
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    use geometry::{Loop3D, Point3D};
    use json5;
    use std::fs;

    #[test]
    fn serde_surface_type(){
        // Hardcode a reference... too verbose
        // Deserialize from hardcoded string and check they are the same
        let hardcoded_ref: SurfaceType = json5::from_str(
        "{
            type: 'ExteriorWall',            
        }",
        )
        .unwrap();
        assert_eq!(SurfaceType::ExteriorWall, hardcoded_ref);
        

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/surface_type";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: SurfaceType = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );


        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/surface_type_2";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: SurfaceType = serde_json::from_str(&json_data).unwrap();
        if let SurfaceType::Custom { name } = from_json_file {
            assert_eq!(name, "My Custom Category");
        }else{
            panic!("Not a custom category!")
        }

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: SurfaceType = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );
        

    }

    #[test]
    fn serde_surface() {

        
        

        
        // Hardcode a reference... too verbose
        // Deserialize from hardcoded string and check they are the same
        let hardcoded_ref: Surface = json5::from_str(
            "{
            name: 'the surface',
            construction:'the construction', 
            vertices: [ 
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ... 
            ]
        }",
        )
        .unwrap();
        assert_eq!(&hardcoded_ref.name, "the surface");
        assert_eq!(&hardcoded_ref.construction, "the construction");

        // Read json file (used in DOC), Deserialize, and compare
        let filename = "./tests/scanner/surface";
        let json_file = format!("{}.json", filename);
        let json_data = fs::read_to_string(json_file).unwrap();
        let from_json_file: Surface = serde_json::from_str(&json_data).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_json_file)
        );

        // Serialize and deserialize again... check that everythin matches the pattern
        let rust_json = serde_json::to_string(&hardcoded_ref).unwrap();
        let from_serialized: Surface = serde_json::from_str(&rust_json).unwrap();
        assert_eq!(
            format!("{:?}", hardcoded_ref),
            format!("{:?}", from_serialized)
        );

        // Check spl
        let (model, ..) = Model::from_file("./tests/scanner/surface.spl").unwrap();
        assert_eq!(1, model.surfaces.len());
        let s = &model.surfaces[0];
        assert_eq!(s.name(), "the surface");
        assert_eq!(s.construction, "the construction");
        assert_eq!(4, s.vertices.outer().len());
    }

    #[test]
    fn surface_from_file() {
        let (model, ..) = Model::from_file("./tests/box.spl").unwrap();
        assert_eq!(model.surfaces.len(), 1);
        assert!(&"the surface".to_string() == model.surfaces[0].name());
        assert_eq!(4, model.surfaces[0].vertices.outer().len());
    }

    #[test]
    fn test_surface_basic() {
        let construction = "the construction".to_string();
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(0., 0., 0.)).unwrap();
        outer.push(Point3D::new(2., 0., 0.)).unwrap();
        outer.push(Point3D::new(2., 2., 0.)).unwrap();
        outer.push(Point3D::new(0., 2., 0.)).unwrap();
        outer.close().unwrap();

        let polygon = Polygon3D::new(outer).unwrap();

        let surf_name = "Some surface".to_string();
        let mut surf = Surface::new(
            surf_name.clone(), 
            polygon, 
            construction,
            Boundary::Outdoor,
            Boundary::Outdoor,
        );

        assert!( matches!(surf.front_boundary, Boundary::Outdoor) );
        assert!( matches!(surf.back_boundary, Boundary::Outdoor) );        
        assert!(surf.first_node_temperature.lock().unwrap().is_none());
        assert!(surf.first_node_temperature_index().is_none());
        assert!(surf.last_node_temperature.lock().unwrap().is_none());
        assert!(surf.last_node_temperature_index().is_none());

        surf.front_boundary = Boundary::Ground;
        surf.set_first_node_temperature_index(31).unwrap();
        surf.set_last_node_temperature_index(39).unwrap();

        
        assert!(surf.first_node_temperature.lock().unwrap().is_some());
        assert_eq!(surf.first_node_temperature_index(), Some(31));
        assert!(surf.last_node_temperature.lock().unwrap().is_some());
        assert_eq!(surf.last_node_temperature_index(), Some(39));

        assert!((surf.area() - 4.).abs() < 1e-5);
    }
}
