# spec-spine constitution

Durable principles that govern this corpus. This document is **tier 2**: it is
subordinate to the bootstrap spec (`specs/000-spec-spine-bootstrap/spec.md`),
whose `unamendable` anchors it may not contradict, and it governs all ordinary
specs (`001`+).

**Normative hierarchy (highest wins):**

1. `specs/000-spec-spine-bootstrap/spec.md`: the bootstrap spec. Non-overridable.
2. `standards/spec/constitution.md`: this document.
3. `standards/spec/contract.md`: a normative summary of the bootstrap spec.
4. Ordinary specs (`001`+): feature-level claims within this envelope.

When two specs conflict, resolve in this order, then by the typed authority graph.

---

## I. Markdown-only authored truth

Authored truth lives only in markdown with YAML frontmatter. There is no
authoritative hand-authored JSON or YAML data file. If a fact governs the system,
it is written in a `spec.md` (or a `standards/` document), never in a derived
artifact. *(Bootstrap anchor: `markdown-truth-boundary`.)*

## II. Compiler-owned JSON machine truth

All machine-consumable truth is emitted by the compiler/indexer into the derived
output tree, and is read only through a typed consumer (the `spec-spine` binary
or the `spec-spine-core` library). Hand-editing a derived artifact is a workflow
violation, and ad-hoc parsing of one (`jq`/`awk`/`sed`) is equally forbidden:
typed reads make schema drift fail at the deserializer, with a clean error,
instead of silently somewhere downstream. *(Bootstrap anchors: `json-truth-boundary`.)*

## III. Spec-first development

A change to behavior begins with a change to a spec. The spec defines the
territory (the units it owns) and the relationships (the typed edges) before the
code is written. The coupling gate enforces this at PR time: a claimed code unit
that changes without its owning `spec.md` changing, or vice versa, refuses the
merge. The escape valve is a named, scoped waiver recorded in the PR body, never
a silent edit to an owner spec.

## IV. Determinism and validation

Every artifact-producing function is a pure function of `(config, file
contents)`; the same inputs produce byte-identical output. Validation is
mechanical: the compiler reports violations and sets a pass/fail flag; the lint
reports conformance warnings; the coupling gate reports drift. No artifact
carries an ambient clock or environment read except the excluded `builtAt` field.
*(Bootstrap anchor: `determinism-requirement`.)*

## V. Legacy as evidence

Code that predates a governing spec is not a violation to be erased; it is
evidence. A spec that claims authority over pre-existing code declares
`origin.retroactive: true` to record that it holds authority it has had since
before the graph existed, rather than masquerading as a fresh `establishes`
claim. History is queryable: "who established this unit" and "who currently owns
it" are different questions, and an amendment patches its predecessor in place
rather than blowing away its history.

Code adopted from outside the corpus is specced **as found**. The adopting spec
describes the behavior that exists, and records the behavior it would not have
chosen under a `## Known defects` heading, with the defect named. A defect
recorded under that heading is not thereby blessed: it is the reason a later spec
can be written against it. Without that heading an adopting spec has only bad
options, since describing the code accurately would ratify its defects as the
specified behavior. `origin.retroactive: true` says *when* the authority began;
`## Known defects` says what the adopting spec makes of what it found.

---

## Amendment

This constitution is changed by an ordinary spec that is `approved`, **claims
the affected text as an authority unit**, and does not contradict any anchor in
the `unamendable` list of `specs/000`. The bootstrap spec's freeze surface is
the hard boundary; everything else here is revisable through the normal governed
flow.

The claim uses the ordinary ownership vocabulary over a section unit of this
file, not the `amends` edge:

- `establishes` a `{ kind: section, file: "standards/spec/constitution.md",
  anchor: <heading-slug> }` unit for a principle the spec adds,
- `refines` that unit, with a named `aspect`, for a principle it tightens or
  restates,
- `co_authority` on that unit where a principle is genuinely shared.

The anchor is the heading slug the indexer computes, so `## V. Legacy as
evidence` is `v-legacy-as-evidence`. `amends` is not the instrument: its targets
are spec ids, and this file is not a spec. Spec 043 states the rule and is its
own worked example.

Unlike an amended `spec.md`, which is a record of what the corpus held when it
was ratified and is therefore never edited to mention its successors (spec 040),
this document is a standing statement of what is true now. It is edited in
place, and its history lives in the specs that claimed each section and in git.

The gate does not defend this: `standards/spec/constitution.md` is on the
coupling gate's built-in bypass floor, so a change here raises no `C-001` and
the ownership claim is a ledger fact that `spec-spine registry relationships`
answers, not a refusal. That is deliberate. A governance document is not code,
and refusing a constitutional edit for want of a matching source change would be
the wrong refusal.
