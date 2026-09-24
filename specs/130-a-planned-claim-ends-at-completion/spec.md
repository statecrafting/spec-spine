---
id: "130-a-planned-claim-ends-at-completion"
title: "A planned claim ends at completion"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  Spec 063's `planned: true` lets a spec claim territory it has not written,
  so a corpus gating on `--fail-on-unresolved` can ratify before it builds.
  063 bound the flag to completion only through the lint (`L-011`): a spec at
  `implementation: complete` whose planned unit is absent passed `check`,
  `index check` and both with `--fail-on-unresolved`, exit 0, because the
  index dropped the diagnostic for a planned unit at every lifecycle. A
  consumer whose hooks run `check` alone never saw the contradiction. The flag
  now holds a claim open only until the spec says `complete`; after that the
  absent unit classifies like any settled claim, the natural `I-0xx` error,
  and `check` exits 1. `index coverage` also stops listing a planned unit that
  has resolved as "not yet written". Nothing changes for an unmarked claim or
  for a spec whose work is not complete.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "063-planned-territory-is-declared-not-inferred"
  - "080-an-unresolved-claim-is-not-stale"
# 3.1: 063 3.2 says an unresolved planned unit MUST NOT produce a diagnostic,
# with no lifecycle bound. It now produces the settled-claim error once the
# spec is complete.
amends: ["063-planned-territory-is-declared-not-inferred"]
amends_sections: ["3.2"]
extends:
  # 3.1: the classification.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # 3.3: the coverage list.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: corrective }
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  - { spec: "063-planned-territory-is-declared-not-inferred", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  # 3.1 at the verbs.
  - { spec: "079-a-blocking-claim-is-not-a-stale-shard", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  # 3.4: the adopter guidance.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/specify-first.md", nature: additive }
references:
  - { unit: { kind: file, path: "specs/063-planned-territory-is-declared-not-inferred/spec.md" }, role: context }
intent:
  goal: "a spec may claim unwritten territory until it says its work is complete, and every freshness verb refuses the claim from then on"
  non_goals:
    - "a new marker, edge or grammar: 063's `planned: true` is the authored form (4)"
    - "changing any verdict for an unmarked claim (3.2)"
    - "the exit code `index coverage` and `couple` give an unresolved claim (4)"
---

# 130: A planned claim ends at completion

## 1. Purpose

A consumer that ratifies before it builds needs to approve a spec claiming a
crate that does not exist yet, under a gate that runs
`check --fail-on-unresolved`. Statecraft measured on 0.23.0 that this was
impossible and planned a one-pull-request exception to its own
ratify-before-build rule for its spec 007.

The measurement used an unmarked claim. Spec 063 (0.18.0) already provides the
authored form for exactly this case, `planned: true` on the unit's object
form. Measured on 0.25.0 at `25d46b9f`, with a Statecraft-shaped fixture (a
Cargo workspace, `require_ownership = true`, an approved spec claiming
`crates/statecraft-bundle/` that is not on disk):

| Case | `check --fail-on-unresolved --fail-on-warn` | `lint --fail-on-warn` | `index coverage --fail-on-untraced` |
|---|---|---|---|
| A: `pending`, planned, absent | 0 | 0 | 0, listed as planned |
| B: `pending`, unmarked, absent | 1 (`W-001` refused) | 0 | 0 |
| C: `complete`, planned, absent | **0** | 1 (`L-011`) | 0, listed as planned |
| D: `in-progress`, planned, present | 0 | 1 (`L-012`) | 0, **listed as planned** |
| E: `pending`, `{kind: crate, planned}`, absent | 0 | 0 | 0 |
| F: `pending`, `amends` only, no territory | 0 | 0 (no `L-001`) | 0 |
| G: `complete`, unmarked, absent | 1 (`I-004`) | 0 | 2 |

Cases A, B, E and F are the behavior the consumer asked for, and they already
hold. Two do not:

- **C.** The completion bound lived only in the lint. The index skipped a
  planned unit's diagnostic at every lifecycle, so `check`, `index check` and
  both with `--fail-on-unresolved` reported a finished spec with an absent unit
  as fresh and clean. A consumer whose hooks run `check` without `lint` never
  sees it.
- **D.** `index coverage` lists planned territory under "declared, not yet
  written" from the registry, which cannot see resolution, so a planned unit
  that has landed is listed as unwritten beside the counts that already count
  it as claimed.

## 2. Territory

No new files. The classification is in `index.rs`, owned by 004; the coverage
list in `coverage.rs`, owned by 029, reading an identity function from
`query.rs`, which 063 extended for this list. The verb-level tests sit beside
079's blocking-claim cases in `cli.rs`. The adopter guidance is in
`docs/specify-first.md`, owned by 057.

**This spec `amends` 063 §3.2**, which says an unresolved unit marked
`planned` MUST NOT produce a diagnostic and names no lifecycle bound. 063 §3.3
bounds the flag at completion through `L-011` and explains why `complete` is
the only bound; this spec applies that same bound in the index, so it changes
§3.2's rule and leaves §3.3's reasoning intact.

It does not amend 023 or 080. A planned unit on a complete spec now classifies
by 023's own table (an owning edge on a settled spec is the natural error),
and 080 decides the exit code for that error.

## 3. Behavior

### 3.1 The flag holds a claim open until completion (amends 063 §3.2)

An unresolved unit marked `planned` on a spec whose `implementation` is
`complete` MUST classify exactly as the same unit unmarked: on an owning edge,
the natural `I-0xx` error. `check` and `index check` therefore exit 1 with the
unresolved-claim report 079 and 080 define, with or without
`--fail-on-unresolved`.

The bound MUST key on `implementation: complete` itself, not on "not in
flight". An `approved` spec with no `implementation` key is not in flight
either, and 063 gave it the flag without a bound; this spec does not change
that case.

`L-011` is unchanged. A lint-clean corpus never holds this state, so no shard
of such a corpus changes.

### 3.2 What stays as it was

- A planned unit on a spec that is not `complete` produces no diagnostic
  (063 §3.2), whatever the status.
- An unmarked unresolved claim classifies as 023 requires: `W-001` in flight,
  refused by `--fail-on-unresolved`; the natural error when settled.
- A planned unit that resolves is resolved for every purpose and draws
  `L-012` (063 §3.4).

### 3.3 Coverage lists only what is not yet written

`index coverage`'s planned list MUST omit a planned unit that the committed
index resolves for the same spec. The unit is still counted as claimed, as
063's 2026-09-08 decision on counting requires.

`registry plan`'s planned territory is unchanged: it reads the registry alone
and makes no claim about what is written.

### 3.4 Adopter guidance

`docs/specify-first.md` MUST show the authored form and the verdict for each
combination of completion and presence, so an adopter who ratifies before
building finds the mechanism without reading 063.

## 4. Out of scope

**A new marker.** 063's flag is the authored form the consumer asked for: it is
per claim, typed, refused by a binary that predates it (exit 3 on the unknown
key), and cannot be written on the bare-string shorthand, so a typo cannot
acquire it.

**`index coverage` and `couple` on an unresolved claim.** Both still refuse
with exit 2 and "index is stale" when the committed index records a blocking
diagnostic (case G above), where `check` says it is not staleness and exits 1.
That predates this spec and is the same defect 080 fixed for `check`; it is
recorded as a finding for the exit-contract spec rather than fixed here.

## 5. Resolved decisions

**D-1 (2026-09-24): bound the existing flag rather than add a marker.** The
request was for a per-claim planned marker with four properties. Measured,
063's flag already had three (accepted before completion, typos still caught,
resolved markers reported by `L-012`); the fourth held only in the lint. A
second marker would give the corpus two spellings for one state.

**D-2 (2026-09-24): the resolved-marker report stays a warning.** `L-012` is a
warning, so `lint --fail-on-warn` refuses it and a corpus that does not run
the flag is told. Nothing measured argued for raising it: the claim is
correct, only the annotation is stale.

**D-3 (2026-09-24): an error, not a new code.** At completion the unit is an
ordinary claim on a settled spec, so it gets that claim's code and 079's
report, including the line that names `implementation: complete`. A code of
its own would make adopters branch on a distinction the gate does not draw.

## Verification

```verify:cli
# 3.1: complete ends the planned window; the controls before completion and
# for an absent implementation key still produce nothing; unmarked claims
# keep their codes.
cargo test -p spec-spine-core --test index --locked planned
cargo test -p spec-spine-core --test index --locked an_unmarked_claim_keeps_its_classification
# 3.3: a resolved planned unit is not listed as unwritten.
cargo test -p spec-spine-core --test coverage --locked planned
# 3.1 at the verbs: accepted before completion, refused after, with and
# without the flag.
cargo test -p spec-spine-cli --test cli --locked spec130
```
