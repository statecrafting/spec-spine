---
id: "128-a-derived-tree-stays-in-its-repository"
title: "A derived tree stays in its repository"
status: approved
kind: "tooling"
created: "2026-09-23"
summary: >
  `[layout] derived_dir` is read from the committed `spec-spine.toml`, and
  nothing checked where it pointed. `derived_dir = "../outside"`, an absolute
  path, or `x/../../outside` made `compile`, `index` and `attest` write their
  shards outside the repository and prune `*.json` files there, all with exit
  0, on `main` and on spec 127's build. Spec 127 checks every component below
  the repository root for links, and stops where a path leaves the root (its
  D-4), so it did not reach this. A `derived_dir` that is absolute, carries a
  `..` segment, or uses a Windows path form is now refused when the
  configuration is loaded, before any verb reads or writes, and the checked
  writer refuses any path that is not wholly below its root. A repository
  reached through a link still works.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "036-declared-state-dir"
  - "092-the-engine-ships-governance-not-an-environment"
  - "127-a-derived-file-is-where-its-path-says"
# 3.3 withdraws 127 D-4 and the last sentence of 127 4's `derived_dir` bullet:
# a path not wholly below the root is refused, not written as before.
amends: ["127-a-derived-file-is-where-its-path-says"]
amends_sections: ["4", "5"]
establishes:
  - { kind: file, path: "crates/spec-spine-cli/tests/derived_dir_contained.rs" }
extends:
  # 3.1, 3.2: the load-time rule, beside 036's `state_dir` rule.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: corrective }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/config.rs", nature: additive }
  # 3.3: the checked writer refuses a path not wholly below its root.
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-core/src/shard.rs" }, nature: corrective }
references:
  - { unit: { kind: file, path: "specs/033-configured-corpus-root/spec.md" }, role: context }
  - { unit: { kind: file, path: "crates/spec-spine-cli/tests/derived_symlinks.rs" }, role: context }
intent:
  goal: "no committed configuration can make a governance verb write, create, remove or prune a file outside the repository it governs"
  non_goals:
    - "confining `specs_dir`, `standards_dir` or any other layout key, which no verb writes through (4)"
    - "validation of configuration passed to the JSON facade, which loads no file (4)"
    - "protection against another process replacing a path while the verb runs"
    - "a claim about Windows links or junctions, which were not measured"
---

# 128: A derived tree stays in its repository

## 1. Purpose

### 1.1 Measured, on spec 127's build (`84c4731e`, over `main` at `f6afdc61`)

Each case is a disposable area holding `repo/` (one spec, a
`spec-spine.toml` setting the key) beside `outside/`, which holds a sentinel
`other.json` in each of `spec-registry/by-spec/`, `codebase-index/by-spec/`
and `attestation/`. The whole of `outside/` is hashed before and after.

