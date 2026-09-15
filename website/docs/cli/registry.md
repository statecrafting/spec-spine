---
id: registry
title: spec-spine registry
sidebar_position: 5
---

# spec-spine registry

Typed read-only queries against the compiled registry.

## Usage

```bash
spec-spine registry list [--status S] [--ids-only] [--json]
spec-spine registry show <id> [--json]
spec-spine registry status-report [--nonzero-only] [--json]
spec-spine registry relationships <id> [--json]
spec-spine registry plan [--json]
```

## Subcommands

### `registry list`

Lists specs from the committed registry.

- **`--status S`**: Filter by status (`draft`, `approved`, `superseded`, `retired`).
- **`--ids-only`**: Print only the spec IDs, one per line.
- **`--json`**: Output a read document: `{ "items": [...], "schemaVersion" }`, where `items` holds the spec records, or the id strings with `--ids-only`.

Every `registry` subcommand's `--json` output is a **read document** (spec 093): a JSON object with sorted keys and a top-level `schemaVersion` on the read-document axis. A read is not a verdict, so it is not wrapped in the `ok` / `exitCode` / `report` envelope the gate verbs use.

### `registry show <id>`

Shows the details of a single spec.

- **`--json`**: Output as JSON.

`contentHash` is the spec's committed registry shard hash, read from the ledger
and never recomputed (spec 055). It is **path-framed**: SHA-256 over the
spec's repo-relative POSIX path, a NUL byte (`0x00`), then the file's
normalized bytes (BOM stripped, CRLF and CR folded to LF). It therefore does not
equal a plain SHA-256 of the file. The unframed digest is `specSourceHash`,
which `spec-spine attest --spec <id>` reports (spec 096).

### `registry status-report`

Shows counts of specs by their lifecycle status.

- **`--nonzero-only`**: Omit statuses with a count of zero.
- **`--json`**: Output as JSON.

### `registry relationships <id>`

Shows the relationship neighborhood (incoming and outgoing edges) for a specific spec.

- **`--json`**: Output as JSON.

### `registry plan`

The ready set (spec 038): which specs a scheduler may hand out now, and what blocks the rest.

- **Excluded** from the output entirely: `status` is `superseded` or `retired`, or `implementation` is `complete`, `n-a` or `deferred`. A spec with no `implementation` key counts as `pending`.
- **Blocked**: at least one `depends_on` target is not finished (neither `complete` nor `n-a`). Each blocker is named with its `state`; a `depends_on` target that does not resolve to a spec blocks with `state: "unresolved"` rather than being ignored.
- **Ready**: everything else, in dependency (topological) order, ties broken by id.

`implementation` is a hint to the scheduler, never evidence that a spec is done; the evidence is the indexer's verdict, recorded by [`attest --spec`](./attest.md). See [Lifecycle and Completion](../concepts/lifecycle.md).

- **`--json`**: Output `{ "ready": [{ "id", "title" }], "blocked": [{ "id", "title", "blockedBy": [{ "id", "state" }] }], ..., "schemaVersion" }`.
- **`--next`**: Only the first ready spec. With `--json`, `{ "next": { "id", "title" }, "schemaVersion" }`, and `"next": null` when nothing is ready (exit 0 either way).

## Exit Codes

- `0`: OK.
- `1`: Spec ID or view not found.
- `3`: I/O, parse, schema, or config error.

## Example

```bash
$ spec-spine registry list --status approved --ids-only
000-bootstrap
001-compile-registry
002-registry-query

$ spec-spine registry plan
045-next-thing
ready: 1, blocked: 0
```
