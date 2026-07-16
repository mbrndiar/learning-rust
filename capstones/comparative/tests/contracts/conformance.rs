use super::subject;
use rusqlite::types::ValueRef;
use serde_json::{Map, Value, json};
use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{Builder, TempDir};

const SPEC_VERSION: &str = "1.0.0";
const PROCESS_TIMEOUT: Duration = Duration::from_secs(15);
static COMMAND_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
struct RunResult {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    envelope: Value,
    duration: Duration,
}

pub fn milestone_1_domain_fixtures() {
    assert_spec_versions();
    let contract = read_fixture("fixtures/contract.json");
    assert_exact_keys(
        object(&contract),
        &[
            "kind",
            "spec_id",
            "spec_version",
            "key_pattern",
            "key_max_bytes",
            "safe_integer_min",
            "safe_integer_max",
            "value_input_max_utf8_bytes",
            "max_container_depth",
            "busy_timeout_ms",
            "commands",
            "set_expectations",
            "delete_expectations",
            "exit_codes",
        ],
    );
    assert_eq!(string(field(&contract, "spec_id")), "comparative-kv");
    assert_eq!(integer(field(&contract, "key_max_bytes")), 128);
    assert_eq!(
        integer(field(&contract, "safe_integer_max")),
        9_007_199_254_740_991
    );
    assert_eq!(
        integer(field(&contract, "value_input_max_utf8_bytes")),
        65_536
    );
    assert_eq!(integer(field(&contract, "max_container_depth")), 32);
    assert_eq!(integer(field(&contract, "busy_timeout_ms")), 10_000);
    assert_eq!(
        field(&contract, "commands"),
        &json!(["set", "get", "delete", "list"])
    );
    assert_eq!(
        field(&contract, "set_expectations"),
        &json!(["any", "absent", "exact_revision"])
    );
    assert_eq!(
        field(&contract, "delete_expectations"),
        &json!(["any", "exact_revision"])
    );
    assert_eq!(
        field(&contract, "exit_codes"),
        &json!({
            "success": 0,
            "validation": 2,
            "conflict": 3,
            "not_found": 4,
            "storage": 5
        })
    );
    let keys = read_fixture("fixtures/keys.json");
    assert_exact_keys(
        object(&keys),
        &["kind", "spec_version", "accepted", "rejected", "ordering"],
    );
    for case in array(field(&keys, "accepted")) {
        assert_exact_keys(object(case), &["id", "key", "key_generator"]);
        let key = generated_key(case);
        let parsed = subject::Key::parse(&key)
            .unwrap_or_else(|error| panic!("accepted key {key:?} failed: {error}"));
        assert_eq!(parsed.as_str(), key);
    }
    for case in array(field(&keys, "rejected")) {
        assert_exact_keys(object(case), &["id", "key", "key_generator"]);
        let key = generated_key(case);
        let error = match subject::Key::parse(&key) {
            Ok(_) => panic!("rejected key {key:?} was accepted"),
            Err(error) => error,
        };
        assert_error(
            &error,
            2,
            "invalid_argument",
            &json!({"field": "key", "reason": "format"}),
        );
    }

    assert!(subject::Revision::new(1).is_ok());
    assert!(subject::Revision::new(9_007_199_254_740_991).is_ok());
    assert!(subject::Revision::new(0).is_err());
    assert!(subject::Revision::new(9_007_199_254_740_992).is_err());

    let accepted = read_fixture("fixtures/values-accepted.json");
    assert_exact_keys(object(&accepted), &["kind", "spec_version", "cases"]);
    for case in array(field(&accepted, "cases")) {
        assert_exact_keys(
            object(case),
            &[
                "id",
                "input_json",
                "input_generator",
                "normalized",
                "normalized_generator",
            ],
        );
        let input = generated_input(case);
        let expected = generated_normalized(case);
        let actual = subject::parse_json_value(&input)
            .unwrap_or_else(|error| panic!("accepted value {input:?} failed: {error}"));
        assert_eq!(
            actual,
            expected,
            "fixture case {}",
            string(field(case, "id"))
        );
    }

    let rejected = read_fixture("fixtures/values-rejected.json");
    assert_exact_keys(object(&rejected), &["kind", "spec_version", "cases"]);
    for case in array(field(&rejected, "cases")) {
        assert_exact_keys(
            object(case),
            &[
                "id",
                "input_json",
                "input_generator",
                "exit",
                "category",
                "details",
            ],
        );
        let input = generated_input(case);
        let error = match subject::parse_json_value(&input) {
            Ok(_) => panic!("rejected value {input:?} was accepted"),
            Err(error) => error,
        };
        assert_error(
            &error,
            integer(field(case, "exit")) as u8,
            string(field(case, "category")),
            field(case, "details"),
        );
    }

    assert_eq!(
        subject::parse_json_value("0e-999999999999999999999")
            .expect("all zero spellings remain exact zero"),
        json!(0)
    );
    assert_eq!(
        subject::parse_json_value(r#"{"x":1.5,"x":1}"#)
            .expect("an overwritten invalid member is not in the normalized tree"),
        json!({"x": 1})
    );
    let syntax_before_value = match subject::parse_json_value("\"\\uD800\" trailing") {
        Ok(_) => panic!("invalid trailing JSON was accepted"),
        Err(error) => error,
    };
    assert_error(
        &syntax_before_value,
        2,
        "invalid_json",
        &json!({"reason": "syntax"}),
    );
}

pub fn milestone_2_cli_and_invalid(program: &Path) {
    run_sequential_fixture(program, "fixtures/scenarios/invalid.json");
    assert_additional_cli_grammar(program);
}

pub fn milestone_3_storage_and_migration(program: &Path) {
    run_sequential_fixture(program, "fixtures/scenarios/normal.json");
    run_sequential_fixture(program, "fixtures/scenarios/migration.json");
    assert_v1_storage_invariants(program);
}

pub fn milestone_4_boundaries_and_mutations(program: &Path) {
    run_sequential_fixture(program, "fixtures/scenarios/boundary.json");
}

pub fn milestone_5_multiprocess(program: &Path) {
    let fixture = read_fixture("fixtures/scenarios/multiprocess.json");
    assert_exact_keys(object(&fixture), &["kind", "spec_version", "scenarios"]);
    assert_eq!(string(field(&fixture, "kind")), "multiprocess_scenarios");
    assert_eq!(string(field(&fixture, "spec_version")), SPEC_VERSION);
    for scenario in array(field(&fixture, "scenarios")) {
        assert_exact_keys(
            object(scenario),
            &["id", "repeat", "database", "setup", "operations"],
        );
        for repetition in 0..integer(field(scenario, "repeat")) {
            run_multiprocess_scenario(program, scenario, repetition);
        }
    }
}

pub fn actor_process() {
    let ready = required_env_path("KV_ACTOR_READY");
    let release = required_env_path("KV_ACTOR_RELEASE");
    let stdout = required_env_path("KV_ACTOR_STDOUT");
    let stderr = required_env_path("KV_ACTOR_STDERR");
    let program = required_env_path("KV_ACTOR_PROGRAM");
    let args: Vec<String> =
        serde_json::from_str(&std::env::var("KV_ACTOR_ARGS").expect("KV_ACTOR_ARGS must be set"))
            .expect("KV_ACTOR_ARGS must contain a string array");
    File::create(&ready).expect("signal actor ready");
    wait_for_file(&release, Duration::from_secs(30));

    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            File::create(stdout).expect("create actor stdout"),
        ))
        .stderr(Stdio::from(
            File::create(stderr).expect("create actor stderr"),
        ));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        panic!("exec actor program: {error}");
    }
    #[cfg(not(unix))]
    {
        let status = command.status().expect("run actor program");
        std::process::exit(status.code().unwrap_or(1));
    }
}

