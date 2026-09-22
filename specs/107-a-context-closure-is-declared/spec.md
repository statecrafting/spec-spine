---
id: "107-a-context-closure-is-declared"
title: "A context closure is declared"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "106-obligations-are-declared-constraints"
summary: >
  An agent given a work order today has no declared answer to "what must I
  have read before I can act on this". A ContextClosure declares that set
  explicitly: the specs, sections and obligations a piece of work is answerable
  to. It is a declaration of what was named, never a proof that the naming is
  complete. Deferred: specified now, built when a consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 107: A context closure is declared

## 1. Purpose

Note 04 §4.5 records B04 and B05: a consumer that hands an agent a task wants
to know, and to record, what that task is answerable to. Today the answer is
assembled ad hoc from `registry show`, `index owner` and a reading of the spec
body, differently by every consumer, and it is never written down, so two runs
of the same task can be judged against different context and nothing notices.

### 1.1 Why it depends on obligations, and on nothing else in this wave

This spec's `depends_on` names spec 106 and only spec 106, because a closure
whose finest grain is "a whole spec" is not worth declaring: the point is to
say *which requirements* apply, and requirement ids are 106's. That is a
semantic dependency.

It does **not** depend on WorkScope, impact sets, move mappings, overlays,
waiver lifecycle or bindings. Those were proposed in the same wave, which is
not a reason for an edge.

## 2. Territory

None, on spec 106 §2's terms: a deferred contract claims no file and declares
no planned unit, so it cannot make the gate red before anyone builds it.

## 3. Behavior

### 3.1 A closure is a declared set, in frontmatter

A ContextClosure MUST be declared in frontmatter, under one key, with no
second fenced syntax. Same reasoning as spec 106 §3.1.

```yaml
context_closure:
  id: "C-1"
  specs:
    - "005-coupling-gate"
    - "081-coupling-sees-the-change-being-committed"
  sections:
    - { spec: "005-coupling-gate", anchor: "35-the-bypass-floor" }
  obligations:
    - "100-a-deleted-path-is-judged-where-it-lived#R-1"
  rationale: "the gate's ownership resolution and the segment rule it turns on"
```

### 3.2 Every reference is qualified and validated

- A `specs` entry MUST name a spec id that exists in the corpus.
- A `sections` entry MUST name an anchor that resolves in that spec's body, on
  spec 106 §3.4's terms: a dangling anchor is a validation error.
- An `obligations` entry MUST be **qualified** (`<spec-id>#<obligation-id>`)
  and MUST resolve. Spec 106 §3.3 forbids an unqualified cross-spec reference
  and this inherits that rule rather than restating a looser one.

### 3.3 What a closure establishes, and what it does not

A closure establishes that somebody **declared** this set as the context a
piece of work is answerable to.

It does **not** establish:

- **that the set is complete.** Nothing verifies that a closure names
  everything relevant, and nothing can: relevance is a judgement. A consumer
  MUST NOT read "not in the closure" as "not applicable".
- **that the context was read.** A declaration is not evidence of an agent's
  having loaded anything.
- **that work satisfying the closure is correct.**

This is spec 106 §3.7's limit, inherited deliberately and restated because a
closure is the artifact most likely to be mistaken for a sufficiency proof.

### 3.4 A closure is content-addressed by what it names

A closure MUST carry a digest over its resolved members: each spec's content
hash, each section's digest (spec 106 §3.5), each obligation's id and text.

That is what lets a consumer say "the context this work was done against has
changed" without re-reading the corpus, and it is why §3.2 requires the
references to resolve: a digest over unresolved names is a digest over
nothing.

The digest MUST be stable under reordering of the declared lists and MUST
change when any member's content changes.

### 3.5 Closures are inert in every gate

No verb changes its verdict because of a closure. `couple` does not consult
one, `check` does not refuse a stale one, and `lint` warns about a dangling
reference rather than refusing, except where §3.2 makes it a compile
validation error.

A closure is a record a consumer writes and reads. Making it a gate would mean
refusing work because a human's judgement about relevance was incomplete,
which §3.3 says nothing can establish.

## 4. Out of scope

- **Assembling a closure automatically** from edges, ownership or prose. A
  computed closure is a different artifact with a different meaning, and
  conflating the two is how "declared" becomes "believed complete".
- **Loading or delivering context to an agent.** That is an orchestration
  concern and, since spec 092, Statecraft's.
- **Any gate.** §3.5.
- **WorkScope.** A closure says what work is answerable to; a scope says what
  it may touch. Spec 108.

## 5. Open design questions

1. **Where does a closure live?** In a spec's frontmatter it is a property of
   the spec; in a work-order record it is a property of one task. The second is
   more useful and has no home in this repository, which is the question a
   consumer has to answer before this is built.
2. **Does a closure nest?** Naming another closure by id would let a consumer
   compose them. It would also need a cycle check and a merged digest, neither
   of which is worth specifying before anyone composes one.

## Acceptance when built

No `verify:cli` block; nothing here is implemented, and a block that passed
would assert nothing.

The build MUST establish, behaviorally:

- a closure whose references all resolve compiles, and appears in the registry
  shard with its members and its digest;
- an unqualified obligation reference is a validation error;
- a reference to a missing spec, a missing anchor or a missing obligation is a
  validation error naming which;
- the digest is unchanged when the declared lists are reordered;
- the digest changes when a referenced section's prose changes, and when a
  referenced obligation's text changes;
- `couple`, `check` and `lint --fail-on-warn` reach the same verdicts on the
  same tree with and without a closure declared (§3.5).
