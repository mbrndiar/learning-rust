//! Shared smoke contract, included verbatim by the starter and solution test
//! crates.
//!
//! Pulled in via `#[path = "../../tests/contracts/smoke.rs"]` and aliased to
//! `super::subject`, this file provides two entry points — one asserting the
//! finished solution's public boundary works, the other asserting the starter
//! scaffold exposes the same surface while remaining explicitly incomplete and
//! free of filesystem side effects. It is fast and never depends on private
//! internals.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::subject;

// Minimal repository double that only counts how many times it was called, used
// to prove whether the service delegates (solution) or short-circuits (starter).
struct SmokeRepository {
    calls: AtomicUsize,
}

impl SmokeRepository {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn record(&self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
    }
}

impl subject::TaskRepository for SmokeRepository {
    fn create(&self, _title: &str) -> subject::TaskResult<subject::Task> {
        self.record();
        Err(subject::TaskError::incomplete("smoke create"))
    }

    fn list(&self, _filter: subject::TaskFilter) -> subject::TaskResult<Vec<subject::Task>> {
        self.record();
        Ok(Vec::new())
    }

    fn get(&self, _id: i64) -> subject::TaskResult<subject::Task> {
        self.record();
        Err(subject::TaskError::incomplete("smoke get"))
    }

    fn update(&self, _id: i64, _patch: subject::TaskPatch) -> subject::TaskResult<subject::Task> {
        self.record();
        Err(subject::TaskError::incomplete("smoke update"))
    }

    fn delete(&self, _id: i64) -> subject::TaskResult<()> {
        self.record();
        Err(subject::TaskError::incomplete("smoke delete"))
    }
}

/// Asserts the completed solution's public boundary: the service delegates to
/// the repository, and the built binaries plus real adapters are wired up.
#[allow(dead_code)]
pub fn assert_solution_public_boundary(api_program: &Path, cli_program: &Path) {
    let repository = Arc::new(SmokeRepository::new());
    let service = subject::TaskService::new(repository.clone());
    assert_eq!(
        service
            .list(subject::TaskFilter::default())
            .expect("milestone 1 list delegates"),
        Vec::<subject::Task>::new()
    );
    assert_eq!(repository.calls(), 1);

    assert_completed_repositories(api_program, cli_program);
}

/// Asserts the starter scaffold: the public surface exists and links, but the
/// service is explicitly incomplete (fails before delegating, so no repository
/// call) and the adapters produce no filesystem side effects.
#[allow(dead_code)]
pub fn assert_starter_public_boundary(api_program: &Path, cli_program: &Path) {
    let repository = Arc::new(SmokeRepository::new());
    let service = subject::TaskService::new(repository.clone());
    let error = service
        .list(subject::TaskFilter::default())
        .expect_err("starter service remains explicitly incomplete");
    assert_eq!(error.incomplete_capability(), Some("application list"));
    assert_eq!(repository.calls(), 0);

    assert_incomplete_adapters_are_side_effect_free(api_program, cli_program);
}

// Incomplete adapters must fail loudly yet touch nothing: no store files are
// created, `--help` still works, and selecting the Actix path errors cleanly.
fn assert_incomplete_adapters_are_side_effect_free(api_program: &Path, cli_program: &Path) {
    let directory = tempfile::tempdir().expect("create isolated smoke directory");
    let sqlite_path = directory.path().join("tasks.db");
    let markdown_path = directory.path().join("tasks.md");
    let sqlite = subject::storage::sqlite::SqliteRepository::open(&sqlite_path)
        .expect_err("starter SQLite remains incomplete");
    let markdown = subject::storage::markdown::MarkdownRepository::open(&markdown_path)
        .expect_err("starter Markdown remains incomplete");
    assert!(sqlite.incomplete_capability().is_some());
    assert!(markdown.incomplete_capability().is_some());
    assert!(!sqlite_path.exists());
    assert!(!markdown_path.exists());

    for program in [api_program, cli_program] {
        let help = Command::new(program)
            .arg("--help")
            .current_dir(directory.path())
            .output()
            .expect("run binary help");
        assert!(help.status.success(), "--help must remain usable");
    }

    let actix_path = directory.path().join("actix-unused.db");
    let api = Command::new(api_program)
        .args([
            "--server",
            "actix",
            "--data",
            actix_path.to_str().expect("UTF-8 test path"),
        ])
        .current_dir(directory.path())
        .output()
        .expect("run incomplete Actix API selection");
    assert!(!api.status.success());
    assert!(String::from_utf8_lossy(&api.stderr).contains("incomplete project capability"));
    assert!(!actix_path.exists());

    let cli = Command::new(cli_program)
        .current_dir(directory.path())
        .output()
        .expect("run incomplete CLI binary");
    assert!(!cli.status.success());
    assert!(
        String::from_utf8_lossy(&cli.stderr)
            .to_ascii_lowercase()
            .contains("usage")
    );

    assert!(!sqlite_path.exists());
    assert!(!markdown_path.exists());
}

// Completed adapters actually initialize their stores (files exist, Markdown
// header is written) and expose the native Actix scope, while the CLI with no
// args still prints usage and exits non-zero.
fn assert_completed_repositories(api_program: &Path, cli_program: &Path) {
    let directory = tempfile::tempdir().expect("create isolated smoke directory");
    let sqlite_path = directory.path().join("tasks.db");
    let markdown_path = directory.path().join("tasks.md");
    let sqlite =
        subject::storage::sqlite::SqliteRepository::open(&sqlite_path).expect("open SQLite");
    let markdown = subject::storage::markdown::MarkdownRepository::open(&markdown_path)
        .expect("open Markdown");
    assert_eq!(sqlite.path(), sqlite_path);
    assert_eq!(markdown.path(), markdown_path);
    assert!(sqlite_path.is_file());
    assert!(markdown_path.is_file());
    assert_eq!(
        std::fs::read_to_string(&markdown_path).expect("read Markdown"),
        "<!-- rest-task-api:v1 next-id=1 -->\n# Tasks\n\n"
    );
    drop((sqlite, markdown));

    for program in [api_program, cli_program] {
        let help = Command::new(program)
            .arg("--help")
            .current_dir(directory.path())
            .output()
            .expect("run binary help");
        assert!(help.status.success(), "--help must remain usable");
    }

    let repository = Arc::new(SmokeRepository::new());
    let service = subject::TaskService::new(repository);
    assert!(
        subject::api::actix::scope(service).is_ok(),
        "solution exposes the native Actix scope"
    );

    let cli = Command::new(cli_program)
        .current_dir(directory.path())
        .output()
        .expect("run incomplete CLI binary");
    assert!(!cli.status.success());
    assert!(
        String::from_utf8_lossy(&cli.stderr)
            .to_ascii_lowercase()
            .contains("usage")
    );
}
