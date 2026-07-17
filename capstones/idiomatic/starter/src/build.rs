//! Milestone 4: bounded worker ownership and cancellation.
//!
//! This is the concurrency core. Discovery should be single-threaded and ordered;
//! reading and tokenizing files is fanned out to a bounded pool, then the results
//! are put back into a canonical order before anything is trusted. The key idea
//! that makes a parallel build reproducible: **document ids are assigned last**.
//! Workers may finish in any order, so `build` must collect every document, sort by
//! `(root order, path)`, and only then number them `1..=N`. A shared
//! [`Cancellation`] token lets any fatal condition stop the pool promptly, and
//! every spawned worker must be joined before `build` returns, even on error paths.

use crate::{FileTree, IndexData, IndexError, IndexSettings, RootSpec};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Cloneable cancellation behavior injected into an index build.
///
/// Cloning must share one underlying flag so a `cancel` on any handle is visible to
/// every worker; `CancellationToken` does this with an `Arc<AtomicBool>`.
pub trait Cancellation: Clone + Send + Sync + 'static {
    /// Requests cancellation; subsequent `is_cancelled` calls return `true`.
    fn cancel(&self);
    /// Reports whether cancellation has been requested.
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
///
/// `F` is the [`FileTree`] seam and `C` the [`Cancellation`] token; keeping both
/// generic lets tests drive the exact same orchestration with fakes.
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
