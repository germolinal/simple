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
use crate::error_msgs::print_warning_no_module;
use crate::scanner::SimpleScanner;
use crate::simulation_state_element::SimulationStateElement;
use crate::{hvac::*, Boundary, SolarOptions, SurfaceType};
use crate::{Float, SiteDetails};
use crate::{Object, SurfaceTrait};
use crate::{Output, SimulationStateHeader};
use serde::{self, de::Visitor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::prelude::*;

use std::path::Path;
use std::sync::Arc;

use crate::{Building, Construction, Fenestration, Luminaire, Material, Space, Substance, Surface};

/// A structure describing a set of built-environment objects.
///
/// It can be a bunch of zones all in the same building (e.g., a house, a hotel)
/// or it can be a bunch of zones in different buildings.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// The [`Building`]s in the model
    pub buildings: Vec<Arc<Building>>,

    /// The [`Construction`]s in the model
    pub constructions: Vec<Arc<Construction>>,

    /// The windows and doors in the surface    
    pub fenestrations: Vec<Arc<Fenestration>>,

    /// The Heating/Cooling devices in the space
    pub hvacs: Vec<HVAC>,

    /// Luminaires
    pub luminaires: Vec<Arc<Luminaire>>,

    /// The [`Material`]s in the model
    pub materials: Vec<Arc<Material>>,

    /// The name of the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The furniture and appliances in the property
    pub objects: Vec<Object>,

    /// The requested outputs
    ///
    /// These aren't checked too much while parsing, but after
    /// building the model. This allows asking for data that
    /// is missing (e.g., asking for temperature in node 192)
    pub outputs: Vec<Output>,

    /// Some information about the site in which the building(s) are located
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_details: Option<SiteDetails>,

    /// The options for the Solar calculations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solar_options: Option<SolarOptions>,

    /// The [`Space`]s in the model
    pub spaces: Vec<Arc<Space>>,

    /// The [`Surface`]s in the model
    pub surfaces: Vec<Arc<Surface>>,

    /// The [`Substance`]s in the model
    pub substances: Vec<Substance>,

    /// Serde
    #[serde(skip)]
    simulation_state: Option<SimulationStateHeader>,
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.buildings.iter() {
            write!(f, "Building {}", b)?;
        }

        for b in self.constructions.iter() {
            write!(f, "Construction {}", b)?;
        }

        for b in self.fenestrations.iter() {
            write!(f, "Fenestration {}", b)?;
        }

        for b in self.hvacs.iter() {
            write!(f, "HVAC {}", b)?;
        }

        for b in self.luminaires.iter() {
            write!(f, "Luminaire {}", b)?;
        }

        for b in self.materials.iter() {
            write!(f, "Material {}", b)?;
        }

        for b in self.objects.iter() {
            write!(f, "Object {}", b)?;
        }

        for b in self.outputs.iter() {
            write!(f, "Output {}\n\n", b)?;
        }

        if let Some(s) = self.site_details.as_ref() {
            write!(f, "SiteDetails {}", s)?;
        }

        for b in self.substances.iter() {
            write!(f, "Substance {}", b)?;
        }

        for b in self.spaces.iter() {
            write!(f, "Space {}", b)?;
        }

        for b in self.surfaces.iter() {
            write!(f, "Surface {}", b)?;
        }

        Ok(())
    }
}
impl std::default::Default for Model {
    fn default() -> Self {
        Self {
            name: None,
            buildings: Vec::default(),
            fenestrations: Vec::default(),
            constructions: Vec::default(),
            hvacs: Vec::default(),
            luminaires: Vec::default(),
            materials: Vec::default(),
            objects: Vec::default(),
            outputs: Vec::default(),
            site_details: None,
            solar_options: None,
            spaces: Vec::default(),
            surfaces: Vec::default(),
            substances: Vec::default(),
            simulation_state: Some(SimulationStateHeader::new()), // yeah... this is the only field that defaults to a non-default value.
        }
    }
}

struct SimpleModelVisitor {}

impl<'de> Visitor<'de> for SimpleModelVisitor {
    type Value = Model;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse Model from JSON")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut model = Model::default();

        while let Some(key) = map.next_key::<&[u8]>()? {
            match key {
                b"buildings" => {
                    let objs: Vec<Building> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_building(o);
                    }
                }
                b"constructions" => {
                    let objs: Vec<Construction> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_construction(o);
                    }
                }
                b"fenestrations" => {
                    let objs: Vec<Fenestration> = map.next_value()?;
                    for o in objs.into_iter() {
                        model
                            .add_fenestration(o)
                            .map_err(serde::de::Error::custom)?;
                    }
                }
                b"hvacs" => {
                    let objs: Vec<HVAC> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_hvac(o).map_err(serde::de::Error::custom)?;
                    }
                }
                b"luminaires" => {
                    let objs: Vec<Luminaire> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_luminaire(o).map_err(serde::de::Error::custom)?;
                    }
                }
                b"materials" => {
                    let objs: Vec<Material> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_material(o);
                    }
                }
                b"name" => {
                    model.name = map.next_value()?;
                }
                b"objects" => {
                    let objs: Vec<Object> = map.next_value()?;
                    for o in objs.into_iter() {
                        // model.add_object(o).map_err(serde::de::Error::custom)?;
                        model.objects.push(o);
                    }
                }
                b"outputs" => {
                    let objs: Vec<Output> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.outputs.push(o);
                    }
                }
                b"site_details" => {
                    model.site_details = map.next_value()?;
                }
                b"solar_options" => {
                    model.solar_options = map.next_value()?;
                }
                b"spaces" => {
                    let objs: Vec<Space> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_space(o);
                    }
                }
                b"surfaces" => {
                    let objs: Vec<Surface> = map.next_value()?;
                    for o in objs.into_iter() {
                        // model.add_surface(o).map_err(serde::de::Error::custom)?;
                        model.surfaces.push(Arc::new(o));
                    }
                }
                b"substances" => {
                    let objs: Vec<Substance> = map.next_value()?;
                    for o in objs.into_iter() {
                        model.add_substance(o);
                    }
                }
                _ => {
                    let k = std::str::from_utf8(key).map_err(serde::de::Error::custom)?;
                    Err(format!("Field '{}' in model is not serialized", k))
                        .map_err(serde::de::Error::custom)?;
                }
            }
        }

        Ok(model)
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(SimpleModelVisitor {})
    }
}

impl Model {
    /// Gets the (lat,lon, stdmer) tuple in degrees. If either `site_details` is not there,
    /// or if any of the `latitude`, `longitude`, or `standard_meridian`
    /// in the `site_details` aren't there,
    /// it returns None.
    ///
    /// ```rust
    /// use model::Model;
    ///
    /// // Successful workflow
    ///
    /// let json_str = r#"{
    ///     "site_details": {
    ///         "latitude": 1.2,
    ///         "longitude" : 5.21,
    ///         "standard_meridian": 123.0
    ///     }
    /// }"#;
    ///
    /// let (model, header) = Model::from_json(&json_str).unwrap();
    /// assert_eq!(Some((1.2, 5.21, 123.0)), model.geolocation());         
    /// ```
    pub fn geolocation(&self) -> Option<(Float, Float, Float)> {
        if let Some(site) = &self.site_details {
            let lat = site.latitude();
            let lon = site.longitude();
            let stdmer = site.standard_meridian();
            if let (Ok(lat), Ok(lon), Ok(stdmer)) = (lat, lon, stdmer) {
                return Some((*lat, *lon, *stdmer));
            }
            return None;
        }
        None
    }

