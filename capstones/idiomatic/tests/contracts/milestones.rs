use super::subject;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex, mpsc};
use subject::IndexStore as _;
use tempfile::{TempDir, tempdir};

pub fn milestone_1_validated_domain() {
    let fixture = tempdir().expect("temporary root");
    let root_value = format!("fixture={}", fixture.path().display());
    let root = subject::RootSpec::parse(&root_value).expect("valid root");
    assert_eq!(root.name(), "fixture");
    assert_eq!(
        root.path(),
        fixture.path().canonicalize().expect("canonical")
    );
    assert_eq!(
        subject::RootSpec::parse("bad name=.")
            .expect_err("invalid root")
            .code(),
        Some(subject::ErrorCode::InvalidRoot)
    );
    assert!(subject::DocumentId::new(0).is_err());
    assert!(subject::valid_portable_path("docs/readme.md"));
    assert!(!subject::valid_portable_path("../readme.md"));
    assert_eq!(
        subject::IndexSettings::new(vec!["txt".into()], 10)
            .expect_err("invalid extension")
            .code(),
        Some(subject::ErrorCode::InvalidExtension)
    );
    assert_eq!(
        subject::IndexSettings::new(vec![".txt".into()], 0)
            .expect_err("invalid byte limit")
            .code(),
        Some(subject::ErrorCode::InvalidArgument)
    );
    assert_eq!(
        subject::IndexSettings::new(vec![".TXT".into(), ".txt".into()], 10)
            .expect("normalized settings")
            .extensions,
        vec![".txt"]
    );
    assert_eq!(
        subject::tokenization::tokenize("Rust_safe-42! İ"),
        vec!["rust", "safe", "42", "i\u{307}"]
    );
    let long = "A".repeat(65);
    let outcome = subject::tokenization::tokenize_with_outcome(&format!("{long} ok {long}"));
    assert_eq!(outcome.tokens, vec!["ok"]);
    assert!(outcome.ignored_long_token);

    let duplicate_values = vec![
        root_value.clone(),
        format!("other={}", fixture.path().display()),
    ];
    let duplicate = subject::validate_roots(&duplicate_values).expect_err("duplicate path");
    assert_eq!(duplicate.code(), Some(subject::ErrorCode::DuplicateRoot));
    let duplicate_build = subject::IndexBuilder::new(
        MemoryTree {
            contents: BTreeMap::new(),
        },
        NonZeroUsize::new(1).expect("worker"),
        subject::CancellationToken::new(),
    )
    .build(
        &[root.clone(), root.clone()],
        &subject::IndexSettings::default(),
    )
    .expect_err("duplicate builder roots");
    assert_eq!(
        duplicate_build.code(),
        Some(subject::ErrorCode::DuplicateRoot)
    );

    let query = subject::SearchQuery::new(
        vec!["SAFE".into(), "rust".into(), "Rust".into()],
        Some("docs".into()),
        5,
    )
    .expect("query");
    assert_eq!(
        query
            .terms
            .iter()
            .map(subject::SearchTerm::as_str)
            .collect::<Vec<_>>(),
        vec!["rust", "safe"]
    );
    assert_eq!(
        subject::SearchQuery::new(vec!["two terms".into()], None, 1)
            .expect_err("invalid term")
            .code(),
        Some(subject::ErrorCode::InvalidSearchTerm)
    );
    assert_eq!(
        subject::SearchQuery::new(vec!["rust".into()], Some("../docs".into()), 1)
            .expect_err("invalid prefix")
            .code(),
        Some(subject::ErrorCode::InvalidPathPrefix)
    );
    assert!(subject::SearchQuery::new(vec!["rust".into()], None, 0).is_err());
    assert!(
        subject::query::search(
            &searchable_index(),
            subject::SearchQuery {
                terms: Vec::new(),
                path_prefix: None,
                limit: 1,
            },
        )
        .is_err()
    );

    let index = searchable_index();
    let result = subject::query::search(&index, query).expect("search");
    assert_eq!(
        result
            .matches
            .iter()
            .map(|found| found.document.path.as_str())
            .collect::<Vec<_>>(),
        vec!["docs/a.txt"]
    );
    assert_eq!(result.matches[0].term_counts[0].count, 2);
}

