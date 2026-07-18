//! Shared compile-time boundary check for both key/value crates.
//!
//! Included via `#[path]` into each crate's `smoke` test as `super::subject`. It
//! implements the `subject::KvStore` trait with a deterministic error stub and takes the
//! validated constructors as function pointers, so it fails to *compile* if the
//! public surface drifts — a fast guard that starter and solution keep the same
//! shape.

use super::subject;
use serde_json::Value;

struct SmokeStore;

impl subject::KvStore for SmokeStore {
    fn set(
        &mut self,
        _key: &subject::Key,
        _value: &Value,
        _expectation: subject::SetExpectation,
    ) -> Result<subject::SetResult, subject::KvError> {
        Err(subject::KvError::Storage {
            operation: "smoke store set",
        })
    }

    fn get(&self, _key: &subject::Key) -> Result<subject::Entry, subject::KvError> {
        Err(subject::KvError::Storage {
            operation: "smoke store get",
        })
    }

    fn delete(
        &mut self,
        _key: &subject::Key,
        _expectation: subject::DeleteExpectation,
    ) -> Result<subject::DeleteResult, subject::KvError> {
        Err(subject::KvError::Storage {
            operation: "smoke store delete",
        })
    }

    fn list(&self) -> Result<subject::ListResult, subject::KvError> {
        Err(subject::KvError::Storage {
            operation: "smoke store list",
        })
    }
}

pub fn assert_public_boundary() {
    let _parse_key: fn(&str) -> Result<subject::Key, subject::KvError> = subject::Key::parse;
    let _parse_revision: fn(u64) -> Result<subject::Revision, subject::KvError> =
        subject::Revision::new;

    assert_eq!(subject::SPEC_VERSION, "1.0.0");

    let mut application = subject::KvApplication::new(SmokeStore);
    let error = application
        .execute(subject::Command::List)
        .expect_err("the injected smoke store must report a storage operation");
    assert_eq!(error.category(), "storage_error");
    let details = error.details();
    assert_eq!(details["reason"], "storage_failure");
    assert!(
        details["operation"]
            .as_str()
            .is_some_and(|operation| !operation.is_empty())
    );
}
