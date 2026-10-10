---
id: "162-capability-catalog"
title: "Every operation is described once, and the description is tested"
status: draft
kind: "governance"
created: "2026-09-27"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "034-machine-readable-verdicts"
  - "074-a-governed-read-names-its-version"
  - "110-an-interface-reference-is-digest-pinned"
  - "132-one-exit-contract-for-the-family"
  - "152-a-refusal-says-what-it-is-in-every-form"
  - "155-selected-content-accessor"
  - "170-a-consumer-is-served-answers-not-access"
summary: >
  Adds a versioned, canonical, digest-bearing capability catalog compiled into
  the binary: every CLI and facade operation with its summary, schema
  references, exact effects, preconditions, exit outcomes, budgets,
  pagination, stability, deprecation and examples. The catalog rides the one
  `capabilities` answer spec 170 established, as an additive member, so a
  binary still states what it supports through one path. Census tests hold it
  to the clap tree, the facade, the verdict verbs and the managed gate; the
  examples run against the built binary. `capabilities verify` checks pinned
  operation digests. The catalog describes; it authorizes nothing.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/capability.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/capability-catalog.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/capability.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/capability_catalog.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/capability_catalog.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-cli/tests/fixtures/capability-catalog/", planned: true }
  - { kind: file, path: "docs/capability-catalog.md", planned: true }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/src/schema.rs" }, nature: additive }
  - { spec: "034-machine-readable-verdicts", unit: { kind: file, path: "crates/spec-spine-types/src/verdict.rs" }, nature: additive }
  - { spec: "170-a-consumer-is-served-answers-not-access", unit: { kind: file, path: "crates/spec-spine-types/src/capabilities.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "170-a-consumer-is-served-answers-not-access", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_capabilities.rs" }, nature: additive }
  - { spec: "170-a-consumer-is-served-answers-not-access", unit: { kind: file, path: "crates/spec-spine-cli/tests/capabilities.rs" }, nature: additive }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_interface.rs" }, nature: corrective }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap (item H, CC-01)" }
  - { unit: { kind: file, path: "scripts/statecraft/gate.sh" }, role: "the managed gate whose spec-spine invocations the gate census reads" }
  - { unit: { kind: file, path: "crates/spec-spine-types/src/error.rs" }, role: "Error::exit_code, the one mapping" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every CLI invocation form and every public facade function is described by exactly one catalog operation record, and no record describes an operation that does not exist."
    anchor: "3-10-the-catalog-is-held-to-the-code"
  - id: "R-2"
    kind: requirement
    text: "Each operation states its read, write, execute, network, environment and authority-sensitive effects in closed vocabularies, and never understates an effect the code performs."
    anchor: "3-4-effects-are-declared-exactly"
  - id: "R-3"
    kind: requirement
    text: "Each operation's declared outcomes follow spec 132's exit contract, and every example exits with a code its operation declares."
    anchor: "3-5-preconditions-and-outcomes"
  - id: "R-4"
    kind: requirement
    text: "The catalog and each operation carry a canonical sha256 digest, and a consumer can verify pinned operation digests against the running binary."
    anchor: "3-11-digests-discovery-and-verification"
  - id: "I-1"
    kind: invariant
    text: "The catalog is static data compiled into the binary: showing it reads no repository content, runs no example, opens no network connection, and grants no authority."
    anchor: "3-14-trust-boundary"
  - id: "V-1"
    kind: verification
    text: "The catalog is checked for canonical bytes, schema conformance, facade and clap census, verdict-verb names, gate agreement, effect audit, stability rule, digest verification, and every example against the built binary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/capability_catalog.rs"
      - "crates/spec-spine-cli/tests/capability_catalog.rs"
      - "crates/spec-spine-cli/src/main.rs"
intent:
  goal: "describe every spec-spine operation once, machine-readably, and keep that description true by test"
  non_goals:
    - "generating reference prose or transport descriptions (spec 166 and a later read-only projection)"
    - "authorizing, sandboxing or rate-limiting any operation"
    - "a dynamic plugin registry or runtime operation discovery beyond this binary"
    - "adding budgets or pagination to operations that have none today"
---

# 162: Every operation is described once, and the description is tested

## 1. Purpose

A consumer of spec-spine today learns what an operation does from several
places that are maintained separately: clap doc comments (`--help`), the
`*_json` functions in `crates/spec-spine-core/src/lib.rs`, the verb constants
in `crates/spec-spine-types/src/verdict.rs`, the `capabilities` document spec
170 added, and prose in `docs/cli-reference.md` and `docs/api.md`. Spec 170's
document answers which verbs exist, their flags and their `--json` schema
axes. Nothing answers what each one reads and writes, whether it runs `git` or
declared commands, which exit codes it returns, or whether its output is
bounded, and the separate surfaces already disagree: `spec-spine interface
verify --help` (`cmd_interface.rs`) says a stale committed registry exits `2`;
spec 132 moved staleness to `1`, the code returns `Error::Stale` (exit `1`),
and `docs/cli-reference.md` says `1`.

The consumers queued behind this spec need those answers without parsing
prose. Spec 166 generates reference documentation, and a later read-only
projection would expose operations as tools; both need to know, per
operation, its effects, outcomes, schema and bounds.

This spec defines one catalog that describes current behavior and changes no
existing operation's arguments, output bytes or exit codes, with one
exception: spec 170's `capabilities --json` document gains the catalog as an
additive member (§3.12). The new behavior is the catalog, its member in that
document, `capabilities --operation`, `capabilities verify`, and a one-line
help-text correction.

## 2. Territory

- `crates/spec-spine-types/src/capability.rs`: the catalog DTOs and closed
  vocabularies; the embedded schema (through `schema.rs`);
  `CATALOG_SCHEMA_VERSION` in `version.rs`; verb constants `capabilities` and
  `capabilities.verify` in `verdict.rs`.
- `crates/spec-spine-core/src/capability.rs`: the one definition, the digest
  construction and the pin verifier, exposed through `lib.rs` as
  `capability_catalog()`, `capability_catalog_json()` and
  `capability_verify_json(request_json)`.
- Spec 170's `capabilities` verb, extended: `capabilities.rs` (the document
  gains `catalog`), `cmd_capabilities.rs` (`--operation`, the `verify`
  subcommand) and `tests/capabilities.rs`; `main.rs` wires the subcommand and
  carries a unit test `capability_census` walking `Cli::command()`, as
  `spec_id_census` does (an integration test cannot import from the binary
  crate).
