use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Shape of a table
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum TableShape {
    /// Other kind of table
    #[default]
    Other,

    /// Rectangular
    Rectangular,

    /// Circular
    Circular,

    /// L-Shaped
    LShaped,
}

/// Type of a table
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum TableType {
    /// Other kind of table
    #[default]
    Other,

    /// Dining table
    Dining,

    /// Cofee table
    Coffee,
}
