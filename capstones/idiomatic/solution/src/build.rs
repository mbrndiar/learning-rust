//! Bounded worker orchestration for deterministic index construction.
//!
//! This is the concurrency core. Discovery is single-threaded and ordered; reading
//! and tokenizing files is fanned out to a bounded pool, then the results are put
//! back into a canonical order before anything is trusted. The key idea that makes
//! a parallel build reproducible: **document ids are assigned last**. Workers may
//! finish in any order, so `build` collects every document, sorts by
//! `(root order, path)`, and only then numbers them `1..=N`. A shared
//! [`Cancellation`] token lets any fatal condition stop the pool promptly, and
//! `thread::scope` guarantees every spawned worker is joined before `build`
//! returns even on the error paths.

use crate::tokenization::tokenize_with_outcome;
use crate::{
    DocumentId, ErrorCode, FileIssue, FileTree, INDEX_SCHEMA_VERSION, IndexData, IndexError,
    IndexIssue, IndexSettings, IndexedDocument, IssueCode, RootSpec, TermCount, TreeEntry,
    TreeEntryKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

/// Cloneable cancellation behavior injected into an index build.
///
/// Cloning must share one underlying flag so a `cancel` on any handle is visible to
/// every worker; `CancellationToken` does this with an `Arc<AtomicBool>`.
pub trait Cancellation: Clone + Send + Sync + 'static {
    /// Requests cancellation; subsequent `is_cancelled` calls return `true`.
    fn cancel(&self);
    /// Reports whether cancellation has been requested.
    fn is_cancelled(&self) -> bool;
}

/// Atomic cancellation token suitable for production and tests.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates an uncancelled token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cancellation for CancellationToken {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// Builder parameterized by file access and cancellation capabilities.
///
/// `F` is the [`FileTree`] seam and `C` the [`Cancellation`] token; keeping both
/// generic lets tests drive the exact same orchestration with fakes.
pub struct IndexBuilder<F, C> {
    tree: F,
    workers: NonZeroUsize,
    cancellation: C,
}

impl<F: FileTree, C: Cancellation> IndexBuilder<F, C> {
    /// Creates a builder with an explicit positive worker count.
    pub const fn new(tree: F, workers: NonZeroUsize, cancellation: C) -> Self {
        Self {
            tree,
            workers,
            cancellation,
        }
    }

    /// Builds a complete deterministic index and joins every started worker.
    pub fn build(
        &self,
        roots: &[RootSpec],
        settings: &IndexSettings,
    ) -> Result<IndexData, IndexError> {
        if roots.is_empty() {
            return Err(IndexError::contract(
                ErrorCode::InvalidRoot,
                "at least one root is required",
            ));
        }
        let mut names = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for root in roots {
            if !names.insert(root.name()) || !paths.insert(root.path()) {
                return Err(IndexError::contract(
                    ErrorCode::DuplicateRoot,
                    "root names and canonical paths must be unique",
                ));
            }
        }
        if self.cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let settings = IndexSettings::new(settings.extensions.clone(), settings.max_bytes)?;

        let mut issues = Vec::new();
        let mut jobs = Vec::new();
        for root in roots {
            let entries = self.tree.entries(root)?;
            for entry in entries {
                // Poll cancellation between entries so a large tree can stop early
                // without waiting for the whole walk to finish.
                if self.cancellation.is_cancelled() {
                    return Err(cancelled());
                }
                match entry {
                    Err(issue) => push_file_issue(&mut issues, root.name(), issue)?,
                    Ok(entry) => {
                        self.classify_entry(root, &settings, entry, &mut jobs, &mut issues)?;
                    }
                }
            }
        }
        if self.cancellation.is_cancelled() {
            return Err(cancelled());
        }

        let documents = self.run_workers(jobs, settings.max_bytes, &mut issues)?;
        let root_order = roots
            .iter()
            .enumerate()
            .map(|(index, root)| (root.name(), index))
            .collect::<BTreeMap<_, _>>();
        // Determinism happens here: workers returned documents in completion order,
        // so sort by (root order, path) to recover a stable order independent of
        // thread scheduling.
        let mut documents = documents;
        documents.sort_by(|left, right| {
            (root_order[left.root.as_str()], left.path.as_str())
                .cmp(&(root_order[right.root.as_str()], right.path.as_str()))
        });
        // Assign contiguous ids only after the final order is fixed, so the same
        // inputs always yield the same id for the same document.
        for (offset, document) in documents.iter_mut().enumerate() {
            document.id = DocumentId::new(offset as u64 + 1)?;
        }
        // Issues share the same canonical ordering key so their output is stable too.
        issues.sort_by(|left, right| {
            (
                root_order[left.root.as_str()],
                left.path.as_deref(),
                left.code.as_str(),
                left.message.as_str(),
            )
                .cmp(&(
                    root_order[right.root.as_str()],
                    right.path.as_deref(),
                    right.code.as_str(),
                    right.message.as_str(),
                ))
        });

        let index = IndexData {
            schema_version: INDEX_SCHEMA_VERSION,
            settings,
            roots: roots.iter().map(|root| root.name().to_owned()).collect(),
            documents,
            issues,
        };
        // Validate our own output: this both guards against a builder bug and proves
        // the produced index satisfies exactly what a reloaded index must.
        index.validate()?;
        Ok(index)
    }

