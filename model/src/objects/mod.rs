use derive::ObjectIO;
use geometry::{Point3D, Vector3D};
use serde::{Deserialize, Serialize};

/// Chair specifications
pub mod chair;

/// Sofa specifications
pub mod sofa;

/// Storage specifications
pub mod storage;

/// Table specifications
pub mod table;

use chair::{ChairArmType, ChairBackType, ChairLegType, ChairType};
use sofa::SofaType;
use storage::StorageType;
use table::{TableShape, TableType};

/// An object category
#[derive(Serialize, Deserialize, Debug, ObjectIO, Clone, Default)]
#[serde(tag = "type")]
pub enum ObjectSpecs {
    /// Other category
    #[default]
    Other,

    /// Bathtub
    Bathtub,

    /// Bed
    Bed,

    /// Chair
    Chair {
        /// The category of the chair
        #[serde(rename = "subtype")]
        #[serde(default)]
        category: ChairType,

        /// The kind of arms
        #[serde(default)]
        arms: ChairArmType,

        /// The kind of back
        #[serde(default)]
        back: ChairBackType,

        /// The kind of legs
        #[serde(default)]
        legs: ChairLegType,
    },

    /// Dishwasher
    Dishwasher,

    /// Fireplace
    Fireplace,

    /// Oven
    Oven,

    /// Refrigerator
    Refrigerator,

    /// Sink
    Sink,

    /// Sofa
    Sofa {
        /// The category of the storage
        #[serde(rename = "subtype")]
        #[serde(default)]
        category: SofaType,
    },

    /// Stairs
    Stairs,

    /// Storage
    Storage {
        /// The category of the storage
        #[serde(rename = "subtype")]
        #[serde(default)]
        category: StorageType,
    },

    /// Stove
    Stove,

    /// Table
    Table {
        /// The category of the storage
        #[serde(rename = "subtype")]
        #[serde(default)]
        category: TableType,

        /// The shape of the table
        #[serde(default)]
        shape: TableShape,
    },

    /// Television
    Television,

    /// Toilet
    Toilet,

    /// Washer Dryer
    WasherDryer,
}

/// An object
#[derive(Serialize, Deserialize, Debug, ObjectIO, Clone)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// The name of the object
    pub name: String,

    /// The size of the object (x, y, z)
    pub dimensions: Point3D,

    /// The location of the center of the objeect
    #[serde(default)]
    pub location: Point3D,

    /// The up
    #[serde(default = "Vector3D::z")]
    pub up: Vector3D,

    /// Front
    #[serde(default = "Vector3D::y")]
    pub front: Vector3D,

    /// The specification of the object
    #[serde(default)]
    pub specifications: ObjectSpecs,

    /// The space in which the object is located
    pub space: Option<String>,
}

impl Object {
    /// Gets the name of the object type
    pub fn as_str(&self) -> &str {
        match &self.specifications {
            ObjectSpecs::Other => "Other",     //0
            ObjectSpecs::Bathtub => "Bathtub", // 1
            ObjectSpecs::Bed => {
                // 2
                if self.dimensions.x < 1.1 {
                    "Single Bed" // 2.1
                } else {
                    "Double Bed" // 2.2
                }
            }
            ObjectSpecs::Chair { category, .. } => {
                // 3
                match category {
                    ChairType::Other => "Generic Chair", // 3.1
                    ChairType::Dining => "Dining Chair", // 3.2
                    ChairType::Office => "Office Chair", // 3.3
                    ChairType::Stool => "Stool",         // 3.4
                }
            }
            ObjectSpecs::Dishwasher => "Dishwasher",     // 4
            ObjectSpecs::Fireplace => "Fireplace",       // 5
            ObjectSpecs::Oven => "Oven",                 // 6
            ObjectSpecs::Refrigerator => "Refrigerator", // 7
            ObjectSpecs::Sink => "Sink",                 // 8
            ObjectSpecs::Sofa { category, .. } => {
                // 9
                match category {
                    SofaType::Other => "Generic Sofa",                 // 9.1
                    SofaType::Rectangular => "Rectangular Sofa",       // 9.2
                    SofaType::SingleSeat => "Single Seat Sofa",        // 9.3
                    SofaType::LShaped => "LShaped Sofa",               // 9.4
                    SofaType::LShapedExtension => "LShaped Extension", // 9.5
                }
            }
            ObjectSpecs::Stairs => "Stairs", // 10
            ObjectSpecs::Storage { category, .. } => {
                match category {
                    StorageType::Cabinet => "Cabinet", // 11.1
                    StorageType::Shelf => "Shelf",     // 11.2
                }
            }
            ObjectSpecs::Stove => "Stove", // 12
            ObjectSpecs::Table { category, .. } => {
                match category {
                    TableType::Other => "Generic Table", // 13.1
                    TableType::Dining => "Dining Table", // 13.2
                    TableType::Coffee => "Coffee Table", // 13.3
                }
            }
            ObjectSpecs::Television => "Television",    // 14
            ObjectSpecs::Toilet => "Toilet",            // 15
            ObjectSpecs::WasherDryer => "Washer Dryer", // 16
        }
    }
}

impl std::default::Default for Object {
    fn default() -> Self {
        Self {
            name: String::default(),
            dimensions: Point3D::default(),
            location: Point3D::default(),
            up: Vector3D::z(),
            front: Vector3D::y(),
            specifications: ObjectSpecs::default(),
            space: None,
        }
    }
}

#[cfg(test)]
mod testing {

    use super::*;

    #[test]
    fn basic() {
        let input = r#"{
            "name": "3280A239-E0F4-47AC-8C74-370EA25A889E",
            "dimensions": [
                0.949104,
                0.8469235,
                0.53894794
            ],
            "up":[0, 0.2, 0.3],
            "specifications":{
                "type": "Chair"
            }            
        }"#;
        let a: Object = serde_json::from_str(input).unwrap();
        println!("{}", serde_json::to_string(&a).unwrap());
    }
}
