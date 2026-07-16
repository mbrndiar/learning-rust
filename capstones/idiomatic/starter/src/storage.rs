//! Milestone 3: validated publish-after-complete persistence.

use crate::{IndexData, IndexError};
use std::path::{Path, PathBuf};

/// Loads and atomically replaces complete index values.
pub trait IndexStore {
    fn load(&self) -> Result<IndexData, IndexError>;
    fn replace(&self, index: &IndexData) -> Result<(), IndexError>;
}

/// Single-writer JSON index stored at one host path.
#[derive(Debug, Clone)]
pub struct JsonFileIndexStore {
    path: PathBuf,
}

impl JsonFileIndexStore {
    /// Creates a store for `path` without reading it.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the exact host path supplied to the store.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl IndexStore for JsonFileIndexStore {
    fn load(&self) -> Result<IndexData, IndexError> {
        todo!("milestone 3: load and validate the versioned JSON index")
    }

    fn replace(&self, _index: &IndexData) -> Result<(), IndexError> {
        todo!("milestone 3: atomically replace with a complete candidate")
    }
}
