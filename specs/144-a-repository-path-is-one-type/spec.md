---
id: "144-a-repository-path-is-one-type"
title: "A repository path is one type"
status: approved
kind: "tooling"
created: "2026-09-25"
summary: >
  Paths the engine joins onto the repository root were checked key by key, or
  not at all. `derived_dir` refused `/`, `..`, `\` and `:` (spec 128);
  `specs_dir` and `standards_dir` accepted anything, so a corpus could be read
  from outside the repository; a compact plan refused a leading slash (WF-8) and
  let the drive-relative `C:foo` through; no key refused a Windows device-name
  segment; and a link inside the repository could point any governed read
  outside it. This spec gives one rule for a repository-relative path, applies
  it to every layout root and to compact plan paths, and refuses a run whose
  tree holds a link that resolves outside the repository, exit 2, while a
  repository reached through a link keeps working.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "096-compaction-is-a-verb-not-a-session"
  - "127-a-derived-file-is-where-its-path-says"
  - "128-a-derived-tree-stays-in-its-repository"
  - "134-the-suite-runs-on-windows"
amends:
  # 3.2: 128's derived_dir rule becomes the shared rule, which also refuses a
  # device-name segment and a segment ending in '.' or a space.
  - "128-a-derived-tree-stays-in-its-repository"
extends:
  # 3.2 every layout root, and the export of the rule.
  - { spec: "128-a-derived-tree-stays-in-its-repository", unit: "crates/spec-spine-types/src/config.rs", nature: corrective }
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # 3.1 the device-name check now delegates to the shared one.
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: additive }
  # 3.3 plan paths, and their cases beside WF-8's.
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: "crates/spec-spine-core/src/compact.rs", nature: corrective }
  - { spec: "097-a-path-leaves-the-corpus-the-way-a-spec-does", unit: "crates/spec-spine-core/tests/retire.rs", nature: additive }
  # 3.4 the walk, and the two entry points that call it.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/pathutil.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.2 spec 129's escape test put a `\` in two layout roots, which 3.1 refuses.
  - { spec: "129-a-json-config-obeys-the-loaders-rules", unit: "crates/spec-spine-core/tests/config_facade.rs", nature: corrective }
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/repo_path.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/repo_path.rs" }
---

# 144: A repository path is one type

## 1. Purpose

The 0.25.0 release record listed what was still open after the containment
work of specs 126 to 128: "read-side symbolic links; `specs_dir` or
`standards_dir` outside the repository; ... a device-named `derived_dir`
segment". Spec 134's WF-8 then found that the compact plan check depended on
the platform's idea of absolute. Each was a separate hand-written check, or a
missing one.

## 2. Territory

- `crates/spec-spine-types/src/repo_path.rs`: the rule, `RepoPath`, and the
  device-name check (moved from `shard.rs`, which now delegates).
- `crates/spec-spine-types/src/config.rs`: every layout root.
- `crates/spec-spine-core/src/compact.rs`: plan paths.
- `crates/spec-spine-core/src/pathutil.rs`, called from `compile` and `index`:
  the link rule.

## 3. Behavior

### 3.1 One rule

A repository-relative path MUST be refused, on every platform and on the
string alone, when it starts with `/` or `\`; contains `:`, `\` or NUL; has a
`..` segment; has a segment that is a reserved Windows device name (`CON`,
`nul.txt`, `COM1`); or has a segment other than `.` ending in `.` or a space.
An empty path and `.` name the repository root. `RepoPath` is the validated
form, and it deserializes only from a string the rule accepts.

### 3.2 Every layout root (amends 128)

`layout.specs_dir`, `layout.standards_dir` and `layout.derived_dir` MUST satisfy
the rule, and `layout.state_dir` MUST satisfy it in addition to its own rules
(spec 036). A violation is `Error::Config`, exit 2, naming the key.

### 3.3 Compact plan paths

A plan's retired `path` and each `historical_files` entry MUST satisfy the
rule, beside the leading-slash, `..` and `.` checks spec 096 and WF-8 already
make. The drive-relative `C:foo`, which is not absolute on Windows either, is
refused.

### 3.4 No read leaves the repository through a link

Before reading, `compile` and `index`, and therefore every verb that reads the
corpus or the code, MUST walk the repository tree and refuse the run, as
`Error::Refused` (exit 2) naming the link, when a symbolic link resolves outside
the repository. The comparison is against the root's own resolution, so a
repository reached through a link works. A link resolving inside the
repository, and a dangling link, are accepted. Links are checked, never
followed.

The walk skips `.git`, the derived and state roots and any link at or above
either of them (the tool's own output, whose links spec 127 refuses on write;
D-4), and the `[index] resolver_exclusions` directory names, which no governed
read enters.

## 4. Out of scope

**Windows links and junctions.** The rule uses the platform's link metadata
and canonicalization. It is tested on Unix, where links can be created without
privilege; the Windows test job runs the rest.

**Hard links.** A hard link is a file, not a pointer, and has no outside.

**Concurrent changes to the tree** during a run, as in spec 127.

## 5. Resolved decisions

**D-1 (2026-09-25): refuse the run, do not skip the link.** Skipping would make
the verdict depend on a file the reader cannot see. A refusal names the link,
and the author can remove it.

**D-2 (2026-09-25): one walk, at the two entry points.** About twenty read sites
open repository files. Every one reaches its file by walking down from the root
or by joining a repository-relative path, so a read can leave only through a
link in the tree. Checking the tree once makes every site safe, including the
next one written. On this repository, `check` takes 0.6 s with the walk.

**D-3 (2026-09-25): an empty historical entry is reported as empty.** A
whitespace-only `historical_files` entry is caught by the empty-entry check
before the path rule, which would call it a segment ending in a space.

**D-4 (2026-09-25): a link at or above the derived or state root is spec
127's.** Spec 127 refuses a write through a linked derived root or any
ancestor of it, naming the derived tree, and its acceptance asserts that
message. This rule leaves those links to it. A governed file stored under
such an ancestor, outside both roots, is read through the link unchecked; no
corpus in the family stores one there.

## Verification

> **Superseded acceptance (2026-09-25).** This block no longer runs.
> `151-carried-acceptance-tests-what-it-names` declares this spec in
> `amends_verification`, so `spec-spine verify 144` builds its plan from that
> spec's block, where these commands are carried with the `repo_path` line made exact and every other line unchanged (spec 082 3.2 and 3.4).

```verify:cli
# 3.1: the rule and the type.
sh -c 'cargo test -p spec-spine-types --locked --lib -- --exact repo_path::tests::plain_relative_paths_pass repo_path::tests::every_escape_and_windows_hazard_is_refused repo_path::tests::the_type_refuses_what_the_rule_refuses 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.2, 3.4: the layout roots and the link rule. Each fails with its rule
# removed (recorded in the PR).
sh -c 'cargo test -p spec-spine-core --locked --test repo_path 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
# 3.3: plan paths in Windows forms.
sh -c 'cargo test -p spec-spine-core --locked --test retire -- --exact a_retired_path_in_a_windows_form_is_refused an_empty_historical_entry_is_refused 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 3.4: this repository holds no link that leaves it.
cargo build --release --locked
target/release/spec-spine check
```