pub fn milestone_2_traversal_and_issues() {
    let fixture = tempdir().expect("temporary root");
    let root = parsed_root(&fixture);
    let tree = ScriptedTree;
    let settings = subject::IndexSettings::new(Vec::new(), 32).expect("settings");
    let index = subject::IndexBuilder::new(
        tree,
        NonZeroUsize::new(2).expect("workers"),
        subject::CancellationToken::new(),
    )
    .build(&[root], &settings)
    .expect("recoverable issues do not fail the batch");

    assert_eq!(index.documents.len(), 1);
    assert_eq!(index.documents[0].path, "long.txt");
    assert!(index.documents[0].terms.is_empty());
    let codes = index
        .issues
        .iter()
        .map(|issue| issue.code)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        codes,
        BTreeSet::from([
            subject::IssueCode::EntryUnreadable,
            subject::IssueCode::FileUnreadable,
            subject::IssueCode::FileDisappeared,
            subject::IssueCode::FileTooLarge,
            subject::IssueCode::NonUtf8Content,
            subject::IssueCode::NonUtf8Path,
            subject::IssueCode::SymlinkSkipped,
            subject::IssueCode::TokenTooLong,
        ])
    );
    assert_eq!(
        index
            .issues
            .iter()
            .find(|issue| issue.code == subject::IssueCode::NonUtf8Path)
            .expect("non UTF-8 path")
            .path,
        None
    );
    assert!(index.issues.iter().all(|issue| !issue.message.is_empty()));

    let error = subject::IndexBuilder::new(
        UnsafePathTree,
        NonZeroUsize::new(1).expect("worker"),
        subject::CancellationToken::new(),
    )
    .build(&[parsed_root(&fixture)], &settings)
    .expect_err("unsafe provider path");
    assert_eq!(error.code(), Some(subject::ErrorCode::WorkerFailed));
}

