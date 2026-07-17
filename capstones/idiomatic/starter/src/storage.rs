//! Milestone 3: validated publish-after-complete persistence.
//!
//! Writing should follow a candidate-then-rename discipline: serialize the full
//! index, write it to a temporary file *in the destination directory*, flush, and
//! only then rename over the destination so readers never observe a half-written
//! file; a failure in any phase must leave an existing valid index untouched. This
//! is a single-writer design and does not promise crash durability or coordination
//! between concurrent writers. `load` must fail closed on any invariant violation,
//! not just JSON syntax.

use crate::{IndexData, IndexError};
use std::path::{Path, PathBuf};

/// Loads and atomically replaces complete index values.
pub trait IndexStore {
    /// Reads, version-checks, and fully revalidates the persisted index.
    fn load(&self) -> Result<IndexData, IndexError>;
    /// Publishes a complete replacement index, or leaves the old one unchanged.
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