    fn classify_entry(
        &self,
        root: &RootSpec,
        settings: &IndexSettings,
        mut entry: TreeEntry,
        jobs: &mut Vec<TreeEntry>,
        issues: &mut Vec<IndexIssue>,
    ) -> Result<(), IndexError> {
        // A trustworthy tree never yields a path outside its root; if it does, the
        // provider is buggy and the whole build fails rather than indexing an escape.
        if !entry.host_path.starts_with(root.path()) {
            return Err(worker_failed("file-tree entry escaped its canonical root"));
        }
        entry.root = root.name().to_owned();
        let Some(path) = entry.relative_path.as_deref() else {
            // Only regular directory entries reach here; a missing portable path
            // means non-UTF-8 and is recorded as an issue, not a job.
            issues.push(issue(root.name(), None, IssueCode::NonUtf8Path));
            return Ok(());
        };
        if !crate::valid_portable_path(path) {
            return Err(worker_failed(
                "file-tree entry used an unsafe portable relative path",
            ));
        }
        match entry.kind {
            TreeEntryKind::Symlink => {
                issues.push(issue(
                    root.name(),
                    Some(path.to_owned()),
                    IssueCode::SymlinkSkipped,
                ));
            }
            // Only regular files with an included extension become read jobs;
            // directories are traversed by the walker and other kinds are ignored.
            TreeEntryKind::RegularFile if settings.includes_path(path) => jobs.push(entry),
            TreeEntryKind::Directory | TreeEntryKind::RegularFile | TreeEntryKind::Other => {}
        }
        Ok(())
    }

