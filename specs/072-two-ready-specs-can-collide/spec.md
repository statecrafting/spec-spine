---
id: "072-two-ready-specs-can-collide"
title: "Two ready specs can collide"
status: approved
kind: "tooling"
created: "2026-09-13"
summary: >
  `registry plan` answers "what can be worked on now", and a scheduler fans
  that answer out. Readiness is computed from `depends_on` alone, so two specs
  with no edge between them are both offered even when they claim the same
  files. The corpus has the case standing today: drafts 084 and 085 both extend
  `crates/spec-spine-cli/src/verify_attestation.rs`, `attest.rs` and `lib.rs`,
  neither depends on the other, and `plan` lists both as ready with no way to
  say so. This spec adds the overlap report: for every pair on the ready set,
  the units both claim. It is a report and never a refusal, and it is a lower
  bound rather than a safety verdict: disjoint territory does not prove
  independence, because a shared lockfile, a regenerated shard or a consumed
  API are collisions no frontmatter declares. `plan` keeps exiting 0.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "035-registry-plan-ready-set"
  - "053-plan-answers-the-whole-question"
  - "063-planned-territory-is-declared-not-inferred"
extends:
  # 3.1 to 3.5: the computation and the `overlaps` field on `Plan`.
  - { spec: "035-registry-plan-ready-set", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  # 3.6: the human rendering, beneath both sets.
  - { spec: "053-plan-answers-the-whole-question", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
  # 3.7: the acceptance.
  - { spec: "035-registry-plan-ready-set", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
---
# 072: Two ready specs can collide

## 1. Purpose

### 1.1 Readiness is computed from one edge

`plan` partitions the corpus with `depends_on` (spec 035). A spec is ready when
every spec it depends on is finished. That is the right question for "may this
start", and it is silent on a second one a scheduler must also answer: "may
these start **together**".

The two are independent. `depends_on` records that one spec's work needs
another's conclusion. Territory records where a spec's work lands. Two specs can
be free of each other in the first sense and land in the same file in the
second.

### 1.2 The case is live in this corpus

Drafts 084 and 085 both carry `extends` edges naming
`crates/spec-spine-cli/src/verify_attestation.rs`,
`crates/spec-spine-core/src/attest.rs` and `crates/spec-spine-core/src/lib.rs`.
Neither depends on the other. `registry plan` offers both, and an orchestrator
reading that answer as a fan-out list would dispatch two sessions into the same
three files. Design note 04 section 4.6 records this and states the gap
plainly: `plan` "has no way to say so".

### 1.3 A report, not a gate

The corpus knows what each spec claims. It does not know what a session will
actually touch, and two disjoint territories still share a lockfile, a
regenerated shard tree, and any API one of them consumes. So the honest output
is the overlap the corpus **can** see, labelled as a lower bound. Reporting a
pair is evidence; reporting none is not a clearance.

This is why the report does not refuse. A refusal would have to mean "these are
safe apart", which is the claim the data cannot support.

## 2. Territory

`Plan` gains an `overlaps` field computed in `query.rs`, rendered by
`cmd_registry.rs` beneath the existing sets, and asserted in `tests/query.rs`.
No shard shape changes: `Plan` is a projection computed on read, never a
committed artifact.

## 3. Behavior

### 3.1 What is compared

Two ready specs overlap when any unit one claims intersects any unit the other
claims. Units are read from the **ownership-bearing** edges only:
`establishes`, `extends`, `refines`, scoped `supersedes`, `co_authority` and
`constrains`. `references` is non-owning (spec 031) and MUST NOT contribute.

A unit marked `planned` (spec 063) counts. A spec that has declared territory it
has not written yet is exactly the spec most likely to collide with one that
has.

### 3.2 How units intersect

- **Path-bearing units** (`file`, `section`, `directory`) compare by path. Two
  paths intersect when they are equal, or when one denotes a subtree containing
  the other. A `file` path with a trailing `/` denotes the subtree rooted at it,
  which is the existing shorthand, and a `directory` unit always does.
  A `section` compares by its file: two anchors in one file are not the same
  bytes, but they are the same file, and this report is a lower bound on
  collision rather than a diff.
- **Identity-bearing units** (`symbol`, `crate`, `module`) compare by exact id.
- **Across the two groups** there is no comparison. Deciding whether a symbol
  lies in a file requires the index's resolution, which `plan` does not read.
  Such a pair MUST NOT be reported, and 3.4's caveat covers it.

### 3.3 What is reported

For each unordered pair of ready specs with a non-empty intersection: the two
ids, and the identity string of every unit on either side that takes part in
the collision. A subtree claim and the file it covers intersect without being
the same identity, so both strings appear: the report names what each spec
declared, not the intersection of the two declarations. Nothing else. The pair
carries no severity, no recommendation and no ordering between the two specs.

### 3.4 The report is a lower bound

The human output states, on the same line as the count, that overlaps are what
the corpus declares and that an absent overlap is not a safety verdict. A
consumer reading `--json` gets the same field with the same meaning; this spec
adds no `safe` boolean, because there is nothing to put in it.

### 3.5 Ordering

`overlaps` is sorted by the first id then the second, each pair's ids ascending,
and each pair's units sorted by identity string. Spec 035 3.2 requires the whole
report to be a pure function of the corpus rather than of a hash-map iteration
order, and this field is part of that document.

### 3.6 Additive

`overlaps` is omitted from `--json` when empty, so a corpus with no overlapping
ready pair emits exactly what it emitted before. The human output prints the
section only when there is one. The ready and blocked sections are unchanged.

### 3.7 The acceptance

`tests/query.rs` gains: a pair that overlaps on a plain file; a pair that
overlaps only through a directory subtree containing the other's file; a pair
whose only shared unit is reached through `references`, which MUST NOT be
reported; a pair claiming the same symbol id; a path unit and a symbol unit,
which MUST NOT be reported; and a blocked spec sharing a ready spec's file,
which MUST NOT be reported because it is not a fan-out candidate.

## 4. Out of scope

**Refusing.** `plan` exits 0 whatever it finds (3.3). A gate that refused a
fan-out would need the safety claim 1.3 says the data cannot support.

**WorkScope as a record.** Design note 04 section 4.6's `WorkScope` carries a
snapshot digest, a closure digest, obligations and declared shared outputs, and
the family plan defers it (revision 4, SP-03) until a named consumer needs it.
This spec ships the one field that answers the question `plan` is already asked,
inside the projection that already answers it. It mints no new record type and
no new schema.

**Resolving symbols.** 3.2's third bullet stays unreported rather than guessing.
Reading the index to resolve a symbol to a file would make `plan` depend on the
index being fresh, which is a different verb's contract.

**Declared shared outputs.** `.derived/`, lockfiles and generated trees collide
between any two specs that regenerate them. Naming them per pair would report
the same three paths on every pair and drown the signal; 3.4's caveat states
the limit instead.

## Verification

Each line is one command. The core suite fails against pre-091 code, where the
`overlaps` field and its tests do not exist.

```verify:cli
cargo test -p spec-spine-core --test query --locked
# 3.1: references is non-owning and cannot contribute an overlap.
grep -qF 'references' specs/072-two-ready-specs-can-collide/spec.md
# 3.3, 3.6: the field exists, is additive, and carries no safety verdict.
grep -qF 'pub overlaps: Vec<Overlap>' crates/spec-spine-core/src/query.rs
grep -qF 'skip_serializing_if = "Vec::is_empty"' crates/spec-spine-core/src/query.rs
! grep -qE 'pub (safe|is_safe|parallel_safe)' crates/spec-spine-core/src/query.rs
# 3.4: the human output states the limit rather than implying clearance.
grep -qF 'lower bound' crates/spec-spine-cli/src/cmd_registry.rs
```
