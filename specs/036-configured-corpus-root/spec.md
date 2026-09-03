---
id: "036-configured-corpus-root"
title: "The coupling gate honors `layout.specs_dir`"
status: draft
kind: "tooling"
created: "2026-09-03"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
amends:
  - "005-coupling-gate"
extends:
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
summary: >
  `layout.specs_dir` is configuration, and every other consumer reads it:
  `compile` scans it, `index` builds each spec's `spec.md` path from it, and
  `init` scaffolds into it. The coupling gate did not. Two functions in
  `couple.rs` hardcoded the literal `specs/`, and both are load-bearing. The
  primary-owner heuristic asked whether `specs/<id>/spec.md` was in the diff, so
  for an adopter whose corpus lives anywhere else the answer was permanently no:
  no edit to an owning spec could clear `C-001`, and every governed change was
  refused with the owner named and no reachable way to satisfy it, short of a
  waiver on every PR. The amends-awareness parser had the same literal, so
  amendment expansion silently never fired. Both now take the configured root,
  and the builder is shared with the indexer (`spec_md_rel`, promoted to
  `pub(crate)`) so a path constructor and its parser cannot drift apart again. A
  trailing slash on the configured value is trimmed, making `specs` and `specs/`
  name one corpus root instead of the second yielding `specs//<id>/spec.md`. No
  DTO, schema, or config shape changes; a default-layout repo is byte-identical.
---

# 036: The coupling gate honors `layout.specs_dir`

## 1. Purpose

`Config::layout.specs_dir` names the corpus root. It defaults to `specs`, and
the rest of the system treats it as configuration throughout: `compile` reads
the directory from it (`compile.rs`), `index` builds each mapping's `spec.md`
path from it (`index.rs::spec_md_rel`), and `init` scaffolds into it
(`scaffold.rs`). An adopter who sets `specs_dir = "contracts"` gets a corpus
that compiles, indexes, and lints correctly.

The coupling gate then refuses their every change.

Two functions in `couple.rs` spelled the root as a literal:

| site | question it answers | with the literal |
|---|---|---|
| `any_owner_in_diff` | is an owning spec's `spec.md` in this diff? | looks for `specs/<id>/spec.md`, a path that does not exist in the repo |
| `spec_id_for_spec_md_path` | is this path a `spec.md`, and whose? | `None` for every real corpus path |

The first is the **primary-owner heuristic**: the single mechanism by which a
`C-001` violation is cleared. Editing the owning spec is how an author says
"this change is governed, and here is the governing text". Under a non-default
root that mechanism could not fire, because the gate searched for a path the
repo does not contain. The violation was reported correctly, named the correct
owner, and was unclearable:

```
C-001 'src/lib.rs' changed without an authoring edit to any owning spec (001-a)
```

with `contracts/001-a/spec.md` sitting in the very same diff. The only exits
were a `Spec-Drift-Waiver:` on every pull request, or moving the corpus back to
`specs/`. Spec-first development, the constitution's principle III, was
unavailable to that adopter.

The second site made amends-awareness (005 §3.3) dead code under the same
condition: the FR-005 expansion parses the changed path back into a spec id, and
a parser that never matches never expands the owner set.

This is a defect of reach, not of judgement. The gate's verdicts were right
about the corpus it was told to look at; it was told to look at the wrong place.

## 2. Territory

`couple.rs`: the two functions above, plus `owners_for_path` and `couple_with`,
which thread the configured value to them. `index.rs`: `spec_md_rel` gains
`pub(crate)` visibility and trims a trailing slash; its two existing callers are
unchanged in behavior. `tests/couple.rs`: acceptance fixtures under a
non-default root.

Nothing about the emitted artifacts changes. This is a read of configuration
that was previously not performed.

## 3. Behavior

### 3.1 One constructor, one parser, one root

`spec_md_rel(specs_dir, id)` builds `<specs_dir>/<id>/spec.md`;
`spec_id_for_spec_md_path(specs_dir, path)` is its exact inverse. The gate uses
the same constructor the indexer uses rather than a second copy of the format
string, because two spellings of one path shape is how this defect arose.

`couple_with` binds `cfg.layout.specs_dir` once and passes it to both readers.
The gate remains a pure function of `(config, registry, index, diff, waiver)`;
the configured root was always part of that first argument, and is now actually
consulted.

### 3.2 Prefix matching is separator-aware

The parser strips the configured root and the `/` separator as two steps. A
single `strip_prefix("specs")` would accept `specsX/005-x/spec.md` and report
its id as `005-x`, inventing authority for a directory that merely shares a
prefix with the corpus root. The separator must be present for the path to be a
corpus path.

### 3.3 A trailing slash names the same root

`specs` and `specs/` denote one corpus root. Both are trimmed before use. This
also repairs the indexer, where `specs_dir = "specs/"` previously produced
`specs//<id>/spec.md`: a path that hashes into the shard as a missing file, so
a spec's own source stopped contributing to its shard hash. Nothing validated
the trailing slash away, so this was reachable configuration.

### 3.4 No change under the default layout

For `specs_dir = "specs"`, every path this spec touches is the string the
literal produced. The committed registry and index of a default-layout repo,
this one included, are byte-identical across the change, and the determinism
gate proves it across the release matrix as usual.

### 3.5 Tests (minimum)

- The parser accepts `<root>/<id>/spec.md` for a configured root, rejects the
  default `specs/` once another root is configured, accepts a nested root
  (`docs/specs`), rejects a sibling sharing the prefix (`specsX/`), and accepts
  a root written with a trailing slash.
- `any_owner_in_diff` matches under the configured root and only there.
- End to end under `specs_dir = "contracts"`: a changed owned file drifts, and
  editing `contracts/<owner>/spec.md` clears it (the regression this spec
  fixes); a file at `specs/<owner>/spec.md` does **not** clear it; and amends
  expansion still names the amender and still clears when the amender is edited.

## 4. Out of scope

**Validating `specs_dir` at config load.** Trimming at the point of use is
sufficient and local. A normalizing constructor over the whole `layout` block is
a larger change to the config contract and belongs to its own spec.

**The other hardcoded corpus assumption.** `compile`'s V-001 requires a spec's
directory name to equal its `id`; that is a tier-1 anchor
(`directory-name-equals-id`), not a layout question, and is unaffected.
