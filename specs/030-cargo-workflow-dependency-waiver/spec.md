---
id: "030-cargo-workflow-dependency-waiver"
title: "Cargo and workflow dependabot bumps self-clear: freshness projection + auto-waiver"
status: approved
kind: "tooling"
created: "2026-07-03"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
amends:
  - "004-codebase-index"
  - "005-coupling-gate"
extends:
  # Freshness half (004 §3.5): Cargo.toml joins package.json in folding as a
  # governance projection, so a cargo version bump does not stale the index.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/manifest.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # Coupling half (005 §3.5): the mechanical auto-waiver gains cargo and
  # workflow classifiers; the CLI pre-filter admits all three manifest classes.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/dep_only.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
summary: >
  Amends 004 §3.5 and 005 §3.5, extending the paired 2026-06-11 npm
  dependency-only mechanism to the cargo and GitHub Actions ecosystems so a
  dependabot-class bump self-clears without a human waiver. Freshness half:
  Cargo.toml folds into the index content hash as a governance projection (its
  dependency tables stripped), mirroring npm, so a version bump leaves the
  committed shards fresh. Coupling half: the mechanical auto-waiver recognizes a
  Cargo.toml whose only change is dependency version specifiers, and a
  `.github/workflows/*.yml` whose only change is the `@ref` of `uses:` action
  references. Every non-version edit (a new or removed dependency, a feature /
  git / path change, a `run:` / `with:` edit, an added step or table, an
  unpinned action, a spec-metadata edit) still refuses the waiver, fail-closed.
  A claimed workflow file (007/008/021's release.yml) and a crate whose direct
  dependencies dependabot bumps are the live acceptance tests.
---

# 030: Cargo and workflow dependabot bumps self-clear

## 1. Purpose

The 2026-06-11 amendment paired two mechanisms so a dependabot-class
`package.json` bump self-clears: spec 004 §3.5 folds npm manifests as a
governance projection (a version bump is not a hashed input, so the index stays
fresh), and spec 005 §3.5 mechanically waives a coupling drift whose only change
is a dependency version string. A bot can neither re-index nor add a
`Spec-Drift-Waiver:` line, so without both halves a routine bump walls on a
stale index (exit 2) or an unwaivable `C-001` (exit 1).

That mechanism stopped at npm. Two other ecosystems dependabot manages hit the
same wall on this repo's own corpus:

- **cargo.** A crate's `Cargo.toml` is a discovered manifest folded as **raw
  bytes** (004 §3.5), so a `[dependencies]` version bump stales the index. It is
  also floor-owned or explicitly claimed, so it fires `C-001`. Both halves are
  missing.
- **GitHub Actions.** `.github/workflows/release.yml` is explicitly claimed by
  specs 007/008/021 (`kind: file`), so spec 009 lifts it above the `.github/`
  floor and a `uses:` action bump fires `C-001`. A `file` unit carries no span
  and is not a hashed input (004 §3.5), so freshness is already fine; only the
  coupling half is missing.

The evidence is this repo's merge history: the cargo group bump (#52) and the
GitHub Actions major bumps (#55) each required a hand-written
`Spec-Drift-Waiver:` line, because the bot could not add one itself. This spec
closes both gaps by extending the existing mechanism, not inventing a new one.

## 2. Territory

`spec-spine-core`'s `manifest.rs` (a new `cargo_hash_projection`, paired with
`npm_hash_projection`) and `index.rs` (the per-shard manifest fold routes
`Cargo.toml` through it); plus `dep_only.rs` (two new fail-closed classifiers
and the generalized waiver dispatch) and the `cmd_couple.rs` pre-filter (it now
admits all three manifest classes), with the CLI end-to-end test. No new config
knob (the existing opt-in `config.coupling.auto_waive_dependency_only` now
covers all three classes); no schema change; no new DTO field; no user-facing
CLI surface change.

## 3. Behavior

### 3.1 Cargo freshness projection (amends 004 §3.5)

A `Cargo.toml` folds into the index content hash as its **governance
projection**: the parsed manifest with its dependency tables removed, rendered
through the same canonical (sorted-key) serializer the npm projection uses so
the hash is deterministic across the release matrix. The stripped tables are
`dependencies`, `dev-dependencies`, and `build-dependencies` at every location
cargo allows them: the top level, under `[workspace]`, and under each
`[target.<cfg>]`. Everything the indexer actually reads is preserved and still
stales the shard on change: `[package]` `name` / `version` / `edition`, the
`[package.metadata.<ns>]` spec binding, and the presence of `[lib]` / `[bin]`
(which determines the package kind). An unparseable or non-table manifest falls
back to raw bytes: over-hashing is the fail-closed direction, exactly as npm.

Like the npm amendment, upgrading across this change re-computes every
cargo-bearing repo's hash once (its crate shards now fold the projection, not
the raw bytes); re-run `spec-spine index` when adopting. `Cargo.lock` is
untouched: it remains on the bypass floor.

