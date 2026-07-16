# Reusable agent prompt: build the Task REST API applied project

Replace every angle-bracket placeholder before giving this prompt to an agent.
The specification and OpenAPI document remain the behavioral source of truth.

## Inputs

- **Target repository:** `<TARGET_REPOSITORY>`
- **Target language:** `<TARGET_LANGUAGE>`
- **Supported versions:** `<SUPPORTED_VERSIONS>`
- **Project path:** `<PROJECT_PATH>`
- **Starter/solution selection mechanism:** `<IMPLEMENTATION_SELECTOR>`
- **Selected server stacks:** `<SERVER_STACKS>`
- **Selected HTTP client stacks:** `<HTTP_CLIENT_STACKS>`
- **SQLite library:** `<SQLITE_LIBRARY>`
- **Package/build tool:** `<PACKAGE_BUILD_TOOL>`
- **Test runner:** `<TEST_RUNNER>`
- **Formatter:** `<FORMATTER>`
- **Linter:** `<LINTER>`
- **Static/type analyzer:** `<STATIC_ANALYZER>`
- **Coverage tool:** `<COVERAGE_TOOL>`
- **Minimum coverage target:** `<COVERAGE_TARGET>`
- **Contract source path:** `<SPEC_PATH>`
- **OpenAPI source path:** `<OPENAPI_PATH>`

## Assignment

Create a required applied learning project at **Project path** for **Target
language**. Implement the bounded Task REST API, SQLite and versioned Markdown
repositories, and the ecosystem-selected server and client stacks defined by
**Contract source path** and **OpenAPI source path**.

Use shared `starter` and `solution` source roots plus shared progressive tests.
The project belongs after learners know SQL/SQLite and HTTP/JSON plus any
concurrency or resource-lifecycle foundations required by the target ecosystem,
and before final capstones. Do not replace, rename, or simplify existing final
projects.

Before editing:

1. inspect the target repository's course structure, documentation conventions,
   dependency files, quality commands, CI, and existing projects;
2. inspect current uncommitted changes and preserve unrelated work;
3. verify version-sensitive library behavior against official documentation;
4. map the reusable roles to idiomatic target-language package names; and
5. record any necessary ecosystem-specific adaptation that does not change
   observable behavior.

## Required architecture

Create equivalent roles for:

- a framework-neutral core with a read-only Task value, validation, domain
  errors, application service, and repository abstraction;
- SQLite and one-file versioned Markdown checklist repositories;
- thin adapters for every stack in **Selected server stacks**;
- one shared client command application with transports for every stack in
  **Selected HTTP client stacks**; and
- shared tests selected by **Starter/solution selection mechanism**.

Every client must interoperate with every server. Do not pair clients to servers
or let a client use framework-private behavior. The core must not import server
frameworks or HTTP client libraries.

Select servers and clients for distinct teaching value rather than to satisfy a
rigid count. A standard-library implementation is useful only when it exposes
concepts appropriate to the target ecosystem; framework-native selections are
equally valid.

## Idiomaticity and engineering requirements

Implement the project as experienced maintainers of the target ecosystem would,
while keeping it approachable for the course level:

- use the language's idiomatic immutable/read-only data representation and
  interface, trait, protocol, or dependency-injection style;
- prefer composition and narrow capabilities over inheritance frameworks,
  service locators, global mutable state, or generic repository base classes;
- keep validation in the shared core and HTTP-specific models at the boundary;
- use explicit composition roots, application factories, or the framework's
  recommended equivalent;
- keep route handlers thin and centralize shared error mapping;
- use parameterized SQL, explicit row mapping, transaction ownership, and
  deterministic connection cleanup;
- treat Markdown persistence as untrusted input, serialize deterministically,
  synchronize one-process writes, and publish by atomic sibling replacement;
- use framework-native request, response, dependency, lifecycle, and test
  facilities instead of building a universal web-framework wrapper;
- set finite client and test timeouts, encode URL components safely, validate
  response status and shape, close resources, and never retry implicitly;
- log unexpected server exceptions internally while returning only sanitized
  documented errors;
- keep examples and tests local, deterministic, and independent of public
  network services; and
- use names, module layout, error types, documentation style, and build commands
  that are conventional for **Target language**, not transliterations from
  Python or another source repository.

When a framework default conflicts with the shared status or error contract,
normalize it at the adapter boundary. Generated OpenAPI may be tested for
compatibility, but **OpenAPI source path** remains the portable source of truth.

## Learner experience

Organize starter work into five milestones:

1. domain and contracts;
2. SQLite and Markdown persistence;
3. first client and shared HTTP boundary;
4. first server adapter; and
5. remaining adapters, generated OpenAPI comparison, and interoperability.

The untouched starter must compile, build, or import successfully. Incomplete
operations should fail with explicit learner-facing messages rather than
missing modules, syntax errors, or accidental null behavior. Keep reference
source separate and direct learners to attempt each milestone first.

Document exact setup, build, run, format, lint, analyze, test, and coverage
commands. Include a concise comparison of what each server and client stack
makes explicit, adds, or hides. State that local development servers are
educational and not production deployment guidance.

## Testing and validation

Use **Test runner** to provide:

- pure domain/service tests with fakes;
- one repository contract parameterized over SQLite and Markdown;
- one black-box HTTP contract parameterized over all servers;
- one client contract parameterized over all transports;
- restart, monotonic-ID, malformed Markdown, and storage-failure coverage;
- malformed JSON, unknown field, invalid title/filter/ID, missing task,
  unsupported method, and sanitized internal-error coverage;
- malformed-response, connection, timeout, and cleanup coverage;
- real loopback tests on ephemeral ports with no public network; and
- an every-selected-client-by-every-selected-server interoperability matrix.

Run **Formatter**, **Linter**, **Static/type analyzer**, **Test runner**, and
**Coverage tool** using the repository's established conventions. Enforce at
least **Minimum coverage target** for this project without relying on coverage
from unrelated projects. Parse the OpenAPI YAML, check local Markdown links,
and compare generated framework OpenAPI semantically where available.

Use **Package/build tool** for dependency and build operations. Add dependencies
through the ecosystem's normal project files and clean-install path; do not
silently depend on globally installed packages.

## Boundaries

Do not add authentication, users, timestamps, priorities, tags, pagination,
search, bulk operations, browser UI, CORS design, WebSockets, async
requirements, deployment infrastructure, ORMs, schema migrations, generated
SDKs, automatic retries, cross-process Markdown locking, or checksum manifests.

Do not weaken existing repository gates, modify unrelated exercises, or rewrite
history. Keep changes staged and committed in meaningful validated phases using
the repository's required commit trailers.

## Completion report

Report:

- files and architecture added;
- the five learner milestones;
- exact quality and run commands;
- test and coverage results;
- OpenAPI, generated-contract, and link validation results;
- interoperability results;
- intentionally unsupported behavior;
- dependency or environment blockers; and
- confirmation that unrelated working-tree changes were preserved.
