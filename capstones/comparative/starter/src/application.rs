//! Storage-independent application seam.

use crate::{Command, CommandResult, KvError, KvStore};

/// Command application parameterized by an injected store.
pub struct KvApplication<S> {
    store: S,
}

impl<S: KvStore> KvApplication<S> {
    /// Wraps a store implementation.
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// Executes one validated command.
    pub fn execute(&mut self, command: Command) -> Result<CommandResult, KvError> {
        let _ = (&mut self.store, command);
        Err(KvError::incomplete("key/value command execution"))
    }

    /// Returns ownership of the injected store.
    pub fn into_store(self) -> S {
        self.store
    }
}
