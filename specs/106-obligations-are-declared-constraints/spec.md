---
id: "106-obligations-are-declared-constraints"
title: "Obligations are declared constraints"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "070-an-authority-snapshot-says-what-it-read"
summary: >
  A spec's normative content is prose today, so nothing downstream can name a
  single requirement, cite it, or say which evidence bears on it. This declares
  three constraint kinds in frontmatter, with stable ids inside a spec and
  qualified references across specs, and states precisely what an obligation
  does and does not establish. Deferred: specified now, built when a consumer
  names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 106: Obligations are declared constraints

## 1. Purpose

Everything a spec requires is prose. The corpus can say which spec owns a file
and which specs an edge connects; it cannot say which **requirement** a change
answers, which requirement an acceptance command tests, or which requirement a
successor amends. A reviewer reads "§3.5" in a commit message and a machine
reads nothing.

That is the gap note 04 §4.4 records as A01, A02 and A08. Its prerequisite,
an authority snapshot that names what it read, has had an owning spec since
spec 070, so this is buildable. No consumer has named it.

### 1.1 Deferred, and what that means here

Under the rule the owner adopted on 2026-09-21 (design note 09 §5):

> Specify now, implement only for a named consumer need.

So this document is a contract and not a work order. `implementation:
deferred` keeps it off `registry plan`'s ready set, which is the corpus's own
way of saying "written down, not scheduled". Filing it changes nothing about
what the engine does today.

## 2. Territory

None. This spec claims no file and declares no planned unit.

That is deliberate and is a rule for every deferred contract in this wave: a
`planned: true` unit for a file nobody is going to write this quarter is a
standing claim on territory, and a speculative `establishes` is worse, because
`index` then carries an unresolved-unit diagnostic that `check
--fail-on-unresolved` refuses. A contract with no consumer must not make the
gate red. Territory is claimed by the spec that builds it, in the change that
builds it.

## 3. Behavior

### 3.1 Declarations live in frontmatter

Obligations MUST be declared in a spec's YAML frontmatter, under one key.
There MUST NOT be a second declaration syntax, in particular a fenced block in
the body.

This is the owner's drafting baseline, and the reason is that the corpus
already has exactly one place a machine reads a spec's structure. A fenced
block would be a second grammar with its own parser, its own error messages,
its own position in the file, and its own way of disagreeing with the
frontmatter about the same spec.

```yaml
obligations:
  - id: "R-1"
    kind: requirement
    text: "A deleted path is judged at the snapshot preceding its segment."
    anchor: "31-two-snapshots-and-which-one-answers"
  - id: "I-1"
    kind: invariant
    text: "The gate never emits a successful verdict it did not compute."
    anchor: "35-required-prior-evidence-that-cannot-be-obtained-is-a-refusal"
  - id: "V-1"
    kind: verification
    text: "A shallow clone that cannot read its range exits 3."
    anchor: "35-required-prior-evidence-that-cannot-be-obtained-is-a-refusal"
    inputs: ["crates/spec-spine-cli/tests/couple.rs"]
```

### 3.2 Three kinds, and no fourth

`kind` MUST be one of exactly three:

| `kind` | What it asserts |
|---|---|
| `requirement` | something the implementation MUST do |
| `invariant` | something that MUST remain true across every state |
| `verification` | something an executable check MUST establish |

A fourth kind, `constraint`, was considered and is **not** adopted in the
initial contract. Note 04 D-5 left it open; the owner's ruling of 2026-09-21
closes it as "no fourth kind". The reason to keep the set at three is that a
fourth whose boundary with the other three is unclear produces
miscategorisation rather than information, and an additional kind is a MINOR
addition later if a consumer can say what it would carry that the three cannot.

### 3.3 Ids are stable within a spec; references across specs are qualified

- An obligation `id` MUST be unique **within its spec** and MUST be stable for
  the life of that obligation. It is not globally unique and MUST NOT be
  treated as such.
- A reference from another spec MUST be **qualified**: `<spec-id>#<obligation-id>`,
  for example `100-a-deleted-path-is-judged-where-it-lived#R-1`.
- An unqualified id in another spec's declaration MUST be a validation error,
  not a silent resolution against the local spec. A reference that resolves
  differently depending on where it is read is worse than one that fails.
- Reusing an id for a **different** obligation after the original is removed
  MUST be a validation error. An id is a name, and a name that has been reused
  makes every citation of it ambiguous forever, including citations in git
  history that nothing can update.

### 3.4 An anchor is validated, not asserted

`anchor` names a heading in the declaring spec's body, in the slug form the
corpus already uses for section units (spec 020). It MUST resolve to a heading
that exists, and a dangling anchor MUST be a validation error at compile.

The whole value of the anchor is that it ties a machine-readable obligation to
the prose that states it in full. An anchor nobody checks is a comment, and a
spec whose obligations point at headings it has since renamed is worse than one
with no anchors, because a reader trusts it.

