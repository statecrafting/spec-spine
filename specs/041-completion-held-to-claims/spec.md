---
id: "041-completion-held-to-claims"
title: "`implementation: complete` defeats draft leniency"
status: draft
kind: "tooling"
created: "2026-09-06"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "025-unresolved-unit-severity"
amends:
  # Narrows 025 3.1 arm 2: the lifecycle tier stops applying to a spec that
  # asserts its own completion. 025's text is unchanged (spec 040).
  - "025-unresolved-unit-severity"
extends:
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
summary: >
  `index.rs::in_flight` is `status == "draft" || implementation == Pending`, and
  spec 025 uses it to downgrade an unresolved owning unit from a blocking
  `I-0xx` error to a counted `W-001` warning. The `Complete` arm of
  `implementation` is never consulted, so `status: draft` alone buys the
  leniency: a spec that states in its own frontmatter that its work is finished
  may claim units that do not exist, and the indexer counts a warning instead of
  refusing. That window is not an edge case in this corpus, it is the normal one.
  A spec is filed as `draft` with `implementation: complete` when its code lands
  (spec 036 did exactly that at 46474a3) and stays that way until a later
  ratification PR flips it to `approved`, so every spec passes through a period
  where its strongest claim about code is the one claim nothing checks. This spec
  makes `implementation: complete` decisive: a spec asserting completion is never
  in flight, whatever its `status`, and its unresolved owning units are hard
  errors. `status` and `implementation` are orthogonal axes (design ratification
  versus code existence), and where they disagree the more specific claim about
  code wins. It verifies existence, not behavior, and 3.3 is explicit that no
  gate can do more than that.
---

# 041: Completion is held to its claims

Wave 2 of `docs/design/02-agentic-builder-substrate.md`, and a narrower thing
than that note anticipated. See §5.

## 1. Purpose

`implementation` is the corpus's statement about code: `pending`, `in-progress`,
`complete`, `n-a`, `deferred`. `status` is the corpus's statement about design:
`draft`, `approved`, `superseded`, `retired`. They are orthogonal, and the
project already treats them that way everywhere else.

One predicate reads both, and reads one of them incompletely:

```rust
// crates/spec-spine-core/src/index.rs:77
fn in_flight(&self) -> bool {
    self.status == "draft" || matches!(self.implementation, Some(Implementation::Pending))
}
```

Spec 025 §3.1 arm 2 uses it to decide severity: an unresolved **owning** unit on
an in-flight spec is a counted `W-001` warning rather than a blocking `I-0xx`
error. That is correct and this spec does not disturb it. Work in progress may
legitimately claim territory that does not exist yet, which is what let specs 037
through 039 be filed against units still to be written.

The defect is that `Implementation::Complete` never enters the expression. Only
the `Pending` arm does, so the `status == "draft"` disjunct decides the outcome
on its own. A spec whose frontmatter says

```yaml
status: draft
implementation: complete
```

asserts that its work is finished and is nonetheless granted the leniency
designed for work that has not started.

### 1.1 This is the normal state, not a corner

In this repo a spec is filed as `draft` with `implementation: complete` in the
same pull request that lands its code, and a separate ratification PR later flips
it to `approved`. Spec 036 was filed exactly so at `46474a3` and ratified at
`f2388ed`; spec 040 sits in that state right now.

So the window is not exotic. Every spec in this corpus passes through a period,
bounded only by how long ratification takes, in which the single strongest claim
it makes about code is the one claim nothing verifies. During that window the
indexer will accept a spec that says "this is built" while pointing at files that
were never written, and report it as a warning in a shard nobody reads (the CLI
prints only errors).

That is the gap. It is not that `implementation` is unread, which is what the
design note assumed: for an **approved** spec, 025 already forces claimed units to
exist. It is that the check switches off during precisely the interval when the
claim is newest and least reviewed.

## 2. Territory

`index.rs::in_flight` and its acceptance fixtures. No DTO changes, no schema
version moves, and no emitted field is added or removed: this changes which tier
an existing diagnostic lands in, for one combination of two existing fields.

## 3. Behavior

### 3.1 The predicate

A spec MUST NOT be treated as in flight when its `implementation` is `complete`,
whatever its `status`. Otherwise the existing rule stands: `status: draft` or
`implementation: pending` means in flight.

| `status` | `implementation` | in flight | unresolved owning unit |
|---|---|---|---|
| draft | pending | yes | `W-001` warning |
| draft | in-progress | yes | `W-001` warning |
| draft | **complete** | **no** | **`I-0xx` error** |
| draft | n-a / deferred / absent | yes | `W-001` warning |
| approved | pending | yes | `W-001` warning |
| approved | in-progress | no (unchanged, see §4) | `I-0xx` error |
| approved | complete | no | `I-0xx` error |
| approved | n-a / deferred / absent | no | `I-0xx` error |

The table is exhaustive over both enums on purpose. Only the `draft` +
`complete` row moves; every other cell states what the predicate already does, so
an implementer cannot read a gap as a licence to guess.

`n-a` and `deferred` assert nothing about code existing, so this spec adds
nothing for them to be held to; they keep taking their answer from `status`, as
does an absent key.

### 3.2 Why the completion claim wins over the draft claim

The two fields are not competing descriptions of one thing, so "which is more
authoritative" is the wrong question. The right one is which field is *about* the
condition being tested, and the condition is whether claimed code exists.