    /// Prints the model into a file called `filename`.
    pub fn print_to_file(&self, filename: &str) -> Result<(), String> {
        let mut file = File::create(filename).map_err(|e| format!("{}", e))?;
        file.write_all(self.to_string().as_bytes())
            .map_err(|e| format!("{}", e))?;
        Ok(())
    }

    /// Adds an element and default value to the model's [`SimulationStateHeader`]. Returns an error
    /// if the state has been taken already
    fn push_to_state(&mut self, e: SimulationStateElement, v: Float) -> Result<usize, String> {
        match &mut self.simulation_state {
            Some(s) => Ok(s.push(e, v)?),
            None => Err(
                "Cannot add this object tot he model because it no longer has a state".to_string(),
            ),
        }
    }

    /// Takes the [`SimulationStateHeader`] from the model.
    ///
    /// This state will very likely be incomplete, as it will only contain the
    /// elements added during the creation of the model. That is to say, only
    /// operational state (e.g., how open a [`Fenestration`] or whether an [`HVAC`]
    /// is on or not.)
    ///
    /// ```rust
    /// use model::{Model, hvac::ElectricHeater};
    ///
    /// let mut model = Model::default();
    /// let heater = ElectricHeater::new("Bedrooms heater");
    ///
    /// // Wrap the ElectricHeater in an enum variant so that we can
    /// // put different kinds of heaters/hvac in the same array
    /// let hvac = heater.wrap();
    /// model.add_hvac(hvac);
    /// let state = model.take_state().expect("There was no state?");
    /// assert_eq!(state.len(), 1); // only the state of the HVAC
    ///
    ///
    /// ```
    ///
    pub fn take_state(&mut self) -> Option<SimulationStateHeader> {
        self.simulation_state.take()
    }

