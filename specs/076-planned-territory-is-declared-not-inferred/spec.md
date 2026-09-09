---
id: "076-planned-territory-is-declared-not-inferred"
title: "Planned territory is declared, not inferred"
status: draft
kind: "tooling"
created: "2026-09-08"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "003-conformance-lint"
  - "004-codebase-index"
  - "025-unresolved-unit-severity"
  - "032-ownership-coverage"
  - "038-registry-plan-ready-set"
  - "041-completion-held-to-claims"
  - "050-index-diagnostics-reach-a-gate"
  - "074-shipped-is-not-the-same-as-working"   # allocates L-010; this spec takes the next two
amends:
  # 025 FR-002 requires an unresolved unit from an owning edge, whose owning
  # spec is `draft` or `implementation: pending`, to be emitted as `W-001`.
  # 3.2 below makes a unit marked `planned` a declared state that produces no
  # W-001 at all, which changes what FR-002 requires.
  - "025-unresolved-unit-severity"
extends:
  - spec: "004-codebase-index"
    nature: additive
    paths:
      - "crates/spec-spine-types/src/unit.rs"        # the optional flag on the unit payload
      - "crates/spec-spine-core/src/index.rs"        # classification and the declared state
      - "crates/spec-spine-types/tests/grammar.rs"   # round-trip and rejection
      - "crates/spec-spine-core/tests/index.rs"
  - spec: "001-compile-registry"
    nature: additive
    paths:
      - "crates/spec-spine-types/src/version.rs"     # REGISTRY_SCHEMA_VERSION 1.1.0 -> 1.2.0
      - "crates/spec-spine-types/tests/dtos.rs"
  - spec: "003-conformance-lint"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/lint.rs"         # L-011 and L-012
      - "crates/spec-spine-core/tests/lint.rs"
  - spec: "032-ownership-coverage"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/coverage.rs"     # planned territory in the report
  - spec: "038-registry-plan-ready-set"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/query.rs"        # planned territory in the plan
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  A spec cannot say what it intends to own. Ownership is only ever inferred
  from a unit resolving on disk, so a corpus that opts into spec 050's
  `--fail-on-unresolved` gate forecloses declaring territory before writing it:
  every claim must arrive with its file. The cost is that `registry plan` and
  `index coverage` are blind to planned ownership, a draft's risk understates
  the work, and two specs can plan the same file with no edge to collide on
  until one of them ships. This spec adds an optional `planned` flag to the
  unit payload. A planned unit is a declared state rather than an unresolved
  one, so it produces no `W-001` and the gate keeps refusing every claim that
  is merely broken. The flag is bound to the lifecycle field that ends it:
  a spec at `implementation: complete` carrying one is refused, and a planned
  unit that has resolved is reported so the flag cannot be left on.
---

# 076: Planned territory is declared, not inferred

## 1. Purpose

Let a spec say what it intends to own, without weakening the gate that refuses
a claim which is simply wrong.

Today ownership is inferred from resolution. A unit that does not resolve is an
unresolved unit, and spec 025 grades it by lifecycle: a warning while the
owning spec is in flight, an error once it is not. Spec 050 then offers
`--fail-on-unresolved`, which refuses any `W-001` or `W-002`, for a corpus past
the specify-first stage. This repository opted in and CI runs it.

The consequence went unnoticed until it bit. **A corpus that sets that flag
cannot declare territory before writing it at all.** Filing spec 075 with
`establishes: crates/spec-spine-cli/src/cmd_check.rs` refused the gate on a
`W-001`, and the claim had to be removed and deferred to the implementing
change. Specs 072 to 075 are `extends`-only, and that is not a property of
those changes; it is the gate shaping the corpus.

What is lost is not cosmetic:

- `spec-spine registry plan` and `spec-spine index coverage` say nothing about
  planned ownership, so the reads that exist to answer "what is being worked
  on and who will own it" can only see the past.
- A draft's `risk` understates the work, because the territory it will take is
  not written down anywhere a tool can read.
- Two drafts can plan the same file with no edge to collide on. The duplicate
  ownership surfaces only when the second one ships, which is the latest and
  most expensive moment to find it.

The obvious fix is to make the gate lifecycle-aware and let a `draft` carry
unresolved units. That fix is wrong, for a reason specific to this being a
tool other people adopt: it ties "not written yet" to `status: draft`, which
only works in a repository that drafts then builds. An adopter who ratifies
first has `approved` specs planning territory, and the gate would have to
special-case them. It would also make `--fail-on-unresolved` a near no-op,
since a settled spec's unresolved unit is already a hard error that plain
`index check` refuses; refusing `W-001` is the flag's entire marginal value.

