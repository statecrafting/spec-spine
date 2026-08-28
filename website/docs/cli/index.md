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
spec-spine index check [--slice NAME]
spec-spine index render
spec-spine index orphans [--json]
spec-spine index coverage [--json] [--fail-on-untraced]
```

## Subcommands

### `index` (default)

Scans the repository for manifests (e.g., `Cargo.toml`, `package.json`) and specs, resolving authority units to their owning specs. Emits per-unit and per-package index shards to `.derived/codebase-index/by-spec/<id>.json` and `.../by-package/<slug>.json`.

### `index check`

The staleness gate. It recomputes the content hash of the current inputs and compares it against the committed index shards.

- **`--slice NAME`**: Checks staleness for a specific named slice defined in `[index.slices]` in the config, rather than the global content hash.

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
  - `2`: Stale (committed index is out of date).
  - `3`: I/O or parse error.
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
Error: Index is stale. Expected hash abc123def456, actual hash fed654cba321.
# (Exits with 2)
```