### 3.5 Section digests accompany full spec identity, never replace it

A spec's identity stays what it is: the content hash over the whole `spec.md`.

Alongside it, an obligation-bearing spec MUST carry a **digest per anchored
section**, so a consumer can tell "the sentence my obligation points at
changed" from "some other part of this spec changed".

Both are required and neither substitutes for the other. The full hash is what
makes a ledger reproducible and is what every attestation already carries;
section digests are the finer grain a consumer needs to decide whether an
obligation it depends on moved. A design that replaced the first with the
second would make two specs with the same sections and different prose
indistinguishable.

### 3.6 A verification obligation declares its inputs

A `verification` obligation MUST declare its inputs explicitly, as a list of
repo-relative paths, commands, or both. They MUST NOT be inferred from the
spec's `## Verification` block, from its territory, or from anything else.

Inference is how a declaration becomes untrue without anyone editing it. An
obligation that says "tested by whatever this spec happens to run" is a claim
about a moving target.

### 3.7 What an obligation establishes, and what it does not

This section is the one an implementer should read twice. An obligation is a
**declaration**, and the corpus's reading of it is narrow on purpose.

An obligation record establishes:

- that a spec **declares** this requirement, invariant or verification;
- that the declaration is **related** to the named evidence: a verification
  obligation names inputs, an amending spec names the obligation it amends,
  and so on.

An obligation record does **not** establish:

- **that a verification result is true.** A `verification` obligation naming a
  test says the corpus declares that test as the evidence. It says nothing
  about whether the test passed, whether it can fail, or whether it tests what
  it claims. This repository has already met a test that passed against
  unbuilt code and a negative assertion that could not fire; an obligation
  record would not have caught either.
- **that the declarations enumerate every dependency.** The set of obligations
  a spec declares is what its author wrote down. It is not a proof of
  completeness, and nothing downstream may treat "no obligation says otherwise"
  as "nothing else is required".

A consumer that reads an obligation graph as a proof of correctness or of
completeness is misreading it, and this section exists so that misreading
cannot be blamed on the contract.

### 3.8 Compatibility

The `obligations` key is **optional**. A spec without it is valid, which is
every spec in the corpus on the day this is built.

Adding the key to the registry DTO is an additive registry-schema MINOR under
`docs/schema-versioning.md`: it restamps every shard's `specVersion` and
leaves `shardHash` alone, because the hash is over `spec.md` source bytes.

No existing verb changes its verdict. In particular `couple` MUST NOT consult
obligations: clearance stays "an owning spec's `spec.md` is in the diff", and
a gate that also demanded the right obligation be cited would refuse most
legitimate work on the day it shipped.

## 4. Out of scope

- **Obligation-aware coupling.** §3.8.
- **A completeness check** of any kind. §3.7.
- **ContextClosure**, which references obligations and is spec 107.
- **Impact and conflict sets**, which are declared against obligation ids and
  are spec 109.
- **A fourth constraint kind.** §3.2.
- **Any migration of existing prose into obligations.** The key is optional and
  a corpus adopts it spec by spec, or never.

## 5. Open design questions

Exposed rather than decided, because each needs a consumer's answer and none
blocks the others.

1. **Does an `amends` edge carry obligation granularity?** A successor that
   amends one requirement of a predecessor could name
   `<spec>#<obligation>` rather than a section anchor. It is the obvious next
   increment and it is not in this contract, because `amends_sections`
   already exists and two overlapping granularities need a consumer to choose
   between them.
2. **Is an obligation removable?** §3.3 forbids reuse of an id for a different
   obligation. Whether an obligation may be deleted at all, or must be marked
   withdrawn in place the way a superseded acceptance block is, is a lifecycle
   question this contract does not settle.
3. **Where does the section digest live?** In the registry shard beside the
   spec's content hash is the obvious home; whether it also belongs in the
   authority snapshot (spec 070) depends on whether a consumer verifies
   obligations offline.

## Acceptance when built

No `verify:cli` block. This spec declares no territory and its acceptance
cannot run against a tree that does not implement it; a block that passed here
would assert nothing, which is a failure mode this repository has met more than
once.

The build that implements this MUST establish, behaviorally:

- a spec with a valid `obligations` key compiles, and its obligations appear in
  the registry shard with their kinds, ids, anchors and inputs;
- a duplicate id within one spec is a validation error;
- an unqualified cross-spec reference is a validation error, and the message
  says a qualified form is required;
- an id reused for a different obligation after removal is a validation error;
- a dangling `anchor` is a validation error, with the heading name in the
  message;
- a `verification` obligation with no declared inputs is a validation error;
- a section digest changes when its section's prose changes and does not change
  when another section's does, and the spec's full content hash changes in both
  cases;
- a corpus with no `obligations` key anywhere compiles to shards whose
  `shardHash` values are unchanged from before the build;
- `couple` reaches the same verdict on the same diff with and without
  obligations declared.
