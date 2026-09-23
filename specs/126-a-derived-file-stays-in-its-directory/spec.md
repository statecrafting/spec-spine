---
id: "126-a-derived-file-stays-in-its-directory"
title: "A derived file stays in its directory"
status: draft
kind: "tooling"
created: "2026-09-23"
summary: >
  `compile` and `index` name each per-spec shard `<id>.json` after the spec's
  frontmatter `id`, and `attest --spec` names its attestation the same way.
  None of the three checked that the id was a file name before joining it to a
  directory. A corpus declaring `id: "../../../package"` made `compile`
  overwrite the repository's `package.json` with shard JSON, an absolute id
  wrote wherever it pointed, and `index` and `attest --spec` did the same while
  exiting 0. `V-012` reported the id, but after the write rather than instead
  of it. These are verbs documented as safe to run against a stranger's branch.
  Every file name derived from a spec id is now checked before anything is
  written, pruned or created: a name that is not one plain file name refuses
  the whole write with exit 3 and leaves the tree as it was.
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "022-index-sharding"
  - "039-per-spec-attestation"
establishes:
  - { kind: file, path: "crates/spec-spine-cli/tests/derived_paths.rs" }
extends:
  # 3.1 and 3.2: the one shard writer both `compile` and `index` call, and the
  # name check beside it.
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-core/src/shard.rs" }, nature: additive }
  # 3.3: the per-spec attestation path.
  - { spec: "039-per-spec-attestation", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_attest.rs" }, nature: additive }
  # D-4: the comment on `validate_spec_id` that said `attest --spec` was
  # already protected; a comment correction, no behavior.
  - { spec: "039-per-spec-attestation", unit: { kind: file, path: "crates/spec-spine-cli/src/verify_attestation.rs" }, nature: corrective }
references:
  - { unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, role: context }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/index.rs" }, role: context }
intent:
  goal: "a spec's id can never make a governance verb write, prune or create a file outside the directory that verb writes into"
  non_goals:
    - "a stricter id grammar: V-012 is unchanged, and an id that fails it but is a plain file name still gets its shard"
    - "confining the paths an authority unit may name, which is a read-side question"
---

# 126: A derived file stays in its directory

## 1. Purpose

### 1.1 Measured, on `main` at `95ef76df` (0.24.0)

Spec 022 §3 puts every per-spec shard at a fixed place:
`<derived>/spec-registry/by-spec/<id>.json` for the registry and
`<derived>/codebase-index/by-spec/<id>.json` for the index. FR-001 makes emit a
directory sync over those two directories. Spec 039 puts a per-spec
attestation at `<derived>/attestation/by-spec/<id>.json`.

The `<id>` in each is the spec's frontmatter `id`, interpolated as given. On a
scratch corpus, with the released binary:

| Frontmatter `id` | Verb | Exit | What was written |
|---|---|---|---|
| `"../../../package"` | `compile` | 1 | the repository's own `package.json`, overwritten with a shard |
| `"<absolute dir>/absolute"` | `compile` | 1 | `absolute.json` in that directory, outside the repository |
| `"../../../../escaped"` | `compile` | 1 | `escaped.json` one level above the repository root |
| `"../../../../escaped"` | `index` | 0 | the same, from the index's `by-spec/` |
| `"../../../../pwned"` | `attest --spec ../../../../pwned` | 0 | `pwned.json` above the repository root |
| `""` | `compile` | 1 | a hidden `by-spec/.json`, which the reader never lists, so the tree reports stale for good |

`compile` did report `V-012` (and `V-001`) for each id. It reports them after
the shards are written, which is the order spec 064 §3.2 fixes for a reason
that has nothing to do with this: the exit code is decided after emission, and
emission never depended on the verdict. `index` does not validate ids at all.

### 1.2 Why it matters

The README states the trust model this breaks: `compile`, `index`, `lint` and
`couple` "run against PR branches whose contents are, in the general case, a
stranger's". A contributor who runs `spec-spine compile` or `spec-spine index`
on such a branch, which is the first step of the gate `AGENTS.md` lists, lets
the branch overwrite any `*.json` file the user can write. CI is not exposed: it
runs the read-only `check` in their place. `verify` is the verb kept out of the gate chain because
it executes; these were believed not to write outside their own tree.

The attestation path carried a comment saying it was safe
(`verify_attestation.rs`): "`attest --spec` is already protected by its
registry lookup, which refuses an unknown id before anything is written." The
lookup refuses an unknown id. It does not refuse a known one, and the corpus
chooses which ids are known.

## 2. Territory

- `crates/spec-spine-core/src/shard.rs` (extends 022): the name check, and its
  use in `sync_dir`, the one function both `compile` and `index` write shards
  through.
- `crates/spec-spine-cli/src/cmd_attest.rs` (extends 039): the same check before
  the per-spec attestation path is created or written.
- `crates/spec-spine-cli/tests/derived_paths.rs` (establishes): the three verbs
  through the shipped binary.
- `crates/spec-spine-cli/src/verify_attestation.rs` (extends 039, D-4): the
  comment that called `attest --spec` already protected.

## 3. Behavior

### 3.1 A plain file name

A name derived from a spec id is **plain** when all of these hold:

- it is not empty, and does not begin with `.`;
- it contains no `/`, `\`, `:` or NUL;
- as a path, it is exactly one ordinary component (so not `.`, `..`, a root or
  a prefix).

The rule is about what the name does to a path, not about what an id should
look like. `V-012` is the id grammar and is unchanged. An id such as `001-Foo`
fails `V-012` and is still a plain file name, so it still gets a shard, exactly
as before; `compile` still reports it and still exits 1.

### 3.2 The shard writer checks every name before it touches the directory

`shard::sync_dir` MUST check every name it was given against §3.1 before it
creates the directory, prunes a file or writes one. If any name is not plain,
it MUST return an I/O error naming that name and the directory, and MUST leave
the directory and every file outside it exactly as they were: nothing pruned,
nothing written, nothing created.

Because `compile` and `index` write every per-spec and per-package shard
through `sync_dir`, both verbs refuse with exit 3. A package shard's name is
already a slug that cannot fail §3.1 (spec 022 §3), so in practice only a spec
id reaches the refusal.

### 3.3 The per-spec attestation checks its name before it is written

`attest --spec` MUST apply §3.1 to the resolved spec id's file name before it
creates `<derived>/attestation/by-spec/` or writes the attestation or its
seal. A name that is not plain refuses with exit 3 and writes nothing.

### 3.4 What the refusal says

The message names the file name, the directory it would have been written
into, and says nothing was written. It points at `spec-spine compile --check`,
which reports the spec's `V-012` without writing.

Exit 3 is the code for a refused filesystem write. Where the corpus also fails
validation, 3 outranks 1, the order `check` already applies (spec 062).

## 4. Out of scope

- **The id grammar.** `V-012` is not tightened, and whether an invalid id
  should get a shard at all is not reopened (§3.1).
- **Authority unit paths.** A unit that names `../outside.rs` or an absolute
  path is read, not written; whether the index may look outside the repository
  is a separate question with its own contract (spec 004's "repo-relative
  path").
- **`verify-attestation --spec`.** It reads, and already refuses a traversing
  argument (`validate_spec_id`). It is unchanged.
- **The `compile` order.** Violations are still printed after the write when a
  write happens (spec 064 §3.2).

## 5. Resolved decisions

**D-1 (2026-09-23): the check sits at the writer, not at validation.** The
alternative was to omit the shard of any spec failing `V-012`. That changes the
bytes emitted for a corpus with an invalid but harmless id, which every
existing corpus in that state would see as churn, and it would leave `attest
--spec` and any future writer unprotected. Checking the name at the point it
becomes a path protects every caller of `sync_dir` at once and changes nothing
for a name that was already safe.

**D-2 (2026-09-23): the whole write is refused, not the one shard.** Skipping
the offending shard and writing the rest would leave a ledger that silently
lacks a spec, and `compile` would still exit 1 for a reason the operator reads
as a validation problem. A refusal before anything moves is the state spec 021
FR-006 asks of a failing mode: visible, and without side effects.

**D-3 (2026-09-23): the artifact root the CLI creates before `sync_dir` is
outside 3.2.** `compile` creates `<derived>/spec-registry` and `index` creates
`<derived>/codebase-index` (`cmd_compile.rs` and `cmd_index.rs`, each a
`create_dir_all` just before the first `sync_dir` call) before any shard name
is seen. On a refused run over a tree that already exists, which is every
committed ledger, that call changes nothing. On a first build it leaves that one
empty directory, and its `.derived` parent, behind. The directory is a
configured path, not one derived from an id, so it cannot be steered outside
the derived tree, and 3.2 scopes its guarantee to `sync_dir`, which creates
nothing on a refusal. Moving the check ahead of those calls would mean claiming
both files, which is territory this spec does not declare; the gap is recorded
rather than closed, and `derived_paths.rs` asserts exactly that one empty
directory so the behavior cannot change unnoticed. The check is lexical: it
does not resolve a symlink already present inside `by-spec/`, which only the
repository's own checkout can put there and which neither 3.2 nor 3.3 speaks to.

**D-4 (2026-09-23): the comment in `verify_attestation.rs` is corrected under
an `extends` edge.** It said `attest --spec` was protected by its registry
lookup, which 1.2 shows was false. Leaving a comment that names the defect as
impossible beside the fix was the alternative; correcting it is a comment-only
change, so the edge is `corrective` on spec 039's unit, which already extends
that file, and no behavior of `verify-attestation` changes (4).

## Verification

```verify:cli
# 3.1 to 3.4, through the shipped binary: compile, index and attest --spec each
# refuse a traversing, an absolute and an empty id with exit 3, the victim files
# are byte-identical afterwards, nothing is written outside the derived tree,
# and a valid corpus in the same fixture still compiles, indexes and attests.
cargo test -p spec-spine-cli --test derived_paths --locked
# 3.1, 3.2: the name rule itself, and sync_dir leaving an existing shard
# directory untouched when one name in the batch is refused.
cargo test -p spec-spine-core --lib shard:: --locked
```