    fn run_workers(
        &self,
        jobs: Vec<TreeEntry>,
        max_bytes: u64,
        issues: &mut Vec<IndexIssue>,
    ) -> Result<Vec<IndexedDocument>, IndexError> {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        // Never spawn more workers than jobs; bounded channels apply backpressure so
        // at most `worker_count` jobs are outstanding at once.
        let worker_count = self.workers.get().min(jobs.len());
        let (job_sender, job_receiver) = mpsc::sync_channel::<TreeEntry>(worker_count);
        let (result_sender, result_receiver) = mpsc::sync_channel::<WorkerOutput>(worker_count);
        // Workers share one receiver behind a mutex, forming a simple work queue.
        let job_receiver = Arc::new(Mutex::new(job_receiver));
        let mut documents = Vec::new();
        let mut failure = None;

        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(worker_count);
            for _ in 0..worker_count {
                let receiver = Arc::clone(&job_receiver);
                let sender = result_sender.clone();
                let cancellation = self.cancellation.clone();
                let tree = &self.tree;
                handles.push(scope.spawn(move || {
                    worker_loop(tree, receiver, sender, cancellation, max_bytes);
                }));
            }
            // Drop the extra sender so the loop below sees disconnect once every
            // worker has exited.
            drop(result_sender);

            // Prime the pool with one job per worker, then top it up on each result
            // so the number of in-flight jobs stays bounded (a scheduling window).
            let mut pending = jobs.into_iter();
            let mut in_flight = 0_usize;
            for _ in 0..worker_count {
                let Some(entry) = pending.next() else {
                    break;
                };
                if job_sender.send(entry).is_err() {
                    failure = Some(worker_failed("job channel closed unexpectedly"));
                    break;
                }
                in_flight += 1;
            }

            while in_flight > 0 {
                // Time out periodically so a cancellation that arrives while all
                // workers are idle is still observed.
                match result_receiver.recv_timeout(Duration::from_millis(10)) {
                    Ok(output) => {
                        in_flight -= 1;
                        // Record the first failure only; once set, later outputs are
                        // drained but ignored so the pool can wind down cleanly.
                        if self.cancellation.is_cancelled() && failure.is_none() {
                            failure = Some(cancelled());
                        }
                        match output {
                            WorkerOutput::Document(document, too_long) => {
                                if failure.is_none() {
                                    if too_long {
                                        issues.push(issue(
                                            &document.root,
                                            Some(document.path.clone()),
                                            IssueCode::TokenTooLong,
                                        ));
                                    }
                                    documents.push(document);
                                }
                            }
                            WorkerOutput::Issue(root, issue) => {
                                if failure.is_none() {
                                    // A fatal/incomplete provider issue promotes to a
                                    // build failure and cancels the remaining work.
                                    if let Err(error) = push_file_issue(issues, &root, issue) {
                                        self.cancellation.cancel();
                                        failure = Some(error);
                                    }
                                }
                            }
                            WorkerOutput::Cancelled => {
                                if failure.is_none() {
                                    failure = Some(cancelled());
                                }
                            }
                            WorkerOutput::Fatal(message) => {
                                if failure.is_none() {
                                    failure = Some(worker_failed(message));
                                }
                            }
                        }

                        // Only feed more work while healthy; otherwise cancel so
                        // workers stop pulling jobs and the queue drains.
                        if failure.is_none() && !self.cancellation.is_cancelled() {
                            if let Some(entry) = pending.next() {
                                if job_sender.send(entry).is_err() {
                                    self.cancellation.cancel();
                                    failure =
                                        Some(worker_failed("job channel closed unexpectedly"));
                                } else {
                                    in_flight += 1;
                                }
                            }
                        } else {
                            self.cancellation.cancel();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if self.cancellation.is_cancelled() && failure.is_none() {
                            failure = Some(cancelled());
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Every worker exited while we still expected results: treat
                        // as cancellation if requested, otherwise a hard failure.
                        if failure.is_none() {
                            failure = Some(if self.cancellation.is_cancelled() {
                                cancelled()
                            } else {
                                worker_failed("result channel closed with jobs in flight")
                            });
                        }
                        break;
                    }
                }
            }

            // Close the job channel and join every worker before leaving the scope,
            // so no thread outlives the build; a panic surfaces as a worker failure.
            drop(job_sender);
            for handle in handles {
                if handle.join().is_err() && failure.is_none() {
                    failure = Some(worker_failed("worker thread panicked"));
                }
            }
        });

        match failure {
            Some(error) => Err(error),
            None => Ok(documents),
        }
    }
}

/// One message a worker sends back per job it processes.
enum WorkerOutput {
    /// A finished document plus whether an over-length token was dropped.
    Document(IndexedDocument, bool),
    /// A recoverable per-file issue tagged with its root name.
    Issue(String, FileIssue),
    /// A non-recoverable failure that aborts the build.
    Fatal(String),
    /// The worker observed cancellation and stopped early.
    Cancelled,
}