A declared field is loop-agnostic and keeps that value. A typo'd path stays a
`W-001` and stays refused, because nobody marked it planned.

## 2. Territory

No new files. Everything is an additive `extends` except the amendment below.

**This spec `amends` 025.** Its FR-002 requires an unresolved unit from an
owning edge, on a spec that is `draft` or `implementation: pending`, to be
emitted as `W-001`. Section 3.2 makes a unit marked `planned` produce no
diagnostic at all, which changes what FR-002 requires.

**It does not amend 000, and that was the read that decided this spec's shape.**
The unit vocabulary looked like it might sit under a non-overridable anchor.
It does not: spec 000 declares `*(anchor: typed-authority-graph)*` explicitly
on section 4's opening paragraph, which fixes the principle (a spec declares
typed edges and the authority units it owns; authority is derived by walking
the graph, never declared directly) and says nothing about the unit payload.
Section 4.2 records the opposite in spec 000's own words: "The schema is
permissive on the unit payload, so `directory`/`crate`/`module` were added as
an additive minor with no schema-file edit." Spec 017, which added those three
kinds, declared no `amends`. A `planned` flag annotates a claim's resolution
state; authority still derives from the edge, so the anchored sentence stands.

**It does not amend 050.** Section 3.2 of that spec is phrased over diagnostic
codes: the flag "MUST exit 1 when the committed index records any `W-001` or
`W-002`". That rule is unchanged here. What changes is what produces a `W-001`.
The mechanism composes with the gate rather than carving an exception into it,
and that is the property that makes it safe.

**It does not amend 041**, which requires that a spec not be treated as in
flight once `implementation` is `complete`. Section 3.3 extends that principle
to the new field rather than contradicting it.

**Coupling falls out for free, and the spec relies on it.** Dropping a
`planned` flag is an edit to `spec.md`, which is exactly the spec-side change
the coupling gate demands when the claimed file lands. So the implementing PR
satisfies `couple` by doing the thing this design already requires, with no
waiver and no special case. That is why this is a mechanism rather than a rule:
it composes with the gate that was already there.

## 3. Behavior

### 3.1 The flag

A unit MAY carry `planned: true`, meaning the spec claims this territory and
has not written it yet.

The flag is valid only in the **object form** of a unit. The bare-string file
shorthand (`"src/thing.rs"`) cannot carry it, and the shorthand grammar MUST
NOT be extended to allow it: a planned claim is written as
`{ kind: file, path: "src/thing.rs", planned: true }`. The friction is
accepted deliberately, because the alternative is a second string grammar to
parse and a second place for the flag to be misspelled silently.

`planned: false` MUST be accepted and MUST mean exactly what omitting it means,
so a consumer that round-trips a unit does not change its meaning.

The flag is valid on any unit kind, since a spec may plan a section or a
symbol as readily as a file.

### 3.2 A planned unit is a declared state, not a diagnostic (amends 025)

An unresolved unit marked `planned` MUST NOT produce `W-001`. It is not an
unresolved claim; it is a claim whose subject is openly not yet written.

Every other classification is unchanged. An unresolved unit that is **not**
marked planned still classifies exactly as spec 025 requires: `W-002` from a
non-owning edge, `W-001` from an owning edge on an in-flight spec, and a hard
`I-003`..`I-009` error otherwise. A path that is simply wrong is therefore
still caught, and still refused by `--fail-on-unresolved`, which is the
property that makes the flag safe to add.

A planned unit MUST NOT contribute a `ResolvedLocation` or a `TraceMapping`
entry, exactly as an unresolved unit does not. It is a declaration of intent,
not of ownership, and no authority query may resolve to it.

### 3.3 The flag cannot outlive the work

A spec whose `implementation` is `complete` while any of its units carries
`planned: true` MUST be refused by the lint, as **`L-011`**, at the **error**
tier. Not a warning: completion asserts the work is done, and a planned unit
asserts it is not, so the two together are a contradiction in the spec's own
frontmatter rather than a gap someone might hold deliberately.

This binds the flag to the lifecycle field that already exists, which is what
keeps it from becoming a state nobody owns the exit from. Spec 041 established
that `complete` ends the in-flight window; this extends the same principle to
the new field.

