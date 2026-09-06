---
id: "045-absent-implementation-defers-to-status"
title: "An absent `implementation` key takes its answer from `status`"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-init-scaffold"
  - "038-registry-plan-ready-set"
  - "041-completion-held-to-claims"
amends:
  # 038 3.1 reads an absent key as `pending` for every `status`. This narrows
  # that to drafts and reads it as settled on a ratified spec, which is what
  # 041 3.1's table already says the gate does. 038's text is unchanged
  # (spec 040).
  - "038-registry-plan-ready-set"
extends:
  - { spec: "038-registry-plan-ready-set", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  - { spec: "038-registry-plan-ready-set", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
references:
  - { unit: { kind: file, path: "specs/038-registry-plan-ready-set/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/041-completion-held-to-claims/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/044-in-progress-is-in-flight/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  `implementation` is optional, and two verbs read its absence in opposite
  directions. `registry plan` (spec 038) reads an absent key as `pending` and
  offers the spec as ready; the index's in-flight predicate (specs 041 and
  044) reads `approved` + absent as settled and holds the spec to its claims.
  The one spec that routinely lacks the key is the bootstrap spec `spec-spine
  init` scaffolds, so every scaffolded corpus reports its bootstrap spec as the
  single ready item in an otherwise finished corpus, forever; one adopter does
  today, and this repository did until its own spec 000 was patched by hand.
  This spec fixes the rule rather than the instance: an absent key takes its
  answer from `status`. On a `draft` it reads as `pending`, as 038 intended;
  on anything ratified it reads as settled, as the gate already concludes. The
  scaffold's bootstrap spec declares `implementation: n-a` and the spec
  template states the key instead of commenting it out, so the absent case
  becomes rare as well as consistent.
---

# 045: An absent `implementation` key takes its answer from `status`

## 1. Purpose

### 1.1 Two verbs, one key, two answers

Spec 038 3.1 partitions the corpus for scheduling and says of the key:

> A spec with no `implementation` key at all is treated as `pending`, because an
> unstated intention is the same input to a scheduler as a stated intention to
> start.

Spec 041 3.1 writes the in-flight predicate down as an exhaustive table, and its
last row is `approved` + `n-a / deferred / absent` with in flight `no`: an
`approved` spec without the key is held to every unit it claims, exactly as a
`complete` one is. `index.rs::in_flight` agrees with the table. Spec 044 3.1
repeats the row unchanged.

So for the same spec, `approved` and silent about its implementation, the gate
says "settled, every claim must resolve" and the scheduler says "not started,
offer it". Both readings are defensible alone. Together they mean a corpus can
be fully built, fully green, and still have `registry plan` hand out work.

The prose beneath each table adds a second, smaller inconsistency: 041 3.5 and
044 3.3 both say "an absent key still behaves as `pending` for this purpose",
which contradicts the row directly above them. The tables and the code are
right; the sentences are slips. Under spec 040 this spec does not edit 041 or
044 to fix them; it records the correct reading here and the tables stay the
authoritative statement.

### 1.2 The instance that made it visible

The spec that routinely lacks the key is the one `spec-spine init` writes.
`scaffold.rs::bootstrap_spec` emitted `status: approved` and no
`implementation`, so in every scaffolded corpus:

```
$ spec-spine registry plan
000-bootstrap
ready: 1, blocked: 0
```

That is the claude-observatory corpus today: forty specs `complete` or `n-a`,
and the bootstrap spec offered as the one thing to do. This repository had the
same defect until #102 added `implementation: complete` to its own spec 000 by
hand, which fixed one corpus and left the rule that produced it in place.

The adopter audit recorded in `docs/design/03-adopter-audit-2026-09.md` found
that every adopter but one had already routed around the ambiguity by declaring
the key on every spec (their templates promote it to required), which is the
right instinct and also the reason the defect stayed invisible for as long as
it did.

## 2. Territory

`query.rs::schedulable` and `query.rs::blocker_state` (spec 038's), their tests,
and the scaffold's bootstrap spec and spec template (spec 006's). No DTO
changes, no schema version moves, no emitted field is added or removed. The
scaffold change alters two generated files' contents; `Scaffold` itself is
unchanged.

## 3. Behavior

### 3.1 The rule

An absent `implementation` key MUST be read as the value `status` implies for
it, not as a fixed value:

| `status` | absent `implementation` reads as | `plan` offers it | as a `depends_on` target |
|---|---|---|---|
| draft | `pending` | yes (unless blocked) | blocks, reported as `pending` |
| approved | settled | no | finished |
| superseded / retired | excluded (unchanged, 038 3.1) | no | finished |

"Settled" means what 041 3.1 means by it for the same row: the spec makes no
claim that work remains, so it is not scheduled and does not block. It is not
`complete`, and nothing here says so; `blocker_state` reports no state for it
because there is no state to report.

The `draft` row is spec 038's original reading, kept where it was correct. A
draft is by definition unratified work, and an author who has not yet said how
far it has got has told a scheduler the same thing `pending` would.

### 3.2 Why `status` is the right fallback

The two fields are orthogonal axes (041 3.2), and reading an absent value on
one axis off the other is exactly what 041 and 044 already do for `n-a` and
`deferred`: "they keep taking their answer from `status`, as does an absent
key." This spec makes `plan` do what the index does, so that the verb that
schedules and the verb that adjudicates agree about which specs have anything
left to say about their code. They remain different mechanisms (038 3.4 still
holds: `implementation` is a hint, never evidence); they merely stop reading
the same input in opposite directions.

The alternative, making `implementation` mandatory, is a frontmatter-grammar
change with an adopter migration attached, and 041 4 already declined it for
the same reason. Reading absence off `status` costs no adopter anything.

### 3.3 The scaffold

`scaffold.rs::bootstrap_spec` MUST emit `implementation: n-a` with a comment
saying why: the bootstrap spec defines what a spec is and owns no code, so
there is nothing to implement. `n-a` is the value the corpus vocabulary already
provides for a record spec, and three of the four audited adopters chose it for
their own bootstrap spec by hand. (`complete` would also keep `plan` quiet and
is what this repository's spec 000 says, because spec 000 does own the types
crate through the manifest floor; the scaffold's does not.)

`scaffold.rs::spec_template` MUST state `implementation: pending` as a live
key with its value enumeration in the trailing comment, not as a commented-out
optional. Every adopter that customised the template promoted the key to
required; the default should not need the promotion.

`plan` over a freshly scaffolded corpus is therefore empty: nothing ready,
nothing blocked.

### 3.4 What this does not change

- Every cell of 041 3.1 and 044 3.1. The index's predicate is not touched;
  this spec brings `plan` to it.
- `approved` + `pending`, which stays schedulable and stays in flight. The
  specify-first corpora live there for months and rely on it.
- The `deferred` reading (038 3.1's argument that deferral is a decision, not
  a report).
- The V-014 cycle refusal and the topological order.

## 4. Out of scope

**Fixing the prose slips in 041 3.5 and 044 3.3.** Spec 040: an amendment
never edits the spec it amends. The tables are correct; this spec is the record
that the sentences are not.

**A lint for a missing key.** Not needed once absence is consistent, and the
template change makes the absent case rare in new corpora.

**Any change to `status`.** Ratification stays a human act on its own axis.

## 5. Verification

- `draft` + absent: offered when unblocked; blocks a dependent, reported as
  `pending`.
- `approved` + absent: never offered; a dependent on it is not blocked by it;
  a corpus of nothing but ratified silent specs plans to nothing ready and
  nothing blocked.
- Every other cell of the 038 partition is unchanged, including the five
  explicit values and `superseded`/`retired` exclusion.
- A freshly scaffolded corpus compiles, lints clean, and plans to an empty
  ready set; its bootstrap spec carries `implementation: n-a` and its template
  states `implementation: pending` uncommented.
- This repository's committed registry and index are unchanged: every spec here
  declares the key.
