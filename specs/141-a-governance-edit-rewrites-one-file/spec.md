---
id: "141-a-governance-edit-rewrites-one-file"
title: "A governance edit rewrites one file"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Every index shard's `shardHash` folds one global scalar over `spec-spine.toml`
  and every `[index] extra_hashed_inputs` file, so a one-line edit to any root
  document restamps every shard. Measured on this repository at `153d35df`: an
  `AGENTS.md` edit rewrites 139 shards, and two pull requests editing two
  different root documents conflict on all 139. An adopter's merge queue pays
  that on every such PR. This spec moves the global inputs out of the shard
  hashes into one committed sidecar, `codebase-index/inputs.json`, with one
  entry per input file, compared byte for byte like a shard. A governance edit
  then rewrites that one file, and two edits to different documents change
  lines far enough apart to merge. The aggregate index content hash folds the
  sidecar, so an attestation still covers every authored input.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "022-index-sharding"
  - "060-a-workflow-bump-is-not-a-governed-change"
  - "069-the-committed-index-is-compared-not-trusted"
  - "070-an-authority-snapshot-says-what-it-read"
amends:
  # 022's shard-hash inputs (the per-spec and per-package bullets of its storage section).
  - "022-index-sharding"
  # 050, 057 and 061 require the L-008 message to say a glob restamps every shard.
  - "050-claimed-but-unwitnessed"
  - "057-the-docs-name-what-adopters-derived"
  - "061-shipped-is-not-the-same-as-working"
  # 060 3.1 names the fold site as "the global-inputs scalar every shard hash carries".
  - "060-a-workflow-bump-is-not-a-governed-change"
  # 069 3.1's comparison gains one file.
  - "069-the-committed-index-is-compared-not-trusted"
