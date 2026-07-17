use std::ffi::OsString;
use std::io::{self, Write};
use std::time::Duration;

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use serde::Serialize;

use crate::client::{DEFAULT_TIMEOUT, TaskClient, normalize_base_url};
use crate::{
    TaskError, TaskFilter, TaskPatch, TaskResult, normalize_patch, normalize_title, validate_id,
};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_API: i32 = 3;
pub const EXIT_UNEXPECTED_RESPONSE: i32 = 4;
pub const EXIT_CONNECTION: i32 = 5;

const USAGE: &str =
    "usage: tasks [--base-url URL] [--timeout SECONDS] <add|list|show|update|complete|remove> ...";

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
        if !self.timeout.is_finite()
            || self.timeout <= 0.0
            || self.timeout > Duration::MAX.as_secs_f64()
        {
            return Err(TaskError::client_configuration(
                "timeout",
                "timeout must be positive and finite",
            ));
        }
        let timeout = Duration::from_secs_f64(self.timeout);
        if timeout.is_zero() {
            return Err(TaskError::client_configuration(
                "timeout",
                "timeout must be positive and finite",
            ));
        }
        Ok(timeout)
    }
}

pub async fn run(cli: Cli) -> TaskResult<()> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let exit = run_parsed(cli, TaskClient::new, &mut stdout, &mut stderr).await;
    if exit == EXIT_SUCCESS {
        Ok(())
    } else {
        Err(TaskError::internal(
            "tasks command",
            io::Error::other(format!("command exited with status {exit}")),
        ))
    }
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
        Err(_) => {
            let _ = writeln!(stderr, "{USAGE}");
            EXIT_USAGE
        }
    }
}

pub async fn run_parsed<F, W, E>(mut cli: Cli, factory: F, stdout: &mut W, stderr: &mut E) -> i32
where
    F: FnOnce(String, Duration) -> TaskResult<TaskClient>,
    W: Write,
    E: Write,
{
    let timeout = match cli.timeout_duration() {
        Ok(timeout) => timeout,
        Err(_) => return usage(stderr),
    };
    cli.base_url = match normalize_base_url(&cli.base_url) {
        Ok(base_url) => base_url,
        Err(_) => return usage(stderr),
    };
    if validate_command(&mut cli.command).is_err() {
        return usage(stderr);
    }
    let client = match factory(cli.base_url, timeout) {
        Ok(client) => client,
        Err(error) => return render_error(&error, stderr),
    };
    let result = execute(&client, &cli.command).await;
    match result {
        Ok(value) => {
            if writeln!(stdout, "{value}").is_err() {
                return transport_failure(stderr);
            }
            EXIT_SUCCESS
        }
        Err(error) => render_error(&error, stderr),
    }
}

fn validate_command(command: &mut Command) -> TaskResult<()> {
    match command {
        Command::Add { title } => {
            *title = normalize_title(title)?;
        }
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

async fn execute(client: &TaskClient, command: &Command) -> TaskResult<String> {
    match command {
        Command::Add { title } => compact(&client.create(title).await?),
        Command::List { completed } => compact(
            &client
                .list(TaskFilter {
                    completed: *completed,
                })
                .await?,
        ),
        Command::Show { id } => compact(&client.get(*id).await?),
        Command::Update {
            id,
            title,
            completed,
        } => compact(
            &client
                .update(
                    *id,
                    TaskPatch {
                        title: title.clone(),
                        completed: *completed,
                    },
                )
                .await?,
        ),
        Command::Complete { id } => compact(
            &client
                .update(
                    *id,
                    TaskPatch {
                        title: None,
                        completed: Some(true),
                    },
                )
                .await?,
        ),
        Command::Remove { id } => {
            client.delete(*id).await?;
            compact(&Deleted { deleted: *id })
        }
    }
}

fn compact(value: &impl Serialize) -> TaskResult<String> {
    serde_json::to_string(value).map_err(|error| TaskError::internal("render client output", error))
}

fn render_error(error: &TaskError, stderr: &mut impl Write) -> i32 {
    if let Some((status, code, message, _)) = error.api_details() {
        let _ = writeln!(stderr, "api: {status} {code}: {message}");
        return EXIT_API;
    }
    if let Some(message) = error.unexpected_response_message() {
        let _ = writeln!(stderr, "malformed-response: {message}");
        return EXIT_UNEXPECTED_RESPONSE;
    }
    if error.is_connection() {
        if error.is_timeout() {
            let _ = writeln!(stderr, "connection: timeout: request timed out");
        } else {
            let _ = writeln!(stderr, "connection: request failed");
        }
        return EXIT_CONNECTION;
    }
    if error.client_configuration_details().is_some() || error.validation_details().is_some() {
        return usage(stderr);
    }
    transport_failure(stderr)
}

fn usage(stderr: &mut impl Write) -> i32 {
    let _ = writeln!(stderr, "{USAGE}");
    EXIT_USAGE
}

fn transport_failure(stderr: &mut impl Write) -> i32 {
    let _ = writeln!(stderr, "transport: request failed");
    EXIT_CONNECTION
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

#[derive(Serialize)]
struct Deleted {
    deleted: i64,
}

#[must_use]
pub const fn default_timeout() -> Duration {
    DEFAULT_TIMEOUT
}
