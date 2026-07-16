//! Validated domain values and deterministic index result shapes.

use crate::tokenization::{is_normalized_term, normalize_search_term};
use crate::{ErrorCode, INDEX_SCHEMA_VERSION, IndexError};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

const DEFAULT_EXTENSIONS: [&str; 3] = [".log", ".md", ".txt"];
const DEFAULT_MAX_BYTES: u64 = 1_048_576;
const MAX_MAX_BYTES: u64 = 16_777_216;

/// A named, readable directory root supplied as `NAME=PATH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSpec {
    name: String,
    path: PathBuf,
}

impl RootSpec {
    /// Parses, canonicalizes, and preflights one root specification.
    pub fn parse(value: &str) -> Result<Self, IndexError> {
        let (name, path) = value.split_once('=').ok_or_else(|| {
            IndexError::contract(ErrorCode::InvalidRoot, "root must use the NAME=PATH form")
        })?;
        if !valid_root_name(name) {
            return Err(IndexError::contract(
                ErrorCode::InvalidRoot,
                "root name must match [A-Za-z0-9][A-Za-z0-9._-]{0,31}",
            ));
        }
        if path.is_empty() {
            return Err(IndexError::contract(
                ErrorCode::InvalidRoot,
                "root path must not be empty",
            ));
        }

        let supplied = PathBuf::from(path);
        let canonical = fs::canonicalize(&supplied)
            .map_err(|source| IndexError::io(ErrorCode::InvalidRoot, &supplied, source))?;
        let metadata = fs::metadata(&canonical)
            .map_err(|source| IndexError::io(ErrorCode::InvalidRoot, &supplied, source))?;
        if !metadata.is_dir() {
            return Err(IndexError::io(
                ErrorCode::InvalidRoot,
                &supplied,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "root path is not a directory",
                ),
            ));
        }
        fs::read_dir(&canonical)
            .map_err(|source| IndexError::io(ErrorCode::InvalidRoot, &supplied, source))?;

        Ok(Self {
            name: name.to_owned(),
            path: canonical,
        })
    }

    /// Returns the portable root name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the canonical host path used only for traversal and containment.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Parses roots and rejects duplicate names or canonical directories.
pub fn validate_roots(values: &[String]) -> Result<Vec<RootSpec>, IndexError> {
    let roots = values
        .iter()
        .map(|value| RootSpec::parse(value))
        .collect::<Result<Vec<_>, _>>()?;
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for root in &roots {
        if !names.insert(root.name.clone()) {
            return Err(IndexError::contract(
                ErrorCode::DuplicateRoot,
                format!("root name {:?} is duplicated", root.name),
            ));
        }
        if !paths.insert(root.path.clone()) {
            return Err(IndexError::contract(
                ErrorCode::DuplicateRoot,
                format!("root path {} is duplicated", root.path.display()),
            ));
        }
    }
    Ok(roots)
}

/// One normalized exact-match search term.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SearchTerm(String);

impl SearchTerm {
    /// Parses and validates a search term with the index tokenizer.
    pub fn parse(value: &str) -> Result<Self, IndexError> {
        normalize_search_term(value).map(Self).ok_or_else(|| {
            IndexError::contract(
                ErrorCode::InvalidSearchTerm,
                "each search term must produce exactly one token of 1 to 64 Unicode scalars",
            )
        })
    }

