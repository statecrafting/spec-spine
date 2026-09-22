---
id: "110-an-interface-reference-is-digest-pinned"
title: "An interface reference is digest-pinned"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "070-an-authority-snapshot-says-what-it-read"
summary: >
  A spec can cite another repository's spec only in prose, so the citation
  cannot go stale visibly. This declares a cross-corpus interface reference
  pinned to a digest the exporting corpus already computes, so a moved
  interface is a detectable fact. Discovery, fetching and trust in the exporter
  stay outside. Deferred: specified now, built when a consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 110: An interface reference is digest-pinned

## 1. Purpose

Note 04 §5 P2 records B22 and A06. Six repositories in this family pin
spec-spine, several cite each other's specs, and every one of those citations
is prose. When the cited document changes, nothing anywhere says so. The
repository that most recently learned this is this one: spec 095 renumbered
the corpus and 638 citations had to be repaired by a spec written for the
purpose (spec 098), most of them inside this repository, where at least the
citations were reachable.

A cross-corpus citation is the same problem with no repair mechanism.

### 1.1 Why the dependency is spec 070 and not spec 106

The thing pinned is a **digest**, and spec 070's authority snapshot is where
this corpus already writes down what it read and what it hashed to. That is
the semantic dependency.

It is not an obligations dependency. A reference to a whole spec, pinned by
that spec's content hash, is useful on its own; pinning a single obligation is
a later increment and is named in §5 rather than assumed.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 A reference names a corpus, a spec and a digest

```yaml
interface_references:
  - corpus: "statecraft-cli"
    spec: "002-environment-lifecycle"
    digest: "sha256:..."
    sections: ["313-the-managed-instruction-file"]
    obtained: "2026-09-21"
    rationale: "this repository inserts the bridge line that section specifies"
```

- **`corpus`** is a name, not a URL and not a path. This spec deliberately
  defines no way to locate a corpus (§4).
- **`spec`** is the cited spec's id in that corpus.
- **`digest`** is the cited spec's content hash, in the form the exporting
  corpus emits.
- **`sections`** narrows the citation to anchors, when the citation is about
  part of a document.
- **`obtained`** is an authored date recording when the digest was read. It is
  authored, never taken from a clock: every artifact-producing function in this
  engine is a pure function of config and file contents, and reading a clock
  here would break that for a field nobody needs to be automatic.

### 3.2 A reference is a recorded fact, and its staleness is detectable

The corpus cannot check the digest: the cited corpus is not here. What it can
do, and MUST do, is make the check possible for somebody who has both.

A reference MUST therefore be:

- **complete enough to check offline**: corpus, spec id, digest and the form
  the digest is in, with no implicit defaults;
- **recomputable in the same way**: the digest MUST be the exporting corpus's
  own spec content hash, not a re-hash of a copy, so the two sides compute the
  same number over the same bytes.

That is the whole mechanism. A verifier holding both corpora compares; this
corpus records.

### 3.3 What a reference establishes, and what it does not

It establishes that this spec **cites** that document **as it was at that
digest**.

It does **not** establish:

- **that the cited document still says that.** The point of the digest is that
  the question is now answerable, not that it has been answered.
- **that the exporting corpus is trustworthy.** Trust in the exporter is the
  consumer's, and §4 keeps it there.
- **that the cited corpus is reachable.** Nothing here fetches anything.

### 3.4 Nothing gates on a reference

No verb refuses because a reference exists, and no verb attempts to resolve
one. `lint` MAY report a reference whose `digest` is malformed, which is a
statement about this file and not about the other corpus.

A gate would have to fetch, and fetching turns a pure function of local file
contents into a network call. That is the invariant this engine is built on
and a citation convenience does not get to break it.

### 3.5 A reference is updated deliberately

Changing a `digest` is an edit to a spec, so it goes through the ordinary
route: the spec that owns the reference is edited, under an authority that
covers it. A reference MUST NOT be machine-refreshed in place, because "the
citation was updated and nobody read the new text" is exactly the failure a
pin exists to prevent.

## 4. Out of scope

- **Discovery.** How a consumer finds the corpus named in `corpus` is not
  specified here and deliberately so: a registry of corpora is a product
  decision with an owner, and it is not this engine.
- **Fetching.** §3.4.
- **Trust in the exporter.** §3.3.
- **Pinning an obligation** rather than a spec. §5.
- **Automatic repair** of a stale reference. §3.5.

## 5. Open design questions

1. **Can a reference pin an obligation?** `spec-id#obligation-id` plus that
   obligation's section digest would be the finer grain, and it is only worth
   building once spec 106 exists and a second corpus in this family declares
   obligations. Two preconditions, neither met.
2. **What is `corpus` a name in?** Today it would be a convention. If a
   registry of corpus names ever exists it will want to be the authority, and
   a reference written before it will need translating.
3. **Does the authority snapshot carry references?** Spec 070 records what
   this corpus read. A cross-corpus citation is arguably an input, which would
   put it in the snapshot; it is arguably a statement rather than a read, which
   would keep it out.

## Acceptance when built

No `verify:cli` block; nothing here is implemented.

The build MUST establish, behaviorally:

- a well-formed reference compiles and reaches the registry shard with every
  member intact, including `obtained` verbatim;
- a malformed digest is reported by `lint` and does not refuse without
  `--fail-on-warn`;
- no verb attempts any filesystem access outside the repository root, and no
  verb attempts a network call, when a reference is present: asserted by
  running the full gate in a corpus carrying a reference to a corpus that does
  not exist anywhere, and observing the same verdicts as without it;
- compiling a corpus with references is a pure function of its files: two runs
  produce byte-identical shards, and `obtained` is never rewritten.
