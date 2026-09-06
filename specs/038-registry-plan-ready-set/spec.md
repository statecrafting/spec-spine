---
id: "038-registry-plan-ready-set"
title: "`registry plan`: the ready set in dependency order"
status: draft
kind: "tooling"
created: "2026-09-05"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "002-registry-query"
  - "033-dependency-cycle-refusal"
amends:
  # Narrows 033 §4's "readiness ... belongs to the consumer" exclusion to the
  # part of it that is genuinely impure. See §4. 033's own text is untouched.
  - "033-dependency-cycle-refusal"
extends:
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  # The acyclicity this walk relies on, and the design note this is wave 1 of.
  - { unit: { kind: file, path: "specs/033-dependency-cycle-refusal/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
summary: >
  `depends_on` records the order specs must be built in, spec 033 guarantees
  that graph is acyclic, and `implementation` records how far each one has got,
  but nothing joins the three: there is no query that answers "which specs can be
  worked on now, and in what order". Every consumer that needs it (a contributor
  picking up the corpus, a release note, an autonomous driver taking one spec per
  session) recomputes it by reading frontmatter by hand, which is both tedious
  and exactly the ad-hoc traversal the typed read verbs exist to replace. This
  spec adds `registry plan`: a read-only projection that partitions the corpus
  into a `ready` set (nothing unfinished blocks it) emitted in deterministic
  topological order, and a `blocked` set, each entry naming the unfinished
  dependencies that block it, so the output explains itself rather than merely
  listing. It is a scheduling projection and nothing more: `implementation` is a
  self-declared field that no gate currently verifies, so `plan` treats it as a
  hint about intent and never as evidence about code, and this spec deliberately
  does not let a scheduling answer imply that anything is done.
---

# 038: `registry plan`

Wave 1 of `docs/design/02-agentic-builder-substrate.md`.

## 1. Purpose

Three pieces of the answer already exist and are never joined:

| Piece | Where | What it says |
|---|---|---|
| `depends_on` | frontmatter, compiled into the registry | which specs must land first |
| acyclicity | spec 033 refuses a cycle at compile | the graph can be walked |
| `implementation` | frontmatter | how far this spec has got |

What no verb answers is the question anyone actually asks of a corpus: *what can
be worked on now?* A contributor arriving at a 37-spec corpus reads frontmatter
until the picture assembles. A driver taking one spec per session has to do the
same traversal in its own code, against fields whose semantics belong to
spec-spine, which is how a consumer ends up with a second, divergent
understanding of the corpus.

The traversal is small and the data is already compiled. The reason to put it
here rather than leave it to each consumer is the reason `registry relationships`
exists: the graph's meaning is the library's to define.

## 2. Territory

`query.rs` gains the partition and the ordering; `cmd_registry.rs` gains the
subcommand; the facade gains its JSON entry point. No compile-time or index-time
behavior changes, no DTO in the committed artifacts changes, and nothing is
emitted to `.derived`. This is a projection over the committed registry, in the
same class as `registry relationships`.

## 3. Behavior

### 3.1 The partition

For each spec in the registry, exactly one classification:

- **Excluded**, and absent from the output entirely: `status` is `superseded` or
  `retired` (the corpus has moved past it), or `implementation` is `complete`,
  `n-a` or `deferred`. A spec with no `implementation` key at all is treated as
  `pending`, because an unstated intention is the same input to a scheduler as a
  stated intention to start.
- **Blocked**: at least one `depends_on` target is itself not finished, meaning
  its `implementation` is neither `complete` nor `n-a`. The entry names every
  such target and the state each one is in.
- **Ready**: everything else.

`deferred` sits in the excluded set beside `complete` even though the two carry
opposite information about whether work remains, so the reason is worth stating.
`plan` does not ask "is this finished", it asks "should this be scheduled", and
to both `complete` and `deferred` the answer is no. `deferred` is the one value
in the enum that is a **decision** rather than a report: someone looked at the
spec and took it off the schedule. A scheduler that offered it anyway would be
overruling that decision, and one that quietly returned it to `ready` when a
dependency landed would make deferral expire on its own, which is not what
deferring something means. Un-deferring is a human edit to the field, exactly as
deferring was.

Note the asymmetry this creates, which §3.3 is built to survive: a `deferred`
spec is excluded from both output sets while still blocking its dependents. It
therefore appears in the document only as a blocker, and every blocker carries
its `state` so that entry explains itself rather than pointing at nothing.

A `depends_on` target that does not resolve to a spec in the registry MUST block,
and MUST be reported with `state: "unresolved"` rather than silently ignored. A
dangling dependency is a corpus defect, and a scheduler that treats "the blocker
does not exist" as "the blocker is satisfied" would hand out work in an order the
corpus does not sanction.

### 3.2 Ordering is total and deterministic

`ready` is emitted in topological order over `depends_on`, with ties broken by
ascending spec id. `blocked` is emitted in ascending spec id, and each entry's
`blockedBy` follows that spec's own authored `depends_on` order. Both orderings
are part of the contract: the determinism requirement below is a property of the
whole report, so a consumer diffing either array across runs is relying on a
stated guarantee rather than on an implementation's iteration order. The tiebreak is what makes the output a pure function of the
corpus rather than of a hash-map iteration order, which the determinism contract
requires of every output this project produces.

The walk assumes acyclicity and is entitled to: spec 033 refuses a cycle at
compile time with the error-tier `V-014`, so a registry that exists is acyclic.
If the invariant is nonetheless violated (a hand-edited shard, a registry written
by a different tool version), `plan` MUST report the offending set as an error
rather than loop or truncate. That error is `Error::Validation` carrying the
cycle path, which is exit `1` under the standing exit-code contract and is
reported the way every other `registry` subcommand reports an error (§3.3). It
reuses 033's existing classification rather than inventing a second vocabulary
for the same defect, and it means the error branch has a specified shape rather
than being left to the implementer.

### 3.3 Output

Prose (default) lists the ready set in order, then a one-line count of what is
blocked. `--json` emits both sets in full:

```json
{
  "ready": ["014-parser", "021-writer"],
  "blocked": [
    { "id": "030-round-trip", "blockedBy": [
        { "id": "014-parser", "state": "pending" },
        { "id": "021-writer", "state": "in-progress" }
    ] },
    { "id": "031-migration", "blockedBy": [
        { "id": "030-round-trip", "state": "deferred" }
    ] }
  ]
}
```

`blockedBy` is what makes the output diagnostic instead of merely enumerative:
the question after "what can I do now" is always "why not that one".

Each blocker carries its `state`, which is its `implementation` value
(`pending`, `in-progress`, `deferred`) or the literal `unresolved` for a
`depends_on` target absent from the registry (§3.1). It is not decoration, and
it closes a hole a bare id list would leave: a `deferred` spec is excluded from
both output sets (§3.1) while still blocking its dependents (§3.5), so a bare id
would name a blocker the reader can find nowhere else in the document. `state`
makes that entry self-explaining, and it is the one case where the answer to "why
not that one" is not "wait" but "someone decided not to".

`plan` emits this report **bare**, exactly as `registry list`, `show`,
`status_report` and `relationships` do. It does not ride inside spec 037's
verdict envelope: that envelope wraps the adjudicating verbs, and 037 §4 keeps it
off the read verbs deliberately, since wrapping a shipped read surface would
break its consumers. `plan` is a read verb and joins them rather than splitting
the `registry` group into two output shapes.

Consequently 037 is not a dependency of this spec, and `depends_on` does not list
it. The two specs are independent and can land in either order.

### 3.4 `implementation` is a hint, never evidence

`plan` reads `implementation` because it is the only statement of intent the
corpus carries. It is a **self-declared** field: no gate verifies that a spec
marked `complete` has any implemented code, and `lint` does not read it at all.

Therefore `plan` MUST NOT be described, documented, or used as an answer to
whether work is done. It answers what is *claimed*, which is the correct input to
scheduling and the wrong input to acceptance. The completion gate that would make
the field trustworthy is wave 2 of the design note and is deliberately not
assumed here.

This distinction is load-bearing for any consumer that drives work automatically:
a scheduler that reads a claim and an adjudicator that verifies one must stay
different mechanisms, or the claim becomes its own proof.

### 3.5 Tests (minimum)

- A linear chain yields one ready spec and the rest blocked, each naming its
  immediate blocker.
- A diamond yields both middle specs ready, ordered by id, and the join blocked
  by both.
- `complete` and `n-a` dependencies do not block; `deferred`, `pending` and
  `in-progress` ones do.
- `superseded`, `retired`, `complete`, `n-a` and `deferred` specs appear in
  neither set.
- Every `blockedBy` entry carries the blocker's `state`, and it matches that
  spec's `implementation` value.
- A spec blocked solely by a `deferred` spec is reported with that blocker at
  `state: "deferred"`, even though the blocker itself appears in neither set:
  the entry must be readable without a second lookup.
- A missing `implementation` key behaves as `pending`.
- A `depends_on` naming a spec absent from the registry blocks and is reported
  with `state: "unresolved"`.
- Ordering is stable across repeated runs and independent of corpus file order.
- A cycle reaching `plan` (constructed directly, since `compile` refuses one)
  yields `Error::Validation` naming the cycle path, exit `1`, and terminates.
- Against this repo's own corpus the command succeeds and the ready set is a
  subset of the non-complete specs.

## 4. Out of scope

**Everything spec 033 §4 excluded, which is why this `amends` it rather than
contradicting it.** 033 placed "readiness, pinning, and invalidation" outside
spec-spine, reasoning that whether a dependency is *shipped*, what its contract
hashed to when a dependent was built, and which dependents an amendment
invalidates are functions of merge history and a run journal, and so cannot be
computed from `(config, file contents)`. That reasoning is correct and this spec
computes none of those three things.

What `plan` computes is strictly weaker and demonstrably pure: a partition over
fields the corpus already carries, which is a function of file contents and
nothing else. The word "readiness" covers both, which is how the collision
arises, and §3.4 keeps them apart in the only way that matters: this answer is a
**claim** about intent, never evidence about code. Readiness-as-shipped remains
the consumer's, exactly as 033 said. Readiness-as-declared becomes a projection
like `registry relationships`.

The `amends` edge records that narrowing in place. 033's text is unchanged, and
its exclusion still governs everything except the pure partition named here.

**Inferring order from any edge other than `depends_on`.** A spec commonly
`extends` a unit owned by a spec it does not list in `depends_on`; twelve
approved specs in this corpus do exactly that, because the two edges answer
different questions (`extends` is an authority relation, `depends_on` a build
order). `plan` walks `depends_on` and nothing else. Deriving order from `extends`
would silently redefine what `depends_on` means for the whole corpus, which is a
change to the frontmatter grammar and not a scheduling feature.

**Any notion of effort, priority or assignment.** `plan` orders by dependency and
nothing else. A corpus that wants priority has `extra_known_keys` and a consumer
that understands them.

**Enriching the entries.** `ready` carries ids, and `blocked` carries ids plus
`blockedBy`. A consumer that wants each spec's title, status or `implementation`
value issues `registry show`, which is the verb for that. Widening `plan` into a
general projection would duplicate `show` and make the scheduling answer harder
to read, and inlining `implementation` in particular would put a self-declared
field next to a computed one where a reader would reasonably assume both were
adjudicated (§3.4).

**Writing anything.** `plan` is a projection; it emits no artifact and updates no
field. In particular it never advances `implementation`.

**Verifying `implementation`.** §3.4 is explicit that this spec does not do it.
That is the completion gate, filed separately.

**Cross-repo planning.** One registry, one repo, one plan.

## 5. Resolved decisions

- **D-1 (2026-09-06): `plan`'s cycle guard agrees with `compile`'s by
  construction, and that agreement is now tested.** 3.2 says the walk is
  entitled to assume acyclicity because spec 033 refuses a cycle at compile
  time. Review of the implementing PR asked what scopes that refusal.
  `compile::detect_dependency_cycle` receives every compiled record with no
  `status` filter, and `plan` walks every registry entry, so both cover the same
  set and the guard is genuinely unreachable on a registry `compile` accepts.

  Nothing in either function enforces the pairing. Were `compile`'s check ever
  scoped to active specs, `plan` would start refusing corpora `compile` accepts,
  and the divergence would surface as a scheduling failure with no test pointing
  at its cause. `plan_and_compile_agree_on_a_cycle_among_retired_specs` pins it
  on the case where the two would diverge first, converting a comment into a
  checked property. This is the same correction the previous round applied to
  `topological`'s degeneracy: an assumption a doc comment states is worth an
  assertion when a later change elsewhere could quietly invalidate it.

- **D-2 (2026-09-06): `blocked`'s ordering was true but unstated.** `ready`
  carried an explicit ordering promise; `blocked` was ascending id only because
  the walk iterates a `BTreeMap`, so a `--json` consumer diffing it across runs
  would have been relying on an accident. 3.2's determinism requirement is a
  property of the report rather than of one of its two arrays, so the guarantee
  existed and only the sentence stating it was missing. 3.2 now states it and
  `plan_blocked_entries_are_ordered_by_id` asserts it.
