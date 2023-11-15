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

#![deny(missing_docs)]

//! This crate contains the data structure utilized for describing
//! a `SIMPLE` building.
//!
//! This structure and its API are built on top of an
//! architecture developed during [Germán Molina's PhD dissertation](https://openaccess.wgtn.ac.nz/articles/thesis/Exploring_modelling_and_simulating_the_Feeling_of_Comfort_in_residential_settings/17085467/1).
//! The purpose of this architecture is to keep the different physical domains of a building—e.g., thermal,
//! visual, air flows, moisture—separated in the code but integrated at runtime. This
//! allows for multi-physics building simulation and a couple of more interesting things
//! while keeping the code maintainable and scalable.
//!
//!
//! # Introduction
//!
//! It contains all the supported elements of a building and methods
//! for manipulating them.
//!
//! This library includes a macro for generating automatic
//! [user documentation](https://simple-buildingsimulation.github.io/model/ioreference/book/html/index.html)
//! and a highly consistent [RUST API](https://simple-buildingsimulation.github.io/model/rustdoc/doc/model/index.html).
//! This macro also enables a [`scanner`] to produce functions for parsing a model from text files.
//!
//! Similarly, it includes a [`rhai_api`]—i.e., the Control API—that allows manipulating certain elements of
//! the building through portable scripts, allowing users to emulate automatic control or
//! occupant behaviour.
//!
//! # Architecture of `SIMPLE` and the model and Simulation Engine
//!
//! The basics of the architecture is that the building simulation is performed through different
//! simulation modules, coresponding to the different domains to simulate (e.g., thermal, visual, etc.).
//! However—because these domains tend to use different abstractions, methods, math and so on—integrating
//! them in a maintainable way can be tricky. The architecture presented here attempts to mitigate this issue
//! by keeping the domains separated in the code but tightly communicated at runtime.
//!
//! Communication is achieved through the [`Model`] structure and its associated `simulation_state`.
//! Specifically, after its creation, the [`Model`] and the simulation modules themselves all remains
//! immutable, and all the changes are applied over the `simulation_state`. This implies that people
//! developing different modules need to only understand a single API: This one. Also, they can retrieve
//! information produced by other domains of the simulation by simply asking for it.
//!
//! # Example
//!
//! ```
//! use model::{Model, Space, SimulationStateHeader, SimulationStateElement};
//!
//! /* INITIALIZE MODEL AND STATE */
//! // The model is simple
//! let mut model = Model::default();
//! // Internally, the State has a header (the element to which the data in it belongs)
//! // and its value. We will decouple them later.
//! let mut header = SimulationStateHeader::new();
//!
//! /* POPULATE THE MODEL */
//! // Let's create a bedroom.
//! let mut bedroom = Space::new("my_bedroom");
//!
//! // Let's put the bedroom in a model. This adds an index of the bedroom object, which
//! // we will use later
//! // (This also wraps the bedroom on an `Rc` and returns a clone of it)
//! let bedroom = model.add_space(bedroom);
//!
//! /*  MAP THE STATE OF THE ROOM
//!    (whis will be done by SimulationModules, as they know what is needed; e.g., temperature or air speed)
//! */
//! // We need to submit a SimulationStateElement associated to the SpaceDryBulbTemperature.
//! // We choose an initial value to 22.0 C
//! // Note that the space is identified through its index.
//! let space_index = 0; // We know this... it was the first to be added.
//! let state_index = header.push(SimulationStateElement::SpaceDryBulbTemperature(space_index), 22.).unwrap();
//! // Then we need to tell the space which index in the SimulationState contains its DryBulb temperature.
//! bedroom.set_dry_bulb_temperature_index(state_index);
//!
//! // After building the model, we decouple the header of the simulation state
//! // (i.e., long and fancy names) from its data. The data is just a compact,
//! // memory-efficient and cool Vector of floating point numbers that will be passed
//! // around simulation modules. This structure is cheaper to read, write and move around.
//! let mut sim_state = header.take_values().unwrap();
//!
//! /* EMULATE A FANCY THERMAL CALCULATION */
//!
//! // Let's assume that—through a fancy calculation—we concluded that
//! // the current temperature is 24.0
//! bedroom.set_dry_bulb_temperature(&mut sim_state, 24.);
//!
//! /* RETREIVE THE DATA FROM A DIFFERENT MODULE */
//! // Let's assume we need the dry bulb temperature in order to calculate the number
//! // of flies that might be in this room, bothering occupants... As biologists, we know
//! // nothing about room heat transfer. Fortunately, this has been calculated already
//! assert!( (24.0 - bedroom.dry_bulb_temperature(&sim_state).unwrap() ).abs() < 1e-5 );
//!
//!
//! ```

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

#[cfg(not(feature = "float"))]
type Float = f64;

/// The module in charge of registering the control API.
///
/// This API is produced automatically through the derive macro included in
/// this library.
pub mod rhai_api;

/// The module containing the functions that allow parsing a Model from text files
pub mod scanner;

/// Contains the structure that has all the data that changes throughout the simulation
mod simulation_state;
pub use simulation_state::{SimulationState, SimulationStateHeader};

/// Contains all the possible elements that can be registered in the simulation state (e.g.,
/// Space Dry Bulb Temperature, Fenestration Solar Irradiance, etc.)
mod simulation_state_element;
pub use simulation_state_element::SimulationStateElement;

/// The model itself
mod model;
pub use crate::model::Model;

/// A Building Object that can conain spaces.
mod building;
pub use building::{Building, ShelterClass};

/// A construction; i.e., a set of materials ordered from Front to Back
mod construction;
pub use construction::Construction;

/// A material; i.e., a Substance with a certain thickness
mod material;
pub use material::Material;

/// A physical substance with physical—i.e., optical, thermal—properties.
pub mod substance;
pub use substance::Substance;

/// Represents the boundary of a [`Surface`] (e.g. it can lead to the ground, outdoors or a space)
mod boundary;
pub use boundary::Boundary;

/// A surface that can potentially be opened and closed.
mod fenestration;
pub use fenestration::{Fenestration, FenestrationPosition, FenestrationType};

/// A fixed (i.e., not movable) surface in the building (or surroundings). This can be of
/// any Construction, transparent or not.
mod surface;
pub use surface::{Surface, SurfaceType};

/// Some details of the site in which the building(s) is located
mod site_details;
pub use site_details::{SiteDetails, TerrainClass};

/// A Luminaire
mod luminaire;
pub use luminaire::Luminaire;

/// The module for requesting Outputs
// mod output;
pub use simulation_state_element::Output;

/// Represents a space within a building. This will
/// often be a room, but it might also be half a room
mod space;
pub use space::{Space, SpaceCategory};

/// An infiltration rate for a `Space`
mod infiltration;
pub use infiltration::Infiltration;

/// A collection of elements heating and cooling systems
pub mod hvac;
pub use hvac::{HVAC, SmallHVAC};

/// For setting options in simulations
pub mod simulation_options;
pub use simulation_options::SolarOptions;

/// For printing warning and error messages to the user
pub mod error_msgs;
pub use error_msgs::{print_error, print_warning};

///
pub mod surface_trait;
pub use surface_trait::{get_orientation, Orientation, SurfaceTrait};
