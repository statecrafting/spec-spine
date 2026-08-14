---
id: "031-registry-freshness-check"
title: "Registry freshness: `spec-spine compile --check`"
status: approved
kind: "tooling"
created: "2026-08-14"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "001-compile-registry"
  - "024-index-sharding"
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/tests/compile.rs", nature: additive }
references:
  # The index-side staleness gate this mirrors for the registry tree, and the
  # exit-code contract it has to agree with (both bypass-floor / non-owning).
  - { unit: { kind: file, path: "specs/004-codebase-index/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  The committed spec-registry shard tree had no freshness gate. `index check`
  (spec 004) covers only the codebase-index tree, and `compile` is a validation
  gate that rewrites the shards without ever asserting the committed copies
  matched, so editing a spec.md and forgetting to recompile lands stale
  shardHash values with CI green (this is what PR #61 did to specs 017 and 021).
  This spec adds `spec-spine compile --check`: a non-writing gate that compiles
  in memory and compares the result byte-for-byte against the committed shards,
  exiting 2 on staleness exactly as `index check` does. It closes the
  registry/index asymmetry with a first-class command rather than a repo-local
  git tree check, so every adopter that commits its registry gets the same
  guarantee spec-spine gets.
---

# 031: Registry freshness check

## 1. Purpose

spec-spine commits both derived views (spec 024), but only one of them has ever
had a freshness gate:

- `.derived/codebase-index/**` is gated by `spec-spine index check` (spec 004
  §3.5, resharded by 024 FR-003), which exits 2 on a stale shard or a changed
  shard set.
- `.derived/spec-registry/by-spec/**` has **nothing**. `spec-spine compile`
  rewrites the shard tree and reports frontmatter violations; it is a
  *validation* gate and says nothing about what the committed shards held
  beforehand.

The asymmetry is historical: only the index side ever got a staleness
subcommand. The cost is real. Commit `9ede89f` (PR #61) edited two `spec.md`
bodies and refreshed their codebase-index shards but skipped `compile`, so both
spec-registry shards landed on `main` carrying pre-edit `shardHash` values with
every CI check green. The ledger's whole claim is that the committed artifact is
what the sources compile to; an ungated half of it is a claim nobody checks.

A repo-local CI step over `git status` closes this for spec-spine alone (and did,
as an interim fix). It does not generalize: it lives in one workflow file, it is
silently vacuous unless a writing `compile` ran first in the same job, and it is
unavailable to any adopter that commits its registry. **Freshness is a property
of the ledger, so the tool that owns the ledger should answer it.**

## 2. Territory

The freshness comparison (`compile.rs`), its re-export and JSON facade
(`lib.rs`), the CLI flag and its exit mapping (`cmd_compile.rs`, `main.rs`), and
the two test files. All additive: no existing signature changes, no schema
change, no new DTO. `Freshness` (spec 004's enum, already public) is reused
rather than duplicated, so both gates report through one type.

## 3. Behavior

### 3.1 `spec-spine compile --check`

Compiles the corpus in memory and compares the result against the committed
shard tree. **It never writes**: no shard, no `build-meta.json`, no pruning of a
removed spec's shard. A `--check` run leaves the working tree byte-identical.

The comparison is over the **serialized shard bytes**, not the `shardHash`
field. Canonical emission (sorted keys, 2-space, LF, trailing newline) makes a
fresh compile's bytes reproducible, so an exact comparison is both the simplest
and the strictest check: it catches a stale hash, a hand-edited shard body, and
a schema-version restamp alike. This is stronger than the index side, which
compares recomputed hashes because its inputs (code spans) are too expensive to
re-resolve.

Three staleness classes, all reported together:

- **modified**: a committed shard whose bytes differ from the compiled one.
- **missing**: a spec with no committed shard (a spec added without recompiling).
- **orphaned**: a committed shard with no corresponding spec (a spec removed
  without recompiling).

Missing and orphaned are set-membership failures and are why the check compares
sets rather than diffing file-by-file: neither is visible to a content
comparison alone.

### 3.2 Exit codes

Per the stable contract (`docs/design/00-architecture.md`; `Error::exit_code()`):

| exit | condition |
|---|---|
| `0` | validation passed and every shard matches |
| `1` | validation failed |
| `2` | validation passed but one or more shards are stale |
| `3` | I/O, parse, or schema failure (unreadable specs dir, corrupt shard) |

**Validation outranks staleness.** A corpus that does not validate cannot vouch
for its shards, so `1` wins when both hold; the operator fixes the frontmatter
first, and the staleness verdict is re-derived on the next run. This keeps
`--check` a single self-contained gate rather than something that must be
sequenced behind a plain `compile`.

An **unbuilt** registry (no `by-spec/` directory, or an empty one against a
non-empty corpus) is **stale (2), not an I/O error**: a registry that was never
built is by definition not vouching for the corpus. This mirrors spec 012 §3.3,
where an index predating a slice config reads as stale rather than as an error.
It is the one place `--check` deliberately diverges from plain `compile`, whose
reader treats a missing registry dir as `Error::Io`.

### 3.3 Output

Fresh prints one line naming the shard count. Stale prints one line per stale
shard, classified, to **stderr** (so CI logs surface it), then a summary line,
and is capped at 20 shards with an `and N more` tail so a corpus-wide restamp
does not flood the log. Message text is not part of the contract; exit codes
are.

### 3.4 Determinism

`--check` is a pure function of `(config, spec sources, committed shards)`. It
reads no clock and no environment: notably it does **not** compare
`build-meta.json`, which carries the wall clock and is gitignored. Two runs over
one tree agree, and the verdict is identical across the release matrix.

### 3.5 Relationship to the existing gates

`--check` does not replace anything. Plain `compile` remains the writing form
and keeps its exit semantics. `index check` is untouched: the two gates cover
disjoint trees and are wired as separate CI steps so a failure names which half
drifted. The coupling gate (spec 005) is unaffected; `.derived/` stays in the
bypass floor, because a derived artifact is not code that can drift from a spec.

### 3.6 Tests (minimum)

- Fresh tree after a compile exits 0 and writes nothing (tree byte-identical,
  verified including mtime-independent content comparison).
- A `spec.md` body edit without recompiling is detected as `modified` (the PR
  #61 regression, pinned).
- A newly added spec dir is detected as `missing`.
- A removed spec's leftover shard is detected as `orphaned`.
- An unbuilt registry is stale (2), not an error.
- Validation failure exits 1 even when shards are also stale (precedence).
- The JSON facade returns `{ "fresh": bool, ... }` for both verdicts.

## 4. Out of scope

A `--fix` / `--write` mode (that is plain `compile`). Extending the check to
`build-meta.json` (wall-clock, gitignored, excluded from every determinism gate
by 001). A combined "check both trees" command: one gate per invocation keeps CI
failure messages single-subject, per spec 012 §3.3. Teaching the coupling gate
to consult registry freshness (005 is a code-vs-spec gate; freshness is a
ledger-vs-source question and stays separate).
