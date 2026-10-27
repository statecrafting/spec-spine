---
id: "203-the-determinism-build-restores-a-cargo-cache"
title: "The determinism build restores a cargo cache, measured"
status: approved
kind: "tooling"
created: "2026-10-21"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "156-statecraft-profile-10-governs-this-repository"
summary: >
  The four `determinism / emit` jobs run on every pull request, queue entry and
  push to `main`, and each builds the CLI and a test target from nothing.
  `Swatinem/rust-cache`, pinned to a commit SHA, restores the cargo registry and
  target for each triple, keyed so a cached tree never outlives the toolchain
  and `Cargo.lock` that built it, and saved from `main` only so pull requests
  do not churn a repository cache that is already over its limit. The wall
  clock before and after is measured, as note 06 3.10 requires.
extends:
  # The emit job, by keypath: 156 makes determinism.yml a required extension
  # of ci-gate, and 3.1 adds one step to that job.
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: { kind: section, file: ".github/workflows/determinism.yml", anchor: "jobs.emit" }, nature: additive }
references:
  # The file as a whole is unowned; 163 3.9 added a step the same way. .github/ is on the bypass floor.
  - { unit: { kind: file, path: ".github/workflows/determinism.yml" }, role: "unowned; 3.1 adds one step" }
  - { unit: { kind: file, path: "docs/design/06-harness-and-distribution-2026-09.md" }, role: "3.10: a cost claim carries measured numbers" }
---

# 203: The determinism build restores a cargo cache, measured

## 1. Purpose

`determinism.yml` is a required extension of `ci-gate`, so its four emit jobs
run on every pull request, every merge-queue entry and every push to `main`.
Each job builds `spec-spine` and the `resolver_matrix` test target with a
cold cargo home and an empty `target/`.

## 2. Territory

- **`.github/workflows/determinism.yml`**: one cache step in the `emit` job,
  after the toolchain is shown and before the first build. The file has no
  owning spec (spec 156 names it a required extension; 163 3.9 added a step
  the same way). This spec claims only the `emit` job, by its `jobs.emit`
  keypath, carried on an additive edge to 156.

## 3. Behavior

### 3.1 The cache step

`Swatinem/rust-cache`, pinned to a commit SHA with its release in a comment,
restores `~/.cargo` and `target/` for the job's triple. Its key is
`determinism-<triple>` joined to what the action folds in itself: the rustc
version and a hash of `Cargo.lock`, the manifests and `rust-toolchain.toml`.
A toolchain or lockfile change therefore misses and rebuilds cold. The rustc
versions folded in are every toolchain installed on the runner, not only the
pinned one, so a runner image that updates its preinstalled stable also
misses, once, until the next save from `main` (3.4 records one such miss).

### 3.2 Saves come from `main`

The cache is saved only when the run's ref is `refs/heads/main`. On
2026-10-21 the repository's Actions caches held 10,377,941,223 bytes across
17 entries, over GitHub's 10 GB per-repository limit, so every new entry
evicts the least recently used one. One stable entry per triple, written from
`main`, adds four entries; a save from every pull request would add four per
pull request.

### 3.3 The determinism claim is unchanged

The cache holds build inputs and intermediate artifacts, not the emitted
shards. The `gather` job still compares the shard trees each triple emits
from this repository's corpus. A cached build that emitted different bytes
would fail that comparison exactly as a cold one would.

### 3.4 Measurement

Wall clock per emit job, from the GitHub Actions jobs API (`completed_at`
minus `started_at`), and the build step on its own.

**Before**: the 20 most recent successful `statecraft-ci` runs on
2026-10-21, all cold:

| Triple | Job median (min to max) | Build step median |
|---|---|---|
| aarch64-apple-darwin | 148 s (70 to 180) | 64.5 s |
| aarch64-unknown-linux-gnu | 92 s (82 to 103) | 41.5 s |
| x86_64-pc-windows-msvc | 206.5 s (135 to 220) | 86 s |
| x86_64-unknown-linux-gnu | 97.5 s (66 to 114) | 42 s |