pub fn milestone_3_versioned_storage() {
    let expected_path = fixture_path("expected_index.json");
    let expected = subject::JsonFileIndexStore::new(&expected_path)
        .load()
        .expect("fixture index");
    expected.validate().expect("fixture invariants");

    let directory = tempdir().expect("storage directory");
    let path = directory.path().join("index.json");
    let store = subject::JsonFileIndexStore::new(&path);
    let missing_error = store.load().expect_err("missing index");
    assert_eq!(
        missing_error.code(),
        Some(subject::ErrorCode::IndexNotFound)
    );
    let directory_error = subject::JsonFileIndexStore::new(directory.path())
        .load()
        .expect_err("directory is not an index file");
    assert_eq!(
        directory_error.code(),
        Some(subject::ErrorCode::IndexReadFailed)
    );
    store.replace(&expected).expect("atomic replacement");
    assert_eq!(store.load().expect("round trip"), expected);

    let before = fs::read(&path).expect("old index bytes");
    let entries_before = directory_entries(directory.path());
    let mut invalid = expected.clone();
    invalid.documents[0].path = "../escape.txt".into();
    let error = store.replace(&invalid).expect_err("invalid candidate");
    assert_eq!(error.code(), Some(subject::ErrorCode::IndexCorrupt));
    assert_eq!(fs::read(&path).expect("unchanged index"), before);
    assert_eq!(directory_entries(directory.path()), entries_before);

    let corrupt = [
        ("corrupt/syntax.json", subject::ErrorCode::IndexCorrupt),
        (
            "corrupt/missing-header.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        (
            "corrupt/unsupported-version.json",
            subject::ErrorCode::UnsupportedIndexVersion,
        ),
        (
            "corrupt/invalid-settings.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        (
            "corrupt/duplicate-roots.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        ("corrupt/id-gap.json", subject::ErrorCode::IndexCorrupt),
        (
            "corrupt/document-order.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        (
            "corrupt/duplicate-path.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        (
            "corrupt/duplicate-term.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        ("corrupt/term-order.json", subject::ErrorCode::IndexCorrupt),
        ("corrupt/zero-count.json", subject::ErrorCode::IndexCorrupt),
        (
            "corrupt/invalid-term.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        ("corrupt/unsafe-path.json", subject::ErrorCode::IndexCorrupt),
        (
            "corrupt/invalid-issue-path.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        (
            "corrupt/invalid-issue-message.json",
            subject::ErrorCode::IndexCorrupt,
        ),
        ("corrupt/issue-order.json", subject::ErrorCode::IndexCorrupt),
        (
            "corrupt/issue-code-order.json",
            subject::ErrorCode::IndexCorrupt,
        ),
    ];
    for (name, code) in corrupt {
        let error = subject::JsonFileIndexStore::new(fixture_path(name))
            .load()
            .unwrap_err();
        assert_eq!(error.code(), Some(code), "fixture {name}");
    }
}

pub fn milestone_4_bounded_concurrency() {
    let fixture = tempdir().expect("temporary root");
    let root = parsed_root(&fixture);
    let other_fixture = tempdir().expect("second temporary root");
    let other_root = subject::RootSpec::parse(&format!("other={}", other_fixture.path().display()))
        .expect("second root");
    let settings = subject::IndexSettings::default();
    let contents = BTreeMap::from([
        ("a.txt".to_owned(), b"alpha rust".to_vec()),
        ("b.txt".to_owned(), b"beta rust rust".to_vec()),
    ]);
    let baseline = subject::IndexBuilder::new(
        MemoryTree {
            contents: contents.clone(),
        },
        NonZeroUsize::new(1).expect("worker"),
        subject::CancellationToken::new(),
    )
    .build(std::slice::from_ref(&root), &settings)
    .expect("single worker");
    let ordered_roots = subject::IndexBuilder::new(
        MemoryTree {
            contents: BTreeMap::from([("a.txt".to_owned(), b"rust".to_vec())]),
        },
        NonZeroUsize::new(2).expect("workers"),
        subject::CancellationToken::new(),
    )
    .build(&[other_root, root.clone()], &settings)
    .expect("root ordering");
    assert_eq!(ordered_roots.roots, vec!["other", "fixture"]);
    assert_eq!(ordered_roots.documents[0].root, "other");
    assert_eq!(ordered_roots.documents[1].root, "fixture");

    let traversal_cancellation = subject::CancellationToken::new();
    let yielded = Arc::new(AtomicUsize::new(0));
    let error = subject::IndexBuilder::new(
        CancellingEntriesTree {
            cancellation: traversal_cancellation.clone(),
            yielded: Arc::clone(&yielded),
        },
        NonZeroUsize::new(2).expect("workers"),
        traversal_cancellation,
    )
    .build(std::slice::from_ref(&root), &settings)
    .expect_err("traversal cancellation");
    assert_eq!(error.code(), Some(subject::ErrorCode::Cancelled));
    assert_eq!(yielded.load(Ordering::SeqCst), 1);

    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let concurrent = subject::IndexBuilder::new(
        ReverseTree {
            contents,
            barrier: Arc::new(Barrier::new(2)),
            release_sender,
            release_receiver: Arc::new(Mutex::new(release_receiver)),
            active: Arc::clone(&active),
            maximum: Arc::clone(&maximum),
        },
        NonZeroUsize::new(2).expect("workers"),
        subject::CancellationToken::new(),
    )
    .build(std::slice::from_ref(&root), &settings)
    .expect("reverse completion");
    assert_eq!(concurrent, baseline);
    assert_eq!(maximum.load(Ordering::SeqCst), 2);
    assert_eq!(active.load(Ordering::SeqCst), 0);

    let cancellation = subject::CancellationToken::new();
    let reads = Arc::new(AtomicUsize::new(0));
    let active = Arc::new(AtomicUsize::new(0));
    let error = subject::IndexBuilder::new(
        CancellingTree {
            cancellation: cancellation.clone(),
            reads: Arc::clone(&reads),
            active: Arc::clone(&active),
        },
        NonZeroUsize::new(1).expect("worker"),
        cancellation,
    )
    .build(std::slice::from_ref(&root), &settings)
    .expect_err("cancelled build");
    assert_eq!(error.code(), Some(subject::ErrorCode::Cancelled));
    assert_eq!(error.exit_code(), 130);
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    assert_eq!(active.load(Ordering::SeqCst), 0);

    for _ in 0..10 {
        let cancellation = subject::CancellationToken::new();
        let reads = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let error = subject::IndexBuilder::new(
            CancellingTree {
                cancellation: cancellation.clone(),
                reads: Arc::clone(&reads),
                active: Arc::clone(&active),
            },
            NonZeroUsize::new(3).expect("workers"),
            cancellation,
        )
        .build(std::slice::from_ref(&root), &settings)
        .expect_err("concurrent cancellation");
        assert_eq!(error.code(), Some(subject::ErrorCode::Cancelled));
        assert!((1..=3).contains(&reads.load(Ordering::SeqCst)));
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    for _ in 0..10 {
        let active = Arc::new(AtomicUsize::new(0));
        let reads = Arc::new(AtomicUsize::new(0));
        let error = subject::IndexBuilder::new(
            PanickingTree {
                active: Arc::clone(&active),
                reads: Arc::clone(&reads),
            },
            NonZeroUsize::new(2).expect("workers"),
            subject::CancellationToken::new(),
        )
        .build(std::slice::from_ref(&root), &settings)
        .expect_err("worker panic is typed");
        assert_eq!(error.code(), Some(subject::ErrorCode::WorkerFailed));
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert!((1..=2).contains(&reads.load(Ordering::SeqCst)));
    }
}

pub fn milestone_5_full_cli(program: &Path) {
    let directory = tempdir().expect("CLI fixture directory");
    let root = directory.path().join("root");
    fs::create_dir_all(root.join("docs")).expect("fixture directories");
    materialize_manifest(&root);
    let symlink_created = create_fixture_symlink(&root);
    let index_path = directory.path().join("index.json");

    let root_argument = format!("fixture={}", root.display());
    let output = run(
        program,
        &[
            "index",
            "--index",
            path_text(&index_path),
            "--root",
            &root_argument,
            "--workers",
            "4",
            "--max-bytes",
            "32",
        ],
    );
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).expect("index report JSON");
    assert_eq!(report["index"], path_text(&index_path));
    assert_eq!(report["documents"], 4);
    assert_eq!(report["issues"], if symlink_created { 3 } else { 2 });
    assert_eq!(report["unique_terms"], 4);
    assert!(output.stderr.is_empty());

    let actual_index: Value =
        serde_json::from_slice(&fs::read(&index_path).expect("index bytes")).expect("index JSON");
    let mut expected_index = read_json_fixture("expected_index.json");
    if !symlink_created {
        expected_index["issues"]
            .as_array_mut()
            .expect("issues")
            .retain(|issue| issue["code"] != "symlink_skipped");
    }
    assert_eq!(actual_index, expected_index);

    let search_output = run(
        program,
        &[
            "search",
            "--index",
            path_text(&index_path),
            "--term",
            "RUST",
            "--format",
            "json",
        ],
    );
    assert_success(&search_output);
    assert_eq!(
        serde_json::from_slice::<Value>(&search_output.stdout).expect("search JSON"),
        read_json_fixture("expected_search.json")
    );

    let stats_output = run(
        program,
        &[
            "stats",
            "--index",
            path_text(&index_path),
            "--format",
            "json",
        ],
    );
    assert_success(&stats_output);
    let mut expected_stats = read_json_fixture("expected_stats.json");
    if !symlink_created {
        expected_stats["issues"] = json!(2);
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&stats_output.stdout).expect("stats JSON"),
        expected_stats
    );

    let text_output = run(
        program,
        &[
            "search",
            "--index",
            path_text(&index_path),
            "--term",
            "rust",
            "--path-prefix",
            "docs",
            "--format",
            "text",
        ],
    );
    assert_success(&text_output);
    assert_eq!(
        String::from_utf8(text_output.stdout).expect("text output"),
        concat!(
            "query terms=rust path_prefix=docs limit=100\n",
            "match id=3 root=fixture path=docs/readme.md bytes=15 rust=2\n",
            "match id=4 root=fixture path=docs/unicode.md bytes=8 rust=1\n"
        )
    );

    let second_index = directory.path().join("index-single.json");
    let single_output = run(
        program,
        &[
            "index",
            "--index",
            path_text(&second_index),
            "--root",
            &root_argument,
            "--workers",
            "1",
            "--max-bytes",
            "32",
        ],
    );
    assert_success(&single_output);
    assert_eq!(
        fs::read(&index_path).expect("parallel index"),
        fs::read(&second_index).expect("single index")
    );

    let markdown_index = directory.path().join("markdown.json");
    let markdown_output = run(
        program,
        &[
            "index",
            "--index",
            path_text(&markdown_index),
            "--root",
            &root_argument,
            "--extension",
            ".MD",
        ],
    );
    assert_success(&markdown_output);
    let markdown_report: Value =
        serde_json::from_slice(&markdown_output.stdout).expect("markdown report");
    assert_eq!(markdown_report["documents"], 2);
    assert_eq!(
        markdown_report["issues"],
        if symlink_created { 1 } else { 0 }
    );

    let duplicate = run(
        program,
        &[
            "--json-errors",
            "index",
            "--index",
            path_text(&index_path),
            "--root",
            &root_argument,
            "--root",
            &format!("again={}", root.display()),
        ],
    );
    assert_eq!(duplicate.status.code(), Some(2));
    assert_json_error(&duplicate, "duplicate_root");

    let invalid_root = run(
        program,
        &[
            "--json-errors",
            "index",
            "--index",
            path_text(&index_path),
            "--root",
            "missing-form",
        ],
    );
    assert_eq!(invalid_root.status.code(), Some(2));
    assert_json_error(&invalid_root, "invalid_root");

    let missing_root = run(
        program,
        &[
            "--json-errors",
            "index",
            "--index",
            path_text(&index_path),
            "--root",
            &format!("missing={}", directory.path().join("absent").display()),
        ],
    );
    assert_eq!(missing_root.status.code(), Some(3));
    assert_json_error(&missing_root, "invalid_root");

    let corrupt = run(
        program,
        &[
            "--json-errors",
            "stats",
            "--index",
            path_text(&fixture_path("corrupt/syntax.json")),
        ],
    );
    assert_eq!(corrupt.status.code(), Some(4));
    assert_json_error(&corrupt, "index_corrupt");
}

fn searchable_index() -> subject::IndexData {
    subject::IndexData {
        schema_version: subject::INDEX_SCHEMA_VERSION,
        settings: subject::IndexSettings::default(),
        roots: vec!["fixture".into()],
        documents: vec![
            subject::IndexedDocument {
                id: subject::DocumentId::new(1).expect("id"),
                root: "fixture".into(),
                path: "docs/a.txt".into(),
                bytes: 12,
                terms: vec![
                    subject::TermCount {
                        term: "rust".into(),
                        count: 2,
                    },
                    subject::TermCount {
                        term: "safe".into(),
                        count: 1,
                    },
                ],
            },
            subject::IndexedDocument {
                id: subject::DocumentId::new(2).expect("id"),
                root: "fixture".into(),
                path: "docs/b.txt".into(),
                bytes: 4,
                terms: vec![subject::TermCount {
                    term: "rust".into(),
                    count: 1,
                }],
            },
        ],
        issues: Vec::new(),
    }
}

struct ScriptedTree;

impl subject::FileTree for ScriptedTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let mut entries = vec![
            Err(subject::FileIssue::message(
                subject::IssueCode::EntryUnreadable,
                Some("denied".into()),
                "injected",
            )),
            Ok(fake_entry(root, None, subject::TreeEntryKind::RegularFile)),
            Ok(fake_entry(
                root,
                Some("link.md"),
                subject::TreeEntryKind::Symlink,
            )),
        ];
        for path in [
            "binary.txt",
            "disappeared.txt",
            "long.txt",
            "oversized.txt",
            "unreadable.txt",
        ] {
            entries.push(Ok(fake_entry(
                root,
                Some(path),
                subject::TreeEntryKind::RegularFile,
            )));
        }
        Ok(Box::new(entries.into_iter()))
    }

    fn read(
        &self,
        entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        let path = entry.relative_path.as_deref().expect("scripted path");
        match path {
            "binary.txt" => Ok(vec![0xff, 0xfe]),
            "disappeared.txt" => Err(subject::FileIssue::message(
                subject::IssueCode::FileDisappeared,
                Some(path.into()),
                "injected",
            )),
            "long.txt" => Ok("A".repeat(65).into_bytes()),
            "oversized.txt" => Err(subject::FileIssue::message(
                subject::IssueCode::FileTooLarge,
                Some(path.into()),
                "injected",
            )),
            "unreadable.txt" => Err(subject::FileIssue::message(
                subject::IssueCode::FileUnreadable,
                Some(path.into()),
                "injected",
            )),
            _ => unreachable!("unexpected scripted path"),
        }
    }
}

