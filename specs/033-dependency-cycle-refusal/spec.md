---
id: "033-dependency-cycle-refusal"
title: "Dependency cycle refusal: the `V-014` corpus-wide check on `depends_on`"
status: draft
kind: "tooling"
created: "2026-09-03"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "001-compile-registry"
  - "016-short-id-resolution"
extends:
  # The detector, its place in the cross-spec code set, and both call sites
  # (the fresh compile and the committed-shard reader).
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/tests/compile.rs", nature: additive }
summary: >
  `depends_on` is the one edge whose graph shape spec-spine never checked. A
  dangling target is a `V-010` warning, but a cycle passes: two specs that
  depend on each other compile with zero warnings and lint clean at every
  severity, so a corpus can be green and structurally unexecutable. This spec
  adds `V-014`, an error-tier, corpus-wide check that reports a cycle in the
  resolved `depends_on` graph and names the path. It is the sibling of `V-010`
  and lives beside it in `compile.rs`, the first step of the gate chain and the
  step `compile --check` verifies, so a cycle fails the build at the earliest
  point and no new verb is introduced. `V-014` joins
  `CROSS_SPEC_CODES`, so it is never stored in a registry shard and is
  recomputed from the assembled records on read; both the fresh-compile and
  committed-shard paths reach one detector, so the two views cannot disagree.
  Scheduling semantics stay out: readiness, contract pinning and downstream
  invalidation depend on merge history and a run journal, which are not a pure
  function of the corpus and therefore not spec-spine's to compute.
---

# 033: Dependency cycle refusal

## 1. Purpose

Of the eight typed edges, `depends_on` is the only one that describes an
ordering. spec-spine validates that its targets exist (`V-010`, a warning since
spec 001) and nothing else about the shape of the graph they form. A cycle
therefore compiled clean, before this spec:

```
spec-spine compile                             -> compiled 3 spec(s), 0 warning(s)
spec-spine lint --fail-on-warn --fail-on-info  -> 0 error(s), 0 warning(s), 0 info
```

That corpus is green and cannot be executed in dependency order by anything.
The defect is real regardless of who consumes the graph: the edge asserts that
one spec's authority rests on another's, and a cycle asserts that of every spec
on it simultaneously, which is not a claim an author can have meant.

Today the failure surfaces late and elsewhere. A consumer that orders work by
`depends_on` discovers it at scheduling time, in its own process, against a
corpus it does not own; the author who wrote the cycle saw a green gate on the
PR that introduced it. Moving the check here puts the diagnosis at the point of
authorship, in the repository that caused it, on the same run that already
answers every other question about the corpus.

The constitution decides how much of the graph belongs here. Principle 2:
every artifact is a pure function of `(config, file contents)`. Cycle detection
is exactly that. Readiness ("is every dependency shipped?"), contract pinning
and downstream invalidation are not: they need merge history and a run journal,
inputs that no compile can see. That line is the whole scope of this spec.

## 2. Territory

`compile.rs`: the `detect_dependency_cycle` detector, the `V-014` entry in
`CROSS_SPEC_CODES`, and the two call sites that reach it. `tests/compile.rs`:
the coverage in section 3.5. Both are spec 001's units, extended additively.

No new subcommand, no new config key, no signature change, no emitted-artifact
change, and no schema version moves: `V-014` is an ordinary `Violation` in the
`validation` report that already carries `V-003` through `V-013`.

## 3. Behavior

### 3.1 The check

Over the assembled `SpecRecord` set, `V-014` reports a cycle in the directed
graph whose edges are each spec's `depends_on` entries:

```
V-014  [specs/010-a/spec.md] depends_on cycle: 010-a -> 011-b -> 010-a
```

Severity **error**, so it fails validation and the CLI's exit code 1, the same
tier as `V-003`, `V-004` and `V-008`. A cycle has no legitimate reading, so
there is nothing for a warning tier to defer to.

- **Edges leaving the corpus are skipped.** A `depends_on` naming a spec that
  does not exist is `V-010`'s finding; a dangling target is never reported as
  a cycle, and never suppresses one elsewhere.
- **A self-dependency is a cycle** of length one, reported as `010-a ->
  010-a`.