pub fn lock_helper_process() {
    let database = required_env_path("KV_LOCK_DATABASE");
    let ready = required_env_path("KV_LOCK_READY");
    let release = required_env_path("KV_LOCK_RELEASE");
    let connection = rusqlite::Connection::open(database).expect("open lock-helper database");
    connection
        .busy_timeout(Duration::from_secs(10))
        .expect("configure lock-helper timeout");
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .expect("acquire immediate lock");
    File::create(&ready).expect("signal lock helper ready");
    wait_for_file(&release, Duration::from_secs(30));
    connection
        .execute_batch("ROLLBACK")
        .expect("rollback lock helper");
}

fn assert_spec_versions() {
    let version = fs::read_to_string(spec_path("SPEC_VERSION")).expect("read SPEC_VERSION");
    assert_eq!(version, format!("{SPEC_VERSION}\n"));
    for path in [
        "fixtures/contract.json",
        "fixtures/keys.json",
        "fixtures/values-accepted.json",
        "fixtures/values-rejected.json",
        "fixtures/scenarios/normal.json",
        "fixtures/scenarios/boundary.json",
        "fixtures/scenarios/invalid.json",
        "fixtures/scenarios/migration.json",
        "fixtures/scenarios/multiprocess.json",
    ] {
        let fixture = read_fixture(path);
        assert_eq!(string(field(&fixture, "spec_version")), SPEC_VERSION);
    }
}

fn run_multiprocess_scenario(program: &Path, scenario: &Value, repetition: i64) {
    let id = string(field(scenario, "id"));
    let directory = scenario_directory(&format!("{id}-{repetition}"));
    let database = directory.path().join("store.db");
    let missing_parent = directory.path().join("missing-parent").join("child");
    match string(field(scenario, "database")) {
        "fresh" => {}
        "sqlite_setup" => setup_database(&database, field(scenario, "setup")),
        other => panic!("unknown database kind {other}"),
    }

    let mut captures = HashMap::<String, Value>::new();
    let mut locks = HashMap::<String, LockHelper>::new();
    let mut running = HashMap::<String, RunningCli>::new();
    for operation in array(field(scenario, "operations")) {
        let operation = object(operation);
        assert_eq!(operation.len(), 1, "parallel operations have one kind");
        if let Some(parallel) = operation.get("parallel") {
            run_parallel_group(program, &database, directory.path(), parallel);
        } else if let Some(run_assert) = operation.get("run_assert") {
            run_single_assert(
                program,
                &database,
                &missing_parent,
                directory.path(),
                run_assert,
                &mut captures,
            );
        } else if let Some(start) = operation.get("start_lock_helper") {
            assert_exact_keys(object(start), &["id"]);
            let lock_id = string(field(start, "id")).to_owned();
            assert!(
                locks
                    .insert(
                        lock_id.clone(),
                        LockHelper::start(&database, directory.path(), &lock_id),
                    )
                    .is_none(),
                "duplicate lock helper {lock_id}"
            );
        } else if let Some(start) = operation.get("start_cli") {
            assert_exact_keys(object(start), &["id", "args"]);
            let cli_id = string(field(start, "id")).to_owned();
            let args = substituted_args(field(start, "args"), &database, &missing_parent, None);
            assert!(
                running
                    .insert(
                        cli_id.clone(),
                        RunningCli::start(program, &args, directory.path(), &cli_id),
                    )
                    .is_none(),
                "duplicate running CLI {cli_id}"
            );
        } else if let Some(value) = operation.get("sleep_ms") {
            thread::sleep(Duration::from_millis(integer(value) as u64));
        } else if let Some(release) = operation.get("release_lock_helper") {
            assert_exact_keys(object(release), &["id"]);
            let lock_id = string(field(release, "id"));
            locks
                .remove(lock_id)
                .unwrap_or_else(|| panic!("unknown lock helper {lock_id}"))
                .release();
        } else if let Some(await_cli) = operation.get("await_cli") {
            assert_exact_keys(object(await_cli), &["id", "expect", "assert"]);
            let cli_id = string(field(await_cli, "id"));
            let result = running
                .remove(cli_id)
                .unwrap_or_else(|| panic!("unknown running CLI {cli_id}"))
                .finish(Duration::from_secs(20));
            assert_optional_expectation(&result, field(await_cli, "expect"));
            if let Some(assertion) = object(await_cli).get("assert") {
                assert_duration(&result, assertion);
            }
        } else {
            panic!("unknown multiprocess operation: {operation:?}");
        }
    }
    assert!(running.is_empty(), "all started CLIs must be awaited");
    assert!(locks.is_empty(), "all lock helpers must be released");
    if database.exists() {
        assert_integrity(&database);
    }
    cleanup_database(&database);
    directory
        .close()
        .expect("remove multiprocess scenario directory");
}