| `[layout]` value | Verb | Exit | `outside/` |
|---|---|---|---|
| `derived_dir = "../outside"` | `compile` | 0 | changed: shard written, `other.json` pruned |
| `derived_dir = "../outside"` | `index` | 0 | changed |
| `derived_dir = "../outside"` | `attest` | 0 | changed |
| `derived_dir = "<area>/outside"` (absolute) | `compile` | 0 | changed |
| `derived_dir = "x/../../outside"` | `compile` | 0 | changed |
| `derived_dir = "..\\outside"` (on macOS) | `compile` | 0 | unchanged: `\` is a name character there |
| `specs_dir = "../outside"` | `compile` | 0 | unchanged |
| default layout (control) | `compile` | 0 | unchanged |

The handoff that recorded this finding measured the first row on `f6afdc61`
with the same result. Spec 127 4 recorded it and left it open: its checked
writer walks a path's components below the root and stops at the first one
that is not a plain name (127 D-4), so a `..` or an absolute `derived_dir` is
written exactly as before.

The configuration file is committed content, like the spec ids 126 guards and
the links 127 guards. These are the verbs the README says run against a
stranger's branch.

### 1.2 Found by reading: Windows path forms

On Windows, `\` is a separator and a drive prefix replaces the path it is
joined to, so `..\outside`, `C:\outside`, `C:outside` and
`\\server\share\outside` would each leave the repository there. None was
measured on a Windows host; the rule in 3.1 refuses them on every platform, as
126 refuses `\` and `:` in a derived file name on every platform.

## 2. Territory

- `crates/spec-spine-types/src/config.rs` (extends 000): the `derived_dir`
  rule, run by `load_config` beside 036's `state_dir` rule.
- `crates/spec-spine-types/tests/config.rs` (extends 000): the values refused
  and the values kept, as unit tests on every platform.
- `crates/spec-spine-core/src/shard.rs` (extends 022): the checked writer
  refuses a path not wholly below its root (3.3).
- `crates/spec-spine-cli/tests/derived_dir_contained.rs` (establishes): every
  row of 1.1 through the shipped binary, the controls, and a repository
  reached through a link.

## 3. Behavior

### 3.1 A `derived_dir` names a directory inside the repository

`load_config` MUST refuse a `[layout] derived_dir` that, read as written:

- begins with `/` (an absolute path, including `//server/share`);
- has a segment, split on `/`, that is exactly `..`;
- contains `\` (a Windows separator, so also `..\x`, `\x` and `\\server`);
- contains `:` (a Windows drive or stream form: `C:\x`, `C:x`, `C:/x`).

The refusal is a configuration error, exit 3, naming the key, the value and
the reason. It happens when the configuration is loaded, so a refused
configuration leaves nothing behind: no verb has read or written anything
yet.

Kept as they are: `.derived` (the default), `.statecraft/derived` (the managed
layout), `./.derived/`, any other relative path of plain segments, and a
segment that merely contains dots (`..a`, `a..b`). An empty value and `.`
name the repository root itself; they are inside the repository and stay
accepted, unchanged.

### 3.2 Every verb that loads the configuration refuses it

The rule is part of loading, so it applies wherever `spec-spine.toml` is
loaded, not only in the three writing verbs: `check`, `lint`, `registry`,
`index` reads, `couple` and `delta` (which load the base revision's
configuration as well as the head's) all exit 3 on such a configuration. This
is how 036's `state_dir` rule already behaves. A derived tree outside the
repository is not a committed ledger that any read could honestly report on.

### 3.3 The checked writer refuses a path not wholly below its root (amends 127)

127's writer checks each path's components below the root and, by its D-4,
writes a path that leaves the root as before, checking only the part that is
below it. That is withdrawn. Every path the writer is asked to write, create,
remove or prune MUST be lexically below its root: the path starts with the
root, and every component after it is a plain name (a `.` component is
ignored). A `..`, a root or a prefix component, or a path that does not start
with the root, MUST refuse the whole run in the preflight, exit 3, with
nothing written, created, removed or pruned, as 127 3.2 refuses a link.

This is the guard for a library caller that builds a `Config` without
`load_config` and hands the result to the writer; through the CLI, 3.1 has
already refused the configuration. The check is lexical: the root is not
resolved, so 127 3.1's promise that a repository reached through a link keeps
working still holds.

## 4. Out of scope, and the limits of the claim

- **`specs_dir` and the other layout keys.** Measured (1.1): `specs_dir =
  "../outside"` makes `compile` read from outside and write nothing there.
  `compact`, the one verb that writes under `specs_dir`, takes the files it
  rewrites or removes from the repository's tracked paths, which never begin
  with `..` or `/`; a `compact` run under `specs_dir = "../corp"` with a plan
  removing a spec that lives only there changed nothing in `corp/` (measured
  on the same build). Reading a corpus from outside the repository is a
  question about what `compile` reads, not about what any verb writes, and is
  left to its own spec. 033 4 deferred validating `specs_dir` for the same
  reason.
- **The JSON facade.** `*_json` functions deserialize a `Config` from JSON
  and run none of `load_config`'s rules (036's included). None of them
  writes a file, and 3.3 guards the writer a library caller would use. Making
  the facade validate is a change to every facade function and is left open.
- **Concurrent replacement, Windows links and junctions.** As 127 4.
- **A device name as a `derived_dir` segment** (`derived_dir = "CON"`). 127
  3.5 refuses device names for derived file names; a device-named directory
  segment is not refused here. Writing through one fails on Windows with an
  I/O error; it names no other file in the repository's parent.

## 5. Resolved decisions

**D-1 (2026-09-23): refuse at load, and also at the writer.** Refusing only
at load leaves any library caller that builds a `Config` directly unguarded;
refusing only at the writer would let every read verb report on a ledger
outside the repository and exit 0. Both are cheap and each covers what the
other cannot.

**D-2 (2026-09-23): the rule is lexical and platform-independent.** The same
corpus gets the same verdict on every release triple, which is what 126 and
127 3.5 chose for file names. No path is resolved against the filesystem, so
the rule cannot be defeated or broken by a link above the root.

**D-3 (2026-09-23): an empty `derived_dir` and `.` stay accepted.** Both name
the repository root, which is inside it. Refusing them would be a
compatibility change this spec has no evidence to make.

**D-4 (2026-09-23): compatibility.** A repository whose committed
`spec-spine.toml` sets an escaping `derived_dir` now fails every verb with
exit 3 until the key is corrected, where before its writing verbs wrote
outside it and exited 0. The pull request that corrects the key is judged
against a base whose configuration no longer loads. Corrected on 2026-09-24
to what was measured (spec 129 4, `docs/configuration.md`), where this
entry first said `couple` and `delta` both refuse it: `delta` exits 3 on
that pull request, because it always classifies by the merge-base's rules;
`couple` exits 3 only when the diff deletes a path, because only a deletion
asks it for the base snapshot (spec 100 3.4), and a correcting diff that
deletes nothing gets `couple` exit 0. A `Spec-Drift-Waiver:` changes neither
result, since this is not drift. The recovery is therefore a correcting pull
request that deletes nothing; where a required check runs `delta`, merging it
is the owner's explicit decision, and deletions follow in a later pull
request. No repository known to this project sets one: this
repository and the managed layout use `.statecraft/derived`, and the default
is `.derived`. No schema, API signature, exit-code mapping or shard name
changes.

**D-5 (2026-09-23): the writer's check has its own mutation case.** Through
the CLI, 3.1 refuses the configuration before the writer is reached, so no
CLI test can fail when only 3.3's check is removed; measured, the CLI suite
still passes with it removed. The unit test in `shard.rs` is the assertion
that fails for that mutation, and it places the escaping output after a
legitimate one so a writer that wrote before checking would be caught too.

## Verification

```verify:cli
# 3.1: the values refused and kept, as unit tests on every platform.
cargo test -p spec-spine-types --test config --locked derived_dir
# 3.1 to 3.3 through the shipped binary: every escaping row of 1.1 exits 3
# with the whole area byte-identical, the controls build, and a repository
# reached through a link still builds.
cargo test -p spec-spine-cli --test derived_dir_contained --locked
# 3.3: the writer refuses a path not wholly below its root.
cargo test -p spec-spine-core --lib shard:: --locked
# 127's own acceptance still holds.
cargo test -p spec-spine-cli --test derived_symlinks --locked
```