**After**: measured on this change's own runs on 2026-10-21, one run each
(D-2). The cold run (`c7f9c2dc`) found no cache and saved one; the warm run
(`cf158ae6`) restored it under the same pull-request ref.

| Triple | Cold job / build | Warm job / build | Warm restore |
|---|---|---|---|
| aarch64-apple-darwin | 96 s / 37 s | 92 s / 31 s | hit, 8 s |
| aarch64-unknown-linux-gnu | 98 s / 37 s | 59 s / 23 s | hit, 2 s |
| x86_64-pc-windows-msvc | 221 s / 81 s | 207 s / 82 s | miss |
| x86_64-unknown-linux-gnu | 95 s / 36 s | 61 s / 20 s | hit, 4 s |

The cold save cost 5 to 14 s per job and wrote 863,347,108 bytes across the
four entries. On the two Linux triples a hit saved about 35 s per job; on
macOS the 8 s restore took back most of a 6 s build saving, so the job is
roughly unchanged. Windows missed: the two runs landed on runner images
20261007 and 20261018, whose preinstalled stable rustc differed (1.99.0 and
1.99.1), so the key differed (3.1). No Windows hit was measured, and this
spec claims no Windows saving. One run per side is a single sample; the
before medians above vary by up to 110 s on one triple.

## 4. Out of scope

- `acceptance.yml`. It runs only on pushes to `main`, on a schedule and by
  hand, so no run before merge can measure it; without a number there is no
  claim.
- `affected-acceptance.yml`. Its sweep builds inside a worktree of the
  queued revision, not in the checkout's `target/`, so this step would not
  reach that build.
- The managed workflows (`statecraft-ci.yml`, `spec-spine-required.yml`).
  They change only through the Statecraft profile.
- Releasing cache space held by other entries.

## 5. Resolved decisions

**2026-10-27: ratified by the owner.** The owner confirmed every decision
below as written and directed ratification in this pull request; the owner
exception approved on its run is the ratification of record.

**D-1 (2026-10-21): measure on the pull request, then restrict saves.** The
first commit saves from any ref, so this pull request's own runs can show a
cold save and a warm restore. The commit that records the numbers restricts
saving to `main` (3.2). The restore path, which is what a pull request runs,
is the same in both.

**D-2 (2026-10-21): the warm run is the next push, and saving is restricted
first.** The cold run's workflow run stays open while `review-exception`
waits on the owner, and GitHub refuses to rerun a job in a run that is still
open. The warm measurement is therefore the run of the next push to this
pull request, which restores the cache the cold run saved under the same
pull-request ref. That push restricts saving to `main`, since restoring does
not read `save-if`; the after numbers are recorded in the commit after it.

## Verification

```verify:cli
# 3.1: one pinned cache step, keyed per triple, before the first build.
test "$(grep -c 'uses: Swatinem/rust-cache@' .github/workflows/determinism.yml)" = 1
grep -qE 'uses: Swatinem/rust-cache@[0-9a-f]{40} # v[0-9]+\.[0-9]+\.[0-9]+$' .github/workflows/determinism.yml
grep -qF 'key: determinism-${{ matrix.triple }}' .github/workflows/determinism.yml
sh -c 'c=$(grep -n "Swatinem/rust-cache@" .github/workflows/determinism.yml | cut -d: -f1); b=$(grep -n "cargo build --bin spec-spine" .github/workflows/determinism.yml | cut -d: -f1); test "$c" -lt "$b"'
# 3.2: saved from main only.
grep -qF "save-if: \${{ github.ref == 'refs/heads/main' }}" .github/workflows/determinism.yml
# 3.4: the after numbers are recorded.
grep -qF '**After**: measured on' specs/203-the-determinism-build-restores-a-cargo-cache/spec.md
sh -c '! sed "/^## Verification/q" specs/203-the-determinism-build-restores-a-cargo-cache/spec.md | grep -qF "numbers are in this section, this spec claims no saving"'
```
