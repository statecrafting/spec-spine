---
id: "127-a-derived-file-is-where-its-path-says"
title: "A derived file is where its path says"
status: draft
kind: "tooling"
created: "2026-09-23"
summary: >
  Spec 126 made every file name derived from a spec id one plain name, so an id
  can no longer walk a write out of its directory. Two ways past that remained.
  A symbolic link committed inside the derived tree (git records it like any
  file) made `compile`, `index` and `attest` write through it and prune `*.json`
  files wherever it pointed, all with exit 0. And a name that passes 126's
  lexical rule can still name a Windows device (`CON.json`, `nul.tar.json`),
  which is not a file at all there. Every path below the repository root that
  these verbs write, create, remove or prune is now checked for a link before
  anything moves, a device name is refused on every platform, and a refusal
  leaves the tree exactly as it was. A repository reached through a link still
  works.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "022-index-sharding"
  - "039-per-spec-attestation"
  - "126-a-derived-file-stays-in-its-directory"
# 3.5: widens what 126 3.1 calls a plain file name (B-13).
amends: ["126-a-derived-file-stays-in-its-directory"]
amends_sections: ["3.1"]
establishes:
  - { kind: file, path: "crates/spec-spine-cli/tests/derived_symlinks.rs" }
extends:
  # 3.1 to 3.5: the one checked writer every derived output goes through, the
  # name rule beside it, and the package slug (3.6).
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-core/src/shard.rs" }, nature: additive }
  # 3.3: each verb's outputs, handed to that writer as one run.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_compile.rs" }, nature: corrective }
  - { spec: "004-codebase-index", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_index.rs" }, nature: corrective }
  - { spec: "039-per-spec-attestation", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_attest.rs" }, nature: corrective }
references:
  - { unit: { kind: file, path: "crates/spec-spine-cli/tests/derived_paths.rs" }, role: context }
intent:
  goal: "no committed content can make a governance verb write, create, remove or prune a file anywhere but the path its derived tree names"
  non_goals:
    - "protection against another process replacing a path while the verb runs"
    - "a claim about Windows links or junctions, which were not measured"
    - "confining a configured derived_dir that itself points outside the repository"
---

# 127: A derived file is where its path says

## 1. Purpose

### 1.1 Measured, on `main` at `f6afdc61` (spec 126 merged)

Spec 126 checks the **name** a derived file gets from a spec id. It says
nothing about the **directories** that name is joined to, or about a file
already sitting at the path, and 126 D-6 recorded that as a separate finding.
Git records a symbolic link (mode `120000`) like any file, so a branch being
inspected can carry one anywhere in the derived tree. On a scratch repository
with the default layout and the binary built from `f6afdc61`:

| Link committed at | Pointing at | Verb | Exit | Effect |
|---|---|---|---|---|
| `spec-registry/by-spec/001-a.json` | a file outside | `compile` | 0 | the outside file overwritten with shard JSON |
| `spec-registry/by-spec/` | a directory outside | `compile` | 0 | shard written there, `other.json` there pruned |
| `.derived` itself | a directory outside | `compile` | 0 | shards and `build-meta.json` written there, a stray `*.json` pruned |
| `codebase-index/by-spec/` | a directory outside | `index` | 0 | shard written there, `other.json` pruned |
| `attestation/by-spec/001-a.json` | a file outside | `attest --spec 001-a` | 0 | the outside file overwritten |
| `attestation/by-spec/001-a.sig` | a file outside | `attest --spec 001-a --sign` | 0 | the outside file overwritten with the seal |

Paths are under `.derived/`. The managed layout (`.statecraft/derived`) adds
one more ancestor that can be a link. The same table reproduced on the pre-126
binary. `check` over the first layout exits 3 and changes nothing (a read, and
out of scope, 4).

These are the verbs the README says run against a stranger's branch, and
`compile` and `index` are the first two steps of the gate `AGENTS.md` lists.

### 1.2 Found by reading: a plain name can still be a device

Windows reserves `CON`, `PRN`, `AUX`, `NUL`, `COM1` to `COM9` and `LPT1` to
`LPT9`, and Microsoft documents that the name followed by an extension
(`NUL.txt`, `NUL.tar.gz`) is the device too, as are the superscript forms
`COM¹`, `COM²`, `COM³`, `LPT¹`, `LPT²`, `LPT³`. Windows also strips trailing
dots and spaces from a path component, so `CON .json` and `a.json.` are not
the names they look like. Each is a plain name under 126 3.1. An id `CON`
passes 126, fails `V-012`, and would still get its shard: `CON.json`, which on
Windows is the console. This was found by reading the rule against the
documentation; it was **not** measured on a Windows host.

## 2. Territory

- `crates/spec-spine-core/src/shard.rs` (extends 022): the checked writer, the
  link and kind check, the device-name rule in `check_file_name`, and the
  package slug's escape (3.6). `sync_dir` becomes a use of the writer.
- `crates/spec-spine-cli/src/cmd_compile.rs` (extends 001),
  `crates/spec-spine-cli/src/cmd_index.rs` (extends 004) and
  `crates/spec-spine-cli/src/cmd_attest.rs` (extends 039): each verb hands its
  whole set of outputs to the writer as one run.
- `crates/spec-spine-cli/tests/derived_symlinks.rs` (establishes): the cases in
  1.1, the controls and the device ids, through the shipped binary.

## 3. Behavior

### 3.1 What is checked: every component below the repository root

A path a verb writes, creates, removes or prunes is checked component by
component, from the first component of `[layout] derived_dir` down to the file
itself. The repository root, as the verb was given it (`--repo`, or the
working directory), and every ancestor of it are **not** checked. A repository
reached through a link, a home directory that is a link, and `$TMPDIR` on
macOS (`/var` is a link to `/private/var`) all keep working, and a test in
`derived_symlinks.rs` proves it.

### 3.2 A link refuses the whole run

If any checked component is a symbolic link, the verb MUST refuse with exit 3
before it writes, creates, removes or prunes anything in that run. It MUST NOT
replace the link with a real file or directory, which would silently repair a
hostile tree. The message names the linked path relative to the repository,
says whether a directory or a file was expected there, says nothing was
written, and says the derived tree must contain no links.

### 3.3 The preflight covers every output of the run

The check runs once, over everything the run is going to touch, before the
first mutation:

- `compile`: `spec-registry/by-spec/` and each shard it writes, each `*.json`
  entry it would prune there, the legacy `registry.json` it removes, and
  `build-meta.json`.
- `index`: both `by-spec/` and `by-package/` with their shards and prune
  entries, `slices.json` (written or removed), and the legacy `index.json`.
- `attest`: `attestation.json`, `snapshot.json` or
  `by-spec/<id>.json`, and under `--sign` the seal beside it. The seal is
  computed before anything is written, so a refused seal path leaves the
  attestation unwritten too.

An entry due for pruning that is itself a link is refused, not removed (3.2).

### 3.4 Directories are created one level at a time

A component that does not exist is created with a single-level create after
the preflight, each level re-checked as it is reached, so no directory is
created through a link. An existing component of the wrong kind (a file where
a directory is expected, or a directory where the output file goes) is refused
in the same preflight, with the same exit 3 and the same no-change guarantee,
rather than failing with an I/O error halfway through the run.

### 3.5 A plain file name is not a device name (amends 126 3.1)

126 3.1 is widened. A name derived from a spec id is plain when, in addition
to 126's conditions:

- it does not end in `.` or a space;
- its stem, the text before the first `.` with trailing spaces removed, is
  not, compared ignoring ASCII case, `CON`, `PRN`, `AUX`, `NUL`, `COM1` to
  `COM9`, `LPT1` to `LPT9`, `COM¹`, `COM²`, `COM³`, `LPT¹`, `LPT²` or `LPT³`.

The rule applies on **every** platform, as 126 already refuses `\` and `:`
everywhere, so a corpus gets the same verdict on every release triple. The
refusal is 126's: exit 3, the name and directory named, nothing written.
`CONSOLE`, `COM0`, `COM10`, `NULL` and `001-con` remain plain.

### 3.6 A package slug stays plain

A package shard is named after a slug of the package name (spec 022, "a
filesystem-safe slug"), and 126 asserts a slug can never fail 3.1. A package
named `aux` or `con` would now fail it, so a slug whose stem is a device name
gets a leading `_`, the escape the slug already uses for a leading dot.

## 4. Out of scope, and the limits of the claim

- **Concurrent replacement.** The threat is committed content, which is static
  once checked out. Another local process that swaps a checked directory for a
  link between the check and the write is outside this spec; 3.4's re-check at
  each created level narrows that window for directories and closes nothing
  else. No claim is made against a concurrent attacker.
- **Windows links and junctions.** The check reads `symlink_metadata`. Whether
  a directory junction reads as a link there was not measured, so no
  protection is claimed for junctions or for Windows links. The link tests are
  Unix-only; the device-name tests (3.5) are platform-independent unit tests
  and run everywhere.
- **A configured `derived_dir` outside the root.** `derived_dir =
  "../outside"` (or an absolute path) is not below the root, so 3.1 does not
  reach it. Measured on `f6afdc61`: `compile` writes and prunes shards there,
  exit 0. The configuration file is committed content too, so this is its own
  finding for its own spec; it is recorded, not closed. A path that is not
  wholly below the root is written as before and checked only for the part
  that is.
- **The read side.** `check`, `index check` and `registry` reading through a
  linked shard.
- **`compact`**, which writes the authored corpus under `specs/`, not the
  derived tree.
- **Other characters Windows refuses** (`<`, `>`, `"`, `|`, `?`, `*`). Writing
  one fails with an I/O error; it cannot name a different file.

## 5. Resolved decisions

**D-1 (2026-09-23): check and refuse; no temporary-file-and-rename.** Writing
each file to a sibling temporary name with create-new and renaming it onto the
checked name would stop a final-component link created after the check. It
would not stop a linked directory, it adds a crash residue that must never end
in `.json`, and the race it narrows is outside this spec (4). `O_NOFOLLOW`
needs the `libc` crate as a direct dependency, which is not added.

**D-2 (2026-09-23): refuse, never repair.** Replacing a link with a real file
would make the run succeed on a tree it was never meant to accept, and the
hostile content would disappear from the evidence. The refusal leaves the link
in place and names it.

**D-3 (2026-09-23): one writer, and `sync_dir` keeps its signature.** The
checked writer in `shard.rs` takes the repository root and the run's outputs
(synchronize a shard directory, write one file, remove one file) and applies
them as a whole. `sync_dir(dir, files)` is kept, because the core integration
tests and any library caller use it, and becomes the writer with `dir`'s
parent as its root: it checks `dir` and every entry it touches, and nothing
above. The CLI no longer calls it; every verb passes the real root.

**D-4 (2026-09-23): a path not wholly below the root is checked as far as it
is.** The check walks the path's components relative to the root and stops at
the first one that is not an ordinary name (`..`, a root, a prefix) or when
the path is not under the root at all. Refusing such a path would change what
a configured `derived_dir` may be, which is 4's separate finding.

**D-5 (2026-09-23): a component that does not exist ends the walk.** Nothing
below a missing component can exist, so nothing below it can be a link. A
component that cannot be read for another reason (permissions) also ends the
walk, and the write then fails with today's I/O error.

**D-6 (2026-09-23): trailing dots and spaces.** Every derived name ends in
`.json` or `.sig`, so no derived name can end in a dot or a space today; the
live form is inside the stem (`CON .json`), which 3.5's trim covers. The
whole-name rule is kept anyway so `check_file_name` stands on its own for any
future writer, and it is tested directly.

**D-7 (2026-09-23): the superscript forms are included.** They are documented
as reserved and cost six strings.

**D-8 (2026-09-23): the slug escape changes a file name, rarely.** A
repository with a package whose slug stem is a device name (`aux`, `con`) sees
that one `by-package/` shard renamed with a leading `_` at its next `index`,
and `check` reports it stale until then. Its content is unchanged. Without the
escape, `index` would refuse such a repository outright.

## Verification

```verify:cli
# 3.1 to 3.4 and the device ids of 3.5, through the shipped binary: every case
# of 1.1 (and the managed layout's `.statecraft` ancestor) exits 3 with the
# whole fixture byte-identical, a link at any later output or prune entry
# refuses before the first shard moves, both index batches are checked first,
# a wrong-kind component refuses, a clean tree builds, rebuilds and prunes, and
# a repository reached through a link still builds.
cargo test -p spec-spine-cli --test derived_symlinks --locked
# 3.5 and 3.6 as platform-independent unit tests: device names with and without
# extensions, trailing dots and spaces, the names that stay plain, and a
# package slug that can never fail; plus the checked writer itself.
cargo test -p spec-spine-core --lib shard:: --locked
# 126's own acceptance still holds under the widened rule.
cargo test -p spec-spine-cli --test derived_paths --locked
```
