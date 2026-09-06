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
- **`--json`**: Output as JSON.

### `registry show <id>`

Shows the details of a single spec.

- **`--json`**: Output as JSON.

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

- **`--json`**: Output `{ "ready": [ids], "blocked": [{ "id", "blockedBy": [{ "id", "state" }] }] }`.

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
