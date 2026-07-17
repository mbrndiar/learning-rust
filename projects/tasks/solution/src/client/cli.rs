//! clap-based CLI: a thin front-end over [`TaskClient`] with a stable exit-code
//! policy.
//!
//! The command mirrors the REST surface (add/list/show/update/complete/remove).
//! Input is validated with the shared domain rules before any request, and
//! outcomes map to fixed exit codes so scripts can branch on them:
//! success `0`, usage `2`, API error `3`, malformed response `4`, connection or
//! timeout `5`. [`run_from_with_factory`] takes a client factory so tests can
//! inject a fake transport; production paths pass [`TaskClient::new`].

use std::ffi::OsString;
use std::io::{self, Write};
use std::time::Duration;

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use serde::Serialize;

use crate::client::http::{DEFAULT_TIMEOUT, TaskClient, normalize_base_url};
use crate::{
    TaskError, TaskFilter, TaskPatch, TaskResult, normalize_patch, normalize_title, validate_id,
};

/// Exit code: the command succeeded.
pub const EXIT_SUCCESS: i32 = 0;
/// Exit code: bad arguments or invalid input (nothing was sent).
pub const EXIT_USAGE: i32 = 2;
/// Exit code: the server returned a well-formed error response.
pub const EXIT_API: i32 = 3;
/// Exit code: the response violated the wire contract.
pub const EXIT_UNEXPECTED_RESPONSE: i32 = 4;
/// Exit code: the request could not reach the server or timed out.
pub const EXIT_CONNECTION: i32 = 5;

const USAGE: &str =
    "usage: tasks [--base-url URL] [--timeout SECONDS] <add|list|show|update|complete|remove> ...";

// Parsed top-level arguments: transport options plus the chosen subcommand.
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

// One CLI subcommand, each corresponding to a REST operation.
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
    /// Converts the `--timeout` seconds into a positive, finite [`Duration`].
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

/// Runs a pre-parsed [`Cli`] against the real client, returning `Ok` only on a
/// success exit code. Provided for callers that already hold a [`Cli`].
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

/// Parses process arguments and runs, returning the exit code; the binary's
/// `main` forwards this to `process::exit`.
pub async fn run_process<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    run_from_with_factory(args, TaskClient::new, &mut stdout, &mut stderr).await
}

/// The DI seam: parses arguments and dispatches, taking the client `factory`
/// and output sinks so tests can inject a fake transport and capture output.
///
/// clap's help/version requests exit successfully; any other parse error prints
/// usage and returns [`EXIT_USAGE`].
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

/// Validates options, builds the client via `factory`, runs the command, and
/// prints the compact JSON result or maps the error to an exit code.
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
    // Validate/normalize input before any network call so bad input never sends.
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

// Applies the shared domain normalization to each command in place, so the rest
// of the CLI works with validated input.
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

// Maps a failure to stderr text and the matching exit code. Order matters:
// API errors and malformed responses are distinguished from transport failures,
// and local validation/config problems collapse back to a usage error.
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

// clap value parser for path-style IDs: accept only ASCII-digit positive
// integers so `--id` matches the server's ID rules before any request.
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

/// The default client timeout, re-exported for callers that build a client
/// without going through the CLI.
#[must_use]
pub const fn default_timeout() -> Duration {
    DEFAULT_TIMEOUT
}
