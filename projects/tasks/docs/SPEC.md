# Task REST API applied project specification

## Purpose

This project teaches how a small domain moves through persistence, an HTTP API,
and command-line clients without becoming coupled to one framework.

The application manages tasks with three fields: an integer ID, a title, and a
completion flag. One shared core is used by every server. One HTTP contract is
used by every client. SQLite and a human-readable Markdown checklist provide
two implementations of the same repository behavior.

The contract is intentionally bounded. It is complete enough to support useful
normal and failure flows, but small enough for a learner to understand every
layer.

The machine-readable HTTP shapes live in
[`openapi.yaml`](openapi.yaml). This document explains how those shapes,
persistence rules, clients, and implementation choices fit together.

## An end-to-end example

Assume any project server is running at `http://127.0.0.1:8000`.

Check that the process is ready:

```console
$ curl -s http://127.0.0.1:8000/health
{"status":"ok"}
```

Create a task. New tasks are always incomplete:

```console
$ curl -s -X POST http://127.0.0.1:8000/tasks \
    -H 'Content-Type: application/json' \
    -d '{"title":"Learn REST"}'
{"id":1,"title":"Learn REST","completed":false}
```

Complete it with a partial update:

```console
$ curl -s -X PATCH http://127.0.0.1:8000/tasks/1 \
    -H 'Content-Type: application/json' \
    -d '{"completed":true}'
{"id":1,"title":"Learn REST","completed":true}
```

The completed filter returns the same task:

```console
$ curl -s 'http://127.0.0.1:8000/tasks?completed=true'
[{"id":1,"title":"Learn REST","completed":true}]
```

Delete it, then ask for it again:

```console
$ curl -s -X DELETE -o /dev/null -w '%{http_code}\n' \
    http://127.0.0.1:8000/tasks/1
204

$ curl -s http://127.0.0.1:8000/tasks/1
{"error":{"code":"not_found","message":"task 1 was not found"}}
```

The same flow works with every server, client, and persistence backend in the
project.

## Architecture and dependency direction

The project has three responsibilities:

| Responsibility | Owns | Does not own |
| --- | --- | --- |
| Shared core | Task rules, service operations, repository abstraction, SQLite and Markdown persistence | HTTP framework objects, client library objects, process configuration |
| HTTP servers | Routing, request decoding, status selection, response encoding, dependency construction | Domain validation rules or storage-specific business logic |
| HTTP clients | Command parsing, safe request construction, response validation, output, exit codes | Direct repository access or server internals |

The core points inward: adapters depend on the core, while the core never
imports Axum, Actix Web, or Reqwest. Rust modules, structs, enums, and narrow
traits express these boundaries.

The Reqwest client interoperates with both servers. It is not paired with one
framework and cannot rely on private Axum or Actix Web behavior.

## Task model

A task response has this shape:

```json
{
  "id": 1,
  "title": "Learn REST",
  "completed": false
}
```

| Field | Rule |
| --- | --- |
| `id` | Positive integer allocated by the repository. IDs start at 1, increase monotonically, and are never reused after deletion. |
| `title` | String trimmed before storage. After trimming it contains 1–120 Unicode characters, occupies one physical line, and contains no control characters. |
| `completed` | JSON Boolean. A number, string, or null is not a Boolean. |

Task lists are ordered by ID ascending. JSON request objects reject unknown
properties so misspellings do not silently change behavior.

Creating a task accepts only `title`; `completed` is set to `false`. Updating a
task accepts `title`, `completed`, or both. An empty update is invalid. A valid
update that happens to preserve the current values still succeeds.

## Shared service and repository behavior

The application service provides five task operations:

1. create a task from a title;
2. list all tasks or tasks with one completion state;
3. get one task by ID;
4. update the title, completion state, or both; and
5. delete one task.

Validation belongs in the shared core rather than being reimplemented in each
route. HTTP boundary models may reject an obviously wrong shape early, but they
must map to the same domain outcomes and messages.

The repository abstraction provides corresponding `create`, `list`, `get`,
`update`, and `delete` capabilities. Its public vocabulary should use domain
values, not framework requests or raw database rows. A missing ID produces a
specific not-found domain error. Malformed or unavailable persisted data
produces a storage error that retains useful diagnostic context inside the
process.

Each mutation is atomic from the caller's perspective. A failed create, update,
or delete does not leave a partially written task or consume an ID unless the
underlying storage has durably committed the successful mutation.

## Persistence

The backend is selected when the server starts. SQLite and Markdown files are
independent stores; switching between them does not copy or synchronize data.
Both must pass the same repository contract.

### SQLite repository

The SQLite implementation uses `rusqlite` with bundled SQLite and parameterized
SQL. It stores tasks in one table with:

- an auto-incrementing integer primary key;
- non-null title text; and
- a checked integer representation of the Boolean completion state.

