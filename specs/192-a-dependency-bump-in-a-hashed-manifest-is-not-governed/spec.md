---
id: "192-a-dependency-bump-in-a-hashed-manifest-is-not-governed"
title: "A dependency bump in a hashed manifest is not a governed change"
status: draft
implementation: complete
kind: "tooling"
created: "2026-10-04"
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
  - "027-cargo-workflow-dependency-waiver"
  - "060-a-workflow-bump-is-not-a-governed-change"
  - "141-a-governance-edit-rewrites-one-file"
amends:
  # 004 3.5 defines the content hash over the normalized bytes of every hashed
  # input, with a projection for package manifests discovered as packages and
  # (via 060) for workflows. 3.1 below adds a projection for a Cargo.toml or
  # package.json matched by `extra_hashed_inputs`, which changes what enters
  # the hash for every adopter that hashes one.
  - "004-codebase-index"
extends:
  # 3.1 the two input projections, beside the package and workflow ones.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/manifest.rs", nature: additive }
  # 3.1 the fold site that applies them.
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: additive }
  # 3.2 the agreement matrices, beside 060's workflow matrix.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/dep_only.rs", nature: additive }
  # 3.4 the end-to-end freshness guard.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # 3.3 the migration note, beside 058's and 060's.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
summary: >
  A Cargo.toml or package.json that an adopter lists in `[index]
  extra_hashed_inputs`, typically because a spec claims it and L-008 then
  requires it to be hashed, enters the inputs record as raw bytes. A
  Dependabot version bump therefore stales `inputs.json`, and the bot cannot
  re-index its own PR, while the coupling gate would have waived the same
  change. Spec 060 closed this gap for workflows only. This spec closes it for
  the two package manifests: a hashed Cargo.toml or package.json folds as a
  projection that blanks exactly the version strings the dependency-only
  waiver forgives and keeps everything else, so the ledger and the waiver
  state one rule.
---
# 192: A dependency bump in a hashed manifest is not a governed change

## 1. Purpose

Stop a Dependabot manifest bump from walling an adopter that hashes the
manifest it bumps.

Since spec 141, every `[index] extra_hashed_inputs` match is recorded one
entry per file in `codebase-index/inputs.json`
(`shard.rs::global_input_pieces`). A workflow folds as its governance
projection (spec 060 3.1). Every other match folds as raw bytes, so a hashed
`Cargo.toml` or `package.json` moves its digest on a one-character version
bump, `check` reports the index stale, and the PR waits for a human to push a
re-index commit. The coupling gate is not the obstacle:
`dep_only::cargo_dependency_only_change` and `dependency_only_change` already
waive exactly this change (specs 005 and 027). The ledger and the gate
disagree, the same half-built state spec 060 described for workflows.

The trigger is ordinary. A spec that claims a workspace manifest as a file
must have it hashed, or `lint --fail-on-warn` refuses with L-008. aicortex, an
adopter, measured this on spec-spine 0.28.0: a simulated `base64` bump on a
file-claimed, hashed root `Cargo.toml` left every shard fresh and staled
`inputs.json`.

A **section** claim on a manifest is a different path and this spec does not
change it: a section unit's backing file folds its raw bytes into the claiming
spec's own shard (`index.rs::span_files_for_mapping`, 004 3.5), so the same
bump stales each claiming shard instead. aicortex's ten section claims on
`[workspace]` and `[workspace.dependencies]` staled ten shards that way. Such
an adopter gets this spec's relief by moving the claims to a file claim on the
manifest and listing it in `extra_hashed_inputs` (3.3). Its interim mitigation, grouping every
cargo bump into one weekly PR, reduces the manual re-index to once a week and
does not remove it.

**The existing package projections are the wrong tool.**
`manifest::cargo_hash_projection` removes whole dependency tables, and
`manifest::npm_hash_projection` keeps only `name`, `version`, `workspaces` and
the metadata namespace. Both were written for per-package shards, where the
question is "did a field discovery reads change". For a governance input that
answer is too permissive: an added dependency, a changed feature set, a
`git`/`path` source, or an npm `scripts` edit would stop staling the ledger,
while the waiver correctly refuses every one of them. Applying either as-is
would trade a wall for a silent governance loss. This spec specifies a
narrower projection instead.

## 2. Territory