fn worker_loop<F: FileTree, C: Cancellation>(
    tree: &F,
    receiver: Arc<Mutex<mpsc::Receiver<TreeEntry>>>,
    sender: mpsc::SyncSender<WorkerOutput>,
    cancellation: C,
    max_bytes: u64,
) {
    loop {
        if cancellation.is_cancelled() {
            break;
        }
        // Hold the queue lock only long enough to take one job, then release it so
        // other workers can pull concurrently.
        let job = match receiver.lock() {
            Ok(receiver) => receiver.recv(),
            Err(_) => {
                let _ = sender.send(WorkerOutput::Fatal(
                    "job receiver lock was poisoned".to_owned(),
                ));
                break;
            }
        };
        let Ok(entry) = job else {
            break;
        };
        // Contain a panic in user-provided tree code: convert it into a fatal output
        // instead of unwinding the worker and poisoning shared state.
        let output = catch_unwind(AssertUnwindSafe(|| {
            process_entry(tree, entry, max_bytes, &cancellation)
        }))
        .unwrap_or_else(|_| WorkerOutput::Fatal("worker read panicked".to_owned()));
        if sender.send(output).is_err() {
            break;
        }
    }
}

fn process_entry<F: FileTree, C: Cancellation>(
    tree: &F,
    entry: TreeEntry,
    max_bytes: u64,
    cancellation: &C,
) -> WorkerOutput {
    // Check cancellation at each step so a long read pipeline abandons work quickly.
    if cancellation.is_cancelled() {
        return WorkerOutput::Cancelled;
    }
    let bytes = match tree.read(&entry, max_bytes) {
        Ok(bytes) => bytes,
        Err(issue) => {
            // A read error during cancellation is reported as cancellation, not as a
            // misleading per-file issue.
            return if cancellation.is_cancelled() {
                WorkerOutput::Cancelled
            } else {
                WorkerOutput::Issue(entry.root, issue)
            };
        }
    };
    if cancellation.is_cancelled() {
        return WorkerOutput::Cancelled;
    }
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(_) => {
            return WorkerOutput::Issue(
                entry.root,
                FileIssue::message(
                    IssueCode::NonUtf8Content,
                    entry.relative_path,
                    IssueCode::NonUtf8Content.message(),
                ),
            );
        }
    };
    let tokenization = tokenize_with_outcome(text);
    // BTreeMap gives per-document terms in sorted order for free, satisfying the
    // sorted-unique term invariant without an extra pass.
    let mut counts = BTreeMap::<String, u64>::new();
    for token in tokenization.tokens {
        *counts.entry(token).or_default() += 1;
    }
    WorkerOutput::Document(
        IndexedDocument {
            // Placeholder id; the real contiguous id is assigned after sorting.
            id: DocumentId::new(1).expect("constant positive document id"),
            root: entry.root,
            path: entry
                .relative_path
                .expect("regular file jobs always have portable paths"),
            bytes: bytes.len() as u64,
            terms: counts
                .into_iter()
                .map(|(term, count)| TermCount { term, count })
                .collect(),
        },
        tokenization.ignored_long_token,
    )
}

fn push_file_issue(
    issues: &mut Vec<IndexIssue>,
    root: &str,
    file_issue: FileIssue,
) -> Result<(), IndexError> {
    match file_issue {
        FileIssue::Io { code, path, .. } | FileIssue::Message { code, path, .. } => {
            // A `non_utf8_path` issue must carry a null path; any other issue with a
            // missing path gets a placeholder so it still validates.
            let path = if code == IssueCode::NonUtf8Path {
                None
            } else {
                path.or_else(|| Some("<entry>".to_owned()))
            };
            issues.push(issue(root, path, code));
            Ok(())
        }
        // Scaffold/provider defects are programming errors, not recoverable issues.
        FileIssue::Incomplete { capability } => Err(worker_failed(format!(
            "file provider capability {capability} is incomplete"
        ))),
        FileIssue::Fatal { message } => Err(worker_failed(message)),
    }
}

fn issue(root: &str, path: Option<String>, code: IssueCode) -> IndexIssue {
    IndexIssue {
        root: root.to_owned(),
        path,
        code,
        message: code.message().to_owned(),
    }
}

fn cancelled() -> IndexError {
    IndexError::contract(ErrorCode::Cancelled, "index build was cancelled")
}

fn worker_failed(message: impl Into<String>) -> IndexError {
    IndexError::contract(ErrorCode::WorkerFailed, message)
}