    /// Parses a model from JSON
    ///
    /// ```rust
    /// use model::Model;
    ///
    /// let json_str = r#"{
    ///     "buildings": [{
    ///         "name": "The Building",
    ///         "shelter_class" : "Urban"
    ///     }]
    /// }"#;
    ///
    /// let (model, header) = Model::from_json(&json_str).unwrap();
    /// assert_eq!(header.len(), 0); // buildings don't have state
    /// assert_eq!(model.buildings.len(), 1);
    /// ```
    pub fn from_json(json: &str) -> Result<(Self, SimulationStateHeader), String> {
        let mut model: Model = match serde_json::from_str(json) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };
        let state = model
            .take_state()
            .expect("Internal Error: No State after parsing JSON?");
        Ok((model, state))
    }

    /// Parses a `Model` from a text file containing a JSON
    ///
    /// ```rust
    /// use model::Model;
    /// use std::fs;
    /// use std::io::Write;
    ///
    /// let mut file = fs::File::create("./model.json").unwrap();
    /// let s = r#"{
    ///     "buildings": [{
    ///         "name": "The Building",
    ///         "shelter_class" : "Urban"
    ///     }]
    /// }"#;
    /// write!(file, "{}", s).unwrap();
    ///
    ///
    /// let (model, header) = Model::from_json_file("./model.json").unwrap();
    /// assert_eq!(header.len(), 0); // buildings don't have state
    /// assert_eq!(model.buildings.len(), 1);
    /// fs::remove_file("./model.json").unwrap()
    /// ```
    pub fn from_json_file<P: AsRef<Path> + Display>(
        filename: P,
    ) -> Result<(Self, SimulationStateHeader), String> {
        let jsonstring = match fs::read_to_string(&filename) {
            Ok(v) => v,
            Err(_) => return Err(format!("Could not read JSON file '{}'", filename)),
        };
        Self::from_json(&jsonstring)
    }

    /// Parses a `Model` from an array of bytes (i.e., a `Vec<u8>`)
    ///
    /// ```rust
    /// use model::Model;
    ///
    /// let s = r#"
    ///     Building{
    ///         name: "The Building",
    ///         shelter_class : "Urban"
    ///     }
    /// "#;
    ///
    /// let (model, header) = Model::from_bytes(s.as_bytes()).unwrap();
    /// assert_eq!(header.len(), 0); // buildings don't have state
    /// assert_eq!(model.buildings.len(), 1);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, SimulationStateHeader), String> {
        let mut scanner = SimpleScanner::new(bytes, 1);
        scanner.parse_model()
    }

    /// Parses a `Model` from a text file
    ///
    /// ```rust
    /// use model::Model;
    /// use std::fs;
    /// use std::io::Write;
    ///
    /// let mut file = fs::File::create("./model.spl").unwrap();
    /// let s = r#"
    ///     Building{
    ///         name: "The Building",
    ///         shelter_class : "Urban"
    ///     }
    /// "#;
    /// write!(file, "{}", s).unwrap();
    ///
    ///
    /// let (model, header) = Model::from_file("./model.spl").unwrap();
    /// assert_eq!(header.len(), 0); // buildings don't have state
    /// assert_eq!(model.buildings.len(), 1);
    /// fs::remove_file("./model.spl").unwrap()
    /// ```
    pub fn from_file<P: AsRef<Path> + Display>(
        filename: P,
    ) -> Result<(Self, SimulationStateHeader), String> {
        let bytes = match fs::read(&filename) {
            Ok(v) => v,
            Err(_) => return Err(format!("Could not read SIMPLE file '{}'", filename)),
        };
        Self::from_bytes(&bytes)
    }

    /// Adds an [`Object`] to the [`Model`]
    ///
    /// ```rust
    ///
    /// use model::{Model, Object, Space, ObjectSpecs};
    /// use geometry::{Vector3D, Point3D};
    ///
    /// let space = "the space".to_string();
    /// let mut obj = Object::new(
    ///     String::default(),
    ///     Point3D::default(),
    ///     Point3D::default(),
    ///     Vector3D::z(),
    ///     Vector3D::y(),
    ///     ObjectSpecs::default(),
    /// );
    /// obj.set_space(space.clone());
    ///
    /// let mut model = Model::default();
    ///
    /// // it will fail because the space does not exist
    /// assert!(model.add_object(obj.clone()).is_err());
    ///
    /// let space = Space::new(space);
    /// model.add_space(space);
    ///
    /// // if does nto fail any more
    /// assert!(model.add_object(obj).is_ok());
    /// ````
    pub fn add_object(&mut self, add: Object) -> Result<(), String> {
        if let Ok(space_name) = add.space() {
            if self.get_space(space_name).is_err() {
                return Err(format!(
                    "Space called '{}' was not found in the model",
                    space_name
                ));
            }
        }

        self.objects.push(add);
        Ok(())
    }

    /// Adds a [`Substance`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, Substance, substance::Normal};
    ///
    /// let sub = Normal::new("some fancy material").wrap();
    /// let mut model = Model::default();
    /// assert!(model.substances.is_empty());
    /// model.add_substance(sub);
    /// assert_eq!(model.substances.len(), 1);
    ///
    /// // Adding the substance again will print a warning, but still add it
    /// let sub = Normal::new("some fancy material").wrap();
    /// model.add_substance(sub);
    /// assert_eq!(model.substances.len(), 2);
    /// ```
    pub fn add_substance(&mut self, add: Substance) -> Substance {
        if self.get_substance(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Substance called '{}'",
                add.name()
            ))
        }
        self.substances.push(add.clone());
        add
    }

    /// Retrieves a reference (`Arc`) to a [`Substance`] based on its name, from the `substances`
    /// field
    ///
    /// ```rust
    /// use model::{Model, Substance, substance::Normal};
    ///
    /// let sub = Normal::new("some fancy material").wrap();
    /// let mut model = Model::default();
    /// model.add_substance(sub);    
    ///
    /// // correct name
    /// let s = model.get_substance("some fancy material").unwrap();
    /// match s {
    ///     Substance::Normal(_) => println!("All good!"),
    ///     _ => assert!(false, "All is lost! this should have been a Normal substance!")
    /// }
    ///
    /// // incorrect name
    /// assert!(model.get_substance("I do not exist").is_err());    
    /// ```
    pub fn get_substance<S: Into<String>>(&self, name: S) -> Result<Substance, String> {
        let name: String = name.into();
        for i in self.substances.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Substance '{}' in model", name))
    }

    /// Adds a reference (`Arc`) to a [`Material`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, Material};
    ///
    /// let mut model = Model::default();
    /// let mat = Material::new("Sweet Panel", "is made of this", 0.2);
    /// assert!(model.materials.is_empty());
    /// model.add_material(mat);
    /// assert_eq!(model.materials.len(), 1);
    ///
    /// // Adding something witht the same name will warn the user... but still adds it
    /// let mat = Material::new("Sweet Panel", "is made of this", 0.2);
    /// model.add_material(mat);
    /// assert_eq!(model.materials.len(), 2);
    /// ```
    pub fn add_material(&mut self, add: Material) -> Arc<Material> {
        if self.get_material(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Material called '{}'",
                add.name()
            ))
        }
        let add = Arc::new(add);
        self.materials.push(Arc::clone(&add));
        add
    }

    /// Retrieves a reference (`Arc`) to a [`Material`] based on its name, from the `materials`
    /// field
    ///
    /// ```rust
    /// use model::{Model, Material};
    ///
    /// let mut model = Model::default();
    /// let mat = Material::new("Sweet Panel", "is made of this", 0.2);
    /// model.add_material(mat);
    ///
    /// assert!(model.get_material("Sweet Panel").is_ok());
    /// assert!(model.get_material("This inexistent Material").is_err());
    /// ```
    pub fn get_material<S: Into<String>>(&self, name: S) -> Result<Arc<Material>, String> {
        let name: String = name.into();
        for i in self.materials.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Material '{}' in model", name))
    }

    /// Adds a [`Construction`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, Construction};
    ///
    /// let c = Construction::new("Cool Construction");
    /// let mut model = Model::default();
    /// assert!(model.constructions.is_empty());
    /// model.add_construction(c);
    /// assert_eq!(model.constructions.len(), 1);
    ///
    /// // adding a new construction with the same name prints a warning
    /// let c = Construction::new("Cool Construction");
    /// model.add_construction(c);
    /// assert_eq!(model.constructions.len(), 2);
    /// ```
    pub fn add_construction(&mut self, add: Construction) -> Arc<Construction> {
        if self.get_construction(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Construction called '{}'",
                add.name()
            ))
        }
        // add.set_index(self.constructions.len());
        let add = Arc::new(add);
        self.constructions.push(Arc::clone(&add));
        add
    }

    /// Retrieves a reference (`Arc`) to a [`Construction`] based on its name, from the `constructions`
    /// field
    ///
    /// ```rust
    /// use model::{Model, Construction};
    ///
    /// let c = Construction::new("Cool Construction");
    /// let mut model = Model::default();    
    /// model.add_construction(c);
    ///
    /// assert!(model.get_construction("Cool Construction").is_ok());
    /// assert!(model.get_construction("Leaky Construction").is_err());
    /// ```
    pub fn get_construction<S: Into<String>>(&self, name: S) -> Result<Arc<Construction>, String> {
        let name: String = name.into();
        for i in self.constructions.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Construction '{}' in model", name))
    }

    /// Adds a [`Surface`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, Surface};
    /// use json5;
    ///
    /// // this is less verbose than creating the whole thing.
    /// let s: Surface = json5::from_str(
    ///     "{
    ///     name: 'the surface',
    ///     construction:'the construction',
    ///     vertices: [
    ///         0, 0, 0, // X, Y and Z of Vertex 0
    ///         1, 0, 0, // X, Y and Z of Vertex 1
    ///         1, 1, 0, // X, Y and Z of Vertex 2
    ///         0, 1, 0  // ...
    ///     ]
    ///  }").unwrap();
    ///
    /// let mut model = Model::default();
    /// assert!(model.surfaces.is_empty());
    /// model.add_surface(s);
    /// assert_eq!(model.surfaces.len(), 1);
    ///
    /// // Adding a new surface with the same name issues a warning, but still works
    /// let s: Surface = json5::from_str(
    ///     "{
    ///     name: 'the surface',
    ///     construction:'the construction',
    ///     vertices: [
    ///         0, 0, 0, // X, Y and Z of Vertex 0
    ///         1, 0, 0, // X, Y and Z of Vertex 1
    ///         1, 1, 0, // X, Y and Z of Vertex 2
    ///         0, 1, 0  // ...
    ///     ]
    ///  }").unwrap();
    /// model.add_surface(s);
    /// assert_eq!(model.surfaces.len(), 2);
    /// ```
    pub fn add_surface(&mut self, add: Surface) -> Result<Arc<Surface>, String> {
        if self.get_surface(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Surface called '{}'",
                add.name()
            ))
        }
        // Check boundaries.
        if let Boundary::Space { space } = &add.front_boundary {
            self.get_space(space)?;
        }
        if let Boundary::Space { space } = &add.back_boundary {
            self.get_space(space)?;
        }
        let add = Arc::new(add);
        self.surfaces.push(Arc::clone(&add));
        Ok(add)
    }

    /// Retrieves a reference (`Arc`) to a [`Surface`] based on its name, from the `surfaces`
    /// field
    ///
    /// ```rust
    /// use model::{Model, Surface};
    /// use json5;
    ///
    /// // this is less verbose than creating the whole thing.
    /// let s: Surface = json5::from_str(
    ///     "{
    ///     name: 'the surface',
    ///     construction:'the construction',
    ///     vertices: [
    ///         0, 0, 0, // X, Y and Z of Vertex 0
    ///         1, 0, 0, // X, Y and Z of Vertex 1
    ///         1, 1, 0, // X, Y and Z of Vertex 2
    ///         0, 1, 0  // ...
    ///     ]
    ///  }").unwrap();
    ///
    /// let mut model = Model::default();    
    /// model.add_surface(s);
    /// assert!(model.get_surface("the surface").is_ok());
    /// assert!(model.get_surface("nope... I am not here").is_err());
    /// ```
    pub fn get_surface<S: Into<String>>(&self, name: S) -> Result<Arc<Surface>, String> {
        let name: String = name.into();
        for i in self.surfaces.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Surface '{}' in model", name))
    }

    /// Adds a [`Space`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Space, Model};
    ///
    /// let space = Space::new("Bedroom");
    /// let mut model = Model::default();
    /// assert!(model.spaces.is_empty());
    /// model.add_space(space);
    /// assert_eq!(model.spaces.len(), 1);
    ///
    /// // Adding a new space with the same name prints a warning, but still works
    /// let space = Space::new("Bedroom");
    /// model.add_space(space);
    /// assert_eq!(model.spaces.len(), 2);
    /// ```
    pub fn add_space(&mut self, add: Space) -> Arc<Space> {
        if self.get_space(add.name()).is_ok() {
            print_warning_no_module(format!("There is already a Space called '{}'", add.name()))
        }
        let add = Arc::new(add);
        self.spaces.push(Arc::clone(&add));
        add
    }

    /// Retrieves a reference (`Arc`) to a [`Space`] based on its name, from the `spaces`
    /// field
    ///
    /// ```rust
    /// use model::{Space, Model};
    ///
    /// let space = Space::new("Bedroom");
    /// let mut model = Model::default();    
    /// model.add_space(space);
    /// assert!(model.get_space("Bedroom").is_ok());
    /// assert!(model.get_space("Walrus Enclosure").is_err());
    /// ```
    pub fn get_space<S: Into<String>>(&self, name: S) -> Result<Arc<Space>, String> {
        let name: String = name.into();
        for i in self.spaces.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Space '{}' in model", name))
    }

    /// Adds a [`Building`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Building, Model};
    ///
    /// let b = Building::new("Main Campus");
    /// let mut model = Model::default();
    /// assert!(model.buildings.is_empty());
    /// model.add_building(b);
    /// assert_eq!(model.buildings.len(), 1);
    ///
    /// // Adding a new building witht the same name warns the user but still works
    /// let b = Building::new("Main Campus");
    /// model.add_building(b);
    /// assert_eq!(model.buildings.len(), 2);    
    /// ```
    pub fn add_building(&mut self, add: Building) -> Arc<Building> {
        if self.get_building(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Building called '{}'",
                add.name()
            ))
        }
        let add = Arc::new(add);
        self.buildings.push(Arc::clone(&add));
        add
    }

    /// Retrieves a reference (`Arc`) to a [`Building`] based on its name, from the `buildings`
    /// field
    ///
    /// ```rust
    /// use model::{Building, Model};
    ///
    /// let b = Building::new("Main Campus");
    /// let mut model = Model::default();
    /// model.add_building(b);
    ///
    /// assert!(model.get_building("Main Campus").is_ok());
    /// assert!(model.get_building("Bar With Free Beer and Coffee").is_err());
    /// ```
    pub fn get_building<S: Into<String>>(&self, name: S) -> Result<Arc<Building>, String> {
        let name: String = name.into();
        for i in self.buildings.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Building '{}' in model", name))
    }

    /// Adds a [`Fenestration`] to the [`Model`]
    ///
    /// > Note: It returns a `Result` because  this method can fail.
    /// Specifically, when the [`Fenestration`] has a `parent_surface`
    /// that does not exist of it does not fit within it.
    ///
    /// ```rust
    /// use model::{Space, Model, Fenestration, SurfaceTrait};
    /// use json5;
    ///
    /// let fen  : Fenestration = json5::from_str("{
    ///     name: 'Window 1',
    ///     construction: 'Double Clear Glass',    
    ///     vertices: [
    ///         0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
    ///         0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
    ///         5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
    ///         5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
    ///     ]
    /// }").unwrap();
    ///
    /// let mut model = Model::default();
    /// assert!(model.fenestrations.is_empty());
    /// model.add_fenestration(fen);
    /// assert_eq!(model.fenestrations.len(), 1);
    ///
    /// // adding a new fenestration with the same name warns the user, but still works.
    /// let fen  : Fenestration = json5::from_str("{
    ///     name: 'Window 1',
    ///     construction: 'Double Clear Glass',
    ///     vertices: [
    ///         0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
    ///         0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
    ///         5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
    ///         5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
    ///     ]
    /// }").unwrap();
    ///
    /// model.add_fenestration(fen).unwrap(); // this returns an error, as it can fail (e.g., when a pare)
    /// assert_eq!(model.fenestrations.len(), 2);
    ///
    /// // Let's add one with a parent surface now.
    /// use model::Surface;
    ///
    /// let mut model = Model::default();
    /// model.add_space(Space::new("Space 1"));
    /// let s: Surface = json5::from_str(
    ///     "{
    ///     name: 'the surface',
    ///     construction:'the construction',
    ///     back_boundary: {
    ///         type: 'Space',
    ///         space: 'Space 1',
    ///     },    
    ///     vertices: [
    ///         0, 0, 0, // X, Y and Z of Vertex 0
    ///         1, 0, 0, // X, Y and Z of Vertex 1
    ///         1, 1, 0, // X, Y and Z of Vertex 2
    ///         0, 1, 0  // ...
    ///     ]
    ///  }").unwrap();
    /// model.add_surface(s);
    ///
    /// let fen  : Fenestration = json5::from_str("{
    ///     name: 'Window 1',
    ///     construction: 'Double Clear Glass',
    ///     parent_surface: 'the surface',
    ///     vertices: [
    ///         0.2, 0.2, 0.,
    ///         0.8, 0.2, 0.,
    ///         0.8, 0.8, 0.,
    ///         0.2, 0.8, 0.,
    ///     ]
    /// }").unwrap();
    ///
    /// model.add_fenestration(fen).unwrap();
    /// assert_eq!(model.fenestrations.len(), 1);
    /// assert!((model.fenestrations[0].area()- 0.36).abs() < 1e-6);
    /// assert_eq!(model.surfaces.len(), 1);
    /// assert!((model.surfaces[0].area() - 0.64).abs() < 1e-6);
    ///
    /// ```
    pub fn add_fenestration(&mut self, mut add: Fenestration) -> Result<Arc<Fenestration>, String> {
        if self.get_fenestration(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already a Fenestration called '{}'",
                add.name()
            ))
        }
        // Check the index of this object
        let fen_index = self.fenestrations.len();

        // Push the OpenFraction state, and map into the object
        let state_index = self.push_to_state(
            SimulationStateElement::FenestrationOpenFraction(fen_index),
            0.,
        )?;
        add.set_open_fraction_index(state_index)?;

        // check the parent surface
        let mut parent: Option<Arc<Surface>> = None;
        if let Ok(parent_name) = add.parent_surface() {
            for s in self.surfaces.iter_mut() {
                if s.name() == parent_name {
                    {
                        // scope so we can drop
                        let s = match std::sync::Arc::get_mut(s) {
                            Some(s) => s,
                            None => {
                                return Err(format!(
                                    "Could not borrow parent surface '{}' when adding surface '{}'",
                                    parent_name,
                                    add.name()
                                ));
                            }
                        };
                        // Cut a hole on the wall
                        let out = add.vertices.clone_outer();
                        s.vertices.cut_hole(out)?;
                    }
                    parent = Some(s.clone());
                    break;
                }
            }
            if parent.is_none() {
                return Err(format!(
                    "Fenestration '{}' has been given parent '{}', which does not exist",
                    add.name(),
                    add.parent_surface()?
                ));
            }
        }
        if let Some(s) = parent {
            // inherit boundaries
            let wall_normal = s.vertices.normal();
            let window_normal = add.vertices.normal();
            if wall_normal.is_same_direction(window_normal) {
                // front with front, back with back
                add.front_boundary = s.front_boundary.clone();
                add.back_boundary = s.back_boundary.clone();
            } else {
                // otherwise, they are diverted
                add.front_boundary = s.back_boundary.clone();
                add.back_boundary = s.front_boundary.clone();
            }
        }

        // Add to model, and return a reference
        let add = Arc::new(add);
        self.fenestrations.push(Arc::clone(&add));
        Ok(add)
    }

    /// Retrieves a reference (`Arc`) to a [`Fenestration`] based on its name, from the `fenestrations`
    /// field
    ///
    /// ```rust
    /// use model::{Model, Fenestration};
    /// use json5;
    ///
    /// let fen  : Fenestration = json5::from_str("{
    ///     name: 'Window 1',
    ///     construction: 'Double Clear Glass',
    ///     back_boundary: {
    ///         type: 'Space',
    ///         space: 'Space 1',
    ///     },
    ///     operation: {
    ///         type: 'Fixed',
    ///     },
    ///     category: 'Window',
    ///     vertices: [
    ///         0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
    ///         0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
    ///         5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
    ///         5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
    ///     ]
    /// }").unwrap();
    ///
    /// let mut model = Model::default();
    /// model.add_fenestration(fen);
    /// assert!(model.get_fenestration("Window 1").is_ok());
    /// assert!(model.get_fenestration("Huge window facing west that creates overheating").is_err());
    /// ```
    pub fn get_fenestration<S: Into<String>>(&self, name: S) -> Result<Arc<Fenestration>, String> {
        let name: String = name.into();
        for i in self.fenestrations.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Fenestration '{}' in model", name))
    }

    /// Adds a [`HVAC`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, hvac::ElectricHeater};
    /// let heater = ElectricHeater::new("15 dollar heater");
    /// let mut model = Model::default();
    /// assert!(model.hvacs.is_empty());
    /// model.add_hvac(heater.wrap()); // wrap it as an HVAC object
    /// assert_eq!(model.hvacs.len(), 1);
    ///
    /// // Adding a new HVAC with the same name warns the user, but works
    ///  
    /// let heater = ElectricHeater::new("15 dollar heater");
    /// model.add_hvac(heater.wrap()); // wrap it as an HVAC object
    /// assert_eq!(model.hvacs.len(), 2);
    /// ```
    pub fn add_hvac(&mut self, add: HVAC) -> Result<HVAC, String> {
        if self.get_hvac(add.name()).is_ok() {
            print_warning_no_module(format!("There is already an HVAC called '{}'", add.name()))
        }

        // Check the index of this object
        let obj_index = self.hvacs.len();
        match &add {
            HVAC::ElectricHeater(hvac) => {
                let state_index = self.push_to_state(
                    SimulationStateElement::HeatingCoolingPowerConsumption(obj_index),
                    0.,
                )?;
                hvac.set_heating_cooling_consumption_index(state_index)?;
            }
            HVAC::IdealHeaterCooler(hvac) => {
                let state_index = self.push_to_state(
                    SimulationStateElement::HeatingCoolingPowerConsumption(obj_index),
                    0.,
                )?;
                hvac.set_heating_cooling_consumption_index(state_index)?;
            }
        }

        // Add to model, and return a reference
        self.hvacs.push(add.clone());
        Ok(add)
    }

    /// Retrieves a reference (`Arc`) to a [`HVAC`] based on its name, from the `hvacs`
    /// field
    ///
    /// ```rust
    /// use model::{Model, HVAC, hvac::ElectricHeater};
    /// let heater = ElectricHeater::new("15 dollar heater");
    /// let mut model = Model::default();
    /// model.add_hvac(heater.wrap()); // wrap it as an HVAC object
    ///
    ///
    /// let hvac = model.get_hvac("15 dollar heater").unwrap(); // should be there
    /// match hvac{
    ///     HVAC::ElectricHeater(_)=>println!("All good!"),
    ///     _ => assert!(false, "Nooooo!")
    /// }
    ///
    ///
    /// assert!(model.get_hvac("15000 million fancy heating").is_err());
    /// ```
    pub fn get_hvac<S: Into<String>>(&self, name: S) -> Result<HVAC, String> {
        let name: String = name.into();
        for i in self.hvacs.iter() {
            let hvac_name = match i {
                HVAC::ElectricHeater(hvac) => hvac.name(),
                HVAC::IdealHeaterCooler(hvac) => hvac.name(),
            };

            if hvac_name == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find HVAC '{}' in model", name))
    }

    /// Adds a [`Luminaire`] to the [`Model`]
    ///
    /// ```rust
    /// use model::{Model, Luminaire};
    ///
    /// let luminaire = Luminaire::new("LED Lightbulb");
    /// let mut model = Model::default();
    /// assert!(model.luminaires.is_empty());
    /// model.add_luminaire(luminaire);
    /// assert_eq!(model.luminaires.len(), 1);
    ///
    /// // Adding a new luminaire with the same name will print a warning but still work
    /// let luminaire = Luminaire::new("LED Lightbulb");        
    /// model.add_luminaire(luminaire);
    /// assert_eq!(model.luminaires.len(), 2);    
    ///
    /// ```
    pub fn add_luminaire(&mut self, add: Luminaire) -> Result<Arc<Luminaire>, String> {
        if self.get_luminaire(add.name()).is_ok() {
            print_warning_no_module(format!(
                "There is already an Luminaire called '{}'",
                add.name()
            ))
        }
        let obj_index = self.luminaires.len();
        // Push the state, and map into the object
        let state_index = self.push_to_state(
            SimulationStateElement::LuminairePowerConsumption(obj_index),
            0.,
        )?;
        add.set_power_consumption_index(state_index)?;

        // Add to model, and return a reference
        let add = Arc::new(add);
        self.luminaires.push(Arc::clone(&add));
        Ok(add)
    }

    /// Retrieves a reference (`Arc`) to a [`Luminaire`] based on its name, from the `fenestrations`
    /// field
    ///     
    /// ```rust
    /// use model::{Model, Luminaire};
    ///
    /// let luminaire = Luminaire::new("LED Lightbulb");
    /// let mut model = Model::default();    
    /// model.add_luminaire(luminaire);
    ///
    /// assert!(model.get_luminaire("LED Lightbulb").is_ok());
    /// assert!(model.get_luminaire("Unnecessarily colourful smart lightbulb").is_err());
    /// ```
    pub fn get_luminaire<S: Into<String>>(&self, name: S) -> Result<Arc<Luminaire>, String> {
        let name: String = name.into();
        for i in self.luminaires.iter() {
            if i.name() == &name {
                return Ok(i.clone());
            }
        }
        Err(format!("Could not find Luminaire '{}' in model", name))
    }

    /// Retrieves a reference (`Arc`) to the [`Substance`] that comprises a [`Material`] called `mat_name`.
    ///
    /// It searches for the material first, and then for the substance
    ///
    /// ```rust
    /// use model::{Model, substance::Normal, Material};
    ///
    /// let mut model = Model::default();
    ///
    /// // Search for inexistent mat, is an error
    /// assert!(model.get_material_substance("Sweet Panel").is_err());
    ///
    /// let mat = Material::new("Sweet Panel", "is made of this", 0.2);
    /// model.add_material(mat);
    ///
    /// // Search for material with inexistent substance is error
    /// assert!(model.get_material_substance("Sweet Panel").is_err());
    /// // (even if the material exists)
    /// assert!(model.get_material("Sweet Panel").is_ok());
    ///
    /// // When the substance is there, it should work.
    /// let exp_sub = Normal::new("is made of this");
    /// model.add_substance(exp_sub.wrap());
    /// let found_sub = model.get_material_substance("Sweet Panel").unwrap();
    ///
    /// assert_eq!("is made of this", found_sub.name());
    /// ```    
    pub fn get_material_substance<S: Into<String>>(
        &self,
        mat_name: S,
    ) -> Result<Substance, String> {
        let mat = self.get_material(mat_name)?;
        let name = &mat.substance;
        self.get_substance(name)
    }

    /// Retrieves a dictionary with the heating/cooling setpoints of the different
    /// HVACs in the model. The dictionary will be empty if
    /// there are no HVACs. If an hvac does not have a setpoint,
    /// the content will be none.
    ///
    /// ```
    /// use model::{Model, hvac::ElectricHeater};
    /// let mut model = Model::default();
    ///
    /// let heater_name = "Heater".to_string();
    /// let mut heater = ElectricHeater::new(heater_name.clone());
    /// heater.set_heating_setpoint(19.);
    /// model.add_hvac(heater.wrap()).unwrap();
    ///
    /// let setpoints = model.get_hvac_setpoints();
    /// assert_eq!(setpoints.len(), 1);
    /// assert!( setpoints[&heater_name][0].is_some() );
    /// assert!( (19.0 - setpoints[&heater_name][0].unwrap()).abs() < 1e-9 );
    /// assert!( setpoints[&heater_name][1].is_none() );
    /// ```
    pub fn get_hvac_setpoints(&self) -> HashMap<String, [Option<Float>; 2]> {
        let mut hvac_setpoints: HashMap<String, [Option<Float>; 2]> =
            HashMap::with_capacity(self.hvacs.len());
        for hvac in self.hvacs.iter() {
            let (name, heating, cooling) = match hvac {
                HVAC::ElectricHeater(heater) => {
                    let heating = match heater.heating_setpoint() {
                        Ok(v) => Some(*v),
                        Err(_) => None,
                    };
                    (hvac.name().clone(), heating, None)
                }
                HVAC::IdealHeaterCooler(h) => {
                    let heating = match h.heating_setpoint() {
                        Ok(v) => Some(*v),
                        Err(_) => None,
                    };
                    let cooling = match h.cooling_setpoint() {
                        Ok(v) => Some(*v),
                        Err(_) => None,
                    };

                    (hvac.name().clone(), heating, cooling)
                }
            };
            hvac_setpoints.insert(name, [heating, cooling]);
        }
        hvac_setpoints
    }

    /// Calculates the total floor area of the model, and the floor areas on each space
    /// within it.
    ///
    /// # NOTE:
    ///
    /// * It does not separate by building or anything
    /// * It only accounts for surfaces labelled as
    ///
    /// ```
    /// use model::Model;
    /// let mut model = Model::default();
    /// let (total_area, areas) = model.get_space_sizes();
    /// ```
    pub fn get_space_sizes(&self) -> (Float, HashMap<String, Float>) {
        let mut total_area = 0.0;

        let mut floor_areas = HashMap::with_capacity(self.spaces.len());
        self.spaces.iter().for_each(|s| {
            floor_areas.insert(s.name().clone(), 0.0);
        });

        fn get_space(s: &Arc<Surface>) -> Option<String> {
            let normal = s.normal();
            if normal.z > 0.0 {
                // Points Up... check front
                if let Boundary::Space { space } = &s.front_boundary {
                    return Some(space.clone());
                }
            } else {
                // Points down... check back
                if let Boundary::Space { space } = &s.back_boundary {
                    return Some(space.clone());
                }
            }
            None
        }

        for s in self.surfaces.iter() {
            if let (Ok(cat), Some(space)) = (s.category(), get_space(s)) {
                let area = s.area();
                match cat {
                    SurfaceType::GroundFloor => {
                        total_area += area;
                        let k = floor_areas
                            .get_mut(&space)
                            .unwrap_or_else(|| panic!("Unexpected space {}", space));
                        *k += area;
                    }
                    SurfaceType::InteriorFloor => {
                        total_area += area;
                        let k = floor_areas
                            .get_mut(&space)
                            .unwrap_or_else(|| panic!("Unexpected space {}", space));
                        *k += area;
                    }
                    SurfaceType::ExteriorFloor => {
                        total_area += area;
                        let k = floor_areas
                            .get_mut(&space)
                            .unwrap_or_else(|| panic!("Unexpected space {}", space));
                        *k += area;
                    }
                    _ => {} // ignore
                }
            }
        }

        (total_area, floor_areas)
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {

    use super::*;

    use crate::substance::Normal;

    #[test]
    fn serde() -> Result<(), String> {
        // test simple
        let (model, state) = Model::from_file("./tests/box.spl")?;
        assert_eq!(2, model.substances.len());
        assert_eq!(2, model.materials.len());
        assert_eq!(1, model.spaces.len());
        assert_eq!(1, model.outputs.len());
        assert_eq!(1, model.surfaces.len());
        assert!(model.solar_options.is_some());

        let model_str = serde_json::to_string(&model).map_err(|e| e.to_string())?;

        let mut model: Model = serde_json::from_str(&model_str).map_err(|e| e.to_string())?;

        // use std::fs::File;
        // use std::io::Write;
        // let mut file = File::create("./model.json").map_err(|e| e.to_string())?;
        // // Write a &str in the file (ignoring the result).
        // writeln!(&mut file, "{}", serde_json::to_string(&model).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;

        let other_state = model.take_state().ok_or("Could not take state")?;
        assert_eq!(2, model.substances.len());
        assert_eq!(2, model.materials.len());
        assert_eq!(1, model.spaces.len());
        assert_eq!(1, model.outputs.len());
        assert_eq!(1, model.surfaces.len());
        assert!(model.solar_options.is_some());

        assert_eq!(state.len(), other_state.len());

        Ok(())
    }

    #[cfg(debug_assertions)]
    #[test]
    fn write_io_doc() -> Result<(), String> {
        use crate::boundary::Boundary;
        use crate::building::Building;
        use crate::fenestration::{FenestrationPosition, FenestrationType};
        use crate::substance;
        use crate::Output;
        use crate::ShelterClass;
        use crate::SolarOptions;
        use crate::TerrainClass;
        use crate::{
            hvac, ChairArmType, ChairBackType, ChairLegType, ChairType, Object, ObjectSpecs,
            SofaType, SpacePurpose, StorageType, TableShape, TableType,
        };

        let dir = "../docs/ioreference/src";

        let summary_template = format!("{}/SUMMARY_TEMPLATE.md", dir);
        if !std::path::Path::new(&summary_template).exists() {
            return Err(format!("File '{}' already exist", &summary_template));
        }

        let mut summary = std::fs::read_to_string(summary_template).map_err(|e| e.to_string())?;

        // Add automatic documentation
        // let dir = "../src";
        let summary_file = format!("{}/SUMMARY.md", dir);

        // clear summary
        let f = std::fs::File::create(&summary_file).map_err(|e| e.to_string())?;
        f.set_len(0).map_err(|e| e.to_string())?;

        /////////////////////

        /*****/
        /* A */
        /*****/

        /*****/
        /* B */
        /*****/
        // Boundary
        Boundary::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        Building::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* C */
        /*****/
        ChairArmType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        ChairBackType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        ChairLegType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        ChairType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Construction::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* D */
        /*****/

        /*****/
        /* E */
        /*****/

        /*****/
        /* F */
        /*****/
        Fenestration::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Fenestration::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        FenestrationPosition::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        FenestrationType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* G */
        /*****/

        /*****/
        /* H */
        /*****/
        HVAC::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        summary.push_str(&format!("\t"));
        hvac::ElectricHeater::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        hvac::ElectricHeater::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        summary.push_str(&format!("\t"));
        hvac::IdealHeaterCooler::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        hvac::IdealHeaterCooler::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* I */
        /*****/
        crate::infiltration::Infiltration::print_doc(&dir, &mut summary)
            .map_err(|e| e.to_string())?;

        /*****/
        /* J */
        /*****/

        /*****/
        /* K */
        /*****/

        /*****/
        /* L */
        /*****/
        Luminaire::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Luminaire::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* M */
        /*****/
        Material::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* N */
        /*****/

        /*****/
        /* O */
        /*****/
        Object::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;
        ObjectSpecs::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;
        Output::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* P */
        /*****/

        /*****/
        /* Q */
        /*****/

        /*****/
        /* R */
        /*****/

        /*****/
        /* S */
        /*****/
        SiteDetails::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;
        SofaType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        SolarOptions::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Space::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Space::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        SpacePurpose::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        StorageType::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;

        Substance::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        summary.push_str(&format!("\t"));
        substance::Normal::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        summary.push_str(&format!("\t"));
        substance::Gas::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        summary.push_str(&format!("\t"));
        substance::gas::GasSpecification::print_doc(dir, &mut summary)
            .map_err(|e| e.to_string())?;

        ShelterClass::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        Surface::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        Surface::print_api_doc(&dir, &mut summary).map_err(|e| e.to_string())?;
        crate::surface::SurfaceType::print_doc(&dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* T */
        /*****/
        TableShape::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;
        TableType::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;
        TerrainClass::print_doc(dir, &mut summary).map_err(|e| e.to_string())?;

        /*****/
        /* U */
        /*****/

        /*****/
        /* V */
        /*****/

        /*****/
        /* W */
        /*****/

        /*****/
        /* X */
        /*****/

        /*****/
        /* Y */
        /*****/

        /*****/
        /* Z */
        /*****/

        /////////////////

        let current_summary =
            fs::read_to_string(summary_file.clone()).expect("Could not read summary file");
        let whole_summary = format!("{}\n\n{}", current_summary, summary);
        std::fs::write(summary_file, whole_summary.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn test_geolocation() -> Result<(), String> {
        // Successful workflow

        let json_str = r#"{
            "site_details": {
                "latitude": 1.2,
                "longitude" : 5.21,
                "standard_meridian": 123.1
            }
        }"#;

        let (model, _header) = Model::from_json(&json_str)?;
        assert_eq!(Some((1.2, 5.21, 123.1)), model.geolocation());

        // No site details
        let json_str = r#"{
            "buildings": [{
                "name": "The Building",
                "shelter_class" : "Urban"
            }]
        }"#;

        let (model, _header) = Model::from_json(&json_str)?;
        assert!(model.geolocation().is_none());

        // No longitude
        let json_str = r#"{
            "site_details": {
                "latitude": 1.2,
                "standard_meridian": 123
            }
        }"#;

        let (model, _header) = Model::from_json(&json_str)?;
        assert!(model.geolocation().is_none());

        // No latitude
        let json_str = r#"{
            "site_details": {
                "longitude": 1.2,
                "standard_meridian": 123
            }
        }"#;

        let (model, _header) = Model::from_json(&json_str)?;
        assert!(model.geolocation().is_none());

        // No standard_meridian
        let json_str = r#"{
            "site_details": {
                "longitude": 1.2,
                "latitude": 123
            }
        }"#;

        let (model, _header) = Model::from_json(&json_str)?;
        assert!(model.geolocation().is_none());

        Ok(())
    }

    #[test]
    fn building_substance() {
        let mut building = Model::default();

        let subs_name = "Substance 0".to_string();
        let substance = Normal::new(subs_name.clone()).wrap();

        let s0 = building.add_substance(substance);

        let s = &building.substances[0];
        assert_eq!(subs_name, s.name().clone());
        assert_eq!(subs_name, s0.name().clone());

        #[allow(irrefutable_let_patterns)]
        if let Substance::Normal(s) = &s0 {
            assert_eq!(&subs_name, s.name());
        } else {
            assert!(false, "asd")
        }
    }

    #[test]
    fn building_hvac() -> Result<(), String> {
        let mut building = Model::default();

        let heater_name = "Heater".to_string();
        let heater = ElectricHeater::new(heater_name.clone());

        let h0 = building.add_hvac(heater.wrap())?;

        if let HVAC::ElectricHeater(h) = h0 {
            assert_eq!(heater_name, h.name);
        }

        if let HVAC::ElectricHeater(h) = &building.hvacs[0] {
            assert_eq!(heater_name, h.name);
        }

        Ok(())
    }

    #[test]
    fn test_add_fenestration() -> Result<(), String> {
        let mut model = Model::default();

        model.add_space(Space::new("Space 1"));
        model.add_space(Space::new("Space 2"));


        let s: Surface = json5::from_str(
            "{
            name: 'the surface',
            construction:'the construction',
            back_boundary: {
                type: 'Space',
                space: 'Space 1',
            },
            front_boundary: {
                type: 'Space',
                space: 'Space 2',
            },    
            vertices: [
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ...
            ]
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s)?;

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 1',
            construction: 'Double Clear Glass',
            parent_surface: 'the surface',
            vertices: [
                0.2, 0.2, 0.,
                0.8, 0.2, 0.,
                0.8, 0.8, 0.,
                0.2, 0.8, 0.,
            ]
        }",
        )
        .map_err(|e| e.to_string())?;

        
        model.add_fenestration(fen)?;

        let fen = model.fenestrations[0].clone();
        let s = model.surfaces[0].clone();
        assert_eq!(model.fenestrations.len(), 1);
        assert!((model.fenestrations[0].area() - 0.36).abs() < 1e-6);
        assert_eq!(model.surfaces.len(), 1);
        assert!((model.surfaces[0].area() - 0.64).abs() < 1e-6);

        assert_eq!(
            format!("{:?}", fen.front_boundary),
            format!("{:?}", s.front_boundary)
        );
        assert_eq!(
            format!("{:?}", fen.back_boundary),
            format!("{:?}", s.back_boundary)
        );

        Ok(())
    }

    #[test]
    fn test_add_fenestration_cross_boundary() -> Result<(), String> {
        let mut model = Model::default();
        model.add_space(Space::new("Space 1"));
        model.add_space(Space::new("Space 2"));
        let s: Surface = json5::from_str(
            "{
            name: 'the surface',
            construction:'the construction',
            back_boundary: {
                type: 'Space',
                space: 'Space 1',
            },
            front_boundary: {
                type: 'Space',
                space: 'Space 2',
            },    
            vertices: [
                0, 0, 0, 
                0, 1, 0,
                1, 1, 0, 
                1, 0, 0, 
            ]
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s)?;

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 1',
            construction: 'Double Clear Glass',
            parent_surface: 'the surface',
            vertices: [
                0.2, 0.2, 0.,
                0.8, 0.2, 0.,
                0.8, 0.8, 0.,
                0.2, 0.8, 0.,
            ]
        }",
        )
        .map_err(|e| e.to_string())?;

        model.add_fenestration(fen)?;

        let fen = model.fenestrations[0].clone();
        let s = model.surfaces[0].clone();
        assert_eq!(model.fenestrations.len(), 1);
        assert!((model.fenestrations[0].area() - 0.36).abs() < 1e-6);
        assert_eq!(model.surfaces.len(), 1);
        assert!((model.surfaces[0].area() - 0.64).abs() < 1e-6);
        assert_eq!(
            format!("{:?}", fen.back_boundary),
            format!("{:?}", s.front_boundary)
        );
        assert_eq!(
            format!("{:?}", fen.front_boundary),
            format!("{:?}", s.back_boundary)
        );

        Ok(())
    }

    use crate::simulation_state::SimulationStateHeader;

    use crate::rhai_api::*;
    use crate::simulation_state_element::SimulationStateElement;
    use std::cell::RefCell;

    #[test]
    fn test_api() -> Result<(), String> {
        let mut model = Model::default();
        let mut state_header = SimulationStateHeader::new();

        let electric = ElectricHeater::new("electric heater".to_string());
        let electric = model.add_hvac(electric.wrap())?;
        let ideal = IdealHeaterCooler::new("ideal hvac".to_string());
        let ideal = model.add_hvac(ideal.wrap())?;

        let space = Space::new("some space".to_string());
        let state_index =
            state_header.push(SimulationStateElement::SpaceInfiltrationVolume(0), 2.1)?;
        space.set_infiltration_volume_index(state_index)?;
        let state_index =
            state_header.push(SimulationStateElement::SpaceDryBulbTemperature(0), 22.2)?;
        space.set_dry_bulb_temperature_index(state_index)?;
        model.add_space(space);

        let mut state = state_header.take_values().ok_or("Could not get values")?;

        if let HVAC::ElectricHeater(hvac) = electric {
            hvac.set_heating_cooling_consumption(&mut state, 91.2)?;
        }

        if let HVAC::IdealHeaterCooler(hvac) = ideal {
            hvac.set_heating_cooling_consumption(&mut state, 23.14)?;
        }

        // Wrap and send to the Heap
        let state = Arc::new(RefCell::new(state));
        let model = Arc::new(model);
        let mut engine = rhai::Engine::new();

        register_control_api(&mut engine, &model, &state, true);

        let ast = engine
            .compile(
                "
            
            let some_space = space(\"some space\");
            let vol = some_space.infiltration_volume;
            print(`Infiltration volume is ${vol} `);
            some_space.infiltration_volume = 3.1415;
            let vol = some_space.infiltration_volume;
            print(`Infiltration volume is ${vol} `);

            let vol = space(0).infiltration_volume;
            print(`Infiltration volume is ${vol} `);

            print(\"NEXT ---->\");

            

            // Electric
            let electric = hvac(\"electric heater\");
            let power = electric.power_consumption;
            print(`Electric power consumption is ${power} W`);
            electric.power_consumption = 99.1;
            let power = electric.power_consumption;
            print(`Electric power consumption is ${power} W`);

            // Ideal
            let ideal = hvac(\"ideal hvac\");
            let power = ideal.power_consumption;
            print(`Ideal power consumption is ${power} W`);
            ideal.power_consumption = 912.1;
            let power = ideal.power_consumption;
            print(`Ideal power consumption is ${power} W`);

            print(\"BY INDEX NOW ---->\");

            

            // Electric
            let electric = hvac(0);
            let power = electric.power_consumption;
            print(`Electric power consumption is ${power} W`);
            
            // Ideal
            let ideal = hvac(1);
            let power = ideal.power_consumption;
            print(`Ideal power consumption is ${power} W`);
            

            // Temperature
            let the_space = space(\"some space\");
            let temp = the_space.dry_bulb_temperature;
            print(`Temp is ${temp}`)            
            
        ",
            )
            .map_err(|e| e.to_string())?;

        let _result: () = engine.eval_ast(&ast).map_err(|e| e.to_string())?;

        Ok(())
    }

    #[test]
    fn test_get_space_sizes() -> Result<(), String> {
        let (model, _) = Model::from_file("./tests/cold_wellington_apartment.spl")?;
        let (total_area, areas) = model.get_space_sizes();

        fn check_close(a: Float, b: Float) -> Result<(), String> {
            if (a - b).abs() > 1e-1 {
                return Err(format!("{} and {} are too different", a, b));
            }
            Ok(())
        }

        check_close(total_area, 61.839)?;

        check_close(areas["Kids Bedroom"], 7.12)?;
        check_close(areas["Bathroom"], 3.877)?;
        check_close(areas["Storage"], 1.12)?;
        check_close(areas["Kitchen"], 5.6918)?;
        check_close(areas["Laundry"], 1.7056)?;
        check_close(areas["Livingroom"], 18.76)?;
        check_close(areas["Main Bedroom"], 17.765)?;
        check_close(areas["Hallway"], 5.884)?;

        Ok(())
    }
}
