---
id: "107-a-context-closure-is-declared"
title: "A context closure is declared"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "106-obligations-are-declared-constraints"
summary: >
  A piece of work has no declared answer to "what must I have read before I act
  on this", and no way to tell later whether that context changed. A
  ContextClosure is that answer as data: the specs, sections and obligations a
  task is answerable to, named by the task's owner. spec-spine resolves a
  closure request against the committed ledger and returns every member with
  its identity and one order-independent digest over all of them, so "the
  context this work was done against has changed" is one comparison. The
  closure lives in the consumer's record; spec-spine only resolves it, and no
  gate reads one.
establishes:
  - { kind: file, path: "crates/spec-spine-core/src/closure.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/closure.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/closure.rs", planned: true }
extends:
  # 3.5: the facade and the CLI verb.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_registry.rs" }, nature: additive }
  # 3.4: the committed content hashes, read and never recomputed.
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-core/src/query.rs" }, nature: additive }
  # 3.7: the documentation a consumer reads.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: context }
---

# 107: A context closure is declared

## 1. Purpose

Note 04 §4.5 records B04 and B05: a consumer that hands an agent a task wants to
know, and to record, what that task is answerable to. Today the answer is
assembled ad hoc from `registry show`, `index owner` and a reading of the spec
body, differently by every consumer, and it is never written down. Two runs of
the same task can be judged against different context, and nothing notices.

### 1.1 Why it is built now (design note 09, D-7)

- **What becomes possible.** A task's context is a value: a set of specs,
  sections and obligations, resolved against the ledger and folded into one
  digest. Recording it at issue and resolving it again at review answers "did
  what this was answerable to change" without re-reading the corpus.
- **Why it is distinctive.** It turns spec 106's identities into the object an
  orchestrator actually handles, while leaving the orchestration state where
  spec 092 put it: with the orchestrator.
- **One integration scenario.** The Statecraft CLI stores a closure request in
  a work order, calls `registry closure` when the order is issued and records
  the digest, and calls it again before accepting the result. A different
  digest names which members moved.
- **What remains hypothetical.** No consumer calls this today, and the request
  shape is designed here rather than taken from one. What is not hypothetical
  is the data it reads: every member identity it folds is already in the
  committed ledger (spec 048's content hash, spec 106's section digests and
  obligations).
- **The smallest coherent contract.** One request document, one resolver, one
  facade function, one CLI verb, no gate.
- **Dependencies, compatibility, evidence.** Spec 106. No schema of any
  artifact moves; the answer is a new read document.

### 1.2 Why it depends on obligations, and on nothing else

A closure whose finest grain is "a whole spec" is not worth declaring: the
point is to say which sections and which requirements apply, and those
identities are spec 106's. It does not depend on WorkScope, impact sets, move
mappings, overlays, waiver lifecycle or bindings.

## 2. Territory

- `crates/spec-spine-core/src/closure.rs` (established): the request, the
  resolved closure, and the resolver.
- The facade in `lib.rs`, the `registry closure` verb in `cmd_registry.rs`, and
  the committed-content-hash read in `query.rs` (`extends`).
- `docs/api.md` (`extends`).
- `crates/spec-spine-core/tests/closure.rs` and
  `crates/spec-spine-cli/tests/closure.rs` (established).

## 3. Behavior

### 3.1 A closure lives with its consumer, and spec-spine resolves it

The draft left open where a closure lives: in a spec's frontmatter, or in a
work-order record. It lives in the consumer's record. A closure is a property
of one task, and a task's record is orchestration state, which spec 092 keeps
out of this repository. spec-spine owns the half that needs the ledger: turning
a request into resolved members and a digest.

So there is no `context_closure` frontmatter key. A closure is a request
document:

```json
{
  "specs": ["005-coupling-gate"],
  "sections": [{ "spec": "100", "anchor": "3-6-the-two-facts" }],
  "obligations": ["106-obligations-are-declared-constraints#R-5"],
  "rationale": "the gate's ownership rule and the section identity it relies on"
}
```

`specs`, `sections` and `obligations` are each optional and default to empty;
at least one member MUST be named in total, or the request is refused as a
parse error (exit 3). `rationale` is optional free text, carried through
unchanged and not digested. Any other member MUST be refused as a parse error:
a misspelt `obligation` that silently dropped a member would produce a digest
over less than its author named.

### 3.2 Every reference is qualified and resolved

- A `specs` entry names a spec by the corpus's one spec-id policy (spec 084): a
  full id or a short form that resolves to exactly one spec.
- A `sections` entry names a spec the same way and an anchor that MUST be one
  of that spec's `sectionDigests` keys (spec 106 §3.5).
- An `obligations` entry MUST be qualified, `<spec-id>#<obligation-id>`, and is
  parsed exactly as spec 106 §3.6 parses one. An unqualified entry MUST be
  refused as a parse error (exit 3) before anything else is resolved.

Every reference that does not resolve MUST be reported, all of them in one
refusal naming each (`NotFound`, exit 1), rather than only the first. A consumer
fixing a closure should not have to rerun it once per typo.

A withdrawn obligation resolves, carrying `withdrawn: true`. A closure citing a
retired requirement says so rather than failing; whether that is acceptable is
the consumer's judgement.

### 3.3 The resolved closure

The answer is a read document (spec 074) with `members`, `digest`, and
`rationale` when one was given. Each member carries `kind` and its identity:

