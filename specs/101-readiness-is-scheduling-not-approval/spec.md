---
id: "101-readiness-is-scheduling-not-approval"
title: "Readiness is scheduling, not approval"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: pending
owner: "The spec-spine Authors"
depends_on:
  - "035-registry-plan-ready-set"
summary: >
  `registry plan` partitions by `implementation` and `depends_on` and never by
  `status`, so a draft appears on the ready set by design. The layering is
  deliberate and stays; what is missing is a governed sentence saying so, which
  a consumer reading the document directly has no way to infer.
extends:
  - spec: "057-the-docs-name-what-adopters-derived"
    unit: { kind: file, path: "docs/api.md" }
    nature: additive
  # 3.3: the claimed file has to be hashed, and the hashed-input list lives in
  # the configuration, which specs 061 and 092 own. Crossing into their
  # territory is declared here rather than waived.
  - spec: "061-shipped-is-not-the-same-as-working"
    unit: { kind: file, path: "spec-spine.toml" }
    nature: additive
references:
  - unit: { kind: file, path: "crates/spec-spine-core/src/query.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 101: Readiness is scheduling, not approval

## 1. Purpose

`spec-spine registry plan` reports a `status: draft` spec on its ready set. A
consumer reading that document directly can reasonably conclude the spec is
ready to be built. It is not: approval is a human act and nothing in the
document says so.

This was reported from an adopter (aicortex, 2026-09-12 with `0.18.0`,
re-reproduced 2026-09-15 with `0.19.0`) and recorded in
`docs/design/05-remaining-waves-2026-09.md` §9.3 as **N2**. Its disposition
there is that the layering is intentional and not reopened, and that what is
owed is guidance. This spec is that guidance, written down where a consumer
reads rather than in a design note they never see.

### 1.1 The layering is deliberate and is not reopened

Two approved places already fix it, and this spec changes neither:

- spec 035 §3.1 partitions the corpus by `status: superseded | retired` and by
  `implementation`. Approval is not a partition key.
- spec 093 puts the approval rule in the `/next` skill, on top of `plan`: a
  draft is dropped from the ready set and reported as awaiting approval.

Separating "what the dependency graph permits" from "what a human has blessed"
is the right layering. A repository whose cadence is file-draft, build,
ratify (this one's) and a repository whose cadence is ratify-then-build both
read the same document and apply their own rule. The defect is not the
partition. The defect is that the contract is truthful only to a reader who has
also read the harness.

### 1.2 What is wrong today, precisely

`docs/api.md` describes `plan`'s document by its shape:

> `plan` (spec 035) returns `{ "ready": [...], "blocked": [{ "id", "blockedBy":
> [{ "id", "state" }] }], ..., "schemaVersion" }`.

Every word is accurate and none of it says what membership of `ready` means. A
consumer that treats the set as a work queue is reading the documentation
correctly and reaching a false conclusion.

## 2. Territory

One sentence of governed documentation: `docs/api.md`'s description of the
`plan` read document.

It claims no code. `crates/spec-spine-core/src/query.rs` is carried as a
`references` unit, which is non-owning (spec 031): the behavior this spec
describes is spec 035's and is not changed here.

## 3. Behavior

### 3.1 The documentation states what membership means

`docs/api.md`'s description of `plan` MUST state all three of:

1. **`ready` is a scheduling answer.** It means every `depends_on` target is
   satisfied and the spec is itself schedulable. It is not an approval, not a
   permission to execute, and not a claim that a human has read the spec.
2. **Approval is not a partition key.** `status` is not consulted except to
   exclude `superseded` and `retired` (spec 035 §3.1), so a `status: draft`
   spec appears on `ready` whenever its dependencies are met, by design and not
   by defect.
3. **The approval rule belongs to the consumer.** A consumer that requires
   approval applies it on top of `plan`, as this repository's own `/next` does
   (spec 093).

### 3.2 It is a documentation requirement and nothing else

This spec MUST NOT change `plan`'s output, its partitioning, its ordering, its
schema version, or any exit code. No emitted byte moves.

The reason is stated so a later reader does not mistake restraint for oversight:
a behavior change to make the document self-describing is a real option, it is
**spec 102**, and it is a separately versioned enhancement rather than the
correction of this defect. Making the contract truthful does not require it.
Conflating the two would let a schema bump ride into the corpus as a bug fix.

### 3.3 The claimed file is hashed

`docs/api.md` is claimed by this spec, so `L-008` requires it to be inside some
content hash, and `lint --fail-on-warn` is in the gate. It is added to
`[index] extra_hashed_inputs` in the same change, which means this spec also
crosses into `spec-spine.toml`, owned by specs 061 and 092. That crossing is
declared as an `extends` edge in this spec's own frontmatter, which is the
route spec 005 offers and the one that needs no waiver.

Adding the glob restales every shard, which is a regeneration and not a problem.
It is the cost the rule exists to impose, and a governed sentence whose bytes
can change without staling the ledger is exactly the sentence that should not
be.

## 4. Out of scope

- **Changing `plan`'s behavior**, including adding any field. See §3.2 and spec
  102.
- **A lint refusing `status: draft` with `implementation: complete`.** The same
  adopter report raised it; note 05 §9.3's disposition is that this repository's
  cadence produces that pair normally, so a default refusal would refuse this
  corpus. A configurable lint for a ratify-then-build adopter is a different
  spec with its own decision.
- **The `/next` skill.** Its approval rule is spec 093's and is already correct.

## 5. Resolved decisions

*(Filed as a draft. Decisions taken during the build are appended here.)*

## Verification

Written to fail against the tree this spec is filed on: none of the three
sentences is present today, and `docs/api.md` is not a hashed input.

```verify:cli
# 3.1.1: membership is named as a scheduling fact.
grep -qi 'scheduling' docs/api.md
# 3.1.2: the document says approval is not a partition key.
grep -qi 'not an approval' docs/api.md
# 3.1.3: the consumer-side rule is named, with the skill that applies it.
grep -qi 'on top of' docs/api.md
# 3.3: the claimed file is inside a content hash.
grep -q '"docs/api.md"' spec-spine.toml
# 3.2: nothing about the emitted document moved. `plan` still carries exactly
# the two members it carried, and `ReadySpec` still has exactly its two fields.
grep -q 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs
./target/release/spec-spine registry plan --json
# The governed loop, including the L-008 tier this spec's claim would otherwise
# trip.
./target/release/spec-spine lint --fail-on-warn
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