struct UnsafePathTree;

impl subject::FileTree for UnsafePathTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        Ok(Box::new(
            [Ok(subject::TreeEntry {
                root: root.name().to_owned(),
                host_path: root.path().join("escape.txt"),
                relative_path: Some("../escape.txt".into()),
                kind: subject::TreeEntryKind::RegularFile,
            })]
            .into_iter(),
        ))
    }

    fn read(
        &self,
        _entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        unreachable!("unsafe paths are rejected before reads")
    }
}

#[derive(Clone)]
struct MemoryTree {
    contents: BTreeMap<String, Vec<u8>>,
}

impl subject::FileTree for MemoryTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let entries = self
            .contents
            .keys()
            .map(|path| {
                Ok(fake_entry(
                    root,
                    Some(path),
                    subject::TreeEntryKind::RegularFile,
                ))
            })
            .collect::<Vec<_>>();
        Ok(Box::new(entries.into_iter()))
    }

    fn read(
        &self,
        entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        Ok(self.contents[entry.relative_path.as_deref().expect("path")].clone())
    }
}

struct ReverseTree {
    contents: BTreeMap<String, Vec<u8>>,
    barrier: Arc<Barrier>,
    release_sender: mpsc::SyncSender<()>,
    release_receiver: Arc<Mutex<mpsc::Receiver<()>>>,
    active: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
}

