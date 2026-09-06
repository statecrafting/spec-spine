---
id: lifecycle
title: Lifecycle and Completion
sidebar_position: 8
---

# Lifecycle and Completion

Three frontmatter keys describe where a spec is in its life, and since specs
038, 041, 043 and 044 they are **lifecycle keys with mechanical consequences**,
not descriptive metadata:

| Key | Values | What the tool does with it |
|---|---|---|
| `status` | `draft`, `approved`, `superseded`, `retired` | The ratification state. Drives `registry list --status` and the in-flight predicate below. |
| `implementation` | `pending`, `in-progress`, `complete`, `n-a`, `deferred` (or absent) | How far the work has got. Drives `registry plan` and the in-flight predicate below. |
| `depends_on` | spec ids | Which specs must land first. A cycle is a compile-time refusal (spec 033); the acyclic graph orders `registry plan`. |

## Done is not self-authored

The central rule (spec 041): **a spec that declares itself complete is held to
its claims.** `implementation: complete` is a claim that every owning unit
exists and resolves, and the indexer checks it. If a complete spec owns a file
that is not there, that is a hard `I-0xx` error on `spec-spine index`, whatever
the spec's `status`. Draft leniency does not survive a completion claim.

The opposite side is that openly unfinished work is legal. A spec that is
**in flight** gets a counted, non-blocking `W-001` warning for an owning unit
that does not exist yet (spec 025), so a spec can be filed, and even ratified,
before its code is written.

## The in-flight predicate

| `status` | `implementation` | in flight | unresolved owning unit |
|---|---|---|---|
| draft | pending, in-progress, n-a, deferred, absent | yes | `W-001` warning |
| draft | **complete** | **no** | **`I-0xx` error** |
| approved | pending | yes | `W-001` warning |
| approved | **in-progress** | **yes** (spec 044) | `W-001` warning |
| approved | complete, n-a, deferred, absent | no | `I-0xx` error |

Two things fall out of the table:

- `complete` is decisive on both sides of the `status` flip. Ratifying a
  complete spec changes nothing about how its units are judged.
- `approved` + `in-progress` is in flight (spec 044). A corpus that ratifies
  first and builds second, which is the natural shape for an autonomous builder,
  is not forced to leave its specs as `draft` to stay green.

## The ready set: `registry plan`

`spec-spine registry plan` (spec 038) partitions the registry into what a
scheduler may hand out now and what it may not:

- **Excluded** (absent from the output): `status` is `superseded` or `retired`,
  or `implementation` is `complete`, `n-a` or `deferred`. A spec with no
  `implementation` key counts as `pending`.
- **Blocked**: at least one `depends_on` target is not finished (neither
  `complete` nor `n-a`). Every blocker is named with its state; a target that
  does not resolve to a spec blocks with `state: "unresolved"` rather than being
  ignored.
- **Ready**: everything else, in dependency order.

`deferred` is the one value that records a decision rather than a state, so a
deferred spec is never offered as ready and never silently returns to the
schedule when its dependencies land. Un-deferring is a human edit.

`implementation` is a hint to the scheduler, never evidence of completion. The
evidence is the indexer's verdict, and the record of it is
[`attest --spec`](../cli/attest.md).

## Amending without editing the predecessor

Spec 040 fixes the authoring rule for `amends`: the edge is declared in the
**amending** spec's frontmatter, and only there. The amended `spec.md` is never
edited to record that it has been amended: no back-pointer, no "narrowed by NNN"
note. Discovery is a compiled read (`registry relationships <id>` shows incoming
`amends` edges), not a prose pointer, so the predecessor's source hash stays
stable and its history stays honest. The coupling gate is unchanged: a path
owned by the predecessor still clears through either spec.

The constitution is the case `amends` does not cover (spec 043). It is not a
spec, so an ordinary approved spec changes it by **claiming the affected
constitution text as an authority unit**: `establishes` a `section` unit on the
anchor for a principle it adds, `refines` it (with a named `aspect`) for one it
tightens, `co_authority` where a principle is shared, never contradicting an
`unamendable` anchor of spec 000. The constitution itself is then edited in
place. That claim is a ledger fact, not a gate refusal: `standards/spec/` sits on
the coupling gate's bypass floor, so the edge buys discoverability
(`registry relationships`, `index render`), not enforcement.

## Adopted code is specced as found

Spec 043 gives constitution principle V its operational half. Code adopted from
outside the corpus is specced **as found**: the adopting spec describes the
behavior that exists and records the behavior it would not have chosen under a
`## Known defects` heading, with each defect named. A defect recorded there is
not blessed; it is the reason a later spec can be written against it.
`origin.retroactive: true` still marks *when* the authority began, and is
orthogonal.

The scaffold `spec-spine init` writes carries all of this: the constitution it
generates states the amendment mechanism and principle V in this form, and the
contract points at it, so an adopter's first corpus does not inherit the gaps the
first adopter hit.
