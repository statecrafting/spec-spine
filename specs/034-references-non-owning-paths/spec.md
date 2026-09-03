---
id: "034-references-non-owning-paths"
title: "`references` seeds no implementing path: closing the C-001 ownership leak"
status: approved
kind: "tooling"
created: "2026-09-03"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
amends:
  - "004-codebase-index"
extends:
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
summary: >
  The contract says `references` is the only non-owning edge and that the
  coupling gate ignores it. The indexer did not implement that. It seeded a
  spec's `implementingPaths` from every resolved unit location, owning or not,
  so a file a spec merely cited became a file it claimed. Every consumer reads
  that field as a claim: `couple.rs` treats a listed path as whole-file
  ownership, the indexer's own `owners_of_unit` does the same, and
  `orphaned_specs` and coverage read a non-empty list as "this spec claims
  something". One wrong seed made all of them wrong together. The observable
  defect is that a spec could not edit its own `spec.md` without a waiver once
  any other spec referenced it: on this corpus, changing
  `specs/020-derived-artifact-merge-driver/spec.md` was refused as `C-001` with
  `024-index-sharding`, which only cites it, named as the sole owner. This spec
  filters the seed by the `ownership` flag the loop already holds, at the single
  source rather than at each consumer. The citation is not lost: it stays in
  `resolvedUnits` with `ownership: false`, so provenance is preserved and only
  the ownership view changes. No DTO or schema shape changes.
---

# 034: `references` seeds no implementing path

## 1. Purpose

`standards/spec/contract.md` states the rule in one line: eight typed edges,
"`references` is the only non-owning one (the coupling gate ignores it)". The
constitution's spec-first principle rests on it, because ownership is what
decides whose `spec.md` must change when a file changes.

The indexer did not implement that rule. Building each spec's traceability
mapping, it walked the spec's units and seeded the mapping's `implementingPaths`
from every resolved location, with the unit's `ownership` flag in hand and
unused. A `references` edge therefore produced a claim indistinguishable from an
`establishes` one.

The consequence is not cosmetic, because `implementingPaths` is read as an
authority claim in four places:

| reader | reads a listed path as |
|---|---|
| `couple.rs::owners_for_path` | whole-file `C-001` ownership |
| `index.rs::owners_of_unit` | ownership of a file unit |
| `orphaned_specs` | "this spec claims something" |
| coverage (spec 032) | a claimed source file |

The `resolvedUnits` loops beside the first two check `ownership` correctly. The
`implementingPaths` loops did not, so the mistake bypassed the guard that was
already there.

On this corpus the defect is reachable today. `024-index-sharding` references
`specs/020-derived-artifact-merge-driver/spec.md` for context. Changing spec
020's own `spec.md` was therefore refused:

```
C-001 'specs/020-derived-artifact-merge-driver/spec.md' changed without an
authoring edit to any owning spec (024-index-sharding)
```

A spec could not be edited without touching an unrelated spec that has no
authority over it, or filing a waiver. That inverts the model: citing a document
conferred power over it.

## 2. Territory

`index.rs`: the unit loop that seeds `paths` for a spec's mapping. `tests/index.rs`:
the acceptance fixtures. Nothing in `couple.rs` changes; its reading of
`implementingPaths` was always correct given a correct field.

## 3. Behavior

### 3.1 The rule

An authority unit contributes to a mapping's `implementingPaths` **only when its
edge is owning**. `references` is the sole non-owning edge, so in practice this
excludes exactly `references` units and nothing else.

The other two seed sources are unaffected and remain owning by construction: a
package manifest naming a spec (`[package.metadata.*].spec`), and a `# Spec:`
comment header in a file. Both are a file declaring its own owner.

### 3.2 Fix at the source, not at the consumers

The filter belongs in the indexer, at the one place the field is built, not in
each of the four readers. Fixing the consumers would mean repeating the same
predicate four times and leaving the emitted index still asserting a claim that
is not true. `implementingPaths` means "paths this spec implements"; a cited
file is not one, and the field should not say it is.

### 3.3 Provenance is preserved

The citation is filtered out of the ownership view only. The unit remains in
`resolvedUnits` with `ownership: false` and its resolved locations intact, so
"which files does this spec cite" stays answerable from the index. Nothing that
was recorded stops being recorded; a claim that was never true stops being
asserted.

### 3.4 Shape and compatibility

No DTO field is added, removed or retyped, and no JSON Schema changes, so
`INDEX_SCHEMA_VERSION` does not move. Only content changes: a mapping whose spec
has `references` units loses those paths from `implementingPaths`, which
restamps the affected index shards.

Downstream effects are the intended ones. A spec whose only units were
`references` now reports as orphaned, which is correct: it implements nothing.
Coverage is unchanged on this corpus, because every referenced target here is a
document or a `spec.md`, not a source file.

### 3.5 Tests (minimum)

1. An owning edge and a `references` edge to two equally-real files: only the
   owning path appears in `implementingPaths`.
2. The `references` unit is still present in `resolvedUnits` with
   `ownership: false` and one resolved location.
3. End to end: a spec that only `references` another spec's `spec.md` is not an
   owner of it, so editing that `spec.md` alone clears the gate.

## 4. Out of scope

- **The `specs/` prefix hardcoded in `couple.rs`** (lines 337 and 422) while
  `Config::specs_dir` is configurable. A real defect for an adopter who moves
  their corpus, and a separate one: it is about locating a spec's authoring
  file, not about which edges confer ownership.
- **Whether a `spec.md` should be ownable by another spec at all.** Amends and
  supersedes deliberately transfer authority over another spec's territory, and
  this spec does not revisit that. It removes only the non-owning edge from the
  ownership computation.
- **Any change to `couple.rs`.** Its ownership logic was correct given a correct
  index, and this spec keeps the fix to the field's producer.
