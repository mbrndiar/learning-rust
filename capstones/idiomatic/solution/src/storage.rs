//! Validated, versioned, publish-after-complete index persistence.

use crate::{ErrorCode, IndexData, IndexError};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Loads and atomically replaces complete index values.
pub trait IndexStore {
    fn load(&self) -> Result<IndexData, IndexError>;
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

    fn replace_with<B: PersistenceBackend>(
        &self,
        index: &IndexData,
        backend: &B,
    ) -> Result<(), IndexError> {
        index.validate()?;
        let parent = parent_directory(&self.path);
        fs::create_dir_all(parent)
            .map_err(|source| IndexError::io(ErrorCode::IndexWriteFailed, parent, source))?;
        let payload = backend.serialize(index)?;
        let mut candidate = backend.create(parent)?;
        backend.write(&mut candidate, &payload)?;
        backend.publish(candidate, &self.path)
    }
}

impl IndexStore for JsonFileIndexStore {
    fn load(&self) -> Result<IndexData, IndexError> {
        let file = File::open(&self.path).map_err(|source| {
            let code = if source.kind() == std::io::ErrorKind::NotFound {
                ErrorCode::IndexNotFound
            } else {
                ErrorCode::IndexReadFailed
            };
            IndexError::io(code, &self.path, source)
        })?;
        let value: serde_json::Value = serde_json::from_reader(file).map_err(|source| {
            let code = if source.io_error_kind().is_some() {
                ErrorCode::IndexReadFailed
            } else {
                ErrorCode::IndexCorrupt
            };
            IndexError::json(code, source)
        })?;
        match value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
        {
            Some(crate::INDEX_SCHEMA_VERSION) => {}
            Some(version) => {
                return Err(IndexError::contract(
                    ErrorCode::UnsupportedIndexVersion,
                    format!(
                        "unsupported index schema {version}; expected {}",
                        crate::INDEX_SCHEMA_VERSION
                    ),
                ));
            }
            None => {
                return Err(IndexError::contract(
                    ErrorCode::IndexCorrupt,
                    "index schema_version must be an unsigned integer",
                ));
            }
        }
        let index: IndexData = serde_json::from_value(value)
            .map_err(|source| IndexError::json(ErrorCode::IndexCorrupt, source))?;
        index.validate()?;
        Ok(index)
    }

    fn replace(&self, index: &IndexData) -> Result<(), IndexError> {
        self.replace_with(index, &FilePersistence)
    }
}

trait PersistenceBackend {
    type Candidate;

    fn serialize(&self, index: &IndexData) -> Result<Vec<u8>, IndexError>;
    fn create(&self, parent: &Path) -> Result<Self::Candidate, IndexError>;
    fn write(&self, candidate: &mut Self::Candidate, payload: &[u8]) -> Result<(), IndexError>;
    fn publish(&self, candidate: Self::Candidate, path: &Path) -> Result<(), IndexError>;
}

struct FilePersistence;

impl PersistenceBackend for FilePersistence {
    type Candidate = tempfile::NamedTempFile;

    fn serialize(&self, index: &IndexData) -> Result<Vec<u8>, IndexError> {
        let mut payload = serde_json::to_vec_pretty(index)
            .map_err(|source| IndexError::json(ErrorCode::IndexWriteFailed, source))?;
        payload.push(b'\n');
        Ok(payload)
    }

    fn create(&self, parent: &Path) -> Result<Self::Candidate, IndexError> {
        tempfile::NamedTempFile::new_in(parent)
            .map_err(|source| IndexError::io(ErrorCode::IndexWriteFailed, parent, source))
    }

    fn write(&self, candidate: &mut Self::Candidate, payload: &[u8]) -> Result<(), IndexError> {
        candidate
            .write_all(payload)
            .and_then(|()| candidate.as_file().sync_all())
            .map_err(|source| IndexError::io(ErrorCode::IndexWriteFailed, candidate.path(), source))
    }

    fn publish(&self, candidate: Self::Candidate, path: &Path) -> Result<(), IndexError> {
        candidate
            .persist(path)
            .map(|_| ())
            .map_err(|error| IndexError::io(ErrorCode::IndexWriteFailed, path, error.error))
    }
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{INDEX_SCHEMA_VERSION, IndexSettings};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::tempdir;

    #[test]
    fn bare_filename_uses_current_directory() {
        assert_eq!(parent_directory(Path::new("index.json")), Path::new("."));
    }

    #[test]
    fn every_failed_persistence_phase_preserves_the_old_file_and_cleans_candidates() {
        let directory = tempdir().expect("directory");
        let path = directory.path().join("index.json");
        fs::write(&path, b"old index").expect("old index");
        let index = IndexData {
            schema_version: INDEX_SCHEMA_VERSION,
            settings: IndexSettings::default(),
            roots: vec!["fixture".into()],
            documents: Vec::new(),
            issues: Vec::new(),
        };
        let store = JsonFileIndexStore::new(&path);

        for phase in [
            FailurePhase::Serialize,
            FailurePhase::Write,
            FailurePhase::Publish,
        ] {
            let cleaned = Arc::new(AtomicBool::new(false));
            let backend = FailingPersistence {
                phase,
                cleaned: Arc::clone(&cleaned),
            };
            let error = store
                .replace_with(&index, &backend)
                .expect_err("injected persistence failure");
            assert_eq!(error.code(), Some(ErrorCode::IndexWriteFailed));
            assert_eq!(fs::read(&path).expect("old bytes"), b"old index");
            assert_eq!(
                cleaned.load(Ordering::SeqCst),
                phase != FailurePhase::Serialize
            );
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum FailurePhase {
        Serialize,
        Write,
        Publish,
    }

    struct FailingPersistence {
        phase: FailurePhase,
        cleaned: Arc<AtomicBool>,
    }

    struct FakeCandidate {
        cleaned: Arc<AtomicBool>,
    }

    impl Drop for FakeCandidate {
        fn drop(&mut self) {
            self.cleaned.store(true, Ordering::SeqCst);
        }
    }

    impl PersistenceBackend for FailingPersistence {
        type Candidate = FakeCandidate;

        fn serialize(&self, _index: &IndexData) -> Result<Vec<u8>, IndexError> {
            if self.phase == FailurePhase::Serialize {
                Err(IndexError::contract(
                    ErrorCode::IndexWriteFailed,
                    "injected serialization failure",
                ))
            } else {
                Ok(b"candidate".to_vec())
            }
        }

        fn create(&self, _parent: &Path) -> Result<Self::Candidate, IndexError> {
            Ok(FakeCandidate {
                cleaned: Arc::clone(&self.cleaned),
            })
        }

        fn write(
            &self,
            _candidate: &mut Self::Candidate,
            _payload: &[u8],
        ) -> Result<(), IndexError> {
            if self.phase == FailurePhase::Write {
                Err(IndexError::contract(
                    ErrorCode::IndexWriteFailed,
                    "injected write failure",
                ))
            } else {
                Ok(())
            }
        }

        fn publish(&self, _candidate: Self::Candidate, _path: &Path) -> Result<(), IndexError> {
            Err(IndexError::contract(
                ErrorCode::IndexWriteFailed,
                "injected publish failure",
            ))
        }
    }
}
