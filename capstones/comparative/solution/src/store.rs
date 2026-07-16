//! Injectable persistence boundary for the comparative capstone.

use crate::{
    DeleteExpectation, DeleteResult, Entry, Key, KvError, ListResult, SetExpectation, SetResult,
};
use serde_json::Value;

/// Persistence operations required by the storage-independent application.
pub trait KvStore {
    fn set(
        &mut self,
        key: &Key,
        value: &Value,
        expectation: SetExpectation,
    ) -> Result<SetResult, KvError>;
    fn get(&self, key: &Key) -> Result<Entry, KvError>;
    fn delete(
        &mut self,
        key: &Key,
        expectation: DeleteExpectation,
    ) -> Result<DeleteResult, KvError>;
    fn list(&self) -> Result<ListResult, KvError>;
}