- `crates/spec-spine-cli/src/cmd_interface.rs`: the stale help line, corrected.
- The planned tests, fixtures and `docs/capability-catalog.md`, plus entries in
  the CLI reference, API and schema-versioning documents.

Every other `cmd_*.rs` and facade function stays owned by its spec. The
catalog describes them; it does not edit them.

## 3. Behavior

### 3.1 One catalog, one definition per operation

The catalog is a single document built by `capability_catalog()` from static
data in `crates/spec-spine-core/src/capability.rs`. It is a pure function of
the build: no file, clock, environment, or repository is read. Top-level
members are exactly `schemaVersion`, `tool` (`"spec-spine"`), `toolVersion`
(the core crate's compile-time package version), `compatibility`,
`discovery`, `operations`, and `catalogDigest`.

`compatibility` maps every schema axis the crate declares (registry, index,
build-meta, attestation, spec-attestation, snapshot, verdict, delta, read,
config, capabilities, catalog) to its current constant, so a consumer
compares the axes it can parse against the running binary in one read.

### 3.2 Operation names

Every operation has a unique `name`, `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*$`:

- Where a verdict verb constant exists in `verdict.rs::verb`, the name MUST
  equal it (`compile.check`, `registry.show`, `content.select`,
  `interface.verify`, `capabilities`, and the rest). A consumer that already
  branches on the envelope `verb` keys the catalog the same way.
- Otherwise the name is the argv path joined by `.` (`index.render`,
  `compile`), and the bare `index` invocation is `index.build`.
- A flag that removes the operation's write or execute effects forms its own
  operation, named with a suffix: `verify.plan`, `compact.plan`, and
  `verify.affected` (`verify --affected-by <base> --plan`, spec 158). A flag
  that adds an effect (`attest --sign`) is a conditional effect (§3.4), and a
  flag that selects another response document (`attest --snapshot`,
  `attest --spec`) is a conditional response (§3.6), of one operation.
- A facade function with no CLI form is named `library.<stem>`, the function
  name without `_json` and with `_` spelled `-` (`library.scaffold-init`,
  `library.load-config`).

Operations are sorted by `name` in the document.

### 3.3 The operation record

Each record has exactly these members, all present, `null` or empty where
nothing applies:

- `name`, `summary` (one sentence), `governedBy` (sorted spec ids).
- `cli`: `null` or `{ argv, flags, output }`. `argv` is the fixed token path;
  `flags` lists every clap argument of that command, sorted by name, each with
  `name` (`--long` for a flag, the bare argument id for a positional),
  `positional`, `valueName`, `required`, `repeatable`, and `conflictsWith`;
  `output` is one of `verdict-envelope`, `read-document`, `text`, or `files`.
  Several operations may share one clap command (`verify`, `verify.plan`,
  `verify.affected`); each lists that command's full argument set.
- `facade`: sorted `{ function, selector, readsRepository }` bindings.
  `selector` is `null` or the discriminator a multi-operation function takes
  (`query_json`'s `op`).
- `request`, `response`: schema references (§3.6).
- `effects`, `effectsWhen`, `preconditions`, `outcomes` (§3.4, §3.5).
- `budget`, `pagination` (§3.7).
- `stability`, `since`, `deprecation` (§3.8).
- `examples` (§3.9) and `operationDigest` (§3.11).

### 3.4 Effects are declared exactly

`effects` describes the CLI binding. Every member is a sorted list over a
closed vocabulary, or a single closed token:

| Member | Vocabulary |
|---|---|
| `reads` | `attestation`, `caller-file`, `clock`, `config`, `corpus`, `derived-ledger`, `export-directory`, `key-material`, `source-tree`, `stdin` |
| `writes` | `attestation`, `caller-path`, `corpus`, `delegated`, `derived-ledger`, `temporary` |
| `executes` | `declared-commands`, `git` |
| `network` | `none` or `delegated` |
| `environment` | the exact variable names read (`SPEC_SPINE_PR_BODY`, `SPEC_SPINE_VERIFY_STACK`) |
| `authority` | `code-execution`, `corpus-rewrite`, `gate-verdict`, `ledger-rewrite`, `signature-verification`, `signing`, `waiver-evaluation` |

Rules the vocabulary encodes:

- `git` means a `git` subprocess that reads objects, refs, the index or the
  working tree. Every invocation spec-spine makes today is such a read. A git
  invocation that writes is not expressible; adding one requires a catalog
  MAJOR.
- `declared-commands` (only `verify`) MUST be accompanied by `writes:
  [delegated]`, `network: delegated` and `authority: [code-execution]`: the
  commands a spec declares can do anything, and the catalog says so rather
  than guessing.
- Every other operation's `network` is `none`. spec-spine opens no connection;
  `interface verify` reads local export directories only (spec 110 §3.3).
- `clock` names a wall-clock read (`compile` writing `build-meta.json`'s
  `builtAt`, `attest`, and the unique temporary names the git-reading
  operations build).
- `gate-verdict` marks an operation whose exit code the managed gate consumes
  (§3.10, item 4).

`effectsWhen` lists `{ flag, effects }` entries added to the base when the
flag is present (`attest --sign` adds `reads: [key-material]` and `authority:
[signing]`). A flag never subtracts: `index coverage --paths-from` skips
`git`, and `git` is still declared in the base.

Facade bindings carry no effects list of their own. The core invariant applies
to every facade function: it writes nothing, executes nothing, opens no
connection, reads no environment or clock, and reads files only when it takes
a `repo_root` argument, in which case it reads `config`, `corpus`,
`derived-ledger` and `source-tree` under that root. Each binding states this
once as `readsRepository: true|false`.

### 3.5 Preconditions and outcomes

`preconditions` is a sorted list over a closed set, each with the exit codes
its failure produces: `version-pin-met` (2, every CLI operation except the two
`capabilities` operations, which `main.rs` answers before reading `[meta]
required_version`, spec 170 D-8), `config-loadable` (2), `registry-fresh` (1:
for example `registry closure` and `interface verify`), `clean-tree` (2:
`content select` without `--revision`, `compact` without `--force`),
`git-repository` (4), `refs-resolvable` (1 or 4, as the code maps it).

`outcomes` is a sorted list of `{ exitCode, outcome, kinds }`. `exitCode` and
`outcome` are one of spec 132's five pairs; `kinds` is a subset of
`verdict.rs::ERROR_KINDS` whose `Error::exit_code()` equals `exitCode`, empty
for `0`. Every CLI operation declares `3` (clap usage) and `4` (I/O); every
operation subject to the pin declares `2`. `delta` declares no `1`, because it
has no finding of its own. The build takes each operation's set from reading
its `cmd_*.rs` and the `Error` variants it can return; §3.9 holds that set to
observed behavior.

### 3.6 Request and response schemas and versions

`request` is `{ kind, schema }` where `kind` is `argv` (flags only) or
`document` (a JSON or YAML request: `content select --request`, `registry
closure`, `scope evaluate`, `compact --plan-file`, and every facade function's
`request_json`). `response` is a list, sorted with the `null` entry first, of
`{ when, axis, version, document, schema }`, where `when` is `null` for the
default answer or the flag that selects another (`attest --snapshot` answers
on the snapshot axis):

- `axis` names a schema axis and `version` is its constant at build time,
  read from the constant, never restated as a literal.
- `document` is a stable name for the shape (`registry-show`,
  `content-selection`, `verdict-envelope`).
- `schema` is the embedded JSON Schema's `$id` where one exists (registry,
  index, build-meta, index inputs, the affected selection, and the catalog
  itself), and `null` otherwise.

Most read documents have no embedded schema today. The catalog records that
`null` honestly; authoring the missing schemas is not this spec (D-3).

### 3.7 Budgets and pagination

`budget` is `null` or `{ maxBytes, maxItems }`, each `{ default, min, max }`.
`pagination` is `none` or `continuation`. Today exactly one operation is
bounded: `content.select` declares `maxBytes` 262144 (1 to 1048576),
`maxItems` 64 (1 to 256) and `continuation` (spec 155 §3.1). Every other
operation declares `budget: null`, `pagination: none`, which a consumer MUST
read as unbounded output. This spec adds no budget to any existing operation.

### 3.8 Stability, deprecation and compatibility

`stability` is `stable`, `experimental`, or `deprecated`.

- `stable` MUST NOT be declared unless every spec in `governedBy` is
  `approved` with `implementation: complete` in this repository's committed
  registry. The rule is one-directional: an approved operation may stay
  `experimental` until someone promotes it, so a ratification never forces a
  catalog edit.
- `deprecated` MUST carry `deprecation: { since, replacement, note }`, where
  `since` is a released package version and `replacement` is an operation name
  or `null`. Any other stability carries `deprecation: null`.
- An operation marked `deprecated` in a published release MAY be removed only
  in a later release; removing a non-deprecated operation is refused by the
  census test unless its record was first deprecated.
- `since` is the package version that first shipped the operation for
  operations added after this spec, and `null` for operations that predate the
  catalog (D-4).

Consumer compatibility is the pair `compatibility` (axes) plus `toolVersion`:
a consumer supports an operation when it parses the operation's response axis
at that MAJOR.

### 3.9 Examples run against the built binary

Every operation with a CLI binding carries at least one example
`{ id, fixture, argv, stdin, exitCode, stdoutIncludes }`. `fixture` names a
directory under `crates/spec-spine-cli/tests/fixtures/capability-catalog/`;
a fixture that needs history has `base/` and `head/` trees the test commits in
order. `stdoutIncludes` is a sorted list of literal substrings, possibly
empty.

`crates/spec-spine-cli/tests/capability_catalog.rs` MUST run every example
against `env!("CARGO_BIN_EXE_spec-spine")` with `--repo` set to a fresh
temporary copy of its fixture, and MUST assert the exit code, that the exit
code is one the operation declares, and each `stdoutIncludes` substring.
Writing and executing examples run only in that copy. No example reaches the
network; the `verify` example's fixture declares only `exit 0` and `exit 1`
commands. Operations with an error outcome other than `3` and `4` carry at
least one example exercising a non-zero code they declare.

Facade-only operations carry examples whose `argv` is empty and whose request
is the `stdin` member; the core test calls the function directly.

### 3.10 The catalog is held to the code

The following checks MUST all exist and fail when the catalog and the code
disagree:

1. **Clap census** (`main.rs` unit test `capability_census`): the set of
   command paths and each path's argument names in `Cli::command()` equals the
   catalog's `cli` bindings, including global `--repo`. Each command's clap
   `about` first line equals the catalog `summary` of the operation named for
   that command path.
2. **Facade census** (core test): the set of `pub fn <name>_json` defined in
   `crates/spec-spine-core/src/lib.rs`, plus `_json` names it re-exports with
   `pub use` (today `selected_content_json`), minus an exclusion list in the
   test where each entry states why it is not a facade entry point (today
   `is_package_json`, a predicate), equals the set of functions named in
   `facade` bindings; each binding's `readsRepository` matches whether the
   signature takes `repo_root`.
3. **Verb census** (core test): every `verdict.rs::verb` constant is the name
   of exactly one operation whose `cli.output` is `verdict-envelope` or
   `read-document`.
4. **Gate census** (core test): the operations carrying `gate-verdict` are
   exactly those `scripts/statecraft/gate.sh` invokes through its
   `spec_spine` function, read the way `harness_skills.rs` reads that file
   (D-9).
5. **Effect audit** (CLI test): for each `cmd_*.rs` module, the test detects
   `Command::new("git")`, `env::var(`, `now_utc`/`SystemTime::now`,
   `temp_dir`, and file-write calls, and asserts every detected effect is
   declared by at least one operation that module serves. The module-to-
   operation map lives in the test. The audit is one-directional: it catches
   understatement, which is the dangerous error.
6. **Stability rule** (core test): §3.8 against the committed registry, and
   every `governedBy` id names a spec in it.
7. **The two lists agree** (CLI test): the CLI command paths the catalog
   binds equal the `verbs` paths of spec 170's document, so the one
   `capabilities` answer cannot disagree with itself (D-8).

### 3.11 Digests, discovery and verification

`operationDigest` is `sha256:` plus the lowercase hex SHA-256 of the canonical
JSON bytes (§3.13) of the record with `operationDigest` omitted.
`catalogDigest` is the same construction over the whole catalog with
`catalogDigest` omitted. A consumer pins operation digests, which exclude
`toolVersion` and so move only when the operation's description moves.

`spec-spine capabilities verify --expect <name>=sha256:<hex>... [--json]`
recomputes each named operation's digest from the running binary's catalog
and reports, sorted by name, one outcome per expectation:

| Outcome | When | Exit |
|---|---|---|
| `current` | the digest matches | 0 |
| `changed` | the operation exists and its digest differs; both digests are reported | 1 |
| `missing` | no operation has that name | 1 |

A malformed or repeated `--expect`, or none at all, is usage (exit 3). The
verifier is `capability_verify_json({ "expect": { "<name>": "<digest>" } })`
in the facade. Nothing ever writes a pin: as in spec 110 §3.6, the observed
digest is printed for a human to copy, never applied.

`discovery` makes the existing digest-pinned interface machinery findable from
the catalog: `{ "interfaceReferences": { "declaredBy": "registry.show",
"member": "interfaceReferences", "pinSources": ["contentHash",
"sectionDigests"], "verifiedBy": "interface.verify" }, "capabilities": {
"verifiedBy": "capabilities.verify" } }`. It names operations; it does not
locate corpora (spec 110 §4 keeps that with the caller).

### 3.12 The verb, the facade and the exit contract

The catalog is served by spec 170's `capabilities` verb, not by a second one
(170 §3.2, D-8):

- `spec-spine capabilities --json` writes spec 170's document with one
  additive member, `catalog`, holding the catalog byte-for-byte as
  `capability_catalog_json()` builds it (parsed, not re-encoded as a string).
  The capabilities axis moves MINOR (`0.1.0` to `0.2.0`); `version` and
  `verbs` are unchanged. Without `--json` the human output is unchanged.
- `spec-spine capabilities --operation <name> [--json]` writes one operation
  record. Under `--json` it is the read document `{ "schemaVersion":
  <catalog axis>, "operation": <record> }`; without it, one line naming the
  operation, its stability and a compact effects string. An unknown name is
  `not-found`, exit 1.
- `spec-spine capabilities verify` is §3.11.

Both operations answer failures with the family envelope under `--json` (spec
152), with verbs `capabilities` and `capabilities.verify`. Both are answered
before the version pin, as spec 170 D-8 answers `capabilities`: they read no
repository, so neither a pin this binary does not meet nor an unreadable
`spec-spine.toml` refuses them (D-5). Both are themselves cataloged.

### 3.13 Determinism and the schema axis

The catalog is emitted as canonical JSON: sorted keys, two-space indent, LF,
trailing newline, no insignificant whitespace variation. Every list is sorted
by the key named for it. Repeated calls MUST produce identical bytes, and
because the input is static data, builds of one revision on different
platforms MUST agree; the core test compares two calls and pins the digest
construction with an independent SHA-256.

`CATALOG_SCHEMA_VERSION` starts at `1.0.0`. Adding an optional member or a
vocabulary token is MINOR; removing or renaming a member or a token, or
changing the digest construction, is MAJOR. The catalog's own axis is
independent of the read, verdict and capabilities axes. The core test
validates the emitted catalog against the embedded schema.

### 3.14 Trust boundary

The catalog is data compiled into the binary. Showing or verifying it reads no
spec, source, derived file or git object, runs no example, and opens no
connection. It is a description, not a permission: an operation listed with
`network: none` is a statement about spec-spine's own code, not a sandbox, and
an operation listed at all is not thereby authorized for any caller.
Consumers own authorization, and MUST treat `declared-commands` and
`delegated` as unbounded.

Examples are published as documentation; only `cargo test` executes them.

### 3.15 Inventory

Measured against `main.rs` and `lib.rs` at the 2026-10-10 refile. The build
MUST reconcile this with the censuses and record any difference as a decision
entry.

CLI operations (37): `attest`, `capabilities`, `capabilities.verify`,
`check`, `compact`, `compact.plan`, `compile`, `compile.check`,
`compile.spec`, `config.show`, `content.select`, `couple`, `delta`,
`index.build`, `index.check`, `index.coverage`, `index.diagnostics`,
`index.orphans`, `index.owner`, `index.render`, `interface.verify`, `lint`,
`registry.closure`, `registry.impacts`, `registry.list`, `registry.moves`,
`registry.obligation`, `registry.plan`, `registry.relationships`,
`registry.show`, `registry.status-report`, `scope.compare`, `scope.evaluate`,
`verify`, `verify.affected`, `verify.plan`, `verify-attestation`.

Effects that are not `reads: [config, corpus, derived-ledger]` alone:
`compile` and `index.build` write `derived-ledger` (`ledger-rewrite`);
`index.build` and `index.coverage` execute `git`; `attest` writes
`attestation`, with `--sign` reading `key-material` (`signing`);
`verify-attestation` reads `attestation` and, with `--signature`,
`key-material` (`signature-verification`); `compact` executes `git`, reads a
`caller-file`, writes `corpus` and `caller-path` (`corpus-rewrite`);
`couple` executes `git`, reads `SPEC_SPINE_PR_BODY` and a `caller-file`,
writes `temporary` (`gate-verdict`, `waiver-evaluation`); `delta` executes
`git` and writes `temporary`; `content.select` executes `git` and writes
`temporary`; `verify.affected` executes `git` and writes `temporary`;
`interface.verify` reads `export-directory`; `verify` executes
`declared-commands` and reads `SPEC_SPINE_VERIFY_STACK`; the two
`capabilities` operations read nothing.

Facade functions: the 30 `pub fn *_json` defined in `lib.rs`, spec 155's
re-exported `selected_content_json`, and this spec's two, 33 in all. Those
without a CLI form become `library.*` operations, as the census finds them.

## 4. Acceptance criteria

1. `capabilities --json` emits spec 170's document whose `catalog` member
   validates against the embedded schema, is byte-identical across two runs,
   and names every operation in §3.15 and no other.
2. Adding a clap argument, a subcommand, or a `pub fn *_json` without a
   catalog change fails a census; so does removing one.
3. A `cmd_*.rs` module that invokes `git`, reads an environment variable or
   the clock, or writes a file that no operation it serves declares fails the
   effect audit.
4. Every example exits with its recorded code against the built binary, and
   that code is among its operation's declared outcomes.
5. `capabilities verify` answers `current`, `changed` and `missing` with exits
   0, 1, 1, and a malformed `--expect` with 3; its `--json` envelope carries
   verb `capabilities.verify`.
6. `interface verify --help` states exit 1 for a stale registry.
7. No existing command's arguments, output bytes, or exit codes change, except
   `capabilities --json` gaining `catalog` on its axis' MINOR; the existing
   CLI and core suites pass unchanged.

## 5. Out of scope

- Generating `docs/cli-reference.md` or any prose from the catalog: spec 166.
  This spec ties clap summaries to the catalog by test, not by generation.
- Tool descriptions, transports, or a server for a read-only projection.
- Authoring JSON Schemas for the read and verdict documents that lack one.
- Adding budgets or pagination to unbounded operations.
- Sandboxing, authorizing, or rate-limiting any operation.
- Runtime plugins or operations not compiled into this binary.
- Changing any operation's behavior, except the one help-text correction.

## 6. Resolved decisions

**D-1 (2026-09-27): the definition lives in core as data, and clap is held to
it by test.** Generating clap's help from the catalog would mean rewriting
`main.rs` in clap's builder API, a large diff for a property a census gives
cheaply. "Generate where practical" is met by spec 166 (prose) and a later
projection (tool descriptions) generating from this document. Rejected:
deriving the catalog from `Cli::command()` at runtime, which yields arguments
but not effects, outcomes or schemas.

**D-2 (2026-09-27): effects describe the CLI; one invariant covers the
facade.** Listing reads per facade function would restate it thirty times.

**D-3 (2026-09-27, draft, owner to confirm): `schema: null` is recorded, not
filled.** Writing JSON Schemas for every read document is a separate and
larger unit. The catalog makes the gap visible per operation. Owner to
confirm whether a follow-up spec should close it before a read-only
projection ships.

**D-4 (2026-09-27, draft, owner to confirm): `since` is null for operations
that predate the catalog.** Reconstructing first-release versions from
history is costly and unverifiable by test. Owner to confirm, or supply the
versions from the release notes.

**D-5 (2026-10-10, refile; replaces the 2026-09-27 draft's "the version pin
applies to `capabilities`"): the capabilities operations are answered before
the pin.** Spec 170 (approved after this draft was written) shipped
`capabilities` and its D-8 answers it before `[meta] required_version` is
read, because it reads no repository. This spec follows that rule for
`--operation` and `verify` rather than amending 170: neither reads a
repository either, and a consumer asking a mismatched binary what it can do
gets the binary's own answer.

**D-6 (2026-09-27): the effect audit is one-directional.** Proving that a
declared effect is performed would need a trace of every code path; detecting
performed effects and requiring their declaration catches the error that
misleads a consumer about safety.

**D-7 (2026-09-27): pins are per operation.** `catalogDigest` moves on every
release because it covers `toolVersion`; a consumer pinning it would see
`changed` on every upgrade. `capabilities verify` therefore verifies operation
digests only; `catalogDigest` identifies one binary's catalog.

**D-8 (2026-10-10, refile): the catalog rides spec 170's answer.** The
2026-09-27 draft added `capabilities show` and claimed
`cmd_capabilities.rs`. Spec 170 has since established that file and a bare
`capabilities` verb, and its R-2 allows one path per question. Two verbs that
both list the binary's operations would be two paths. The catalog is
therefore a member of 170's document, `--operation` is a flag of the same
verb, and `verify` its subcommand; `capabilities.show` is renamed
`capabilities`. 170's `verbs` stays walked from the clap tree, and §3.10
item 7 holds it equal to the catalog's CLI bindings, so the one answer
cannot disagree with itself. Rejected: deriving 170's `verbs` from the
catalog, which would change how an approved spec computes its answer.

**D-9 (2026-10-10, refile): the gate census reads the managed gate.** The
draft read "the fenced gate list in `AGENTS.md`". Since spec 156 that list
runs `scripts/statecraft/gate.sh` modes, and the spec-spine invocations live
in that script's `spec_spine` lines, which `harness_skills.rs` already parses.
The census reads every such line, so `gate-verdict` marks `check`, `lint`,
`index.coverage`, `index.check`, `index.owner`, `registry.list` and `couple`:
each one's non-zero exit stops the gate.

**D-10 (2026-10-10, refile): the inventory is remeasured.** Since the draft,
spec 158 added `verify --affected-by <base> --plan`, a git-reading,
non-executing form that is its own operation (`verify.affected`) by §3.2's
flag rule, and the facade gained `affected_json` and lost nothing; 30
`*_json` functions are defined in `lib.rs`, not 28. Global `--repo` and
positional arguments are listed with a `positional` member so a consumer can
build an argv from the record alone.

**D-11 (2026-10-10, refile): renumbered references.** The draft named spec
167 as the read-only projection; no such spec is filed, so the text names
the consumer without an ordinal. Spec 166 keeps its name because its draft
exists.

## Verification

Before the build, the first three lines fail (the two test targets do not
exist; the census is absent from the binary's test list) and so does the last
(`capabilities verify` is not a subcommand: clap usage, exit 3). The fourth
runs the census by name, and the fifth builds the binary the last line drives.

```verify:cli
cargo test -p spec-spine-core --test capability_catalog --locked
cargo test -p spec-spine-cli --test capability_catalog --locked
cargo test -p spec-spine-cli --bin spec-spine --locked -- --list | grep -q 'capability_census: test'
cargo test -p spec-spine-cli --bin spec-spine --locked capability_census
cargo build --release -p spec-spine-cli --locked
./target/release/spec-spine capabilities verify --expect capabilities.verify=sha256:0000000000000000000000000000000000000000000000000000000000000000 --json | grep -q '"changed"'
```
