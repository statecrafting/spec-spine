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

The staleness gate. It indexes the corpus in memory, without writing, and compares the result byte-for-byte with the committed shard tree (spec 086), the way [`compile --check`](./compile.md) does for the registry. Each stale shard is named with its class:

- **`modified`**: a committed shard whose bytes differ from the shard a fresh index emits. A stale `shardHash`, a hand-edited body and a schema restamp all read this way.
- **`missing`**: a spec or package with no committed shard.
- **`orphaned`**: a committed shard with no spec or package behind it.
- **`blocking-diagnostics`**: **machine surface only.** Not printed in prose since spec 098; it still reaches `--json` and library callers, as the paragraph below describes.

A spec claiming a unit that does not resolve is a different refusal, and since spec 098 the prose says so separately rather than calling it staleness. What spec 098 3.3 fixes is the content, not the wording: the report is classed as an unresolved claim distinct from staleness, it carries one line per diagnostic naming the diagnostic code, the owning spec and the unit, it states that regenerating the index does not clear it, and it points at `spec-spine index diagnostics` when the list is capped. When the owning spec declares `implementation: complete`, the report adds that the spec and the tree disagree about what exists. Where shards moved as well, both halves are reported and regeneration is attributed to the stale half alone.

Current output puts that section under an `UNRESOLVED CLAIM:` heading carrying the claim and spec counts. Treat the classed facts above as the contract and the exact phrasing as illustrative: 098's acceptance pins the code, the spec id, the unit and the regeneration statement, and deliberately does not pin the heading text. Match on the diagnostic code, never on the sentence.

The exit code is unchanged either way: `2`. In the [verdict envelope](./overview.md#machine-readable-verdicts---json) and in the library verdict, an unresolved claim still appears in the drift vector as a **`blocking-diagnostics`** line (spec 050), one per shard rather than one per diagnostic, so a machine caller reads what it read before.

`check`, and the freshness guard in front of `couple`, `index coverage` and `index owner`, run the same comparison, so a committed index that reads fresh is exactly what the corpus indexes to.

- **`--slice NAME`**: Checks staleness for a specific named slice defined in `[index.slices]` in the config, against its `slices.json` sidecar hash, rather than the whole tree.
- **`--json`**: Emit the [verdict envelope](./overview.md#machine-readable-verdicts---json) (`verb: "index.check"`) instead of prose.

Before 0.19.0 this verb compared only each shard's recomputed `shardHash`, and derived the files to hash from the committed body itself, so a hand-edited body that owned different `file`, `directory` or `crate` units could read fresh.

### `index render`

Renders the committed index as Markdown. This provides a human-readable view of the codebase index.
*(Note: `render` does not support `--json`.)*

### `index orphans`

Lists specs that have no resolved code units (i.e., specs that claim authority over paths that do not exist or cannot be resolved).

- **`--json`**: Output `{ "orphaned": [ids], "inFlight": [ids], "schemaVersion" }`.

The `--json` output of `index owner`, `index coverage`, `index diagnostics` and `index orphans` is a **read document** (spec 093): a JSON object with sorted keys and a top-level `schemaVersion`. `index diagnostics --json` carries its listing under `items`.

### `index coverage`

Reports, per source file inside a discovered package, whether a spec *specifically* claims it (a resolved unit or a `// Spec:` comment header), whether only a package's manifest floor covers it (**floor-only**), or whether nothing does (**unclaimed**). Freshness-guarded like `couple`: a stale committed index exits `2` rather than reporting against the wrong ledger. Prose, manifests, workflows, config, and paths under `resolver_exclusions` or the bypass set are never counted.

- **`--json`**: Output the `CoverageReport` (totals, the two sorted file lists, per-package counts, and `nearMissHeaders` when there are any).
- **`--fail-on-untraced`**: Exit `1` unless every source file is specifically claimed. The whole-tree "fully specified" assertion for CI.

The same classifier drives the coupling gate's `C-002` when `[coupling] require_ownership` is on, so this report lists exactly the files that flag would refuse.

#### A declared governed scope (spec 097)

By default the universe is inferred: a source extension, inside a discovered package. `[coverage] governed_scope` declares more: glob patterns (as in `extra_hashed_inputs`, so `dir/**/*`, not `dir/**`) naming files that join the universe whatever their extension and wherever they sit, and so join `C-002` under `require_ownership`. `governed_scope_exclusions` carves files back out of that addition only. A resolver exclusion or a bypass prefix still wins; a unit claim still overrides a bypass exactly as spec 009 says. Such a file is claimed by a frontmatter unit, never by a comment header.

With the scope set, `index coverage` matches it against the tracked files (`git ls-files --cached --others --exclude-standard`, minus missing files), or against `--paths-from FILE` where git is not available; a git failure exits `3`. The report adds `declaredScopeFiles` and `enumeration` (`tracked`, `supplied`, or `walk` for a library caller that supplied no list). Both are absent while the scope is empty, and nothing else changes.

#### How a comment header claims (spec 094)

A comment header claims the file it sits in, and only when it is in the **first 16 lines** of that file. For each of those lines, in order: leading whitespace is trimmed; at most one leading `//` or `#` is stripped (the marker is optional); the rest must begin with `Spec:`; and after every trailing `/spec.md` is removed, the final `/`-separated segment of the reference must be the id of a spec in the corpus. So `// Spec: specs/042-x/spec.md`, `# Spec: specs/042-x/spec.md` (for `.py` and `.sh`) and `// Spec: 042-x` all claim for `042-x`.

The **first** `Spec:` line in the window decides. If its reference names no spec in the corpus, the file claims nothing, even when a correct header follows it. `//! Spec:` never claims: an inner doc comment is prose.

`index coverage` lists the headers that tried and failed, as `nearMissHeaders` in `--json` and a `near-miss comment headers` block in prose: `outside-window` (a resolving header on lines 17 to 64 of a file that claimed nothing above it), `unknown-spec` (the first header names no spec) and `doc-comment-marker` (`//! Spec:` in the window). The list explains a classification and changes none; `--fail-on-untraced` does not read it. A header lower than line 16 needs a unit in spec frontmatter instead, which has no positional rule.

## Exit Codes

- **`index` (write):**
  - `0`: OK.
  - `3`: I/O, parse, schema, or config error.
- **`index check`:**
  - `0`: Fresh.
  - `2`: Stale (at least one shard is `modified`, `missing`, `orphaned`, or carries a blocking diagnostic).
  - `3`: I/O, parse or schema error: no committed index, or a committed shard from a schema MAJOR this build does not understand. A committed shard file that does not parse is drift, not an error: `orphaned` when the recompute does not expect it, `modified` when it does, and `--json` counts it as `skippedShards` (spec 095).
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