- **Short ids resolve first.** The detector reads the records, not the parsed
  frontmatter, so a cycle written with spec 016's short form (`depends_on:
  ["011"]`) is seen in its resolved form and reported with full ids.
- **The path is the loop, not the walk.** When a cycle is reached through
  specs that are not on it, only the loop is named: for `010 -> 011 -> 012 ->
  013 -> 011`, the message is `011-b -> 012-c -> 013-d -> 011-b`. The
  violation's `path` is the `spec.md` of the first spec on that loop.
- **One cycle per compile.** The first cycle found is reported and the walk
  stops. A corpus can hold several, and enumerating all elementary cycles is
  worth neither the cost nor the wall of output; breaking the reported one is
  what reveals the next.

### 3.2 Cross-spec, so never sharded

`V-014` joins `CROSS_SPEC_CODES`. A cycle is a property of the record set, not
of any one spec, so storing it in a shard would let a sibling spec's PR stale
that shard, which is precisely what spec 024's sharding exists to prevent. It
is therefore recomputed from the assembled records by
`load_committed_registry`, as `V-003`, `V-004`, `V-008` and `V-010` already
are.

The fresh-compile path and the committed-shard path call **one** detector.
The four older cross-spec codes are duplicated between the two and kept equal
by value; `V-014` is kept equal by linkage instead, which is the stronger
guarantee and the shape a future consolidation should take.

### 3.3 Determinism

The detector is a pure function of the record set. Start order is the record
order, which both callers sort by id; child order is the authored `depends_on`
order with out-of-corpus targets filtered out. So for a corpus with more than
one cycle, *which* cycle is reported is fixed by the corpus alone, and two
compiles of one tree emit byte-identical JSON. The walk is an iterative DFS
over an explicit stack rather than recursion, so corpus depth cannot overflow
the real stack: core stays panic-free on user input.

### 3.4 Placement: compile, not lint

The two code families are disjoint by design and neither reprints the other's
findings: `compile` emits the structural `V-` codes, `lint` emits the
convention `L-` codes and discards the violations of the compile it runs
internally. That is long-standing behavior and is not specific to this code: a
`V-008` corpus is `exit 1` naming the violation under `compile` and `exit 0`,
silent, under `lint --fail-on-warn`. `V-014` behaves exactly like its siblings,
and putting it in `lint` would have meant a second reporting path for a
structural finding rather than a shared one.

Compile is the right home on all three counts:

- **The question is structural.** `V-010` answers "does this edge's target
  exist?"; "do these edges form a cycle?" is the same question about the same
  edge, one level up. Corpus well-formedness, not corpus convention.
- **The cross-spec machinery only exists here.** Being a property of the
  record set, `V-014` must be excluded from the shards and re-derived on read
  (section 3.2), and `load_committed_registry` is the only place that happens.
- **It fails at the earliest point.** `compile` is step one of the
  `compile -> index -> lint -> couple` chain and the first thing CI calls, and
  `compile --check` reports it too (exit 1, validation failed), so a corpus
  cannot reach the later gates with a cycle in it.

What this does **not** do is make `index`, `lint`, `couple` or the `registry`
queries print the cycle; they exit on their own findings as before. The
validation report is carried on the `Registry` those consumers hold, so a
programmatic reader sees `V-014` without a new API; a CLI reader sees it from
`compile`.

### 3.5 Tests (minimum)

- A two-spec cycle is a `V-014` error naming `010-a -> 011-b -> 010-a`,
  anchored at the first spec's path, with validation failing and no `V-010`.
- A self-dependency is a cycle.
- A cycle reached through a longer chain names only the loop.
- A cycle authored with short ids is reported with resolved ids.
- A dangling `depends_on` is `V-010` and never `V-014`.
- A diamond (two paths to one dependency) is not a cycle.
- With two disjoint cycles, the reported one is identical across runs and the
  emitted JSON is byte-identical.
- `V-014` appears in no shard's local violations, and
  `load_committed_registry` re-derives exactly one, with the same message.

## 4. Out of scope

**Readiness, pinning, and invalidation.** Whether a dependency is *shipped*,
what its contract hashed to when a dependent was built, and which dependents an
amendment invalidates are all functions of merge history and a run journal.
They cannot be computed from `(config, file contents)`, so they belong to the
consumer that owns that state, not to the compiler. A consumer must keep its
own cycle refusal too: it drives corpora compiled by spec-spine versions it
does not choose, and this check is defense in depth for it, not a replacement.

**Ordinal monotonicity.** Requiring every `depends_on` target to be
lower-numbered makes the ordinal the build order, which is a useful convention
for corpora that adopt it and false for ones that do not: a corpus can depend
upward and still be perfectly executable, ordering itself by the graph rather
than by the number. If it lands it is opt-in config, never a default, and it is
a separate spec.

**Cycles in the other edges.** `extends`, `refines`, `co_authority` and
`constrains` form graphs too, and a cycle in them is a different question with
a different answer (a mutual `co_authority` pair is ordinary and correct). This
spec is about `depends_on` only.

**Promoting `V-010`.** Whether a dangling `depends_on` should be an error
rather than a warning is a defensible change and a breaking one for adopters
who rely on the current tier. It is not bundled here.
