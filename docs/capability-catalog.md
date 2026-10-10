# The capability catalog

Spec 162. One machine-readable description of every operation a spec-spine
binary performs, compiled into the binary and kept true by tests.

## Where it is

- `spec-spine capabilities --json`: spec 170's document, whose `catalog`
  member is the catalog.
- `spec-spine capabilities --operation <name> [--json]`: one record.
- `spec_spine_core::capability_catalog_json()`: the same catalog from the
  library facade, byte-identical to the `catalog` member.

All three read no repository, run nothing and open no connection. The CLI
forms are answered before `[meta] required_version` is read, so they answer
in any directory (spec 170 D-8, spec 162 D-5).

## What a record says

Each operation has a `name`. Where a verdict verb exists, the name is that
verb (`registry.show`, `content.select`); otherwise it is the argv path joined
by `.`; a flag that removes an operation's writes or executions forms its own
operation (`verify.plan`, `compact.plan`, `verify.affected`); a facade function
with no CLI form is `library.<stem>`.

| Member | Meaning |
|---|---|
| `summary`, `governedBy` | One sentence (the clap `about`), and the specs that govern it |
| `cli` | The argv path, every argument (`--long` or a positional id, value name, required, repeatable), and the output kind |
| `facade` | The `*_json` functions serving it, and whether each reads under a repository root |
| `request`, `response` | Argv or a request document; each answer's schema axis, version and embedded schema `$id` where one exists |
| `effects`, `effectsWhen` | What the CLI form reads, writes and executes, its network and environment use, and its authority-sensitive effects; what a flag adds |
| `preconditions`, `outcomes` | What must hold and the exit code its failure gives; every exit code with the error kinds that produce it |
| `budget`, `pagination` | Today only `content.select` is bounded; `null` / `none` means unbounded output |
| `stability`, `since`, `deprecation` | `stable` only when every governing spec is approved and complete |
| `examples` | Invocations the test suite runs against the built binary |
| `operationDigest` | `sha256:` over the canonical record without this member |

The effect vocabulary is closed and is a statement about spec-spine's own
code, not a sandbox. `declared-commands` (only `verify`) always comes with
`writes: [delegated]`, `network: delegated` and `authority:
[code-execution]`: what a spec declares can do anything. Treat it, and
`delegated`, as unbounded.

A response whose document carries no schema axis names the axis
`unversioned` with an empty version.

## Pinning an operation

A consumer that relies on an operation pins its `operationDigest`. The digest
excludes the tool version, so it moves only when the description moves.

```sh
spec-spine capabilities verify --expect content.select=sha256:<hex> --json
```

`current` exits 0; `changed` (both digests reported) and `missing` exit 1; a
malformed, repeated or absent `--expect` is usage, exit 3. Nothing writes a
pin: copy the observed digest yourself. `catalogDigest` identifies one
binary's catalog and moves on every release, so it is not a pin.

## How it stays true

- The clap census (`main.rs::capability_census`) holds every command path,
  argument and summary equal to the catalog's CLI bindings.
- The facade census holds the `*_json` functions in `lib.rs` (and the
  re-exported `selected_content_json`) equal to the facade bindings.
- Every verdict verb names exactly one operation; `gate-verdict` marks exactly
  the operations `scripts/statecraft/gate.sh` runs.
- The effect audit reads each command module for `git` invocations, clock and
  environment reads, temporary directories and file writes, and fails on any
  its operations do not declare.
- Every example runs against the built binary in a fresh copy of its fixture
  (`crates/spec-spine-cli/tests/fixtures/capability-catalog/`), and must exit
  with a code its operation declares.
- The catalog validates against `capability-catalog.schema.json`, and its
  digests match an independent construction.

## What it is not

Not an authorization, a sandbox or a rate limit. Not generated prose: the
reference documentation and any tool projection are separate consumers of
this document.