No new files. Five units, all `extends`-ed:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/manifest.rs` | 004 | the two input projections |
| `crates/spec-spine-core/src/shard.rs` | 022 | the fold site applies them |
| `crates/spec-spine-core/src/dep_only.rs` | 005 | the agreement matrices |
| `crates/spec-spine-core/tests/index.rs` | 004 | the end-to-end freshness guard |
| `docs/adoption-guide.md` | 057 | the migration note |

**This spec `amends` 004.** Section 3.5 of spec 004 defines what enters the
content hash. Section 3.1 below changes it for a hashed `Cargo.toml` or
`package.json`, which changes the recorded digest for every adopter that
hashes one. Spec 060 declared the same edge for the same reason. Per spec 037
the edge is declared here only; 004's `spec.md` is not edited.

## 3. Behavior

### 3.1 A hashed manifest folds as its input projection (amends 004 3.5)

A hashed input (a `spec-spine.toml` sibling matched by `extra_hashed_inputs`,
outside a declared state root) whose path satisfies `dep_only::is_cargo_toml`
MUST fold as its **cargo input projection**; one whose path satisfies
`dep_only::is_package_json` MUST fold as its **npm input projection**. A
workflow continues to fold as spec 060's projection; any other file continues
to fold as raw bytes.

The **cargo input projection** is the parsed TOML document in which:

- inside every table whose key is in `dep_only::CARGO_DEPENDENCY_TABLES`, at
  any depth the waiver recognizes (top level, `[workspace]`,
  `[target.<cfg>]`, or deeper), each entry that is a string has that string
  replaced by one fixed placeholder, and each entry that is a table with a
  string `version` field has that field's value replaced by the same
  placeholder;
- a dependency table is not descended into further;
- everything else is preserved: package keys, every other field of a
  dependency entry (`features`, `optional`, `default-features`, `git`,
  `path`, `package`, `workspace`, ...), the presence of a `version` field, a
  non-string `version`, and every key outside a dependency table.

The **npm input projection** is the parsed JSON object in which, inside each
top-level table in `dep_only::DEPENDENCY_TABLES` whose value is an object,
each string value is replaced by the same placeholder. Everything else,
including `scripts`, `engines`, `overrides` and the package key sets, is
preserved.

Both render through the canonical, sorted-key serializer the other
projections use, so the result is byte-identical across the release matrix.
An unparseable document, or one that is not a table (cargo) or object (npm),
MUST fall back to its raw bytes: over-hashing is the fail-closed direction.

The per-package shard projections (`cargo_hash_projection`,
`npm_hash_projection`) are unchanged. They answer a different question, and a
manifest that is both a discovered package and a hashed input is judged by
both, each with its own projection.

### 3.2 The projections and the waivers state one rule

For a manifest whose base and head both parse:

- a change the waiver (`cargo_dependency_only_change` or
  `dependency_only_change`) waives MUST leave the input projection unchanged;
- a change that alters the input projection MUST refuse the waiver.

This MUST be asserted by test over a shared matrix per ecosystem, in both
directions, as spec 060 3.2 does for workflows. The matrix MUST include at
least: a bare-string bump, a table `version` bump, an added and a removed
dependency, a shape flip (string to table), an added or changed feature, an
added `version` field, a `git` or `path` edit, a change outside every
dependency table, a dependency table under `[workspace]` and under
`[target.<cfg>]`, and for npm a `scripts` edit and a non-string dependency
value change.

The one permitted disagreement MUST be pinned by its own test: a key named
like a dependency table whose value is not a table (cargo) or not an object
(npm). The waiver refuses any change involving it, and the projection keeps it
verbatim, so an edit to it still changes the projection. Both are
fail-closed.

### 3.3 The upgrade rewrites one file, once

An adopter that hashes a `Cargo.toml` or `package.json` sees that file's
digest in `inputs.json` move once on upgrading, and nothing else: since spec
141 no shard folds a global input. The remedy is the ordinary one: run
`spec-spine index` and commit the regenerated `inputs.json`. No schema version
changes; only a recorded digest moves, so `INDEX_SCHEMA_VERSION` MUST NOT be
bumped.

The migration note MUST say so, and MUST say that an attestation sealed before
the change reports spec 021's named `VersionMismatch` outcome rather than a
content mismatch, as spec 060 3.3 recorded for workflows.

This repository hashes neither manifest, so its own committed tree does not
move.

The note MUST also tell an adopter that claims a manifest by section that the
projection does not reach a section claim, and that the remedy is a file claim
on the manifest plus an `extra_hashed_inputs` entry (section 1).

### 3.4 Tests (minimum)

- The 3.2 matrices, both directions, and the pinned disagreement.
- Comment-only and reformat-only edits leave each projection unchanged.
- An unparseable manifest falls back to raw bytes, so any edit changes its
  digest.
- End to end through `shard::input_digests`: with `Cargo.toml` and
  `package.json` listed in `extra_hashed_inputs`, a dependency version bump
  leaves both digests unchanged, and an added dependency changes each.

## 4. Out of scope

**Changing the package shard projections.** They are correct for discovery,
and narrowing them would restale every package shard in every adopter for no
governed benefit.

**Deriving the waivers from the projections.** As spec 060 4 says for
workflows: attractive, and a rewrite of a classifier on the coupling path.
The agreement test buys the same guarantee.

**Span-backed manifests.** A section or symbol claim whose backing file is a
manifest still folds raw bytes into the claiming spec's shard. Projecting
there means deciding what a span over a dependency table governs, and a lint
hint that names a section claim on a manifest is the cheaper first step. Both
are a separate spec.

**Lockfiles.** `Cargo.lock` and `package-lock.json` sit on the bypass floor
and are not hashed by any shipped default. An adopter that hashes one opts in
to raw-byte staleness.

**Other ecosystems.** pip, go and docker manifests have no projection and no
waiver; spec 027 4 and spec 060 4 still apply.

## 5. Resolved decisions

**D-1 (2026-10-04, the projection mirrors the waiver, not the shard
projection).** Considered and rejected: applying `cargo_hash_projection` and
`npm_hash_projection` to hashed inputs, as first proposed by the adopter
session that measured the defect. Both remove facts the waiver refuses to
waive (section 1), so the ledger would go fresh on changes the coupling gate
still refuses. A governance input's projection must hide exactly what the
waiver forgives and nothing more.

## Verification

```verify:cli
sh -c 'cargo test -p spec-spine-core --locked --lib -- --exact dep_only::tests::the_cargo_input_projection_and_the_cargo_waiver_agree dep_only::tests::the_npm_input_projection_and_the_npm_waiver_agree dep_only::tests::a_non_table_dependency_key_is_refused_by_both 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact a_dependency_bump_leaves_a_hashed_manifest_digest_unchanged an_added_dependency_changes_a_hashed_manifest_digest 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
grep -qF 'spec 192' docs/adoption-guide.md
```
