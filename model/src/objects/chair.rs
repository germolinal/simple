use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Type of chair
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum ChairType {
    /// Other
    #[default]
    Other,

    /// Dining
    Dining,

    /// Office
    Office,

    /// Stool
    Stool,
}

/// Types of armchair
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum ChairArmType {
    /// Has arms
    Existing,

    /// Does not have arms
    #[default]
    Missing,
}

/// Types of chair back
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum ChairBackType {
    /// Has arms
    Existing,

    /// Does not have arms
    #[default]
    Missing,
}

/// Types of legs in a chair
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum ChairLegType {
    /// Four legs    
    Four,

    /// Three legs
    Three,

    /// Star
    Star,

    /// Other
    #[default]
    Other,
}
