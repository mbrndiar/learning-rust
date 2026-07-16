//! Injectable versioned-index persistence boundary.

use crate::{IndexData, IndexError};

/// Loads and atomically replaces complete index values.
pub trait IndexStore {
    fn load(&self) -> Result<IndexData, IndexError>;
    fn replace(&self, index: &IndexData) -> Result<(), IndexError>;
}
