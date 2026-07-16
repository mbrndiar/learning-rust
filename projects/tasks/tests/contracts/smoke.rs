use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use super::subject;

struct IncompleteRepository;

impl subject::TaskRepository for IncompleteRepository {
    fn create(&self, _title: &str) -> subject::TaskResult<subject::Task> {
        Err(subject::TaskError::incomplete("smoke create"))
    }

    fn list(&self, _filter: subject::TaskFilter) -> subject::TaskResult<Vec<subject::Task>> {
        Err(subject::TaskError::incomplete("smoke list"))
    }

    fn get(&self, _id: u64) -> subject::TaskResult<subject::Task> {
        Err(subject::TaskError::incomplete("smoke get"))
    }

    fn update(&self, _id: u64, _patch: subject::TaskPatch) -> subject::TaskResult<subject::Task> {
        Err(subject::TaskError::incomplete("smoke update"))
    }

    fn delete(&self, _id: u64) -> subject::TaskResult<()> {
        Err(subject::TaskError::incomplete("smoke delete"))
    }
}

pub fn assert_public_boundary(api_program: &Path, cli_program: &Path) {
    let service = subject::TaskService::new(Arc::new(IncompleteRepository));
    let error = service
        .list(subject::TaskFilter::default())
        .expect_err("phase 1 service operations must be explicitly incomplete");
    assert_eq!(error.incomplete_capability(), Some("application list"));

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
