---
id: index
title: spec-spine index
sidebar_position: 4
---

# spec-spine index

Scans manifests and specs to emit the codebase index, and provides staleness checks.

## Usage

```bash
spec-spine index
spec-spine index check [--slice NAME] [--json]
spec-spine index render
spec-spine index orphans [--json]
spec-spine index coverage [--json] [--fail-on-untraced]
```

## Subcommands

### `index` (default)

Scans the repository for manifests (e.g., `Cargo.toml`, `package.json`) and specs, resolving authority units to their owning specs. Emits per-unit and per-package index shards to `.derived/codebase-index/by-spec/<id>.json` and `.../by-package/<slug>.json`.

### `index check`

The staleness gate. It indexes the corpus in memory, without writing, and compares the result byte-for-byte with the committed shard tree (spec 086), the way [`compile --check`](./compile.md) does for the registry. Each drifted shard is named with its class:

- **`modified`**: a committed shard whose bytes differ from the shard a fresh index emits. A stale `shardHash`, a hand-edited body and a schema restamp all read this way.
- **`missing`**: a spec or package with no committed shard.
- **`orphaned`**: a committed shard with no spec or package behind it.

A shard carrying a blocking unresolved-unit diagnostic is reported as `blocking-diagnostics` instead, because regenerating does not fix it. `check`, and the freshness guard in front of `couple`, `index coverage` and `index owner`, run the same comparison, so a committed index that reads fresh is exactly what the corpus indexes to.

- **`--slice NAME`**: Checks staleness for a specific named slice defined in `[index.slices]` in the config, against its `slices.json` sidecar hash, rather than the whole tree.
- **`--json`**: Emit the [verdict envelope](./overview.md#machine-readable-verdicts---json) (`verb: "index.check"`) instead of prose.

Before 0.19.0 this verb compared only each shard's recomputed `shardHash`, and derived the files to hash from the committed body itself, so a hand-edited body that owned different `file`, `directory` or `crate` units could read fresh.

### `index render`

Renders the committed index as Markdown. This provides a human-readable view of the codebase index.
*(Note: `render` does not support `--json`.)*

### `index orphans`

Lists specs that have no resolved code units (i.e., specs that claim authority over paths that do not exist or cannot be resolved).

- **`--json`**: Output the list of orphaned spec IDs as a JSON array.

### `index coverage`

Reports, per source file inside a discovered package, whether a spec *specifically* claims it (a resolved unit or a `// Spec:` comment header), whether only a package's manifest floor covers it (**floor-only**), or whether nothing does (**unclaimed**). Freshness-guarded like `couple`: a stale committed index exits `2` rather than reporting against the wrong ledger. Prose, manifests, workflows, config, and paths under `resolver_exclusions` or the bypass set are never counted.

- **`--json`**: Output the `CoverageReport` (totals, the two sorted file lists, per-package counts).
- **`--fail-on-untraced`**: Exit `1` unless every source file is specifically claimed. The whole-tree "fully specified" assertion for CI.

The same classifier drives the coupling gate's `C-002` when `[coupling] require_ownership` is on, so this report lists exactly the files that flag would refuse.

## Exit Codes

- **`index` (write):**
  - `0`: OK.
  - `3`: I/O, parse, schema, or config error.
- **`index check`:**
  - `0`: Fresh.
  - `2`: Stale (at least one shard is `modified`, `missing`, `orphaned`, or carries a blocking diagnostic).
  - `3`: I/O, parse or schema error: no committed index, a committed shard from a schema MAJOR this build does not understand, or a committed shard file that does not parse.
- **`index coverage`:**
  - `0`: Reported (or, with `--fail-on-untraced`, fully claimed).
  - `1`: `--fail-on-untraced` and at least one source file is floor-only or unclaimed.
  - `2`: Stale (run `spec-spine index` first).
  - `3`: I/O or parse error (no committed index).

## Example

```bash
# Write the index
$ spec-spine index

# Check if the committed index is fresh
$ spec-spine index check
index is fresh

# After deleting one shard, hand-editing another, and copying a third under a
# name no spec has:
$ spec-spine index check
index is STALE (run `spec-spine index` to refresh)
3 stale shard(s):
  missing by-spec/004-codebase-index.json
  modified by-spec/005-coupling-gate.json
  orphaned by-spec/099-ghost.json
# (Exits with 2)
```

In a repository with unwitnessed claims (spec 057), the fresh report adds an `unwitnessed claims` count line beneath the verdict.
