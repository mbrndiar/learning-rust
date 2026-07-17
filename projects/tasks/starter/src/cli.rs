use std::ffi::OsString;
use std::io::{self, Write};
use std::time::Duration;

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};

use crate::client::{DEFAULT_TIMEOUT, TaskClient, normalize_base_url};
use crate::{TaskError, TaskPatch, TaskResult, normalize_patch, normalize_title, validate_id};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_API: i32 = 3;
pub const EXIT_UNEXPECTED_RESPONSE: i32 = 4;
pub const EXIT_CONNECTION: i32 = 5;

#[derive(Debug, Parser)]
#[command(name = "tasks", about = "Call a local Task REST API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8000")]
    pub base_url: String,
    #[arg(long, default_value_t = 5.0)]
    pub timeout: f64,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Add {
        title: String,
    },
    List {
        #[arg(long)]
        completed: Option<bool>,
    },
    Show {
        #[arg(value_parser = parse_positive_id)]
        id: i64,
    },
    Update {
        #[arg(value_parser = parse_positive_id)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        completed: Option<bool>,
    },
    Complete {
        #[arg(value_parser = parse_positive_id)]
        id: i64,
    },
    Remove {
        #[arg(value_parser = parse_positive_id)]
        id: i64,
    },
}

impl Cli {
    pub fn timeout_duration(&self) -> TaskResult<Duration> {
        if !self.timeout.is_finite() || self.timeout <= 0.0 {
            return Err(TaskError::client_configuration(
                "timeout",
                "timeout must be positive and finite",
            ));
        }
        let timeout = Duration::try_from_secs_f64(self.timeout).map_err(|_| {
            TaskError::client_configuration("timeout", "timeout must be positive and finite")
        })?;
        if timeout.is_zero() {
            return Err(TaskError::client_configuration(
                "timeout",
                "timeout must be positive and finite",
            ));
        }
        Ok(timeout)
    }
}

pub async fn run(_cli: Cli) -> TaskResult<()> {
    Err(TaskError::incomplete("tasks command execution"))
}

pub async fn run_process<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    run_from_with_factory(args, TaskClient::new, &mut stdout, &mut stderr).await
}

pub async fn run_from_with_factory<I, T, F, W, E>(
    args: I,
    factory: F,
    stdout: &mut W,
    stderr: &mut E,
) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    F: FnOnce(String, Duration) -> TaskResult<TaskClient>,
    W: Write,
    E: Write,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => run_parsed(cli, factory, stdout, stderr).await,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            let _ = write!(stdout, "{error}");
            EXIT_SUCCESS
        }
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            EXIT_USAGE
        }
    }
}

pub async fn run_parsed<F, W, E>(mut cli: Cli, _factory: F, _stdout: &mut W, stderr: &mut E) -> i32
where
    F: FnOnce(String, Duration) -> TaskResult<TaskClient>,
    W: Write,
    E: Write,
{
    if validate_cli(&mut cli).is_err() {
        return EXIT_USAGE;
    }
    let _ = writeln!(stderr, "transport: incomplete project capability");
    EXIT_CONNECTION
}

fn validate_cli(cli: &mut Cli) -> TaskResult<()> {
    cli.timeout_duration()?;
    cli.base_url = normalize_base_url(&cli.base_url)?;
    match &mut cli.command {
        Command::Add { title } => *title = normalize_title(title)?,
        Command::List { .. } => {}
        Command::Show { id } | Command::Complete { id } | Command::Remove { id } => {
            validate_id(*id)?;
        }
        Command::Update {
            id,
            title,
            completed,
        } => {
            validate_id(*id)?;
            let patch = normalize_patch(TaskPatch {
                title: title.take(),
                completed: *completed,
            })?;
            *title = patch.title;
            *completed = patch.completed;
        }
    }
    Ok(())
}

fn parse_positive_id(raw: &str) -> Result<i64, String> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("ID must be a positive integer".to_owned());
    }
    raw.parse::<i64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| "ID must be a positive integer".to_owned())
}

#[must_use]
pub const fn default_timeout() -> Duration {
    DEFAULT_TIMEOUT
}
