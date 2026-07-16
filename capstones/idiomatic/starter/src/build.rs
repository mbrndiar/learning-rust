//! Index-building orchestration seam.

use crate::{FileTree, IndexData, IndexError, IndexSettings, RootSpec};
use std::num::NonZeroUsize;

/// Cloneable cancellation behavior injected into an index build.
pub trait Cancellation: Clone + Send + Sync + 'static {
    fn cancel(&self);
    fn is_cancelled(&self) -> bool;
}

/// Builder parameterized by file access and cancellation capabilities.
pub struct IndexBuilder<F, C> {
    tree: F,
    workers: NonZeroUsize,
    cancellation: C,
}

impl<F: FileTree, C: Cancellation> IndexBuilder<F, C> {
    /// Creates a builder with an explicit positive worker count.
    pub const fn new(tree: F, workers: NonZeroUsize, cancellation: C) -> Self {
        Self {
            tree,
            workers,
            cancellation,
        }
    }

    /// Builds a complete deterministic index.
    pub fn build(
        &self,
        roots: &[RootSpec],
        settings: &IndexSettings,
    ) -> Result<IndexData, IndexError> {
        let _ = (
            &self.tree,
            self.workers,
            &self.cancellation,
            roots,
            settings,
        );
        Err(IndexError::incomplete("index building"))
    }
}
