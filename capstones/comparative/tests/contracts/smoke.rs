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
        Err(subject::KvError::incomplete("smoke store set"))
    }

    fn get(&self, _key: &subject::Key) -> Result<subject::Entry, subject::KvError> {
        Err(subject::KvError::incomplete("smoke store get"))
    }

    fn delete(
        &mut self,
        _key: &subject::Key,
        _expectation: subject::DeleteExpectation,
    ) -> Result<subject::DeleteResult, subject::KvError> {
        Err(subject::KvError::incomplete("smoke store delete"))
    }

    fn list(&self) -> Result<subject::ListResult, subject::KvError> {
        Err(subject::KvError::incomplete("smoke store list"))
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
        .expect_err("the injected smoke store must report an incomplete operation");
    assert!(
        error.incomplete_capability().is_some(),
        "starter and solution must preserve typed incomplete test seams"
    );
}