Schema creation is idempotent for a new or already initialized project
database. Mutations use one transaction, rows are mapped explicitly into domain
values, and every connection is closed. Deleting rows—including every row—does
not reset future IDs.

This project does not require migrations, write-ahead logging configuration,
cross-process compare-and-set behavior, an ORM, or advanced connection-pool
tuning. An unexpected schema or database failure is reported as a storage error
rather than being "fixed" by dropping user data.

### Markdown checklist repository

The second repository stores everything in one UTF-8 Markdown file:

```markdown
<!-- rest-task-api:v1 next-id=3 -->
# Tasks

- [ ] 1: Learn SQLite
- [x] 2: Build an API
```

The metadata comment records the format version and the next ID. It is required
in an existing file. A missing file is initialized as version 1 with
`next-id=1`; an existing empty or malformed file is not treated as a new store.

Checklist rows use `- [ ]` for incomplete tasks and `- [x]` for completed tasks,
followed by the positive ID, `: `, and the validated title. Titles are stored
literally after that separator. Rows are written in ID order, with one final
newline, so repeated saves are deterministic and the file remains pleasant to
read.

The repository validates metadata and every row before constructing domain
values. Unsupported versions, invalid `next-id` values, duplicate or
out-of-order IDs, malformed checklist rows, invalid titles, and a `next-id` that
is not greater than every stored ID are storage errors. It never silently skips
bad lines or guesses how to repair them.

A load-modify-save operation is protected from other threads in the same
process. Saving writes a complete temporary sibling file, flushes and closes
it, then atomically replaces the target. This protects readers from seeing a
half-written document. Cross-process file locking and recovery from a machine
failure between filesystem operations are outside this project.

## HTTP conventions

Examples use a loopback base URL, but the path and JSON contract does not depend
on a specific port.

- JSON request and response bodies use UTF-8 and
  `Content-Type: application/json`.
- Body endpoints require a JSON content type. A missing or unsupported content
  type, invalid UTF-8, or malformed JSON produces the documented `400` error.
- A valid JSON value with the wrong shape, an unknown property, or an invalid
  domain value produces `422`.
- Path IDs use base-10 digits and must be positive. Invalid IDs produce `422`.
- Success responses and errors never include framework tracebacks or raw
  exception text.
- An unsupported method for a known path produces `405` and an `Allow` header.
- Unknown paths use the shared `404` error envelope.
- Implementations may include standard headers such as `Date`, `Server`, and
  `Content-Length`; clients must not depend on them.

Only the methods and paths below are part of the project contract. Redirects,
trailing-slash aliases, `HEAD`, and `OPTIONS` behavior are not portable
extensions and should not be used by project clients.

## HTTP operations

| Method | Path | Request | Success |
| --- | --- | --- | --- |
| `GET` | `/health` | None | `200` with `{"status":"ok"}` |
| `POST` | `/tasks` | `{"title":"..."}` | `201` with the created Task |
| `GET` | `/tasks` | Optional `completed=true` or `completed=false` query | `200` with a Task array |
| `GET` | `/tasks/{id}` | None | `200` with one Task |
| `PATCH` | `/tasks/{id}` | `title`, `completed`, or both | `200` with the updated Task |
| `DELETE` | `/tasks/{id}` | None | `204` with no body |

### Health

`GET /health` shows that the HTTP process can answer requests. It does not
create data or require a repository write. It is a small readiness exercise,
not a production health-check design.

### Create

The request contains exactly one property:

```json
{"title": "  Learn REST  "}
```

The stored and returned title is trimmed:

```json
{"id": 1, "title": "Learn REST", "completed": false}
```

Missing `title`, a non-string title, an empty title after trimming, a multiline
title, more than 120 characters, or an unknown property produces `422`.

### List and filter

`GET /tasks` returns a JSON array, including `[]` when the store is empty.
`completed` may appear once with the exact lowercase value `true` or `false`.
Any other value produces `422`.

The filter is applied by Boolean value and results remain ordered by ID:

```json
[
  {"id": 2, "title": "Write contract tests", "completed": false},
  {"id": 5, "title": "Compare frameworks", "completed": false}
]
```

### Get, update, and delete

A valid but absent positive ID produces `404`.

Patch bodies are partial:

```json
{"title": "Read the OpenAPI document", "completed": true}
```

At least one supported property is required. `id` cannot be changed. Unknown
properties, nulls, and values of the wrong type produce `422`.

A successful delete returns `204` and no response body. Deleting the same ID
again returns `404`. A deleted ID is not reused.

## Error responses

Every JSON error uses one envelope:

```json
{
  "error": {
    "code": "validation_error",
    "message": "title must contain between 1 and 120 characters",
    "details": {
      "field": "title"
    }
  }
}
```

`details` is optional and contains small JSON values that help a caller locate
the problem. Clients classify errors by HTTP status and `code`, not by parsing
English messages.