impl subject::FileTree for ReverseTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let entries = self
            .contents
            .keys()
            .map(|path| {
                Ok(fake_entry(
                    root,
                    Some(path),
                    subject::TreeEntryKind::RegularFile,
                ))
            })
            .collect::<Vec<_>>();
        Ok(Box::new(entries.into_iter()))
    }

    fn read(
        &self,
        entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        let _guard = ActiveGuard::new(&self.active, Some(&self.maximum));
        self.barrier.wait();
        let path = entry.relative_path.as_deref().expect("path");
        if path == "a.txt" {
            self.release_receiver
                .lock()
                .expect("release lock")
                .recv()
                .expect("release");
        } else {
            self.release_sender.send(()).expect("release send");
        }
        Ok(self.contents[path].clone())
    }
}

struct CancellingTree {
    cancellation: subject::CancellationToken,
    reads: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
}

impl subject::FileTree for CancellingTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let entries = ["a.txt", "b.txt", "c.txt"]
            .into_iter()
            .map(|path| {
                Ok(fake_entry(
                    root,
                    Some(path),
                    subject::TreeEntryKind::RegularFile,
                ))
            })
            .collect::<Vec<_>>();
        Ok(Box::new(entries.into_iter()))
    }

    fn read(
        &self,
        _entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        let _guard = ActiveGuard::new(&self.active, None);
        self.reads.fetch_add(1, Ordering::SeqCst);
        subject::Cancellation::cancel(&self.cancellation);
        Ok(b"cancelled".to_vec())
    }
}

