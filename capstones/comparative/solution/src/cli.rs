//! Exact command-line grammar and JSON envelopes.

use crate::store::SqliteStore;
use crate::{
    Command, CommandResult, DeleteExpectation, Key, KvApplication, KvError, Revision,
    SetExpectation, parse_json_value,
};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Shared key/value command-line arguments.
#[derive(Debug, Parser)]
#[command(
    about = "Versioned SQLite-backed configuration store",
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Cli {
    /// Literal SQLite database path.
    #[arg(long, allow_hyphen_values = true)]
    pub db: PathBuf,
    #[command(subcommand)]
    pub command: CliCommand,
}

/// Four commands frozen by the shared specification.
#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Set {
        #[arg(allow_hyphen_values = true)]
        key: String,
        #[arg(long, allow_hyphen_values = true)]
        value_json: String,
        #[arg(long, allow_hyphen_values = true)]
        expect: Option<String>,
    },
    Get {
        #[arg(allow_hyphen_values = true)]
        key: String,
    },
    Delete {
        #[arg(allow_hyphen_values = true)]
        key: String,
        #[arg(long, allow_hyphen_values = true)]
        expect: Option<String>,
    },
    List,
}

/// Complete process result, including the normative exit code.
pub struct ProcessOutput {
    pub stdout: String,
    pub exit_code: u8,
}

/// Parses raw arguments without accepting Clap's convenience aliases or forms.
pub fn parse_exact<I>(arguments: I) -> Result<Cli, KvError>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.into_string().map_err(|_| KvError::Usage))
        .collect::<Result<Vec<_>, _>>()?;
    if arguments.len() < 3 || arguments[0] != "--db" {
        return Err(KvError::Usage);
    }

    let db = PathBuf::from(&arguments[1]);
    let command = match arguments[2].as_str() {
        "list" if arguments.len() == 3 => CliCommand::List,
        "get" if arguments.len() == 4 => CliCommand::Get {
            key: arguments[3].clone(),
        },
        "delete" if arguments.len() == 4 => CliCommand::Delete {
            key: arguments[3].clone(),
            expect: None,
        },
        "delete"
            if arguments.len() == 6
                && arguments.get(4).is_some_and(|value| value == "--expect") =>
        {
            CliCommand::Delete {
                key: arguments[3].clone(),
                expect: Some(arguments[5].clone()),
            }
        }
        "set"
            if arguments.len() == 6
                && arguments
                    .get(4)
                    .is_some_and(|value| value == "--value-json") =>
        {
            CliCommand::Set {
                key: arguments[3].clone(),
                value_json: arguments[5].clone(),
                expect: None,
            }
        }
        "set"
            if arguments.len() == 8
                && arguments
                    .get(4)
                    .is_some_and(|value| value == "--value-json")
                && arguments.get(6).is_some_and(|value| value == "--expect") =>
        {
            CliCommand::Set {
                key: arguments[3].clone(),
                value_json: arguments[5].clone(),
                expect: Some(arguments[7].clone()),
            }
        }
        _ => return Err(KvError::Usage),
    };
    Ok(Cli { db, command })
}

/// Runs raw process arguments and always returns one compact JSON response.
pub fn run_process<I>(arguments: I) -> ProcessOutput
where
    I: IntoIterator<Item = OsString>,
{
    match parse_exact(arguments).and_then(run) {
        Ok(stdout) => ProcessOutput {
            stdout,
            exit_code: 0,
        },
        Err(error) => ProcessOutput {
            stdout: serde_json::to_string(&error.envelope())
                .expect("normative error envelopes are serializable"),
            exit_code: error.exit_code(),
        },
    }
}

/// Runs a parsed command and returns the exact stdout payload without its LF.
pub fn run(cli: Cli) -> Result<String, KvError> {
    validate_db_path(&cli.db)?;
    let command = validate_command(cli.command)?;
    let delete_key = match &command {
        Command::Delete { key, .. } => Some(key.as_str().to_owned()),
        _ => None,
    };
    let store = SqliteStore::open(&cli.db)?;
    let mut application = KvApplication::new(store);
    let result = application.execute(command)?;
    serde_json::to_string(&success_envelope(result, delete_key.as_deref()))
        .map_err(|_| KvError::Storage { operation: "write" })
}

fn validate_db_path(path: &Path) -> Result<(), KvError> {
    let text = path.as_os_str().to_string_lossy();
    if text.is_empty() {
        return Err(KvError::InvalidArgument {
            field: "db",
            reason: "empty",
        });
    }
    if text == ":memory:" || text.starts_with("file:") {
        return Err(KvError::InvalidArgument {
            field: "db",
            reason: "unsupported_form",
        });
    }
    Ok(())
}

fn validate_command(command: CliCommand) -> Result<Command, KvError> {
    match command {
        CliCommand::Set {
            key,
            value_json,
            expect,
        } => {
            let key = Key::parse(&key)?;
            let expectation = parse_set_expectation(expect.as_deref())?;
            let value = parse_json_value(&value_json)?;
            Ok(Command::Set {
                key,
                value,
                expectation,
            })
        }
        CliCommand::Get { key } => Ok(Command::Get {
            key: Key::parse(&key)?,
        }),
        CliCommand::Delete { key, expect } => Ok(Command::Delete {
            key: Key::parse(&key)?,
            expectation: parse_delete_expectation(expect.as_deref())?,
        }),
        CliCommand::List => Ok(Command::List),
    }
}

fn parse_set_expectation(value: Option<&str>) -> Result<SetExpectation, KvError> {
    match value {
        None | Some("any") => Ok(SetExpectation::Any),
        Some("absent") => Ok(SetExpectation::Absent),
        Some(value) => parse_exact_revision(value).map(SetExpectation::Exact),
    }
}

fn parse_delete_expectation(value: Option<&str>) -> Result<DeleteExpectation, KvError> {
    match value {
        None | Some("any") => Ok(DeleteExpectation::Any),
        Some(value) => parse_exact_revision(value).map(DeleteExpectation::Exact),
    }
}

fn parse_exact_revision(value: &str) -> Result<Revision, KvError> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(KvError::InvalidArgument {
            field: "expect",
            reason: "format",
        });
    }
    value
        .parse::<u64>()
        .ok()
        .and_then(|value| Revision::new(value).ok())
        .ok_or(KvError::InvalidArgument {
            field: "expect",
            reason: "format",
        })
}

fn success_envelope(result: CommandResult, delete_key: Option<&str>) -> Value {
    let result = match result {
        CommandResult::Set(result) => json!({
            "key": result.entry.key.as_str(),
            "value": result.entry.value,
            "revision": result.entry.revision.get(),
            "created": result.created,
        }),
        CommandResult::Get(entry) => json!({
            "key": entry.key.as_str(),
            "value": entry.value,
            "revision": entry.revision.get(),
        }),
        CommandResult::Delete(result) => json!({
            "key": delete_key.expect("delete results preserve their command key"),
            "deleted_revision": result.deleted_revision.get(),
            "revision": result.revision.get(),
        }),
        CommandResult::List(result) => json!({
            "entries": result.entries.into_iter().map(|entry| json!({
                "key": entry.key.as_str(),
                "value": entry.value,
                "revision": entry.revision.get(),
            })).collect::<Vec<_>>(),
            "global_revision": result.global_revision,
        }),
    };
    json!({"ok": true, "result": result})
}
