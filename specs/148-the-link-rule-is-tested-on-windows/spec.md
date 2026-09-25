---
id: "148-the-link-rule-is-tested-on-windows"
title: "The link rule is tested on Windows"
status: approved
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

- `crates/spec-spine-core/tests/repo_path.rs`: two `#[cfg(windows)]` tests,
  `a_junction_follows_the_link_rule` and
  `a_windows_symbolic_link_follows_the_link_rule`.

## 3. Behavior

### 3.1 Both link kinds are tested on Windows

The two `#[cfg(windows)]` tests MUST assert, on the `test (windows)` job, each
case the Unix tests assert, one test per link kind (a directory junction; a
directory symbolic link): a link pointing outside the repository refuses
`compile` and `index` with exit 2 naming the link; one pointing inside is read;
a dangling one is read; a repository reached through a link of that kind
works.

### 3.2 A missing privilege is reported, never passed silently

When creating a symbolic link fails with `ERROR_PRIVILEGE_NOT_HELD`, the
symbolic-link half of the test MUST print that it did not run and why, and the
junction half MUST still run. When the `CI` environment variable is `true` (GitHub
Actions sets it on every runner, so no workflow edit is needed), the test MUST
fail instead of skipping, so the required job cannot go green without having
created a link.

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
# 3.1, 3.2 on a Windows host: both tests run and pass, exactly.
sh -c 'case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) cargo test -p spec-spine-core --locked --test repo_path -- --exact a_junction_follows_the_link_rule a_windows_symbolic_link_follows_the_link_rule 2>&1 | grep -q "test result: ok. 2 passed; 0 failed";; *) exit 0;; esac'
# On any other host, the most a sweep can witness: each test exists, directly
# under `#[cfg(windows)]` and `#[test]`, and the file creates a junction, names
# the privilege error, and reads `CI`. The required `test (windows)` job runs
# the tests themselves (spec 134).
sh -c 'for t in a_junction_follows_the_link_rule a_windows_symbolic_link_follows_the_link_rule; do grep -B2 "^fn $t()" crates/spec-spine-core/tests/repo_path.rs | tr -d "\n" | grep -q "#\[cfg(windows)\]#\[test\]" || exit 1; done'
grep -q 'mklink' crates/spec-spine-core/tests/repo_path.rs
grep -q 'ERROR_PRIVILEGE_NOT_HELD\|1314' crates/spec-spine-core/tests/repo_path.rs
grep -q '"CI"' crates/spec-spine-core/tests/repo_path.rs
```