**`complete` is the only bound, deliberately.** A spec that is `approved` and
`implementation: pending` may carry planned units for as long as that state is
honest, which on a specify-first corpus is months. Adding an intermediate
deadline would mean inventing a clock, and every clock this corpus has
considered has been refused for the same reason: it makes an artifact depend on
when it was built rather than on what it contains. The exit is owned by the
lifecycle field, and `L-012` covers the other direction by catching a flag that
has become false in fact.

### 3.4 A planned unit that has resolved is reported

When a unit marked `planned` **does** resolve on disk, the lint MUST emit
**`L-012`** at the **warning** tier, naming the unit and saying to drop the
flag.

Without this the flag rots in the other direction: the file lands, the claim is
satisfied, and nothing notices that the spec still describes it as future work.
Warning tier means `lint --fail-on-warn` refuses it, so a corpus running the
gate is told at the first opportunity rather than at completion.

A resolved planned unit MUST still be treated as resolved for every other
purpose: it contributes its `TraceMapping`, it participates in ownership, and
it is hashed like any other resolved unit. `L-012` is a signal about the
frontmatter, not a downgrade of the claim.

### 3.5 Collisions reuse the ownership rules

Planned territory is subject to the ownership rules that already exist; this
spec introduces no second set.

- Two specs planning the same unit is the **duplicate-ownership refusal**, with
  the same `co_authority` escape as any other shared claim.
- A spec planning a unit another spec **already owns** MUST be refused. The
  correct declaration there is an `extends` edge naming that spec and unit,
  which is what crossing into owned territory has always meant.
- A planned unit MUST NOT appear as the target of an `extends` edge. There is
  nothing to extend: the unit does not exist, and its planner does not yet own
  it.

**These checks run over declared units at compile time, not over the resolved
graph.** Naming the pipeline stage matters here, because the obvious reading is
wrong: the existing duplicate-ownership machinery operates on `TraceMapping`,
and 3.2 excludes a planned unit from ever producing one. A collision between
two planned claims would therefore have nothing to ride on, and a rule that
cannot fire is worse than no rule, because the spec would promise a refusal
that never happens.

`compile` sees every declared edge and unit before any resolution, which is
exactly the view these three rules need: two specs declaring the same planned
unit, a planned unit whose path another spec already owns, and an `extends`
naming a planned unit are all decidable from the corpus frontmatter alone. The
refusals are therefore compile-time validation errors in the `V-` band, not
index diagnostics, and they fire whether or not the path exists on disk.

### 3.6 The ledger records planned territory as a state

A planned unit MUST be recorded in the emitted artifacts as a **declared
state**, not as a suppressed diagnostic. A consumer MUST be able to ask what a
spec plans to own without reading the diagnostics band.

`spec-spine registry plan` MUST be able to report planned territory alongside
the ready set, and `spec-spine index coverage` MUST be able to distinguish a
source file that no spec claims from one that a spec has planned. Both are the
reads this spec exists to unblind, and both become possible only because this
section puts the fact in the artifact rather than leaving it in a warning that
3.2 has already suppressed.

### 3.7 Schema and version

The unit DTO gains one optional boolean. `REGISTRY_SCHEMA_VERSION` MUST bump
**MINOR, `1.1.0` to `1.2.0`**, following the precedent spec 028 set when the
`Provenance` struct gained an optional field: additive, no MAJOR, loaders that
know `1.x` keep working.

`Config` and the DTOs derive `deny_unknown_fields`, so a binary predating this
spec meets a `planned` key with a parse error and exits **3**, rather than
silently ignoring a claim about territory. That is the fail-closed direction
and it is the reason the flag is a typed field rather than a convention in a
comment.

Emitting the field where it is absent MUST NOT change existing output:
`planned` is serialized only when true, so every shard of a corpus that uses no
planned units is byte-identical across this change and no re-index is needed.

**A written `planned: false` MUST normalize to absent.** Section 3.1 accepts it
on input and gives it the meaning of omission, so a tool that read it and wrote
it back verbatim would emit a shard that differs from the one the same corpus
compiles to from scratch. That is a determinism break of exactly the kind the
four-triple gate exists to catch, and it would surface as an inexplicable
staleness rather than as a bug in the round-trip. Normalizing on write keeps
one canonical form for one meaning, which is the rule the rest of the emitter
already follows.

## 4. Out of scope