    /// Borrows the normalized term.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A positive contiguous document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Validates a positive document identifier.
    pub fn new(value: u64) -> Result<Self, IndexError> {
        if value == 0 {
            Err(IndexError::contract(
                ErrorCode::InvalidArgument,
                "document id must be positive",
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the identifier value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for DocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        if value == 0 {
            Err(serde::de::Error::custom("document id must be positive"))
        } else {
            Ok(Self(value))
        }
    }
}

/// Settings recorded in a version-1 index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSettings {
    pub extensions: Vec<String>,
    pub max_bytes: u64,
}

impl IndexSettings {
    /// Validates, normalizes, deduplicates, and sorts index settings.
    pub fn new(extensions: Vec<String>, max_bytes: u64) -> Result<Self, IndexError> {
        if !(1..=MAX_MAX_BYTES).contains(&max_bytes) {
            return Err(IndexError::contract(
                ErrorCode::InvalidArgument,
                "max-bytes must be in 1..=16777216",
            ));
        }
        let supplied = if extensions.is_empty() {
            DEFAULT_EXTENSIONS
                .iter()
                .map(|extension| (*extension).to_owned())
                .collect()
        } else {
            extensions
        };
        let mut normalized = BTreeSet::new();
        for extension in supplied {
            if !valid_extension(&extension) {
                return Err(IndexError::contract(
                    ErrorCode::InvalidExtension,
                    format!("invalid extension {extension:?}"),
                ));
            }
            normalized.insert(extension.to_ascii_lowercase());
        }
        Ok(Self {
            extensions: normalized.into_iter().collect(),
            max_bytes,
        })
    }

    /// Returns whether a portable path has an included extension.
    #[must_use]
    pub fn includes_path(&self, path: &str) -> bool {
        let filename = path.rsplit('/').next().unwrap_or(path);
        filename.rfind('.').is_some_and(|index| {
            index > 0
                && self
                    .extensions
                    .iter()
                    .any(|extension| filename[index..].eq_ignore_ascii_case(extension))
        })
    }

    fn is_valid_persisted(&self) -> bool {
        if !(1..=MAX_MAX_BYTES).contains(&self.max_bytes) || self.extensions.is_empty() {
            return false;
        }
        let mut previous: Option<&str> = None;
        for extension in &self.extensions {
            if !valid_extension(extension) || extension != &extension.to_ascii_lowercase() {
                return false;
            }
            if previous.is_some_and(|value| value >= extension) {
                return false;
            }
            previous = Some(extension);
        }
        true
    }
}

impl Default for IndexSettings {
    fn default() -> Self {
        Self {
            extensions: DEFAULT_EXTENSIONS
                .iter()
                .map(|extension| (*extension).to_owned())
                .collect(),
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }
}

/// One normalized term count within a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermCount {
    pub term: String,
    pub count: u64,
}

/// One fully indexed document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDocument {
    pub id: DocumentId,
    pub root: String,
    pub path: String,
    pub bytes: u64,
    pub terms: Vec<TermCount>,
}

/// Stable recoverable issue codes recorded in an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    EntryUnreadable,
    FileUnreadable,
    FileDisappeared,
    FileTooLarge,
    NonUtf8Content,
    NonUtf8Path,
    SymlinkSkipped,
    TokenTooLong,
}

impl IssueCode {
    /// Returns the stable snake-case code stored in the index.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EntryUnreadable => "entry_unreadable",
            Self::FileUnreadable => "file_unreadable",
            Self::FileDisappeared => "file_disappeared",
            Self::FileTooLarge => "file_too_large",
            Self::NonUtf8Content => "non_utf8_content",
            Self::NonUtf8Path => "non_utf8_path",
            Self::SymlinkSkipped => "symlink_skipped",
            Self::TokenTooLong => "token_too_long",
        }
    }

    /// Returns the deterministic report message for this issue category.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::EntryUnreadable => "directory entry could not be read",
            Self::FileUnreadable => "file could not be read",
            Self::FileDisappeared => "file disappeared before it could be read",
            Self::FileTooLarge => "file exceeds the configured byte limit",
            Self::NonUtf8Content => "file content is not valid UTF-8",
            Self::NonUtf8Path => "relative path is not valid UTF-8",
            Self::SymlinkSkipped => "symbolic links are not indexed",
            Self::TokenTooLong => "document contains a token longer than 64 Unicode scalar values",
        }
    }
}

/// One recoverable path issue emitted by indexing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexIssue {
    pub root: String,
    pub path: Option<String>,
    pub code: IssueCode,
    pub message: String,
}

