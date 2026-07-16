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
        match command {
            Command::Set {
                key,
                value,
                expectation,
            } => self
                .store
                .set(&key, &value, expectation)
                .map(Box::new)
                .map(CommandResult::Set),
            Command::Get { key } => self.store.get(&key).map(Box::new).map(CommandResult::Get),
            Command::Delete { key, expectation } => self
                .store
                .delete(&key, expectation)
                .map(CommandResult::Delete),
            Command::List => self.store.list().map(CommandResult::List),
        }
    }

    /// Returns ownership of the injected store.
    pub fn into_store(self) -> S {
        self.store
    }
}