fn run_parallel_group(program: &Path, database: &Path, output_directory: &Path, parallel: &Value) {
    assert_exact_keys(object(parallel), &["actors_generator", "assert"]);
    let generator = field(parallel, "actors_generator");
    assert_exact_keys(object(generator), &["kind", "count", "pad_width", "args"]);
    assert_eq!(string(field(generator, "kind")), "indexed_commands");
    let count = integer(field(generator, "count")) as usize;
    let pad_width = generator.get("pad_width").map(integer).unwrap_or_default() as usize;
    let group_id = COMMAND_ID.fetch_add(1, Ordering::Relaxed);
    let release = output_directory.join(format!("parallel-{group_id}.release"));
    let current_test = std::env::current_exe().expect("locate conformance test process");
    let mut actors = Vec::with_capacity(count);
    for index in 0..count {
        let number = index + 1;
        let mut replacements = Map::new();
        replacements.insert("i".into(), Value::String(index.to_string()));
        replacements.insert("n".into(), Value::String(number.to_string()));
        replacements.insert(
            "padded_n".into(),
            Value::String(format!("{number:0pad_width$}")),
        );
        let args = substituted_args(
            field(generator, "args"),
            database,
            &output_directory.join("missing"),
            Some(&replacements),
        );
        let ready = output_directory.join(format!("parallel-{group_id}-{index}.ready"));
        let stdout = output_directory.join(format!("parallel-{group_id}-{index}.stdout"));
        let stderr = output_directory.join(format!("parallel-{group_id}-{index}.stderr"));
        let child = Command::new(&current_test)
            .args([
                "--ignored",
                "--exact",
                "conformance_actor_process",
                "--nocapture",
            ])
            .env("KV_ACTOR_READY", &ready)
            .env("KV_ACTOR_RELEASE", &release)
            .env("KV_ACTOR_STDOUT", &stdout)
            .env("KV_ACTOR_STDERR", &stderr)
            .env("KV_ACTOR_PROGRAM", program)
            .env(
                "KV_ACTOR_ARGS",
                serde_json::to_string(&args).expect("serialize actor args"),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn barrier actor");
        actors.push(BarrierActor {
            child: ManagedChild::new(child),
            args,
            ready,
            stdout,
            stderr,
        });
    }
    for actor in &actors {
        wait_for_file(&actor.ready, Duration::from_secs(15));
    }
    File::create(&release).expect("release actor barrier");
    let start = Instant::now();
    let mut results = Vec::with_capacity(count);
    for mut actor in actors {
        let status = actor.child.wait(Duration::from_secs(30));
        let stdout = fs::read_to_string(&actor.stdout).expect("read actor stdout");
        let stderr = fs::read_to_string(&actor.stderr).expect("read actor stderr");
        let envelope = parse_stdout(&stdout);
        assert_command_result_shape(&actor.args, &envelope);
        results.push(ActorResult {
            args: actor.args,
            result: RunResult {
                status,
                envelope,
                stdout,
                stderr,
                duration: start.elapsed(),
            },
        });
        remove_file(&actor.ready);
        remove_file(&actor.stdout);
        remove_file(&actor.stderr);
    }
    remove_file(&release);
    assert_parallel(
        program,
        database,
        output_directory,
        &results,
        field(parallel, "assert"),
    );
}

fn assert_parallel(
    program: &Path,
    database: &Path,
    output_directory: &Path,
    actors: &[ActorResult],
    assertion: &Value,
) {
    assert_exact_keys(
        object(assertion),
        &[
            "all_exit",
            "all_ok",
            "stdout_semantic_all",
            "success_count",
            "category_counts",
            "result_revision_set",
            "success_revision",
            "conflict_actual",
            "not_found_count",
            "winner_value_matches_final",
            "duration_less_than_ms",
            "duration_at_least_ms",
        ],
    );
    for actor in actors {
        assert_eq!(actor.result.stderr, "", "{actor:?}");
    }
    if let Some(exit) = assertion.get("all_exit") {
        for actor in actors {
            assert_eq!(
                actor.result.status.code(),
                Some(integer(exit) as i32),
                "{actor:?}"
            );
        }
    }
    if let Some(ok) = assertion.get("all_ok") {
        for actor in actors {
            assert_eq!(field(&actor.result.envelope, "ok"), ok);
        }
    }
    if let Some(expected) = assertion.get("stdout_semantic_all") {
        for actor in actors {
            assert_eq!(&actor.result.envelope, expected);
        }
    }
    let successes = actors
        .iter()
        .filter(|actor| field(&actor.result.envelope, "ok") == &Value::Bool(true))
        .collect::<Vec<_>>();
    if let Some(count) = assertion.get("success_count") {
        assert_eq!(successes.len(), integer(count) as usize);
    }
    if let Some(expected_counts) = assertion.get("category_counts") {
        let mut counts = HashMap::<&str, usize>::new();
        for actor in actors
            .iter()
            .filter(|actor| field(&actor.result.envelope, "ok") == &Value::Bool(false))
        {
            *counts
                .entry(string(field(
                    field(&actor.result.envelope, "error"),
                    "category",
                )))
                .or_default() += 1;
        }
        for (category, expected) in object(expected_counts) {
            assert_eq!(
                counts.get(category.as_str()).copied().unwrap_or_default(),
                integer(expected) as usize
            );
        }
        assert_eq!(counts.len(), object(expected_counts).len());
    }
    if let Some(range) = assertion.get("result_revision_set") {
        let revisions = successes
            .iter()
            .map(|actor| integer(field(field(&actor.result.envelope, "result"), "revision")))
            .collect::<BTreeSet<_>>();
        assert_integer_range(&revisions, range);
    }
    if let Some(revision) = assertion.get("success_revision") {
        for actor in &successes {
            assert_eq!(
                field(field(&actor.result.envelope, "result"), "revision"),
                revision
            );
        }
    }
    if let Some(actual) = assertion.get("conflict_actual") {
        for actor in actors
            .iter()
            .filter(|actor| error_category(&actor.result.envelope) == Some("conflict"))
        {
            assert_eq!(
                field(
                    field(field(&actor.result.envelope, "error"), "details"),
                    "actual"
                ),
                actual
            );
        }
    }
    if let Some(count) = assertion.get("not_found_count") {
        let actual = actors
            .iter()
            .filter(|actor| error_category(&actor.result.envelope) == Some("not_found"))
            .count();
        assert_eq!(actual, integer(count) as usize);
    }
    if assertion
        .get("winner_value_matches_final")
        .is_some_and(|value| value == &Value::Bool(true))
    {
        assert_eq!(successes.len(), 1);
        let winner = successes[0];
        let key_index = winner
            .args
            .iter()
            .position(|argument| argument == "set")
            .expect("winner set command")
            + 1;
        let value_index = winner
            .args
            .iter()
            .position(|argument| argument == "--value-json")
            .expect("winner value argument")
            + 1;
        let final_value = run_program(
            program,
            &[
                "--db".into(),
                path_string(database),
                "get".into(),
                winner.args[key_index].clone(),
            ],
            output_directory,
            PROCESS_TIMEOUT,
        );
        let expected = subject::parse_json_value(&winner.args[value_index])
            .expect("winner argument is valid JSON");
        assert_eq!(
            field(field(&final_value.envelope, "result"), "value"),
            &expected
        );
    }
    assert_duration_set(actors.iter().map(|actor| &actor.result), assertion);
}

fn run_single_assert(
    program: &Path,
    database: &Path,
    missing_parent: &Path,
    output_directory: &Path,
    run_assert: &Value,
    captures: &mut HashMap<String, Value>,
) {
    assert_exact_keys(object(run_assert), &["args", "expect", "assert", "capture"]);
    let args = substituted_args(field(run_assert, "args"), database, missing_parent, None);
    let result = run_program(program, &args, output_directory, PROCESS_TIMEOUT);
    assert_optional_expectation(&result, field(run_assert, "expect"));
    if let Some(assertion) = object(run_assert).get("assert") {
        assert_structural(&result, assertion, captures);
    }
    if let Some(capture) = object(run_assert).get("capture") {
        captures.insert(
            string(capture).to_owned(),
            field(&result.envelope, "result").clone(),
        );
    }
}

fn assert_optional_expectation(result: &RunResult, expectation: &Value) {
    assert_exact_keys(object(expectation), &["exit", "stdout", "stderr"]);
    assert_eq!(
        result.status.code(),
        Some(integer(field(expectation, "exit")) as i32),
        "{result:?}"
    );
    assert_eq!(result.stderr, string(field(expectation, "stderr")));
    if let Some(stdout) = object(expectation).get("stdout") {
        assert_eq!(&result.envelope, stdout);
    }
}

fn assert_structural(result: &RunResult, assertion: &Value, captures: &HashMap<String, Value>) {
    assert_exact_keys(
        object(assertion),
        &[
            "keys_in_order",
            "global_revision",
            "entry_count",
            "entry_revision_set",
            "values_by_key",
            "revision_by_key",
            "state_unchanged_from",
            "duration_less_than_ms",
            "duration_at_least_ms",
        ],
    );
    let result_value = field(&result.envelope, "result");
    let entries = result_value.get("entries").map(array);
    if let Some(keys) = assertion.get("keys_in_order") {
        let actual = entries
            .expect("keys assertion requires entries")
            .iter()
            .map(|entry| field(entry, "key").clone())
            .collect::<Vec<_>>();
        assert_eq!(Value::Array(actual), *keys);
    }
    if let Some(revision) = assertion.get("global_revision") {
        assert_eq!(field(result_value, "global_revision"), revision);
    }
    if let Some(count) = assertion.get("entry_count") {
        assert_eq!(
            entries.expect("entry count requires entries").len(),
            integer(count) as usize
        );
    }
    if let Some(range) = assertion.get("entry_revision_set") {
        let revisions = entries
            .expect("revision set requires entries")
            .iter()
            .map(|entry| integer(field(entry, "revision")))
            .collect::<BTreeSet<_>>();
        assert_integer_range(&revisions, range);
    }
    if let Some(values) = assertion.get("values_by_key") {
        let by_key = entries_by_key(entries.expect("values require entries"));
        for (key, value) in object(values) {
            assert_eq!(field(by_key[key.as_str()], "value"), value);
        }
    }
    if let Some(revisions) = assertion.get("revision_by_key") {
        let by_key = entries_by_key(entries.expect("revisions require entries"));
        for (key, revision) in object(revisions) {
            assert_eq!(field(by_key[key.as_str()], "revision"), revision);
        }
    }
    if let Some(capture) = assertion.get("state_unchanged_from") {
        assert_eq!(
            result_value,
            captures
                .get(string(capture))
                .unwrap_or_else(|| panic!("unknown capture {}", string(capture)))
        );
    }
    assert_duration(result, assertion);
}

fn entries_by_key(entries: &[Value]) -> HashMap<&str, &Value> {
    entries
        .iter()
        .map(|entry| (string(field(entry, "key")), entry))
        .collect()
}

fn error_category(envelope: &Value) -> Option<&str> {
    envelope
        .get("error")
        .and_then(|error| error.get("category"))
        .and_then(Value::as_str)
}

fn assert_integer_range(actual: &BTreeSet<i64>, range: &Value) {
    assert_exact_keys(object(range), &["from", "to"]);
    let expected =
        (integer(field(range, "from"))..=integer(field(range, "to"))).collect::<BTreeSet<_>>();
    assert_eq!(actual, &expected);
}

fn assert_duration(result: &RunResult, assertion: &Value) {
    assert_duration_set(std::iter::once(result), assertion);
}

fn assert_duration_set<'a>(results: impl IntoIterator<Item = &'a RunResult>, assertion: &Value) {
    let results = results.into_iter().collect::<Vec<_>>();
    if let Some(limit) = assertion.get("duration_less_than_ms") {
        for result in &results {
            assert!(
                result.duration < Duration::from_millis(integer(limit) as u64),
                "{:?} is not less than {limit}",
                result.duration
            );
        }
    }
    if let Some(limit) = assertion.get("duration_at_least_ms") {
        for result in &results {
            assert!(
                result.duration >= Duration::from_millis(integer(limit) as u64),
                "{:?} is less than {limit}",
                result.duration
            );
        }
    }
}

