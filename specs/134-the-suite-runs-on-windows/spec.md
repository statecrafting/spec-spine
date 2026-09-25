---
id: "134-the-suite-runs-on-windows"
title: "The suite runs on Windows"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  spec-spine ships a Windows binary and proves its shard trees byte-identical
  on Windows, but CI only ever built there: the workspace test suite ran on
  Linux alone, so a Windows behavior difference could ship with every check
  green. A `test (windows)` job now runs `cargo test --workspace` on
  `windows-latest` as a check of its own, required through `ci-gate`. A test
  that cannot run on Windows is gated or quarantined in its source with a
  tracked finding, never skipped by the workflow, and Windows results are
  reported apart from build results.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "094-one-gate-and-the-boundaries-it-holds"
extends:
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  # 3.2: the POSIX-shell suites, gated to Unix with a tracked finding each.
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: "crates/spec-spine-core/tests/ai_review_policy.rs", nature: additive }
  - { spec: "117-the-gate-resolves-the-binary-it-documents", unit: "crates/spec-spine-core/tests/gate_binary.rs", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_hooks.rs", nature: additive }
  - { spec: "122-a-commit-refuses-an-unresolved-merge", unit: "crates/spec-spine-core/tests/commit_boundary.rs", nature: additive }
  - { spec: "123-a-startup-verdict-names-its-reader", unit: "crates/spec-spine-core/tests/reader_identity.rs", nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: additive }
  # WF-8: the plan-path refusal decided on the string.
  - { spec: "097-a-path-leaves-the-corpus-the-way-a-spec-does", unit: "crates/spec-spine-core/src/compact.rs", nature: corrective }
  # WF-7: the quarantined tests.
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "100-a-deleted-path-is-judged-where-it-lived", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/windows-findings.md" }, role: context }
---

# 134: The suite runs on Windows

## 1. Purpose

The determinism job builds on four triples, Windows included, and the release
ships a Windows binary. The workspace test suite ran only on Linux. The first
Windows run of it (this spec's pull request, run 1) did not compile: a test
file used `std::os::unix` unconditionally. The second run compiled and ran
1196 tests; 1191 passed and 5 failed, in two groups that are both real:

- **WF-8, a product defect, fixed here.** `compact`'s retire and historical
  plan paths refused a rooted path through `Path::is_absolute`, which is false
  on Windows for `/etc/passwd`: the refusal failed open there.
- **WF-7, most likely a product defect, quarantined here.** Three tests commit
  shards to a scratch repository and read them back through git; on the
  Windows runner the snapshot's shards come back reported `modified` (stale).
  The inferred cause is CRLF conversion on checkout meeting spec 086's byte
  comparison.

## 2. Territory

`.github/workflows/ci.yml` is established by 094; the job is an additive
`extends`. The gated test files and `compact.rs` are `extends` crossings on
their owners, listed in the frontmatter. `docs/windows-findings.md` is the
tracked findings list, referenced rather than claimed (a claimed file must be
hashed, L-008, and a findings list should not restamp every shard when it
changes).

## 3. Behavior

### 3.1 A Windows test check of its own

CI MUST run `cargo test --workspace --locked --no-fail-fast` on
`windows-latest` as a job named `test (windows)`, separate from the Linux
`build · test · clippy` job, and `ci-gate` MUST require it. A Windows result
is reported under that name and never folded into a build result.

### 3.2 Nothing is skipped silently

The workflow MUST NOT skip or filter tests. A test that does not run on
Windows MUST be gated or quarantined in its own source, and each MUST be listed
in `docs/windows-findings.md` with where it is, how it is gated, why, and what
lifts it:

- a suite that executes POSIX shell (a workflow's `run:` scalars, the
  `Makefile` gate, the hooks, the pre-commit guard, the reader-identity
  script) is compiled on Unix only (`#![cfg(unix)]` or `#[cfg(unix)]`, WF-1
  to WF-6). It still runs on Linux on every change;
- a test that fails on Windows because of a suspected product defect is
  quarantined with `#[cfg_attr(windows, ignore = "WF-N: ...")]`, so the
  runner prints the finding id beside the test it skipped (WF-7).

### 3.3 WF-8 is fixed

A `compact` plan path MUST be refused as rooted when it starts with `/` or
`\`, or is absolute, on every platform: the check is on the string, as spec
128 decides `derived_dir`.

## 4. Out of scope

**WF-7's product fix.** The proposed fix (a `.gitattributes` fragment in the
scaffold pinning the derived tree to LF, and a byte comparison that names CRLF
as the cause when normalizing it would have matched) changes adopter-facing
behavior and is its own spec. The quarantine lifts when it lands.

**Windows junction and path-swap semantics.** Nothing here claims them.

## 5. Resolved decisions

**D-1 (2026-09-24): gate the shell suites rather than port them.** They test
POSIX shell that only runs on Linux (this repository's harness and CI steps).
Porting them would test a Windows execution nobody performs.

**D-2 (2026-09-24): required, not advisory.** A check that can fail without
blocking is read as green. With the quarantine list explicit, requiring it
costs nothing today and catches the next Windows regression.

## Verification

```verify:cli
# 3.1: the job exists under its own name and the gate requires it.
grep -q 'name: test (windows)' .github/workflows/ci.yml
grep -q 'needs: \[test, test_windows,' .github/workflows/ci.yml
grep -q 'cargo test --workspace --locked --no-fail-fast' .github/workflows/ci.yml
# 3.2: every gate or quarantine names a finding the list carries.
sh -c 'for id in WF-1 WF-2 WF-3 WF-4 WF-5 WF-6 WF-7 WF-8; do grep -q "| $id |" docs/windows-findings.md || { echo "missing $id"; exit 1; }; done'
sh -c 'n=$(grep -rl "WF-[0-9]" crates/*/tests/*.rs crates/*/src/*.rs | wc -l | tr -d " "); test "$n" -ge 8'
# 3.3: WF-8 on the host platform; the Windows job runs the same cases.
cargo test -p spec-spine-core --test retire --locked outside_the_corpus
```