struct CancellingEntriesTree {
    cancellation: subject::CancellationToken,
    yielded: Arc<AtomicUsize>,
}

impl subject::FileTree for CancellingEntriesTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let cancellation = self.cancellation.clone();
        let yielded = Arc::clone(&self.yielded);
        let entry = fake_entry(
            root,
            Some("cancel.txt"),
            subject::TreeEntryKind::RegularFile,
        );
        Ok(Box::new((0..3).map(move |_| {
            yielded.fetch_add(1, Ordering::SeqCst);
            subject::Cancellation::cancel(&cancellation);
            Ok(entry.clone())
        })))
    }

    fn read(
        &self,
        _entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        unreachable!("traversal cancellation prevents reads")
    }
}

struct PanickingTree {
    active: Arc<AtomicUsize>,
    reads: Arc<AtomicUsize>,
}

impl subject::FileTree for PanickingTree {
    fn entries<'a>(
        &'a self,
        root: &'a subject::RootSpec,
    ) -> Result<
        Box<dyn Iterator<Item = Result<subject::TreeEntry, subject::FileIssue>> + 'a>,
        subject::IndexError,
    > {
        let entries = ["a.txt", "b.txt", "c.txt"]
            .into_iter()
            .map(|path| {
                Ok(fake_entry(
                    root,
                    Some(path),
                    subject::TreeEntryKind::RegularFile,
                ))
            })
            .collect::<Vec<_>>();
        Ok(Box::new(entries.into_iter()))
    }

    fn read(
        &self,
        _entry: &subject::TreeEntry,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, subject::FileIssue> {
        let _guard = ActiveGuard::new(&self.active, None);
        self.reads.fetch_add(1, Ordering::SeqCst);
        panic!("injected worker panic");
    }
}