#[derive(Debug)]
struct ActorResult {
    args: Vec<String>,
    result: RunResult,
}

struct BarrierActor {
    child: ManagedChild,
    args: Vec<String>,
    ready: PathBuf,
    stdout: PathBuf,
    stderr: PathBuf,
}

struct RunningCli {
    child: ManagedChild,
    args: Vec<String>,
    stdout: PathBuf,
    stderr: PathBuf,
    started: Instant,
}

impl RunningCli {
    fn start(program: &Path, args: &[String], directory: &Path, id: &str) -> Self {
        let unique = COMMAND_ID.fetch_add(1, Ordering::Relaxed);
        let stdout = directory.join(format!("running-{id}-{unique}.stdout"));
        let stderr = directory.join(format!("running-{id}-{unique}.stderr"));
        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::from(
                File::create(&stdout).expect("create running stdout"),
            ))
            .stderr(Stdio::from(
                File::create(&stderr).expect("create running stderr"),
            ))
            .spawn()
            .expect("start asynchronous CLI");
        Self {
            child: ManagedChild::new(child),
            args: args.to_vec(),
            stdout,
            stderr,
            started: Instant::now(),
        }
    }

    fn finish(mut self, timeout: Duration) -> RunResult {
        let status = self.child.wait(timeout);
        let duration = self.started.elapsed();
        let stdout = fs::read_to_string(&self.stdout).expect("read running stdout");
        let stderr = fs::read_to_string(&self.stderr).expect("read running stderr");
        remove_file(&self.stdout);
        remove_file(&self.stderr);
        let envelope = parse_stdout(&stdout);
        assert_command_result_shape(&self.args, &envelope);
        RunResult {
            status,
            envelope,
            stdout,
            stderr,
            duration,
        }
    }
}