| Status | Code | Used for |
| --- | --- | --- |
| `400` | `invalid_json` | Missing/unsupported JSON content type, invalid UTF-8, or malformed JSON on a body endpoint |
| `404` | `not_found` | A missing task or unknown route |
| `405` | `method_not_allowed` | A method not supported by a known path |
| `422` | `validation_error` | A valid JSON value or URL component that violates the request or domain rules |
| `500` | `internal_error` | Unexpected server or persistence failure |

For example, an unknown request property is a semantic error:

```http
HTTP/1.1 422 Unprocessable Content
Content-Type: application/json

{
  "error": {
    "code": "validation_error",
    "message": "unknown property: done",
    "details": {"field": "done"}
  }
}
```

Malformed persisted Markdown is different. The server logs the precise storage
error for the developer, then returns a sanitized response:

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "error": {
    "code": "internal_error",
    "message": "the server could not complete the request"
  }
}
```

Status phrases such as `Unprocessable Content` may vary by HTTP library; the
numeric status and JSON body are the portable contract.

## Command-line client contract

Each client exposes the same commands:

```text
add TITLE
list [--completed true|false]
show ID
update ID [--title TITLE] [--completed true|false]
complete ID
remove ID
```

The client also accepts `--base-url URL` and a positive finite `--timeout`
value, defaulting to 5 seconds. It encodes path and query values safely, sends
an explicit JSON content type for bodies, closes responses and client/session
objects, and never retries implicitly.

`complete ID` is a convenience command that sends
`PATCH /tasks/{id}` with `{"completed":true}`. Clients never read or write the
repository directly.

On success:

- `add`, `show`, `update`, and `complete` print the returned Task as JSON;
- `list` prints the returned Task array as JSON; and
- `remove` prints `{"deleted":ID}` generated after the `204` response.

JSON property spacing and order are presentation details; parsed values are the
contract. Normal output goes to stdout. Errors go to stderr as a concise line
that includes the category and useful message.

| Exit code | Meaning |
| --- | --- |
| `0` | Success |
| `2` | Command usage error, such as a missing argument or non-positive ID |
| `3` | The server returned a valid documented API error |
| `4` | The server response had an unexpected status, content type, or JSON shape |
| `5` | Connection, DNS, TLS, or timeout failure |

The client checks status before trusting a success body, validates every decoded
Task and error field, and does not expose library-specific exceptions as its
public behavior.

## Implementation freedom

Observable behavior is shared; internal design is not. An implementation may
choose names and decomposition that are idiomatic for its language and
framework. In particular:

- the repository abstraction is a narrow injected Rust trait;
- domain values use structs with read-only public access;
- route registration and dependency injection should follow the chosen
  framework's conventions;
- asynchronous framework and client boundaries may coordinate a synchronous
  repository without changing observable behavior;
- SQL helper structure and private schema bootstrap functions are not part of
  the public contract; and
- test utilities may call an in-process adapter or a real loopback server as
  long as the black-box contract is also exercised.

Do not erase useful library differences with a home-grown universal framework.
Share domain rules and client policy, but let each adapter demonstrate the
idiomatic lifecycle, error handling, and testing style of its ecosystem.

## Acceptance criteria

The project is complete when:

- both repositories pass one shared create/list/get/update/delete contract,
  including persistence restart and corruption cases;
- Axum and Actix Web pass the same black-box HTTP contract;
- the Reqwest client passes the shared command/transport contract;
- a one-client-by-two-server matrix proves Reqwest can call either server;
- IDs remain monotonic after deletion for both repositories;
- Markdown saves are deterministic and atomically published;
- malformed JSON, invalid fields, invalid filters and IDs, missing tasks,
  unsupported methods, storage failures, malformed responses, and connection
  failures have tests;
- tests use temporary storage, finite timeouts, ephemeral loopback ports, and no
  public network;
- resources and server processes are cleaned up even when a test fails; and
- formatting, static analysis, tests, coverage, OpenAPI validation, and
  documentation-link checks pass using the repository's established tools.

## Explicit non-goals

This project does not include:

- authentication, authorization, users, or multi-tenancy;
- timestamps, priorities, tags, due dates, subtasks, or attachments;
- pagination, sorting options, search, bulk operations, or background jobs;
- browser UI, CORS policy, WebSockets, streaming, or asynchronous endpoints;
- public deployment, TLS termination, containers, or production server tuning;
- ORM use, schema migrations, advanced connection-pool tuning, or distributed
  transactions;
- cross-process Markdown locking or synchronization between backends;
- automatic retries, offline client caching, or generated client SDKs; or
- checksum manifests and exhaustive cross-language fixture machinery.

These omissions keep attention on domain boundaries, persistence adapters,
HTTP/JSON behavior, client interoperability, and idiomatic library comparison.
