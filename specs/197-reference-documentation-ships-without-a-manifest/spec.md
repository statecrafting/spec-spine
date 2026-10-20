---
id: "197-reference-documentation-ships-without-a-manifest"
title: "Reference documentation ships without a manifest, on the evidence it has"
status: draft
kind: "governance"
created: "2026-10-19"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "166-deterministic-reference-documentation"
summary: >
  Records the owner's 2026-10-19 rulings on spec 166 D-22 and D-23. Section
  3.11 (`reference generate --manifest`) leaves 166 for a later spec, because
  no spec 159 packet request can yet name every page's sources. Acceptance 2's
  cross-platform clause and acceptance 7's harness clause are restated as the
  evidence the build actually provides. With both recorded, 166 D-24's
  condition is met.
amends:
  - "166-deterministic-reference-documentation"
references:
  - { unit: { kind: file, path: "specs/166-deterministic-reference-documentation/spec.md" }, role: "the amended spec; D-22, D-23 and D-24 are the decisions ruled on" }
  - { unit: { kind: file, path: "crates/spec-spine-core/tests/reference.rs" }, role: "the tests that carry the restated acceptance 2 and 7" }
---

# 197: Reference documentation ships without a manifest, on the evidence it has

## 1. Purpose

Spec 166 was built in four pull requests (#483 to #486) and left at
`implementation: in-progress` by its D-24. D-24 records that a decision entry
cannot narrow a stated clause, and that two entries did:

- **D-22** deferred 3.11, the `--manifest` binding. A spec 160 manifest's
  inputs must be members of one spec 159 packet, and no packet request yet
  expresses "every page's sources" (a closure names one root).
- **D-23** approximated acceptance 2's third clause (the fixture page tree is
  byte-identical across the determinism workflow's release triples) and
  acceptance 7 (a harness that denies environment reads and records opened
  paths).

The owner ruled on 2026-10-19 to accept both, recorded by an amendment rather
than by editing 166's requirements in place. This spec is that amendment.

D-23 also says the committed fixture page is compared "by the core tests,
which CI runs on every release triple, including Windows". That overstates
it: the required workflow runs the core tests on Linux and Windows only, and
the four-triple determinism workflow emits the registry and index shard trees,
not reference pages. The restated acceptance 2 below says what is run.

## 2. Territory

None. This spec changes what 166 requires and claims no file. The tests that
carry the restated criteria are 166's (`crates/spec-spine-core/tests/reference.rs`).

## 3. Behavior

### 3.1 Section 3.11 leaves spec 166

166 3.11 ("Manifest binding") no longer binds spec 166. `reference generate`
takes no `--manifest` argument. A later spec MAY add it, and SHOULD do so once
a spec 159 packet request can carry a page set's sources; that spec states the
manifest binding afresh rather than reviving 3.11 by reference.

Nothing else in 166 changes: page bytes, regions, the check, the model and the
limits are as 166 states them.

### 3.2 Acceptance 2, restated

This replaces 166 acceptance 2 in full. Acceptance 2 now reads:

> Reordering frontmatter, obligations, edges, and catalog entries leaves page
> bytes unchanged; LF and CRLF spec sources, and LF, CRLF and CR authored
> notes, yield identical pages; and a committed fixture page is reproduced
> byte for byte by the core tests on every platform the required workflow
> runs them on.

The CR case moves to the authored notes because a CR-only `spec.md` is not a
spec (its frontmatter fence is not found), as D-23 records. The determinism
workflow is not extended to the reference page tree.

### 3.3 Acceptance 7, restated

This replaces 166 acceptance 7 in full. Acceptance 7 now reads:

> Generation is a pure function of the reference inputs that
> `gather_reference_inputs` alone reads from 3.8's paths; a test shows that an
> undeclared file, a `.git` directory and a nested page change no generated
> byte.

No harness that denies environment reads or records opened paths is built.
Core's existing invariant (no environment reads, no `git`) is what 3.8 relies
on, and the test pins the observable consequence.

### 3.4 Spec 166's status

With 3.1 to 3.3 recorded, D-22 and D-23 are ruled on and D-24's condition is
met. Its CLI pull request (#486) is on `main` (9384b8fa), so spec 166 MAY
claim `implementation: complete`. Its ratification is a separate owner
approval.

## 4. Acceptance criteria

1. The committed fixture page test (the fixture corpus's `005-epsilon`, a
   `status: retired` spec, hence the test's name
   `the_retired_fixture_page_matches_its_committed_bytes`) and the
   undeclared-input test pass on `main`.
2. The ordering and line-ending test
   (`authoring_order_and_line_endings_never_change_bytes`, restated
   acceptance 2's first two clauses) passes on `main`.

Spec 166's own Verification block is not changed by this amendment and is
not repeated here; it remains 166's evidence, run by `spec-spine verify 166`.

## 5. Out of scope

- Building `--manifest`, or the spec 159 packet request it needs.
- A general harness that records every file a process opens.
- Extending the determinism workflow to the reference page tree.

## 6. Resolved decisions

**D-1 (2026-10-19, owner): amend, do not edit.** The owner accepted
deferring 3.11 and the two approximations, and chose an amending spec so that
166's text keeps what it originally required and this spec records the change.

## Verification

```verify:cli
cargo test -p spec-spine-core --locked --test reference the_retired_fixture_page_matches_its_committed_bytes
cargo test -p spec-spine-core --locked --test reference undeclared_files_and_git_never_reach_a_page
cargo test -p spec-spine-core --locked --test reference authoring_order_and_line_endings_never_change_bytes
```