extends:
  # 3.2 the per-file input digests and their fold
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: additive }
  # 3.1 to 3.4 shard hashes, the sidecar, its comparison and the read-side fold
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.2 `index` writes the sidecar
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  # 3.2 the sidecar entry points are exported
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.5 the L-008 message
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  # 3.5 the scaffolded config comment that said a hashed file stales every shard
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  # 3.2 the sidecar DTO
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/codebase.rs", nature: additive }
  # 3.2 its export
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # 3.2 its embedded schema
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/schema.rs", nature: additive }
  # 3.7 INDEX_SCHEMA_VERSION 1.2.0
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.7 the pinned version
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
  # 3.5 the message assertion
  - { spec: "046-depends-on-ordinal-monotonicity", unit: "crates/spec-spine-core/tests/lint.rs", nature: additive }
  # test helpers write the sidecar as `index` does
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  # same
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  # same
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/tests/delta.rs", nature: additive }
  # same
  - { spec: "044-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-core/tests/diagnostics.rs", nature: additive }
  # same
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # same
  - { spec: "069-the-committed-index-is-compared-not-trusted", unit: "crates/spec-spine-core/tests/index_body.rs", nature: additive }
  # same
  - { spec: "114-authoring-adapters-and-intent-are-separated", unit: "crates/spec-spine-core/tests/intent.rs", nature: additive }
  # same
  - { spec: "112-typed-overlays-ride-the-existing-seam", unit: "crates/spec-spine-core/tests/overlays.rs", nature: additive }
  # same
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-core/tests/scope.rs", nature: additive }
  # same
  - { spec: "070-an-authority-snapshot-says-what-it-read", unit: "crates/spec-spine-core/tests/snapshot.rs", nature: additive }
  # 3.7 the root instruction's paragraph on hashed inputs, the adopter
  # migration note, and the schema history line.
  - { spec: "105-governed-scope-is-enabled-here", unit: "CLAUDE.md", nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
  - { spec: "055-a-version-pin-the-cli-can-check", unit: "docs/schema-versioning.md", nature: additive }
establishes:
  - { kind: file, path: "crates/spec-spine-types/schemas/codebase-index-inputs.schema.json" }
  - { kind: file, path: "crates/spec-spine-core/tests/governance_inputs.rs" }
references:
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
---

# 141: A governance edit rewrites one file

## 1. Purpose

### 1.1 One edit, every shard

`shard.rs::global_inputs_hash` hashes `spec-spine.toml` and every file an
`[index] extra_hashed_inputs` pattern matches, and `index.rs` folds that scalar
into every per-spec and per-package shard's `shardHash` (spec 022). The design
note on the function says a config edit is "rare and inherently global".
In practice the hashed inputs are the root documents an adopter edits often:
`AGENTS.md`, `CLAUDE.md`, the constitution, the workflows.

Measured on this repository at `153d35df` with the 0.26.0 binary, in a scratch
clone:

| Change | Files `index` rewrites | Conflicting files when the two branches merge |
|---|---|---|
| branch A appends a line to `AGENTS.md` | 140 (139 shards) | |
| branch B appends a line to `CLAUDE.md` | 140 (139 shards) | |
| `git merge` A into B | | 139 |

Every pull request that touches a root document therefore conflicts with every
other one that does, on every shard, and a merge queue must regenerate the
whole tree for each. Statecraft's queue pays this on every such PR.

### 1.2 What the fold is for, and what else provides it

The scalar does two jobs. It makes the committed tree go stale when a
governance input changes, and it makes the aggregate index content hash, which
`attest` records as `indexHash`, cover those inputs. Neither job needs the
scalar inside every shard. Staleness needs one committed record of the inputs
that the byte comparison of spec 069 reads. Coverage needs the aggregate to
fold that record.

## 2. Territory

- `crates/spec-spine-core/src/shard.rs`: the per-file input digests.
- `crates/spec-spine-core/src/index.rs`: shard hashes without the scalar, the
  sidecar's emission, comparison and read-side fold.
- `crates/spec-spine-core/src/lint.rs`: the `L-008` message.
- `crates/spec-spine-core/src/scaffold.rs`: the scaffolded config's comment on
  `extra_hashed_inputs`, which said a hashed file stales every shard.
- The core test helpers that write an index as `spec-spine index` does now
  write the sidecar too.
- `crates/spec-spine-types`: the sidecar DTO, its embedded schema and the
  `INDEX_SCHEMA_VERSION` bump.
- `.gitattributes`: the sidecar joins the derived merge driver's globs.
- `docs/schema-versioning.md`, `docs/adoption-guide.md`: the history line and
  the one-time migration note.

## 3. Behavior

### 3.1 Shard hashes cover only their own inputs (amends 022)

A per-spec index shard's `shardHash` MUST cover its `spec.md` and the source
files backing its resolved spans, and nothing global. A per-package shard's
MUST cover its manifest's governance projection and nothing global. The
registry is unchanged: its shard hash never folded the scalar.

### 3.2 The inputs sidecar

`spec-spine index` MUST write `<derived_dir>/codebase-index/inputs.json`:

```json
{
  "inputs": {
    "AGENTS.md": {
      "contentHash": "<sha256>"
    },
    "spec-spine.toml": {
      "contentHash": "<sha256>"
    }
  },
  "schemaVersion": "<INDEX_SCHEMA_VERSION>"
}
```

- `inputs` has one member per file the global scalar covered before this spec:
  `spec-spine.toml` if present, and every `extra_hashed_inputs` match outside a
  declared state root, keyed by its repository-relative POSIX path.
- Each `contentHash` is `hash::content_hash` over that one file's piece: its
  path and its normalized bytes, or, for a workflow, spec 060's governance
  projection. A `uses:` ref bump therefore still leaves the sidecar unchanged.
- The file is canonical JSON like every shard. With one member per line pair,
  two edits to different input files change lines separated by unchanged lines,
  so git merges them without a conflict.
- It is written on every run, with `"inputs": {}` when nothing is hashed, so its
  absence is never a state to interpret.

### 3.3 Freshness compares it (amends 069)

`index check`, `check`, and the freshness guard in front of `couple`,
`index coverage` and `index owner` MUST compare the committed sidecar's bytes
with the one the in-memory index emits, exactly as spec 069 compares a shard:

- `missing inputs.json` when there is none;
- `modified inputs.json` when its bytes differ.

A governance edit then reports one drifted file, and regenerating rewrites one
file.

### 3.4 The aggregate still covers the inputs

The index's `build.contentHash` MUST fold the sidecar: the keyed pairs it hashes
gain `("inputs", <digest>)`, where the digest is `hash::content_hash` over the
sidecar's `(path, contentHash)` pairs. The same value is computed at emit and
when the aggregate is assembled from the committed tree, so `attest`'s
`indexHash` changes when any governance input changes, as it did before.

### 3.5 The L-008 message states the new cost (amends 050, 057, 061)

`L-008` MUST still name both remedies and how they differ. A covering glob now
records the file in the index's inputs record, so an edit rewrites that one
file, which is right for a handful of governance files and wrong for a tree of
source whose every edit would then be a governance change. A `section` or
`symbol` unit is still hashed through its span and stales only the claiming
spec's shard. The words "restamps EVERY shard" are gone, because they are no
longer true.

### 3.6 The fold site (amends 060)

Spec 060 3.1's projection is applied where the per-file digest is computed
(3.2), rather than inside a scalar every shard hash carries. The projection
itself is unchanged.

### 3.7 Version and migration

`INDEX_SCHEMA_VERSION` moves to `1.2.0` (MINOR): the shard documents keep their
shape, and the sidecar is a new file. Every index shard's `shardHash` value
changes once, so an adopter regenerates once on upgrade, which any schema bump
already requires (D-1). The merge driver's globs gain the sidecar.

## 4. Out of scope

**The snapshot's `governanceInputs`** (spec 070) already hashes the same files
as raw bytes on its own; it is unchanged.

**Per-input shards.** One file per input would make even two edits to adjacent
entries merge, at the cost of a directory of one-line files. The single sidecar
is measured to merge in the case that matters (D-2).

## 5. Resolved decisions

**D-1 (2026-09-25): MINOR, not MAJOR.** The shard documents keep their shape and
the sidecar is additive. What changes is the value inside `shardHash`. A reader
of a different version already judges every shard stale after any schema bump,
because `schemaVersion` is part of the bytes 069 compares, so a MAJOR would add
a refusal without adding information.

**D-2 (2026-09-25): one sidecar with nested entries.** Each entry spans three
lines (`"path": {`, `"contentHash": ...`, `},`), so the hash lines of two
adjacent entries are separated by two unchanged lines, and git's merge treats
the two edits as separate hunks.

**D-3 (2026-09-25): an absent sidecar folds nothing on read.** A tree written
before this spec has no `inputs.json`. The freshness comparison reports it
`missing`, which is the answer that matters. The aggregate assembled from such
a tree folds nothing for it rather than failing every read verb (`render`,
`owner`) on a tree whose only fault is its age. A present sidecar that does not
parse, or whose MAJOR is foreign, is still refused (exit 4) like a shard.

**D-4 (2026-09-25): the scaffold's comment changes.** The scaffolded
`spec-spine.toml` said a hashed file "stales every shard", which this spec makes
false, so the two comment lines now say it rewrites the inputs record. The
scaffold's output therefore differs from 0.26.0's in those two comment lines as
well as in the version pin comment. The Statecraft handoff names it, because
Statecraft's positive control compares scaffold output across versions.

**D-5 (2026-09-25): the self-repository measurement.** Measured with this
build, from a `git archive` of this branch: appending a line to `AGENTS.md`
and running `spec-spine index` changes exactly one file under
`.statecraft/derived/`, `codebase-index/inputs.json`. The same edit at
`153d35df` changed 140 (1.1).

## Verification

```verify:cli
cargo build --release --locked
# 3.1 to 3.4 and D-2: the blast radius, the freshness report, the missing
# sidecar, the workflow projection, the aggregate, the schema, and a real git
# merge of two edits to adjacent inputs. Written first and red with the old
# per-shard fold restored (3 of 7).
sh -c 'cargo test -p spec-spine-core --locked --test governance_inputs 2>&1 | grep -q "test result: ok. 7 passed; 0 failed"'
# 3.5: the L-008 message states the new cost and not the old.
sh -c 'cargo test -p spec-spine-core --locked --test lint -- --exact an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 3.7: the index schema is 1.2.0.
sh -c 'cargo test -p spec-spine-types --locked --test dtos -- --exact schema_versions_are_pinned 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 3.1: no shard hash folds a global value any more.
sh -c '! grep -q "GLOBAL_INPUTS_KEY" crates/spec-spine-core/src/index.rs'
# 3.2: this repository commits its sidecar, listing its root documents.
grep -q '^    "AGENTS.md": {$' .statecraft/derived/codebase-index/inputs.json
grep -q '^    "spec-spine.toml": {$' .statecraft/derived/codebase-index/inputs.json
# D-5: on a copy of this tree, an AGENTS.md edit rewrites one derived file.
sh -c 'T="${TMPDIR:-/tmp}/ss141"; rm -rf "$T" && mkdir -p "$T" && git archive HEAD | tar -x -C "$T" && B="$PWD/target/release/spec-spine" && cd "$T" && git init -q && git add -A && git -c user.email=t@example.invalid -c user.name=t -c commit.gpgsign=false commit -qm base && printf "
One more line.
" >> AGENTS.md && "$B" index >/dev/null && n=$(git status --porcelain -- .statecraft/derived | wc -l | tr -d " ") && f=$(git status --porcelain -- .statecraft/derived | awk "{print \$2}") && cd / && rm -rf "$T" && test "$n" -eq 1 && test "$f" = .statecraft/derived/codebase-index/inputs.json'
# The tree this block runs in is fresh under the new construction.
target/release/spec-spine check
```