struct LockHelper {
    child: ManagedChild,
    ready: PathBuf,
    release: PathBuf,
}

impl LockHelper {
    fn start(database: &Path, directory: &Path, id: &str) -> Self {
        let unique = COMMAND_ID.fetch_add(1, Ordering::Relaxed);
        let ready = directory.join(format!("lock-{id}-{unique}.ready"));
        let release = directory.join(format!("lock-{id}-{unique}.release"));
        let child = Command::new(std::env::current_exe().expect("locate test executable"))
            .args([
                "--ignored",
                "--exact",
                "conformance_lock_helper_process",
                "--nocapture",
            ])
            .env("KV_LOCK_DATABASE", database)
            .env("KV_LOCK_READY", &ready)
            .env("KV_LOCK_RELEASE", &release)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn lock helper");
        let helper = Self {
            child: ManagedChild::new(child),
            ready,
            release,
        };
        wait_for_file(&helper.ready, Duration::from_secs(15));
        helper
    }

    fn release(mut self) {
        File::create(&self.release).expect("signal lock helper release");
        let status = self.child.wait(Duration::from_secs(15));
        assert!(status.success(), "lock helper failed: {status}");
        remove_file(&self.ready);
        remove_file(&self.release);
    }
}

struct ManagedChild {
    child: Child,
    completed: bool,
}

impl ManagedChild {
    fn new(child: Child) -> Self {
        Self {
            child,
            completed: false,
        }
    }

    fn wait(&mut self, timeout: Duration) -> ExitStatus {
        let status = wait_for_child(&mut self.child, timeout);
        self.completed = true;
        status
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if !self.completed {
            match self.child.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                }
            }
        }
    }
}

fn wait_for_file(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(5));
    }
}

fn required_env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("{name} must be set")))
}

fn remove_file(path: &Path) {
    if path.exists() {
        fs::remove_file(path).unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
    }
}

fn run_sequential_fixture(program: &Path, relative_path: &str) {
    let fixture = read_fixture(relative_path);
    assert_exact_keys(object(&fixture), &["kind", "spec_version", "scenarios"]);
    assert_eq!(string(field(&fixture, "kind")), "sequential_scenarios");
    assert_eq!(string(field(&fixture, "spec_version")), SPEC_VERSION);

    for scenario in array(field(&fixture, "scenarios")) {
        assert_exact_keys(object(scenario), &["id", "database", "setup", "steps"]);
        run_sequential_scenario(program, scenario);
    }
}

fn assert_additional_cli_grammar(program: &Path) {
    let directory = scenario_directory("exact-cli");
    let database = directory.path().join("store=with-equals.db");
    let database_text = path_string(&database);
    for args in [
        vec![format!("--db={database_text}"), "list".into()],
        vec![
            "--db".into(),
            database_text.clone(),
            "list".into(),
            "extra".into(),
        ],
        vec![
            "--db".into(),
            database_text.clone(),
            "set".into(),
            "key".into(),
            "--value-json=1".into(),
        ],
        vec![
            "--db".into(),
            database_text.clone(),
            "set".into(),
            "key".into(),
            "--value-json".into(),
            "1".into(),
            "--expect=any".into(),
        ],
    ] {
        let result = run_program(program, &args, directory.path(), PROCESS_TIMEOUT);
        assert_standard(
            &result,
            2,
            &json!({
                "ok": false,
                "error": {"category": "usage", "details": {"reason": "invalid_cli"}}
            }),
            "",
        );
        assert!(!database.exists());
    }

    let valid = run_program(
        program,
        &[
            "--db".into(),
            database_text,
            "set".into(),
            "equals".into(),
            "--value-json".into(),
            "\"a=b\"".into(),
        ],
        directory.path(),
        PROCESS_TIMEOUT,
    );
    assert_eq!(valid.status.code(), Some(0), "{valid:?}");
    assert_eq!(
        field(field(&valid.envelope, "result"), "value"),
        &json!("a=b")
    );
    cleanup_database(&database);
    directory.close().expect("remove exact CLI directory");
}

fn assert_v1_storage_invariants(program: &Path) {
    let cases = [
        (
            "invalid-key",
            vec![
                "INSERT INTO store_metadata(singleton, schema_version, global_revision) VALUES (1, 1, 1)",
                "INSERT INTO entries(key, value_json, revision) VALUES ('-bad', 'null', 1)",
            ],
            json!({"reason": "invalid_key", "key": "-bad"}),
        ),
        (
            "invalid-value",
            vec![
                "INSERT INTO store_metadata(singleton, schema_version, global_revision) VALUES (1, 1, 1)",
                "INSERT INTO entries(key, value_json, revision) VALUES ('good', '1.5', 1)",
            ],
            json!({"reason": "invalid_value", "key": "good"}),
        ),
        (
            "duplicate-revision",
            vec![
                "INSERT INTO store_metadata(singleton, schema_version, global_revision) VALUES (1, 1, 1)",
                "INSERT INTO entries(key, value_json, revision) VALUES ('a', 'null', 1)",
                "INSERT INTO entries(key, value_json, revision) VALUES ('b', 'true', 1)",
            ],
            json!({"reason": "revision_invariant"}),
        ),
        (
            "revision-ahead",
            vec![
                "INSERT INTO store_metadata(singleton, schema_version, global_revision) VALUES (1, 1, 1)",
                "INSERT INTO entries(key, value_json, revision) VALUES ('a', 'null', 2)",
            ],
            json!({"reason": "revision_invariant"}),
        ),
    ];

    for (id, statements, details) in cases {
        let directory = scenario_directory(id);
        let database = directory.path().join("store.db");
        let connection = rusqlite::Connection::open(&database).expect("open v1 invariant database");
        create_v1_tables(&connection);
        for statement in statements {
            connection.execute(statement, []).expect("seed invalid v1");
        }
        drop(connection);
        let result = run_program(
            program,
            &["--db".into(), path_string(&database), "list".into()],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_standard(
            &result,
            5,
            &json!({
                "ok": false,
                "error": {"category": "invalid_storage", "details": details}
            }),
            "",
        );
        assert_integrity(&database);
        cleanup_database(&database);
        directory.close().expect("remove v1 invariant directory");
    }

    let directory = scenario_directory("nondefault-pragmas");
    let database = directory.path().join("store.db");
    let connection = rusqlite::Connection::open(&database).expect("open pragma database");
    connection
        .execute_batch("PRAGMA user_version = 7;")
        .expect("set user version");
    create_v1_tables(&connection);
    connection
            .execute(
                "INSERT INTO store_metadata(singleton, schema_version, global_revision) VALUES (1, 1, 0)",
                [],
            )
            .expect("seed metadata");
    drop(connection);
    let result = run_program(
        program,
        &["--db".into(), path_string(&database), "list".into()],
        directory.path(),
        PROCESS_TIMEOUT,
    );
    assert_standard(
        &result,
        5,
        &json!({
            "ok": false,
            "error": {
                "category": "invalid_storage",
                "details": {"reason": "malformed_schema"}
            }
        }),
        "",
    );
    cleanup_database(&database);
    directory
        .close()
        .expect("remove nondefault pragma directory");
}

fn create_v1_tables(connection: &rusqlite::Connection) {
    connection
        .execute_batch(
            "CREATE TABLE store_metadata (
                    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
                    global_revision INTEGER NOT NULL
                        CHECK (global_revision BETWEEN 0 AND 9007199254740991)
                 );
                 CREATE TABLE entries (
                    key TEXT PRIMARY KEY COLLATE BINARY,
                    value_json TEXT NOT NULL CHECK (json_valid(value_json)),
                    revision INTEGER NOT NULL
                        CHECK (revision BETWEEN 1 AND 9007199254740991)
                 );",
        )
        .expect("create exact v1 tables");
}