| `kind` | Members |
|---|---|
| `spec` | `spec` (the full id), `contentHash` |
| `section` | `spec`, `anchor`, `digest` |
| `obligation` | `spec`, `id`, the obligation's `kind` as `obligationKind`, `text`, `anchor`, `inputs` when declared, `withdrawn` when true, and `sectionDigest` |

Members are sorted by `kind` and then by identity, and a member named twice
appears once.

### 3.4 The digest is over what the members are, not how they were named

`digest` MUST be the corpus's one hash construction (spec 077) over one piece
per member:

- `spec:<id>` over the spec's `contentHash`;
- `section:<id>#<anchor>` over the section's digest;
- `obligation:<id>#<obligation-id>` over the obligation member's canonical
  JSON (sorted keys), which includes its text, its kind, its withdrawal and its
  section's digest.

Consequences, each asserted by the acceptance:

- reordering or repeating the declared lists does not change the digest, and
  neither does naming a spec by its short id instead of its full one;
- editing a named section's prose changes the digest, and so does editing the
  prose of the section a named obligation points at, or the obligation's text,
  or withdrawing it;
- editing a section nobody named does not change the digest, unless a named
  whole spec contains it, because a named spec is digested by its full content
  hash.

The content hashes are read from the committed shards, never recomputed (spec
048 §3.3), and section digests and obligations from the committed registry
(spec 106).

### 3.5 A digest over a stale ledger is refused

A closure digest is evidence about the corpus, and a digest computed over a
ledger that no longer matches `spec.md` would vouch for text nobody can read.
`registry closure` and the facade MUST check registry freshness first (spec 028)
and refuse a stale ledger with the staleness exit (2). This is stricter than
`registry show` on purpose: `show` reports what the ledger says, and a closure
digest is the ledger vouching for the corpus.

### 3.6 The surfaces

- `spec_spine_core::resolve_closure(&Registry, &content_hashes, &ClosureRequest)`,
  pure: it reads nothing but its arguments.
- `spec_spine_core::closure_json(config_json, repo_root, request_json)`: the
  facade, `&str` in and `String` out, which checks freshness and reads the
  committed ledger.
- `spec-spine registry closure --request <FILE | -> [--json]`: the same, with
  `-` reading the request from stdin. Without `--json`, one line per member and
  the digest.

### 3.7 What a closure establishes, and what it does not

A closure establishes that somebody **declared** this set as the context a
piece of work is answerable to, and what each member's identity was when it was
resolved.

It does **not** establish:

- **that the set is complete.** Relevance is a judgement, and nothing verifies
  that a closure names everything relevant. A consumer MUST NOT read "not in
  the closure" as "not applicable".
- **that the context was read.** A declaration is not evidence of an agent's
  having loaded anything.
- **that work satisfying the closure is correct.**

### 3.8 Closures are inert in every gate

No verb changes its verdict because of a closure. `couple`, `check`, `lint`
and `index coverage` never read one. A closure is a record a consumer writes and
reads, and making it a gate would refuse work because a human's judgement about
relevance was incomplete, which 3.7 says nothing can establish.

`docs/api.md` MUST document the request, the answer and the facade.

## 4. Out of scope

- **Assembling a closure automatically** from edges, ownership or prose. A
  computed closure is a different artifact with a different meaning.
- **Storing, loading or delivering a closure.** Orchestration state (spec 092).
- **Nesting closures.** Naming another closure by id would need a cycle check
  and a merged digest; not worth specifying before anyone composes one.
- **A per-member change report.** A consumer holding two resolved closures can
  compare their members; a verb that does it is a later, additive read.
- **WorkScope** (108): a closure says what work is answerable to, a scope says
  what it may touch.

## 5. Resolved decisions

**D-1 (2026-09-22, built under D-7, not deferred).** Filed on 2026-09-21 as a
deferred contract claiming no territory. The owner's D-7 authorizes it (design
note 09 section 10), and 1.1 records the evaluation.

**D-2 (2026-09-22, correction: the closure is a request, not frontmatter).**
The draft declared `context_closure` in a spec's frontmatter and asked, as its
own open question 1, whether a closure belongs to a spec or to a task. It
belongs to a task, and a task's record is not this repository's (spec 092). A
frontmatter key would have made every closure a property of a spec, which is the
less useful answer the draft itself named. 3.1 records the replacement.

**D-3 (2026-09-22, correction: refusals are reads, not compile codes).** The
draft made a dangling reference a compile validation error. With no frontmatter
key there is nothing for compile to validate; the same references are refused
where they are resolved (3.2), with the exit codes the read verbs already use.

**D-4 (2026-09-22, stricter than `show`: freshness first).** 3.5. A read that
emits a digest is making a claim about the corpus, and spec 028's freshness
check is the only thing that makes that claim true.

## Verification

Written to fail against the tree it is filed on: nothing named here exists.

```verify:cli
cargo build --release --locked
# 3.1 - 3.4, 3.8: resolution, refusals, the digest's properties, gate neutrality.
cargo test -p spec-spine-core --test closure --locked
# 3.5, 3.6: the shipped verb, including stdin, the stale refusal and exit codes.
cargo test -p spec-spine-cli --test closure --locked
# A closure over this corpus resolves.
sh -c 'printf "%s" "{\"specs\":[\"107\"],\"obligations\":[\"106#R-5\"]}" | ./target/release/spec-spine registry closure --request - --json | grep -q "\"digest\""'
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
