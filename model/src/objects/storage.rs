use derive::ObjectIO;
use serde::{Deserialize, Serialize};

/// Types of storage
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, ObjectIO)]
#[inline_enum]
pub enum StorageType {
    /// Enclosed storage
    #[default]
    Cabinet,

    /// Open storage (i.e., shelves)
    Shelf,
}
