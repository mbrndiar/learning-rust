use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::subject;

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

    assert_incomplete_adapters_are_side_effect_free(api_program, cli_program);
}

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

fn assert_incomplete_adapters_are_side_effect_free(api_program: &Path, cli_program: &Path) {
    let directory = tempfile::tempdir().expect("create isolated smoke directory");
    let sqlite_path = directory.path().join("tasks.db");
    let markdown_path = directory.path().join("tasks.md");
    let sqlite = subject::storage::sqlite::SqliteRepository::new(&sqlite_path);
    let markdown = subject::storage::markdown::MarkdownRepository::new(&markdown_path);
    assert_eq!(sqlite.path(), sqlite_path);
    assert_eq!(markdown.path(), markdown_path);
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

    let api = Command::new(api_program)
        .current_dir(directory.path())
        .output()
        .expect("run incomplete API binary");
    assert!(!api.status.success());
    assert!(String::from_utf8_lossy(&api.stderr).contains("incomplete project capability"));

    let cli = Command::new(cli_program)
        .current_dir(directory.path())
        .output()
        .expect("run incomplete CLI binary");
    assert!(!cli.status.success());
    assert!(String::from_utf8_lossy(&cli.stderr).contains("Usage"));

    assert!(!sqlite_path.exists());
    assert!(!markdown_path.exists());
}
