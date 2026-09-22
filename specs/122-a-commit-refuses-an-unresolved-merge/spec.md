---
id: "122-a-commit-refuses-an-unresolved-merge"
title: "A commit refuses an unresolved merge"
status: draft
kind: "tooling"
created: "2026-09-22"
summary: >
  The commit-boundary hook (spec 094) refuses a stale or invalid derived tree
  and nothing else, so a merge whose conflict was "resolved" by staging the
  markers committed cleanly, and so did a Rust change whose formatting check
  had failed earlier in the same command chain. Both happened on this
  repository's expansion branches. The hook now also refuses unmerged index
  entries, conflict markers on the lines a commit adds (git's own detector,
  with git's own attribute as the only opt-out), and a staged Rust change that
  fails `cargo fmt --all --check`. It still writes and stages nothing.
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "094-one-gate-and-the-boundaries-it-holds"
extends:
  # 3.1 to 3.4: three refusals added to the hook 094 establishes.
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: directory, path: ".githooks/" }, nature: additive }
establishes:
  # 3.5: the hook run for real, in a throwaway repository.
  - { kind: file, path: "crates/spec-spine-core/tests/commit_boundary.rs", planned: true }
references:
  - { unit: { kind: file, path: ".gitattributes" }, role: context }
---

# 122: A commit refuses an unresolved merge

## 1. Purpose

### 1.1 Two commits the boundary let through

Spec 094 §4.2 made `.githooks/pre-commit` the one guard that is a property of
the change rather than of the actor: whatever wrote the bytes, a commit runs
it. What it refuses is a stale derived tree, a corpus that does not validate,
and regenerated shards left out of the commit. On 2026-09-22 two commits on
this repository's expansion branches passed it and should not have:

- **A merge carrying conflict markers.** Merging `main` into spec 102's branch
  stopped on a conflict in the spec's decisions. The markers were staged with
  `git add` and the merge committed (`6dc77cf1`). `git add` clears the
  unmerged index stages without reading the file, so git itself raised no
  objection, and the hook's freshness read passed: markers in a spec's prose
  are valid Markdown, `compile` and `index` had been rerun, and the committed
  tree was exactly as fresh as it claimed. `43a2e89f` removed them afterwards.