/// Complete versioned index data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexData {
    pub schema_version: u64,
    pub settings: IndexSettings,
    pub roots: Vec<String>,
    pub documents: Vec<IndexedDocument>,
    pub issues: Vec<IndexIssue>,
}

impl IndexData {
    /// Revalidates every invariant after loading untrusted JSON.
    pub fn validate(&self) -> Result<(), IndexError> {
        if self.schema_version != INDEX_SCHEMA_VERSION {
            return Err(IndexError::contract(
                ErrorCode::UnsupportedIndexVersion,
                format!(
                    "unsupported index schema {}; expected {INDEX_SCHEMA_VERSION}",
                    self.schema_version
                ),
            ));
        }
        if !self.settings.is_valid_persisted() {
            return corrupt("invalid persisted index settings");
        }
        if self.roots.is_empty() {
            return corrupt("index must contain at least one root");
        }

        let mut root_order = BTreeMap::new();
        for (order, root) in self.roots.iter().enumerate() {
            if !valid_root_name(root) || root_order.insert(root.as_str(), order).is_some() {
                return corrupt("root names must be valid and unique");
            }
        }

        let mut previous_document: Option<(usize, &str)> = None;
        for (offset, document) in self.documents.iter().enumerate() {
            if document.id.get() != offset as u64 + 1 {
                return corrupt("document ids must be contiguous from 1");
            }
            let order = root_order
                .get(document.root.as_str())
                .copied()
                .ok_or_else(|| corrupt_error("document references an unknown root"))?;
            if !valid_portable_path(&document.path) {
                return corrupt("document path is not a safe portable relative path");
            }
            let key = (order, document.path.as_str());
            if previous_document.is_some_and(|previous| previous >= key) {
                return corrupt("documents must be unique and sorted by root and path");
            }
            previous_document = Some(key);

            let mut previous_term: Option<&str> = None;
            for term in &document.terms {
                if term.count == 0 || !is_normalized_term(&term.term) {
                    return corrupt("document term or count is invalid");
                }
                if previous_term.is_some_and(|previous| previous >= term.term.as_str()) {
                    return corrupt("document terms must be unique and sorted");
                }
                previous_term = Some(&term.term);
            }
        }

        let mut previous_issue: Option<(usize, Option<&str>, &'static str, &str)> = None;
        for issue in &self.issues {
            let order = root_order
                .get(issue.root.as_str())
                .copied()
                .ok_or_else(|| corrupt_error("issue references an unknown root"))?;
            match (&issue.path, issue.code) {
                (None, IssueCode::NonUtf8Path) => {}
                (Some(path), code) if code != IssueCode::NonUtf8Path => {
                    if !valid_portable_path(path) {
                        return corrupt("issue path is not a safe portable relative path");
                    }
                }
                _ => return corrupt("only non_utf8_path issues may use a null path"),
            }
            if issue.message != issue.code.message() {
                return corrupt("issue message does not match its stable code");
            }
            let key = (
                order,
                issue.path.as_deref(),
                issue.code.as_str(),
                issue.message.as_str(),
            );
            if previous_issue.is_some_and(|previous| previous > key) {
                return corrupt("issues must be sorted deterministically");
            }
            previous_issue = Some(key);
        }
        Ok(())
    }
}

/// Validated search input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchQuery {
    pub terms: Vec<SearchTerm>,
    pub path_prefix: Option<String>,
    pub limit: usize,
}

impl SearchQuery {
    /// Normalizes terms, validates the optional portable prefix, and bounds limit.
    pub fn new(
        terms: Vec<String>,
        path_prefix: Option<String>,
        limit: usize,
    ) -> Result<Self, IndexError> {
        if !(1..=10_000).contains(&limit) {
            return Err(IndexError::contract(
                ErrorCode::InvalidArgument,
                "limit must be in 1..=10000",
            ));
        }
        if let Some(prefix) = &path_prefix {
            if !valid_portable_path(prefix) {
                return Err(IndexError::contract(
                    ErrorCode::InvalidPathPrefix,
                    "path-prefix must be a safe portable relative path",
                ));
            }
        }
        let terms: Vec<SearchTerm> = terms
            .iter()
            .map(|term| SearchTerm::parse(term))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();
        if terms.is_empty() {
            return Err(IndexError::contract(
                ErrorCode::InvalidSearchTerm,
                "at least one search term is required",
            ));
        }
        Ok(Self {
            terms,
            path_prefix,
            limit,
        })
    }