struct ActiveGuard<'a> {
    active: &'a AtomicUsize,
}

impl<'a> ActiveGuard<'a> {
    fn new(active: &'a AtomicUsize, maximum: Option<&AtomicUsize>) -> Self {
        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(maximum) = maximum {
            maximum.fetch_max(now, Ordering::SeqCst);
        }
        Self { active }
    }
}

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

fn fake_entry(
    root: &subject::RootSpec,
    path: Option<&str>,
    kind: subject::TreeEntryKind,
) -> subject::TreeEntry {
    subject::TreeEntry {
        root: root.name().to_owned(),
        host_path: root.path().join(path.unwrap_or("non-utf8")),
        relative_path: path.map(str::to_owned),
        kind,
    }
}

fn parsed_root(directory: &TempDir) -> subject::RootSpec {
    subject::RootSpec::parse(&format!("fixture={}", directory.path().display()))
        .expect("parsed temporary root")
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("idiomatic directory")
        .join("tests/fixtures")
        .join(name)
}

fn read_json_fixture(name: &str) -> Value {
    serde_json::from_slice(&fs::read(fixture_path(name)).expect("fixture bytes"))
        .expect("fixture JSON")
}

fn directory_entries(path: &Path) -> BTreeSet<String> {
    fs::read_dir(path)
        .expect("directory")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn materialize_manifest(root: &Path) {
    let manifest = read_json_fixture("tree_manifest.json");
    for file in manifest["files"].as_array().expect("files") {
        let path = root.join(file["path"].as_str().expect("file path"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        if let Some(content) = file.get("content").and_then(Value::as_str) {
            fs::write(path, content).expect("text fixture");
        } else if let Some(repeat) = file.get("repeat").and_then(Value::as_u64) {
            fs::write(path, vec![b'x'; repeat as usize]).expect("repeated fixture");
        } else {
            let bytes = file["bytes"]
                .as_array()
                .expect("byte fixture")
                .iter()
                .map(|byte| byte.as_u64().expect("byte") as u8)
                .collect::<Vec<_>>();
            fs::write(path, bytes).expect("binary fixture");
        }
    }
}

#[cfg(unix)]
fn create_fixture_symlink(root: &Path) -> bool {
    std::os::unix::fs::symlink("readme.md", root.join("docs/link.md")).expect("fixture symlink");
    true
}

#[cfg(windows)]
fn create_fixture_symlink(root: &Path) -> bool {
    std::os::windows::fs::symlink_file("readme.md", root.join("docs/link.md")).is_ok()
}

#[cfg(not(any(unix, windows)))]
fn create_fixture_symlink(_root: &Path) -> bool {
    false
}

fn run(program: &Path, arguments: &[&str]) -> Output {
    Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_json_error(output: &Output, code: &str) {
    assert!(output.stdout.is_empty());
    let diagnostic: Value = serde_json::from_slice(&output.stderr).expect("JSON diagnostic");
    assert_eq!(diagnostic["error"]["code"], code);
    assert_eq!(diagnostic["error"]["details"], json!({}));
}

fn path_text(path: &Path) -> &str {
    path.to_str().expect("UTF-8 fixture path")
}