**Making `--fail-on-unresolved` lifecycle-aware.** Considered first and
rejected in section 1: it ties the exemption to `status: draft`, which only
works in a draft-then-build repository, and it would reduce the flag to a
near no-op since a settled spec's unresolved unit is already a hard error.

**A `plans:` edge.** A ninth edge type would put the intent in the graph rather
than on the unit, and the graph's eight types are enumerated in spec 000
section 4.1 under the tier-1 anchor's own section. A field on the payload
achieves the same visibility without touching the edge vocabulary, and section
4.2 already establishes that the payload is the extensible part.

**A config allowlist of planned paths.** `[lint] unresolved_allowed`, mirroring
`unwitnessed_allowed`, would make the gap explicit but would keep it outside
the edge graph, so two specs planning one file would still not collide. The
whole value of putting the flag on the unit is that the existing ownership
machinery does the collision detection.

**Planning a unit in another repository, or a unit kind that does not exist
yet.** The flag says a claim's subject is not written; it does not relax any
other validation. A malformed unit is still malformed.

**Retrofitting the corpus.** Specs 072 to 075 deferred their claims to their
implementing changes and that remains correct for them. Nothing here requires
an existing spec to be rewritten to use the flag.

## 5. Resolved decisions

**2026-09-08: a declared field, not a lifecycle-aware gate.** The gate fix was
proposed first and is the smaller change. It fails on adopters: it works only
where drafts precede builds, and an adopter who ratifies first has `approved`
specs planning territory that the rule would have to special-case. It also
guts the flag it modifies. The field is loop-agnostic, and it preserves the
distinction that actually matters, which is between a claim that is early and
a claim that is wrong.

**2026-09-08: no `amends` on 000, checked rather than predicted.** The unit
vocabulary looked like tier-1 territory. The anchor is declared explicitly on
section 4's principle and does not reach the payload, section 4.2 says the
payload is permissive, and spec 017 extended the vocabulary with no amendment.
This is the fourth consecutive spec where reading the owning spec's words gave
a different edge than the size of the change suggested, and the second where
the answer was lighter rather than heavier.

**2026-09-08: no `amends` on 050, because its rule is phrased over codes.**
The gate refuses any `W-001` or `W-002`. This spec does not exempt anything
from that rule; it changes what produces a `W-001`. Had 050 been written over
"unresolved units" in the abstract, this spec would have owed it an amendment,
and the difference is one sentence of drafting in a spec written months
earlier.

**2026-09-08: the shorthand stays as it is.** A planned claim must use the
object form, so `"src/thing.rs"` cannot be marked planned. Extending the bare
string to carry a flag would mean a second grammar, parsed in a second place,
where a misspelling degrades silently into a path. The friction is one pair of
braces and it buys a flag that is either typed correctly or refused.

**2026-09-08: `L-011` is an error and `L-012` a warning.** Completion beside a
planned unit is a contradiction inside one file's frontmatter and cannot be a
state a corpus holds deliberately, so it refuses. A planned unit that has
resolved is a stale annotation on correct work, which is a nudge; warning tier
means `--fail-on-warn` still refuses it in a corpus that runs the gate, which
is the behavior this repository wants without imposing it on one that does not.
`L-010` is allocated by spec 074, which is why this spec takes the next two and
depends on it.

## Verification

Each line below is one command: spec 049 3.2 makes a fence's body line a
command, so no line may depend on a variable another line set.

Every assertion fails against pre-076 code. The decisive case is the one that
motivated the spec: a draft that declares territory it has not written passes
`index check --fail-on-unresolved`, while a draft with a typo'd path still
fails it.

```verify:cli
cargo build --release --locked
# 3.1 and 3.7 the flag round-trips, and the shorthand still refuses it.
cargo test -p spec-spine-types --test grammar --locked
# 3.7 the registry schema bumped MINOR and the DTOs still conform.
cargo test -p spec-spine-types --test dtos --locked
cargo test -p spec-spine-core --test conformance --locked
# 3.2 planned suppresses W-001; an unmarked bad path still produces one.
cargo test -p spec-spine-core --test index --locked
# 3.3 and 3.4 the flag cannot outlive the work, in either direction.
cargo test -p spec-spine-core --test lint --locked
# 3.6 the plan and the coverage report can see planned territory.
cargo test -p spec-spine-core --test coverage --locked
cargo test -p spec-spine-core --test query --locked
```