### 3.2 Cargo dependency-only auto-waiver (amends 005 §3.5)

When opted in and no explicit PR-body waiver is present, a `Cargo.toml` among
the non-bypassed changed paths is dependency-only iff its parsed base and head
are equal everywhere except the **version specifiers of existing
dependencies**. Inside a dependency table the package key sets must match; each
entry is either a bare version string on both sides (which may differ) or a
table on both sides with an identical field set where only `version` may differ
(both string). A dependency added or removed, a shape flip (string to table or
back), a change to any non-version field (`features`, `git`, `path`,
`optional`, `default-features`, a rename), or any change outside a dependency
table refuses the waiver.

### 3.3 Workflow dependency-only auto-waiver (amends 005 §3.5)

A `.github/workflows/*.yml` among the non-bypassed changed paths is
dependency-only iff its parsed base and head are equal everywhere except the
`@ref` of `uses:` action references: a `uses:` scalar may differ only when both
sides are `owner/action@ref` strings with an identical, non-empty
`owner/action` and both pinned. YAML comments (where a SHA-pinned action records
its human-readable version) are dropped by the parser on both sides, so a
SHA-pin bump with a moving `# vX.Y.Z` comment is dependency-only. A new or
removed step or job, a `with:` / `run:` / `env:` edit, an added key, a changed
action path, or an unpinned action (`@ref` removed) refuses the waiver.

A workflow file reaches this checker only when it is **not bypassed**, which
under spec 009 means it is explicitly claimed by a spec (`.github/` is otherwise
floor-bypassed). So this half governs exactly the claimed-workflow surface, for
example release.yml: it mechanically proves the same property a human waiver on
#55 asserted by hand ("changes only pinned `uses:` action versions, no job
logic").

### 3.4 The whole-diff rule is unchanged

The mechanical waiver still fires only when **every** non-bypassed changed path
is a recognized dependency manifest whose change is dependency-only, evaluated
at the merge base and head (three-dot diff, git-diff mode only; `--paths-from`
carries no content). A diff mixing classes (a `package.json` bump, a `Cargo.toml`
bump, and a `uses:` bump together) waives iff each qualifies. A single
non-manifest path, a created or deleted manifest, or one non-version edit
anywhere sinks the whole waiver: the drift is reported unwaived and the gate
exits 1. The claim-aware bypass verdict (spec 009) still decides which paths are
candidates, so a claimed floor path cannot slip past the pre-filter.

### 3.5 Config, determinism, and report shape

No new config: the existing opt-in `config.coupling.auto_waive_dependency_only`
(default `false`) now covers all three classes. Broadening an opt-in,
fail-closed flag is additive; a repo that wants npm bumps waived wants its cargo
and claimed-workflow bumps waived on the same terms. The core functions stay
pure (no clock, env, or git); `cargo_hash_projection` and the two classifiers
are deterministic pure functions of their input bytes. The `CoupleReport` and
the index DTOs gain no fields; the auto-waived report is indistinguishable from
the npm case.

### 3.6 Tests (minimum)

- Cargo bare-string and table `version` bump across `[dependencies]`,
  `[dev-dependencies]`, and `[workspace.dependencies]`: dependency-only.
- Cargo added/removed dependency, feature-list edit, shape flip, package
  `version` edit, spec-metadata edit, added table: each refuses.
- Cargo reformat-only: dependency-only (alters no governed fact).
- Workflow `uses:` tag bump and SHA-pin bump: dependency-only. A `with:` /
  `run:` edit, an added step, a changed action path, an unpin: each refuses.
- End to end: a floor-owned crate's dependency bump stays index-fresh and
  auto-waives; a claimed workflow file's `uses:` bump stays index-fresh (file
  unit, no span) and auto-waives; a cargo feature edit drifts.

## 4. Out of scope

Other manifest ecosystems dependabot manages (`pyproject.toml`,
`requirements.txt`, `go.mod`, Gradle, Docker `FROM` pins): the same pattern
extends to each behind its own fail-closed classifier, filed when a real bump
needs it. Lockfiles stay on the bypass floor (they are never a governed input).
The auto-waiver remains opt-in and per-repo; this spec does not change its
default. Semantic drift that preserves version pins (a dependency bump that also
silently changes behavior) is out of scope for the same reason it is for npm:
the gate detects line-and-pin coupling, not behavioral equivalence.