`implementation: complete` is a direct assertion about that. `status: draft` says
the design has not been ratified, which is a statement about review, not about
the filesystem. Letting the second silence a check on the first lets an unrelated
axis grant an exemption it knows nothing about.

Put the other way: an author who writes `complete` has volunteered a falsifiable
claim. Refusing to falsify it because a different field says the design is still
under review is not leniency toward work in progress, it is declining to read
what the author wrote.

### 3.3 What this does not do

It verifies **existence**, not behavior. After this change, `implementation:
complete` means every unit the spec claims resolves to something real. It does
not mean the code does what the spec's prose says, and no mechanical gate in this
project can mean that. The coupling gate refuses drift between a changed unit and
its owning spec; it does not read English.

This matters for anything consuming the field, an autonomous builder above all.
`complete` after this spec is "the claimed territory exists and the corpus
compiles, lints and resolves around it", which is a real and checkable fact, and
it is strictly weaker than "the work is correct". A consumer that treats the
field as acceptance is still trusting an author's word; it is merely no longer
trusting an author's word about something falsifiable that nobody falsified.

### 3.4 Blast radius: none in this corpus

Of the 37 approved specs, 36 are `implementation: complete` and the 37th
(`000-spec-spine-bootstrap`) carries no `implementation` key at all; an absent
key is not `Pending`, so an approved spec without one is already settled too.
Neither group's tier moves. Specs 037, 038 and 039 are `draft` with
`implementation: pending` and stay in flight, which is what keeps spec 037's
claim on the not-yet-written `verdict.rs` a warning. Spec 040 is `draft` with
`implementation: complete` and its one section unit resolves, so it passes the
stricter test it newly falls under.

The committed index is therefore byte-identical across this change, and the
determinism gate should prove it across the release matrix as usual.

### 3.5 Tests (minimum)

- A `draft` + `complete` spec with an unresolved owning unit yields a blocking
  `I-0xx` error, not `W-001`, and `index` exits non-zero on it.
- A `draft` + `pending` spec with the same unresolved unit still yields `W-001`
  (the 025 behavior this must not regress).
- A `draft` + `in-progress` spec still yields `W-001`.
- A `draft` + `complete` spec whose units all resolve produces no diagnostic.
- `n-a` and `deferred` continue to take their tier from `status`.
- An unresolved **non-owning** (`references`) unit stays `W-002` in every
  combination: this spec touches the lifecycle arm only, never edge authority.
- The committed index of this repo is unchanged by the predicate change.

## 4. Out of scope

**Verifying that code matches the spec's prose.** §3.3 is explicit. No gate does
this, and one that claimed to would be worse than none.

**Freezing the verdict.** Recording that the checks passed at the moment
completion was claimed is a signed artifact, not a severity tier. Spec 042.

**Requiring `implementation` at all.** An absent key still behaves as `pending`
for this purpose. Making the field mandatory is a frontmatter-grammar change with
an adopter migration attached, and it is not needed here.

**Fixing `approved` + `in-progress`, which is very likely a defect.** That
combination is *not* in flight today, so a ratified spec whose implementation is
openly underway gets hard `I-0xx` errors for units it has not written yet. Spec
025 arm 2 named `draft` and `pending` and did not name `in-progress`, and nothing
in 025 argues that omission, which is what makes it look accidental rather than
chosen: `pending` and `in-progress` are both statements that the work is not
finished, and only one of them buys leniency.

It bites an adopter, not this repo. Nothing here is `approved` + `in-progress`
(the local flow files `draft` + `pending`, builds, then ratifies), but a corpus
that ratifies a design before building it lives in that state for the whole
build, which is the same class of adopter-flow problem the OAP dry run raised
against 025 in the first place.

It is nonetheless left alone here. This spec argues one thing, that a completion
claim defeats leniency; widening leniency for `in-progress` runs the opposite
direction, and bundling the two would make a single spec that both tightens and
loosens the same predicate for unrelated reasons. It deserves its own spec and
its own argument, and §3.1 now states the current behavior explicitly so that
spec has something exact to amend.

**The `approved` + `pending` combination.** A ratified spec whose code is not
yet written stays in flight, and its unresolved owning units stay `W-001`
indefinitely. That is deliberate and is left alone: `pending` is an honest
statement that the work has not been done, and this corpus files specs that are
ratified before they are built. The asymmetry with §3.1 is the point rather than
an oversight, because `complete` and `pending` are opposite claims and only one
of them is falsifiable by looking at the filesystem.

It does carry a consequence worth stating for anything reading the corpus
programmatically: `status: approved` says nothing whatsoever about code. A
consumer that wants to know whether work exists MUST read `implementation`, and
after this spec that field is worth reading. One that reads `status` as a
completeness signal will be wrong about every approved-and-unbuilt spec, and no
gate will contradict it.

**Any change to `status`.** Ratification stays a human act on its own axis.

## 5. Relationship to the design note

`docs/design/02-agentic-builder-substrate.md` §4 (G1) records the gap as
"`implementation: complete` is unverified self-assertion" and sketches a
completion gate requiring "every unit the spec claims resolves and a matching
per-spec attestation is present".

Reading the code rather than the note narrows that. The resolution half already
exists for settled specs via 025, so what was actually missing is one arm of one
predicate, which is this spec. The attestation half is a separate artifact with a
separate purpose (§4, spec 042) and is not a precondition for holding a
completion claim to its territory. The note's own constraint about committed
artifact classes is what makes keeping them apart worthwhile: a severity fix
needs no new committed file and therefore no new freshness gate.
