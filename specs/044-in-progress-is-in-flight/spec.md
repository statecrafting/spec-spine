---
id: "044-in-progress-is-in-flight"
title: "`implementation: in-progress` is in flight"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "025-unresolved-unit-severity"
  - "041-completion-held-to-claims"
amends:
  # 041 3.1's table states the current behavior for `approved` + `in-progress`
  # and 4 argues it is very likely a defect while deliberately not fixing it.
  # This is the fix it deferred. 041's text is unchanged (spec 040).
  - "041-completion-held-to-claims"
extends:
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
references:
  - { unit: { kind: file, path: "specs/041-completion-held-to-claims/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/025-unresolved-unit-severity/spec.md" }, role: context }
summary: >
  Spec 025 grants an in-flight spec leniency: an unresolved owning unit is a
  counted `W-001` warning rather than a blocking error, because work under way
  may legitimately claim territory that does not exist yet. Its predicate names
  `status: draft` and `implementation: pending` and does not name `in-progress`,
  so a spec whose implementation is openly underway gets hard errors for every
  file it has not written. Spec 041 3.1 wrote that cell down and 4 called it
  very likely a defect, then left it alone on the grounds that one spec should
  not both tighten and loosen the same predicate. This is the spec 4 deferred.
  `in-progress` and `pending` are the same claim about the filesystem, that the
  work is not finished, and only one of them buys the leniency designed for
  exactly that state. The asymmetry has no argument behind it: 025 never
  discussed `in-progress`, which is what makes the omission look accidental
  rather than chosen. It bites a corpus that ratifies a design before building
  it, which lives in `approved` + `in-progress` for the whole build; the rahi
  corpus is in that state today.
---

# 044: `in-progress` is in flight

## 1. Purpose

Spec 025 3.1 arm 2 downgrades an unresolved **owning** unit from a blocking
`I-0xx` error to a counted `W-001` warning while a spec is *in flight*, because
work under way may legitimately claim territory that does not exist yet. Spec
041 made `implementation: complete` defeat that leniency, and left the predicate
otherwise as it found it:

```rust
fn in_flight(&self) -> bool {
    if matches!(self.implementation, Some(Implementation::Complete)) {
        return false;
    }
    self.status == "draft" || matches!(self.implementation, Some(Implementation::Pending))
}
```

`Implementation::InProgress` never enters the expression. An `approved` spec at
`in-progress` is therefore **not** in flight, and every unit it claims but has
not yet written is a hard error.

### 1.1 The two values make the same claim

`pending` says the work has not started. `in-progress` says it has started and
has not finished. Both are statements that the claimed territory is incomplete,
which is precisely the condition 025's leniency exists to accommodate. Granting
it to one and refusing it to the other requires an argument, and 025 does not
make one: it names `draft` and `pending` and never discusses `in-progress` at
all. That silence is what distinguishes this from a deliberate asymmetry.

Compare the case spec 041 *did* argue. `complete` is a claim that the work is
finished, so holding it to its territory tests something the author asserted.
Refusing leniency to `in-progress` tests the opposite of what its author
asserted.

### 1.2 It bites the ratify-then-build corpus, and one exists

This repository files a spec as `draft`, builds it, then ratifies, so nothing
here is ever `approved` + `in-progress` and the defect is invisible locally.
That is not true of every adopter, and spec 041 4 said so before an example was
in hand:

> a corpus that ratifies a design before building it lives in that state for the
> whole build, which is the same class of adopter-flow problem the OAP dry run
> raised against 025 in the first place.

The rahi corpus works that way, and `specs/013-ledger-decision-chain` is
`status: approved` with `implementation: in-progress` while a driven session
builds it. It is the only `in-progress` spec in that corpus. For the whole
duration of such a build, every not-yet-written claimed unit is a blocking
diagnostic, and the gate chain refuses a corpus whose only fault is that the
work is honestly declared as under way.

The severity of that is worth stating precisely rather than overstating. The
window is transient and closes as the files land, and a corpus can route around
it by leaving `implementation` at `pending` until the work is done, which is
what a scheduler would then read as "not started". The cost is that the field
becomes less honest the more carefully an adopter avoids the defect.

## 2. Territory

`index.rs::in_flight` and its acceptance fixtures. No DTO changes, no schema
version moves, and no emitted field is added or removed: this changes which
tier an existing diagnostic lands in, for one combination of two existing
fields, exactly as spec 041 did for a different combination.

## 3. Behavior

### 3.1 The predicate

A spec MUST be treated as in flight when its `implementation` is `in-progress`,
unless it also asserts completion, which cannot both hold.

The table is exhaustive over both enums, as 041 3.1's is, so that the one cell
this spec moves is legible against every cell it does not:

| `status` | `implementation` | in flight | change |
|---|---|---|---|
| draft | pending | yes | unchanged |
| draft | in-progress | yes | unchanged |
| draft | complete | no | unchanged (spec 041) |
| draft | n-a / deferred / absent | yes | unchanged |
| approved | pending | yes | unchanged |
| approved | **in-progress** | **yes** | **this spec** |
| approved | complete | no | unchanged |
| approved | n-a / deferred / absent | no | unchanged |

`n-a` and `deferred` still assert nothing about code existing, so they keep
taking their answer from `status`. An absent key is unchanged.

### 3.2 Why this does not weaken spec 041

041 made a completion claim falsifiable. This makes an incompleteness claim
believed. They are the same rule applied consistently: **the field is read as
what it says.** `complete` says the files exist, so the indexer checks;
`in-progress` says they do not all exist yet, so it does not.

The two specs would conflict only if `in-progress` were treated as evidence of
completeness, and nothing here does that. A spec at `in-progress` whose units
all resolve produces no diagnostic either way; a spec at `in-progress` whose
units do not resolve produces a counted warning naming each one, which is a
report, not a pass.

### 3.3 What is deliberately not added

There is no time limit, no staleness on the flag, and no gate that refuses a
spec for sitting at `in-progress` too long. A lifecycle value that expired on
its own would be a clock in a system contracted to be a pure function of
`(config, file contents)`, and 023 3.2's split is explicit about where the only
wall clock lives.

Nor does this make `implementation` mandatory. An absent key still behaves as
`pending` for this purpose (spec 038 3.1 reads it the same way for scheduling).

## 4. Out of scope

**The `approved` + `pending` combination.** Already in flight, unchanged, and
spec 041 4 explains why it stays that way.

**Verifying that code matches a spec's prose.** No gate does this. Spec 041 3.3
is explicit and this spec adds nothing to it.

**Any change to `status`.** Ratification stays a human act on its own axis, and
the whole point of both this spec and 041 is that the two axes are read
separately.

## 5. Verification

- The `approved` + `in-progress` cell yields `W-001`, not `I-0xx`, and `index`
  exits 0 on it.
- Every other cell of the 3.1 table is unchanged, asserted exhaustively.
- A spec at `in-progress` whose units all resolve produces no diagnostic.
- An unresolved **non-owning** `references` unit stays `W-002` in every
  combination: this touches the lifecycle arm only, never edge authority.
- This repo's committed index is unchanged, since nothing here is
  `approved` + `in-progress`.
