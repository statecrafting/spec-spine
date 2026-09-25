---
id: "147-a-link-above-the-derived-root-is-checked-on-read"
title: "A link above the derived root is checked on read"
status: approved
kind: "security"
created: "2026-09-25"
summary: >
  Spec 144's link walk skips every symbolic link at or above the derived and
  state roots and leaves them to spec 127, whose refusal fires only on write.
  A read-only verb therefore reads through such a link unchecked: with
  `.statecraft` a link to a directory outside the repository, `check` reads the
  committed derived tree and a governed file stored beside it from outside and
  reports the repository fresh, exit 0. This spec checks those links on read
  with 144's own rule, while still not descending into either root.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "127-a-derived-file-is-where-its-path-says"
  - "144-a-repository-path-is-one-type"
amends:
  - "144-a-repository-path-is-one-type"
amends_sections: ["3.4"]
extends:
  - { spec: "144-a-repository-path-is-one-type", unit: "crates/spec-spine-core/src/pathutil.rs", nature: corrective }
  - { spec: "144-a-repository-path-is-one-type", unit: "crates/spec-spine-core/tests/repo_path.rs", nature: additive }
---

# 147: A link above the derived root is checked on read

## 1. Purpose

Measured on the 0.27.0 tree (`7ce4935f` plus the ratifications), 2026-09-25.
A repository with `derived_dir = ".statecraft/derived"`, a spec establishing
`.statecraft/notes.md`, and that file in `[index] extra_hashed_inputs`, is
compiled and indexed normally. Then `.statecraft` is moved outside the
repository and replaced by a symbolic link to it:

```
$ spec-spine check                  # exit 0
spec-registry: fresh
codebase-index: fresh
$ spec-spine index owner .statecraft/notes.md   # exit 0
.statecraft/notes.md
  001-a  unit       .statecraft/notes.md (establishes)
```

Editing the file outside the repository makes `check` report
`modified inputs.json`: the verdict is computed over bytes the repository does
not hold. `compile` and `index` refuse (spec 127, exit 2) because they write;
`check`, `couple`, `index owner`, `index coverage` and `lint` never write, so
spec 127 never runs for them.

Spec 144 D-4 recorded the hole ("a governed file stored under such an ancestor,
outside both roots, is read through the link unchecked") on the ground that no
corpus stores one there. This repository's own layout does: `.statecraft/` is
ordinary governed territory outside its two roots (`CLAUDE.md`, "The boundary
with Statecraft"), and the committed derived tree under it is read by every
freshness verb.

## 2. Territory

- `crates/spec-spine-core/src/pathutil.rs`: `refuse_links_leaving`.
- `crates/spec-spine-core/tests/repo_path.rs`: the test
  `a_linked_ancestor_of_the_derived_root_is_checked_on_read`, covering 3.1 and
  3.2 through the library.

## 3. Behavior

### 3.1 The roots and their ancestors are checked, not entered (amends 144 3.4)

`refuse_links_leaving` MUST apply 144 3.4's rule to a symbolic link at a
derived or state root or at any ancestor of one: a link whose target, fully
resolved, is not below the repository root's own resolution refuses the run,
`Error::Refused`, exit 2, naming the link. The walk MUST still not descend
into either root.

A link at or above a root that resolves inside the repository stays spec
127's: it is accepted on read, and refused on write with 127's message.

### 3.2 The message says which read it guards

The refusal MUST name the link and say that the derived tree or a governed
file is read through it, so the author is not sent to spec 127's remedy for a
read-only verb.

## 4. Out of scope

- Windows links and junctions: spec 148.
- Links below the derived root (for example `.statecraft/derived/spec-registry`
  as a link): spec 127 refuses them on write. Whether a read-only verb reading a
  committed shard through one needs the same check is not decided here; it is
  measured first and, if needed, filed as its own spec.

## 5. Resolved decisions

None yet.

## Verification

```verify:cli
# 3.1, 3.2 through the library: the one named test, exactly.
sh -c 'cargo test -p spec-spine-core --locked --test repo_path -- --exact a_linked_ancestor_of_the_derived_root_is_checked_on_read 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 1's reproduction through the shipped binary: with `.statecraft` a link out of
# the repository, check, lint, index coverage and index owner each exit 2 with a
# refusal naming `.statecraft`; with it a link inside, check still reads (exit 0).
# On the 0.27.0 binary all four exit 0 (recorded in the ratification PR).
cargo build --release --locked
sh -c 'T="${TMPDIR:-/tmp}/ss147"; B="$PWD/target/release/spec-spine"; rm -rf "$T" "$T.away" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && printf "pub fn a() {}\n" > "$T/src/a.rs" && printf "[layout]\nderived_dir = \".statecraft/derived\"\n" > "$T/spec-spine.toml" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 && mv "$T/.statecraft" "$T.away" && ln -s "$T.away" "$T/.statecraft" || exit 1; r=0; for v in check lint "index coverage" "index owner src/a.rs"; do "$B" --repo "$T" $v >/dev/null 2>"$T.err"; rc=$?; { test $rc -eq 2 && grep -q "^spec-spine: refused: .*\.statecraft" "$T.err"; } || { echo "$v: exit $rc, expected 2 naming .statecraft"; r=1; }; done; rm "$T/.statecraft" && mkdir "$T/store" && mv "$T.away" "$T/store/sc" && ln -s store/sc "$T/.statecraft" && "$B" --repo "$T" check >/dev/null 2>&1 || { echo "a link inside is not read"; r=1; }; rm -rf "$T" "$T.away" "$T.err"; exit $r'
```