fn run_sequential_scenario(program: &Path, scenario: &Value) {
    let id = string(field(scenario, "id"));
    let directory = scenario_directory(id);
    let database = directory.path().join("store.db");
    let missing_parent = directory.path().join("missing-parent").join("child");
    match string(field(scenario, "database")) {
        "fresh" => {}
        "sqlite_setup" => setup_database(&database, field(scenario, "setup")),
        other => panic!("unknown database kind {other}"),
    }

    for step in array(field(scenario, "steps")) {
        let step = object(step);
        if let Some(references) = step.get("fixture_references") {
            assert_exact_keys(step, &["fixture_references"]);
            for reference in array(references) {
                match string(reference) {
                    "../keys.json" => run_key_cases_cli(program),
                    "../values-accepted.json" => run_accepted_value_cases_cli(program),
                    "../values-rejected.json" => run_rejected_value_cases_cli(program),
                    other => panic!("unknown fixture reference {other}"),
                }
            }
        } else if let Some(run) = step.get("run") {
            assert_exact_keys(step, &["run", "expect"]);
            assert_exact_keys(object(run), &["args"]);
            let args = substituted_args(field(run, "args"), &database, &missing_parent, None);
            let result = run_program(program, &args, directory.path(), PROCESS_TIMEOUT);
            assert_expectation(
                &result,
                field(step.get("expect").expect("run step expect"), "exit"),
                step.get("expect").expect("run step expect"),
            );
        } else if let Some(assertion) = step.get("sqlite_assert") {
            assert_exact_keys(step, &["sqlite_assert"]);
            run_sqlite_assertions(&database, assertion);
        } else {
            panic!("unknown sequential operation: {step:?}");
        }
    }

    if database.exists() {
        assert_integrity(&database);
    }
    cleanup_database(&database);
    directory
        .close()
        .expect("remove sequential scenario directory");
}