- **An unformatted Rust change.** `23bb8aed` on spec 106's branch was
  committed and pushed with a tree that fails `cargo fmt --all --check`: the
  command chain that ran the check did not make the commit conditional on it.
  `acc6d981`, carrying the same subject, formatted it. The hook never looks at
  formatting, so it had nothing to say. Replaying the check over every commit
  since 2026-09-21 that touches a `.rs` file finds one more, `02c19e9e` (spec
  117's build), and the merge `f2f720fe` that carried `23bb8aed` into spec
  107's branch. All three are in published history and stay there.

Both are execution mistakes, and the procedure that produced them has been
corrected separately. The durable correction belongs at the boundary spec 094
already chose, for the reason it chose it: a hook bound to the commit fires
whatever chain of commands led there.

### 1.2 What the boundary can see and what it cannot

At commit time the hook sees the index and the working tree. It can see an
unmerged path, a marker being added, and whether the tree is formatted. It
cannot run the test suite or clippy in a time a contributor tolerates on every
commit; those stay the gate's (`make gate`, CI), as 094 §3 already has it.

## 2. Territory

- `.githooks/pre-commit`, through an `extends` edge on spec 094's `.githooks/`
  directory. Nothing else under `.githooks/` changes.
- `crates/spec-spine-core/tests/commit_boundary.rs`, established here.

## 3. Behavior

### 3.1 Placement

The checks in 3.2 to 3.4 MUST run after the "no corpus" exit 094 §4.2
requires and **before** the `spec-spine` binary is resolved, so a machine
without the binary still gets them. Each refusal MUST use the hook's existing
refusal shape, which names `git commit --no-verify`.

### 3.2 Unmerged index entries

The hook MUST refuse when `git ls-files -u` lists any entry, naming the
unmerged paths. git refuses such a commit on its own; the hook's check makes
the refusal hold for a direct run of the hook and states it in the hook's
terms.

### 3.3 Conflict markers on added lines

The hook MUST refuse when `git diff --cached --check` reports a leftover
conflict marker, naming each file and line. Whitespace findings from the same
command MUST NOT refuse: only the conflict-marker class is this spec's.

Two properties follow from using git's own detector over the staged change
and are part of the contract:

- **Only added lines are judged.** A marker already in history is never
  reported again, so an unrelated commit is never blocked by one.
- **An intentional marker is declared, not guessed.** A file that carries
  markers on purpose (a merge fixture) opts out through the git attribute
  `conflict-marker-size` set on that path in `.gitattributes`. The hook MUST
  NOT carry a path pattern, extension list or directory exemption of its own:
  an exemption the hook infers is one a real conflict can match.

### 3.4 Formatting, when the change is Rust

When `<root>/Cargo.toml` exists and the staged change adds, copies, modifies
or renames a `.rs` file, the hook MUST run `cargo fmt --all --check` from the
repository root and refuse on a non-zero exit, showing the tool's output. When
`cargo` is not on `PATH` it MUST print a skip and continue, as the binary
resolution in 094 §4.2 does. A change that stages no `.rs` file MUST NOT run
it. The check reads the working tree; a partially staged file is judged as
written, which errs towards refusing.

### 3.5 Evidence is behavioral

The refusals MUST be asserted by running the checked-in hook through
`git commit` in a throwaway repository with `core.hooksPath` pointing at this
repository's `.githooks/`, not by reading the script's text. `PATH` in that
repository MUST be narrowed so neither an installed `spec-spine` nor an
installed `cargo` answers for the hook. The tests MUST cover:

1. a conflicted merge staged with its markers is refused, HEAD does not move,
   and the same merge commits once the file is resolved;
2. unmerged entries are refused by the hook run directly;
3. an undeclared marker file is refused, and the same file declared with
   `conflict-marker-size` is accepted;
4. a marker already in history does not block an unrelated commit;
5. a staged Rust change is refused when the format check fails and accepted
   when it passes;
6. a change staging no Rust does not run the format check.

Cases 1, 2, 3 (its refusal half) and 5 MUST fail against the hook as it stood
before this spec.

### 3.6 Unchanged

Everything 094 §4.2 requires still holds: the hook writes nothing, stages
nothing, runs no `couple`, and resolves its binary in the documented order.
Generated shards are still judged by the freshness read that follows: after a
conflict is resolved, a derived tree that was not regenerated is refused as
stale, exactly as before.

## 4. Out of scope

- Running clippy or the test suite at commit time.
- A `pre-merge-commit` hook. A merge that completes without conflict produced
  no markers, and its shard freshness is the gate's to judge.
- Adopters' hooks. The kit is gone (spec 092); this is the hook this
  repository runs.

## 5. Resolved decisions

**D-1 (2026-09-22, the detector).** `git diff --cached --check` rather than a
grep for `^<<<<<<<`. A grep of staged files re-reports markers already in
history and needs its own exemption list; git's detector judges added lines
only and honours `conflict-marker-size`, which is a per-path declaration a
reviewer can read.

**D-2 (2026-09-22, formatting only).** Of the required checks, only
formatting runs here. It takes about a second, and it is the one whose failure
in a command chain produced a commit. Clippy and tests take minutes and stay
at the gate.

## Verification

Written to fail against the tree this spec is filed on: the test file does
not exist and the hook has none of the three checks.

```verify:cli
# 3.2 to 3.4: the three checks are in the hook.
grep -qF 'ls-files -u' .githooks/pre-commit
grep -qF 'diff --cached --check' .githooks/pre-commit
grep -qF 'leftover conflict marker' .githooks/pre-commit
grep -qF 'cargo fmt --all --check' .githooks/pre-commit
# 3.3: no exemption of the hook's own; the attribute is the only opt-out.
grep -qF 'conflict-marker-size' .githooks/pre-commit
# 3.5: behavioral, through git commit, every case.
cargo test -p spec-spine-core --test commit_boundary --locked > "${TMPDIR:-/tmp}/ss122.txt" 2>&1
grep -qE 'test result: ok\. 6 passed' "${TMPDIR:-/tmp}/ss122.txt"
rm -f "${TMPDIR:-/tmp}/ss122.txt"
# 3.6: 094's invariants still hold.
! grep -qE '^[^#]*git add' .githooks/pre-commit
grep -qF -- '--no-verify' .githooks/pre-commit
cargo test -p spec-spine-core --test gate --locked
```
