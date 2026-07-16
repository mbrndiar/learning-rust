//! Milestone 4: bounded worker ownership and cancellation.

use crate::{FileTree, IndexData, IndexError, IndexSettings, RootSpec};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Cloneable cancellation behavior injected into an index build.
pub trait Cancellation: Clone + Send + Sync + 'static {
    fn cancel(&self);
    fn is_cancelled(&self) -> bool;
}

/// Atomic cancellation token suitable for production and tests.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates an uncancelled token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cancellation for CancellationToken {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
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

    /// Builds a complete deterministic index and joins every started worker.
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
        todo!("milestone 4: schedule bounded jobs, cancel, drain, and join")
    }
}
