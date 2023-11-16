use derive::ObjectIO;
use serde::{Deserialize, Serialize};


/// Types of armchair
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum SofaType {
    /// Other
    #[default]
    Other,

    /// Rectangular
    Rectangular,

    /// Single Seat
    SingleSeat,

    /// L-Shaped
    LShaped,

    /// L-Shaped Extension
    LShapedExtension,    
}