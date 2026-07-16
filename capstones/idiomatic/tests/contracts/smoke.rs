use super::subject;
use std::num::NonZeroUsize;

#[derive(Clone)]
struct SmokeCancellation;

impl subject::Cancellation for SmokeCancellation {
    fn cancel(&self) {}

    fn is_cancelled(&self) -> bool {
        false
    }
}

struct SmokeTree;

impl subject::FileTree for SmokeTree {
    fn entries<'a>(
        &'a self,
        _root: &'a subject::RootSpec,
    ) -> Result<Box<dyn Iterator<Item = subject::TreeEntry> + 'a>, subject::IndexError> {
        Ok(Box::new(std::iter::empty()))
    }

    fn read(
        &self,
        _entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        Err(subject::FileIssue::incomplete("smoke tree read"))
    }
}

struct SmokeStore;

impl subject::IndexStore for SmokeStore {
    fn load(&self) -> Result<subject::IndexData, subject::IndexError> {
        Err(subject::IndexError::incomplete("smoke store load"))
    }

    fn replace(&self, _index: &subject::IndexData) -> Result<(), subject::IndexError> {
        Err(subject::IndexError::incomplete("smoke store replace"))
    }
}

pub fn assert_public_boundary() {
    let _parse_root: fn(&str) -> Result<subject::RootSpec, subject::IndexError> =
        subject::RootSpec::parse;
    let _parse_term: fn(&str) -> Result<subject::SearchTerm, subject::IndexError> =
        subject::SearchTerm::parse;
    let _new_document_id: fn(u64) -> Result<subject::DocumentId, subject::IndexError> =
        subject::DocumentId::new;
    let _store = SmokeStore;
    let _builder = subject::IndexBuilder::new(SmokeTree, NonZeroUsize::MIN, SmokeCancellation);

    assert_eq!(subject::INDEX_SCHEMA_VERSION, 1);

    let error =
        subject::tokenization::tokenize("Rust").expect_err("tokenization must remain incomplete");
    assert_eq!(error.incomplete_capability(), Some("tokenization"));
}
