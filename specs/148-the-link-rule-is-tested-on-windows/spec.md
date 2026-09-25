---
id: "148-the-link-rule-is-tested-on-windows"
title: "The link rule is tested on Windows"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Spec 144's rule that no governed read leaves the repository through a link is
  tested only on Unix: both link tests in `tests/repo_path.rs` are
  `#[cfg(unix)]`, and 144 §4 puts Windows links and junctions out of scope.
  Windows is a shipped platform with a required `test (windows)` job (spec
  134), and it has two link kinds 144's walk must see: symbolic links and
  directory junctions. This spec adds a Windows test for both, and records
  what a runner without the symbolic-link privilege does.
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "134-the-suite-runs-on-windows"
  - "144-a-repository-path-is-one-type"
amends:
  - "144-a-repository-path-is-one-type"
amends_sections: ["4"]
extends:
  - { spec: "144-a-repository-path-is-one-type", unit: "crates/spec-spine-core/tests/repo_path.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/windows-findings.md" }, role: context }
---

# 148: The link rule is tested on Windows

## 1. Purpose

`crates/spec-spine-core/tests/repo_path.rs` holds the two tests that prove
144 §3.4 (a link inside is read, a dangling link is read, a link outside
refuses with exit 2; a repository reached through a link works). Both are
`#[cfg(unix)]`, so the required `test (windows)` job never runs them, and the
rule's Windows behavior (link metadata, `canonicalize` returning `\\?\`
verbatim paths, junctions) is unproven.

A Windows test can exist:

- A **directory junction** needs no privilege. `mklink /J` creates one, and
  `std::fs::symlink_metadata` reports it as a symbolic link on stable Rust.
- A **symbolic link** needs `SeCreateSymbolicLinkPrivilege` or Developer Mode.
  GitHub's `windows-latest` runner runs elevated, so
  `std::os::windows::fs::symlink_dir` and `symlink_file` succeed there. A
  developer machine without the privilege gets `ERROR_PRIVILEGE_NOT_HELD`
  (1314).

## 2. Territory

- `crates/spec-spine-core/tests/repo_path.rs`: a `#[cfg(windows)]` module.

## 3. Behavior

### 3.1 Both link kinds are tested on Windows

A `#[cfg(windows)]` test MUST assert, on the `test (windows)` job, each case
the Unix tests assert: a junction and a directory symbolic link pointing
outside the repository refuse `compile` with exit 2 naming the link; one
pointing inside is read; a dangling one is read; a repository reached through
a junction works.

### 3.2 A missing privilege is reported, never passed silently

When creating a symbolic link fails with `ERROR_PRIVILEGE_NOT_HELD`, the
symbolic-link half of the test MUST print that it did not run and why, and the
junction half MUST still run. On CI the test MUST fail instead: an environment
variable the workflow already sets (`CI=true`) turns the skip into a failure,
so the required job cannot go green without having created a link.

### 3.3 144 §4 is corrected

144 §4's "tested on Unix" line is superseded by this spec's test; this spec's
own §1 is the record, since 144 is not edited (spec 037).

## 4. Out of scope

- Links at or above the derived root on read: spec 147.
- File symbolic links on Windows beyond one case: the rule is kind-agnostic.

## 5. Resolved decisions

None yet.

## Verification

```verify:cli
# 3.1, 3.2: the Windows cases compile on every host and run on Windows. On a
# Unix host this asserts the module is present and gated, which is the most a
# non-Windows sweep can witness; the required `test (windows)` job runs them.
grep -q 'cfg(windows)' crates/spec-spine-core/tests/repo_path.rs
grep -q 'mklink' crates/spec-spine-core/tests/repo_path.rs
grep -q 'ERROR_PRIVILEGE_NOT_HELD\|1314' crates/spec-spine-core/tests/repo_path.rs
```