    /// Revalidates a query assembled through its public result-shape fields.
    pub fn validate(&self) -> Result<(), IndexError> {
        if !(1..=10_000).contains(&self.limit) {
            return Err(IndexError::contract(
                ErrorCode::InvalidArgument,
                "limit must be in 1..=10000",
            ));
        }
        if self
            .path_prefix
            .as_ref()
            .is_some_and(|prefix| !valid_portable_path(prefix))
        {
            return Err(IndexError::contract(
                ErrorCode::InvalidPathPrefix,
                "path-prefix must be a safe portable relative path",
            ));
        }
        if self.terms.is_empty() || self.terms.windows(2).any(|terms| terms[0] >= terms[1]) {
            return Err(IndexError::contract(
                ErrorCode::InvalidSearchTerm,
                "query terms must be non-empty, unique, and sorted",
            ));
        }
        Ok(())
    }
}

/// Document fields exposed by a search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentSummary {
    pub id: DocumentId,
    pub root: String,
    pub path: String,
    pub bytes: u64,
}

/// One matching document and its requested term counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchMatch {
    pub document: DocumentSummary,
    pub term_counts: Vec<TermCount>,
}

/// Complete deterministic search response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchResult {
    pub query: SearchQuery,
    pub matches: Vec<SearchMatch>,
}

/// Deterministic index summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexStats {
    pub schema_version: u64,
    pub roots: usize,
    pub documents: usize,
    pub issues: usize,
    pub unique_terms: usize,
    pub indexed_bytes: u64,
}

impl IndexStats {
    /// Computes summary statistics from a validated index.
    pub fn from_index(index: &IndexData) -> Result<Self, IndexError> {
        index.validate()?;
        let unique_terms = index
            .documents
            .iter()
            .flat_map(|document| document.terms.iter().map(|term| term.term.as_str()))
            .collect::<BTreeSet<_>>()
            .len();
        Ok(Self {
            schema_version: index.schema_version,
            roots: index.roots.len(),
            documents: index.documents.len(),
            issues: index.issues.len(),
            unique_terms,
            indexed_bytes: index.documents.iter().map(|document| document.bytes).sum(),
        })
    }
}

/// Converts a host path under `root` into the portable persisted form.
#[must_use]
pub fn portable_relative_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let mut segments = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(segment) => segments.push(segment.to_str()?.to_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments.join("/"))
    }
}

/// Checks the persisted portable relative-path grammar.
#[must_use]
pub fn valid_portable_path(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.as_bytes().get(1) == Some(&b':')
    {
        return false;
    }
    value
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn valid_root_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=32).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn valid_extension(value: &str) -> bool {
    let bytes = value.as_bytes();
    (2..=17).contains(&bytes.len())
        && bytes[0] == b'.'
        && bytes[1..].iter().all(u8::is_ascii_alphanumeric)
}

fn corrupt(message: &str) -> Result<(), IndexError> {
    Err(corrupt_error(message))
}

fn corrupt_error(message: &str) -> IndexError {
    IndexError::contract(ErrorCode::IndexCorrupt, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_extensions_paths_and_searches() {
        assert_eq!(
            IndexSettings::new(vec![".TXT".into(), ".txt".into()], 10)
                .expect("settings")
                .extensions,
            vec![".txt"]
        );
        assert!(IndexSettings::new(vec!["txt".into()], 10).is_err());
        assert!(valid_portable_path("docs/readme.md"));
        assert!(!valid_portable_path("../readme.md"));
        assert_eq!(SearchTerm::parse("İ").expect("term").as_str(), "i\u{307}");
        assert!(SearchTerm::parse("two terms").is_err());
    }
}