fn run_key_cases_cli(program: &Path) {
    let fixture = read_fixture("fixtures/keys.json");
    for case in array(field(&fixture, "accepted")) {
        let directory = scenario_directory("accepted-key");
        let database = directory.path().join("store.db");
        let key = generated_key(case);
        let set = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "set".into(),
                key.clone(),
                "--value-json".into(),
                "null".into(),
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_eq!(set.status.code(), Some(0));
        let get = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "get".into(),
                key.clone(),
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_eq!(get.status.code(), Some(0));
        assert_eq!(field(field(&get.envelope, "result"), "key"), &json!(key));
        cleanup_database(&database);
        directory.close().expect("remove accepted-key directory");
    }

    for case in array(field(&fixture, "rejected")) {
        let directory = scenario_directory("rejected-key");
        let database = directory.path().join("store.db");
        let key = generated_key(case);
        let result = run_program(
            program,
            &["--db".into(), path_string(&database), "get".into(), key],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_standard(
            &result,
            2,
            &json!({
                "ok": false,
                "error": {
                    "category": "invalid_argument",
                    "details": {"field": "key", "reason": "format"}
                }
            }),
            "",
        );
        assert!(!database.exists(), "invalid key must not create storage");
        directory.close().expect("remove rejected-key directory");
    }

    let directory = scenario_directory("key-ordering");
    let database = directory.path().join("store.db");
    let ordering = array(field(&fixture, "ordering"));
    for key in ordering.iter().rev() {
        let result = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "set".into(),
                string(key).into(),
                "--value-json".into(),
                "null".into(),
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_eq!(result.status.code(), Some(0));
    }
    let listed = run_program(
        program,
        &["--db".into(), path_string(&database), "list".into()],
        directory.path(),
        PROCESS_TIMEOUT,
    );
    let actual = array(field(field(&listed.envelope, "result"), "entries"))
        .iter()
        .map(|entry| string(field(entry, "key")))
        .collect::<Vec<_>>();
    let expected = ordering.iter().map(string).collect::<Vec<_>>();
    assert_eq!(actual, expected);
    cleanup_database(&database);
    directory.close().expect("remove key-ordering directory");
}

fn run_accepted_value_cases_cli(program: &Path) {
    let fixture = read_fixture("fixtures/values-accepted.json");
    for case in array(field(&fixture, "cases")) {
        let directory = scenario_directory("accepted-value");
        let database = directory.path().join("store.db");
        let input = generated_input(case);
        let expected = generated_normalized(case);
        let set = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "set".into(),
                "value".into(),
                "--value-json".into(),
                input,
                "--expect".into(),
                "absent".into(),
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_eq!(set.status.code(), Some(0));
        assert_eq!(field(field(&set.envelope, "result"), "value"), &expected);
        assert_eq!(field(field(&set.envelope, "result"), "revision"), &json!(1));
        assert_eq!(
            field(field(&set.envelope, "result"), "created"),
            &json!(true)
        );
        let get = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "get".into(),
                "value".into(),
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_eq!(field(field(&get.envelope, "result"), "value"), &expected);
        cleanup_database(&database);
        directory.close().expect("remove accepted-value directory");
    }
}

fn run_rejected_value_cases_cli(program: &Path) {
    let fixture = read_fixture("fixtures/values-rejected.json");
    for case in array(field(&fixture, "cases")) {
        let directory = scenario_directory("rejected-value");
        let database = directory.path().join("store.db");
        let input = generated_input(case);
        let result = run_program(
            program,
            &[
                "--db".into(),
                path_string(&database),
                "set".into(),
                "value".into(),
                "--value-json".into(),
                input,
            ],
            directory.path(),
            PROCESS_TIMEOUT,
        );
        assert_standard(
            &result,
            integer(field(case, "exit")) as i32,
            &json!({
                "ok": false,
                "error": {
                    "category": string(field(case, "category")),
                    "details": field(case, "details"),
                }
            }),
            "",
        );
        assert!(!database.exists(), "invalid value must not create storage");
        directory.close().expect("remove rejected-value directory");
    }
}

fn setup_database(database: &Path, setup: &Value) {
    assert_exact_keys(object(setup), &["statements"]);
    let connection = rusqlite::Connection::open(database).expect("open fixture setup database");
    for statement in array(field(setup, "statements")) {
        connection
            .execute(string(statement), [])
            .unwrap_or_else(|error| panic!("execute fixture SQL {statement}: {error}"));
    }
}

fn run_sqlite_assertions(database: &Path, assertion: &Value) {
    assert_exact_keys(object(assertion), &["queries"]);
    let connection = rusqlite::Connection::open(database).expect("open assertion database");
    for query in array(field(assertion, "queries")) {
        assert_exact_keys(object(query), &["sql", "rows"]);
        let mut statement = connection
            .prepare(string(field(query, "sql")))
            .expect("prepare assertion query");
        let column_count = statement.column_count();
        let rows = statement
            .query_map([], |row| {
                (0..column_count)
                    .map(|index| sqlite_value(row.get_ref(index)?))
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .expect("query assertion rows")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("collect assertion rows");
        assert_eq!(
            Value::Array(rows.into_iter().map(Value::Array).collect()),
            *field(query, "rows")
        );
    }
}

fn sqlite_value(value: ValueRef<'_>) -> rusqlite::Result<Value> {
    Ok(match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(value) => json!(value),
        ValueRef::Real(value) => json!(value),
        ValueRef::Text(value) => Value::String(String::from_utf8_lossy(value).into_owned()),
        ValueRef::Blob(value) => {
            Value::Array(value.iter().copied().map(|byte| json!(byte)).collect())
        }
    })
}

fn run_program(
    program: &Path,
    args: &[String],
    output_directory: &Path,
    timeout: Duration,
) -> RunResult {
    let id = COMMAND_ID.fetch_add(1, Ordering::Relaxed);
    let stdout_path = output_directory.join(format!("command-{id}.stdout"));
    let stderr_path = output_directory.join(format!("command-{id}.stderr"));
    let stdout_file = File::create(&stdout_path).expect("create command stdout");
    let stderr_file = File::create(&stderr_path).expect("create command stderr");
    let start = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .unwrap_or_else(|error| panic!("spawn {}: {error}", program.display()));
    let status = wait_for_child(&mut child, timeout);
    let duration = start.elapsed();
    let stdout = fs::read_to_string(&stdout_path).expect("read command stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read command stderr");
    fs::remove_file(stdout_path).expect("remove command stdout");
    fs::remove_file(stderr_path).expect("remove command stderr");
    let envelope = parse_stdout(&stdout);
    assert_command_result_shape(args, &envelope);
    RunResult {
        status,
        stdout,
        stderr,
        envelope,
        duration,
    }
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().expect("poll child") {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("terminate timed-out child");
            let _ = child.wait();
            panic!("child {} exceeded {timeout:?}", child.id());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn parse_stdout(stdout: &str) -> Value {
    assert!(
        !stdout.starts_with('\u{feff}'),
        "stdout must not contain a BOM"
    );
    let body = stdout
        .strip_suffix('\n')
        .expect("stdout must end with exactly one LF");
    assert!(
        !body.contains(['\n', '\r']),
        "stdout must contain exactly one JSON line: {stdout:?}"
    );
    assert_compact_json(body);
    let value: Value = serde_json::from_str(body).expect("stdout must be one JSON object");
    assert!(value.is_object(), "stdout root must be an object");
    match field(&value, "ok") {
        Value::Bool(true) => assert_key_set(object(&value), &["ok", "result"]),
        Value::Bool(false) => {
            assert_key_set(object(&value), &["ok", "error"]);
            assert_key_set(object(field(&value, "error")), &["category", "details"]);
            assert_error_details_shape(field(&value, "error"));
        }
        _ => panic!("stdout ok field must be a boolean"),
    }
    assert_normalized_numbers(&value);
    value
}

fn assert_command_result_shape(args: &[String], envelope: &Value) {
    if field(envelope, "ok") != &Value::Bool(true) {
        return;
    }
    let result = object(field(envelope, "result"));
    match args.get(2).map(String::as_str) {
        Some("set") => assert_key_set(result, &["key", "value", "revision", "created"]),
        Some("get") => assert_key_set(result, &["key", "value", "revision"]),
        Some("delete") => {
            assert_key_set(result, &["key", "deleted_revision", "revision"]);
        }
        Some("list") => {
            assert_key_set(result, &["entries", "global_revision"]);
            for entry in array(field(
                envelope.get("result").expect("list result"),
                "entries",
            )) {
                assert_key_set(object(entry), &["key", "value", "revision"]);
            }
        }
        command => panic!("unexpected successful command shape: {command:?}"),
    }
}

fn assert_error_details_shape(error: &Value) {
    let details = object(field(error, "details"));
    match string(field(error, "category")) {
        "usage" | "invalid_json" | "invalid_value" => assert_key_set(details, &["reason"]),
        "invalid_argument" => assert_key_set(details, &["field", "reason"]),
        "conflict" => assert_key_set(details, &["key", "expected", "actual"]),
        "not_found" => assert_key_set(details, &["key"]),
        "busy" => assert_key_set(details, &["timeout_ms"]),
        "unsupported_schema" => assert_key_set(details, &["found", "supported"]),
        "invalid_storage" => {
            let expected = if details.contains_key("key") {
                vec!["reason", "key"]
            } else {
                vec!["reason"]
            };
            assert_key_set(details, &expected);
        }
        "revision_exhausted" => assert_key_set(details, &["maximum"]),
        "storage_error" => assert_key_set(details, &["operation", "reason"]),
        category => panic!("unknown error category {category}"),
    }
}

fn assert_compact_json(text: &str) {
    let mut in_string = false;
    let mut escaped = false;
    for character in text.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
        } else {
            assert!(
                !character.is_whitespace(),
                "JSON output contains insignificant whitespace"
            );
        }
    }
}

fn assert_normalized_numbers(value: &Value) {
    match value {
        Value::Number(number) => {
            let integer = number
                .as_i64()
                .map(i128::from)
                .or_else(|| number.as_u64().map(i128::from))
                .expect("output numbers must be integers");
            assert!((-9_007_199_254_740_991_i128..=9_007_199_254_740_991_i128).contains(&integer));
        }
        Value::Array(values) => values.iter().for_each(assert_normalized_numbers),
        Value::Object(values) => values.values().for_each(assert_normalized_numbers),
        Value::Null | Value::Bool(_) | Value::String(_) => {}
    }
}

fn assert_expectation(result: &RunResult, expected_exit: &Value, expectation: &Value) {
    assert_exact_keys(object(expectation), &["exit", "stdout", "stderr"]);
    let expected_stdout = field(expectation, "stdout");
    assert_standard(
        result,
        integer(expected_exit) as i32,
        expected_stdout,
        string(field(expectation, "stderr")),
    );
}

fn assert_standard(result: &RunResult, exit: i32, stdout: &Value, stderr: &str) {
    assert_eq!(result.status.code(), Some(exit), "{result:?}");
    assert_eq!(result.stderr, stderr, "{result:?}");
    assert_eq!(&result.envelope, stdout, "{result:?}");
    assert_eq!(
        result.stdout,
        format!("{}\n", result.stdout.trim_end_matches('\n'))
    );
}

fn assert_error(error: &subject::KvError, exit: u8, category: &str, details: &Value) {
    assert_eq!(error.exit_code(), exit);
    assert_eq!(error.category(), category);
    assert_eq!(&error.details(), details);
}

fn generated_key(case: &Value) -> String {
    if let Some(key) = case.get("key") {
        return string(key).to_owned();
    }
    let generator = field(case, "key_generator");
    assert_exact_keys(object(generator), &["kind", "prefix", "character", "count"]);
    assert_eq!(string(field(generator, "kind")), "repeat_suffix");
    format!(
        "{}{}",
        string(field(generator, "prefix")),
        string(field(generator, "character")).repeat(integer(field(generator, "count")) as usize)
    )
}

fn generated_input(case: &Value) -> String {
    if let Some(input) = case.get("input_json") {
        return string(input).to_owned();
    }
    generate_input(field(case, "input_generator"))
}

fn generate_input(generator: &Value) -> String {
    match string(field(generator, "kind")) {
        "nested_arrays" => {
            assert_exact_keys(object(generator), &["kind", "depth", "leaf"]);
            let value = nested_value(
                integer(field(generator, "depth")) as usize,
                field(generator, "leaf").clone(),
            );
            serde_json::to_string(&value).expect("serialize generated nested value")
        }
        "ascii_string_total_bytes" => {
            assert_exact_keys(object(generator), &["kind", "character", "total_bytes"]);
            let total = integer(field(generator, "total_bytes")) as usize;
            let character = string(field(generator, "character"));
            assert_eq!(character.len(), 1);
            let value = format!("\"{}\"", character.repeat(total - 2));
            assert_eq!(value.len(), total);
            value
        }
        other => panic!("unknown input generator {other}"),
    }
}

fn generated_normalized(case: &Value) -> Value {
    if let Some(value) = case.get("normalized") {
        return value.clone();
    }
    let generator = field(case, "normalized_generator");
    match string(field(generator, "kind")) {
        "nested_arrays" => {
            assert_exact_keys(object(generator), &["kind", "depth", "leaf"]);
            nested_value(
                integer(field(generator, "depth")) as usize,
                field(generator, "leaf").clone(),
            )
        }
        "ascii_string_total_bytes" => {
            assert_exact_keys(object(generator), &["kind", "character", "total_bytes"]);
            let total = integer(field(generator, "total_bytes")) as usize;
            Value::String(string(field(generator, "character")).repeat(total - 2))
        }
        other => panic!("unknown normalized generator {other}"),
    }
}

fn nested_value(depth: usize, mut leaf: Value) -> Value {
    for _ in 0..depth {
        leaf = Value::Array(vec![leaf]);
    }
    leaf
}

fn substituted_args(
    value: &Value,
    database: &Path,
    missing_parent: &Path,
    replacements: Option<&Map<String, Value>>,
) -> Vec<String> {
    array(value)
        .iter()
        .map(|argument| {
            let mut argument = string(argument)
                .replace("${DB}", &path_string(database))
                .replace("${MISSING_PARENT}", &path_string(missing_parent));
            if let Some(replacements) = replacements {
                for (key, value) in replacements {
                    argument = argument.replace(&format!("${{{key}}}"), string(value));
                }
            }
            argument
        })
        .collect()
}

fn assert_integrity(database: &Path) {
    let connection = rusqlite::Connection::open(database).expect("open database for integrity");
    let result: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("run integrity check");
    assert_eq!(result, "ok");
}

fn cleanup_database(database: &Path) {
    for path in database_files(database) {
        if path.exists() {
            fs::remove_file(&path)
                .unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
        }
        assert!(!path.exists(), "{} remained after cleanup", path.display());
    }
}

fn database_files(database: &Path) -> Vec<PathBuf> {
    let base = database.as_os_str().to_string_lossy();
    ["", "-wal", "-shm", "-journal"]
        .into_iter()
        .map(|suffix| PathBuf::from(format!("{base}{suffix}")))
        .collect()
}

fn scenario_directory(label: &str) -> TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join("comparative-conformance");
    fs::create_dir_all(&root).expect("create repository-local conformance root");
    Builder::new()
        .prefix(&format!("{label}-"))
        .tempdir_in(root)
        .expect("create scenario directory")
}

fn read_fixture(relative_path: &str) -> Value {
    let text = fs::read_to_string(spec_path(relative_path))
        .unwrap_or_else(|error| panic!("read fixture {relative_path}: {error}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("parse fixture {relative_path}: {error}"))
}

fn spec_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../spec")
        .join(relative_path)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    object(value)
        .get(name)
        .unwrap_or_else(|| panic!("missing fixture field {name}"))
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("fixture value must be an object")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("fixture value must be an array")
}

fn string(value: &Value) -> &str {
    value.as_str().expect("fixture value must be a string")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().expect("fixture value must be an integer")
}

fn assert_exact_keys(object: &Map<String, Value>, allowed: &[&str]) {
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    assert!(
        actual.is_subset(&allowed),
        "unknown fixture keys: {:?}",
        actual.difference(&allowed).collect::<Vec<_>>()
    );
}

fn assert_key_set(object: &Map<String, Value>, expected: &[&str]) {
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "JSON object has an unexpected member set");
}
