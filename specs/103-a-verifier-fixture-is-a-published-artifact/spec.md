---
id: "103-a-verifier-fixture-is-a-published-artifact"
title: "A verifier fixture is a published artifact"
status: draft
kind: "tooling"
created: "2026-09-21"
implementation: pending
owner: "The spec-spine Authors"
depends_on:
  - "021-ledger-seal"
  - "068-a-verifier-checks-the-bytes-it-was-given"
summary: >
  The tamper and cross-version cases an independent verifier must reproduce
  exist only as commands inside spec 068's acceptance block and as a measured
  table in docs/authority-evidence.md. A verifier outside this repository
  cannot run either. This emits them as a named, versioned fixture set: the
  bytes, the corpus each payload is about, and the outcome each case must
  reach, all measured against the shipped verifier rather than asserted.
establishes:
  - { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/verifier_fixtures.rs", planned: true }
references:
  - unit: { kind: file, path: "crates/spec-spine-core/src/attest.rs" }
    role: "context"
  - unit: { kind: file, path: "crates/spec-spine-cli/src/verify_attestation.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/authority-evidence.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 103: A verifier fixture is a published artifact

## 1. Purpose

Three consumers outside this repository are asked to verify spec-spine
attestations independently: statecraft-cli, the hosted control plane, and hqgit
(`docs/design/04-authority-evidence-extension.md` §7). Note 04 offers all three
"the tamper and cross-version cases of AE §8, as runnable commands in 068's
`## Verification` block".

That offer cannot be taken up. Those commands run against **this** repository's
tree, this repository's scratch key and a binary built here. A verifier written
in TypeScript against a stored payload has no way to execute them, and the
table in `docs/authority-evidence.md` §8 is a measurement report, not an input.
So the four contract gaps spec 068 closed are asserted for this repository's
own verifier and for nobody else's, and a consumer that gets them wrong finds
out from a tampered payload rather than from a test.

The gap is narrow and so is this spec. spec-spine emits the **bytes, the corpus
they are about, and the expected outcomes**. Everything a consumer builds
around them stays with the consumer.

## 2. Territory

A fixture directory under `crates/spec-spine-core/fixtures/verifier/`, the
documented generator inside it, and the test that proves its cases still
describe the shipped verifier.

The test lives in `crates/spec-spine-cli/tests/`, not beside the fixtures in
core, because it must invoke the **shipped verifier**: the decision sequence
under test (the loose `schemaVersion` read, the MAJOR gate, the strict parse,
the recompute, the byte comparison, and the exit code each produces) is
assembled in the CLI. A core test would have to reassemble that sequence from
library calls and would then be asserting its own reimplementation, which is
exactly the second-implementation failure this repository refuses elsewhere.

Both paths are declared `planned: true` (spec 063): they are this spec's
territory and they are not written yet, so the claim is a declaration rather
than an unresolved unit.

It claims **no source file**. The draft this spec replaces claimed
`attest.rs` and `verify_attestation.rs` through an `extends` edge. Nothing here
changes either: the fixtures describe the verifier as it stands, and a claim
that buys nothing makes two files look like this spec's territory to everyone
who reads the frontmatter afterwards. They are carried as `references`, which
is non-owning.

## 3. Behavior

### 3.1 The boundary, stated first

| Owned here | Owned by the consumer |
|---|---|
| the fixture bytes, byte for byte | the evidence bundle that carries them |
| the payload type name and its version | the envelope, its media type, any in-toto predicate |
| the subject each case is about | which subject a bundle is willing to trust |
| the outcome each case must reach | verification orchestration and scheduling |
| the reason class of a refusal | the trust decision taken on a refusal |

This spec MUST NOT emit an envelope, a bundle manifest, a trust policy, a key,
a signature over anything but a fixture's own payload, or any statement that a
fixture's subject is trustworthy. Spec 021 §7's vocabulary holds: these are
attestations, recomputable records, and nothing here is testimony. It MUST NOT
invent a consumer-side evidence bundle or trust policy of any kind.

It MUST NOT reintroduce a distributed harness in any form. A fixture is data a
verifier reads; it is not an installed file, not a template and not a
generator.

### 3.2 Three kinds of bytes, never conflated

The draft this spec replaces described every `payload.json` as bytes "written
by `attest` and never re-serialized". That is true of some cases and false of
most, because a tamper case is by construction not producer output. A fixture
set that describes a mutated payload as unmodified producer output is lying
about its own evidence.

Every case MUST declare which of exactly three kinds its `payload.json` is:

| `bytes` | Meaning |
|---|---|
| `producer` | verbatim output of `spec-spine attest` over the case's corpus, byte for byte, never re-serialized |
| `mutated` | `producer` bytes with one named, mechanical change applied; `mutation` says what |
| `authored` | hand-written bytes that `attest` could not produce at all, such as a payload that is not valid JSON |

A `producer` case MUST record the command that produced it. A `mutated` case
MUST record the `producer` case it derives from and the single change applied.
Re-serializing a `mutated` payload through a JSON library after the change is
permitted and is what most of the mutations are; what is forbidden is calling
the result producer output.

### 3.3 What a fixture case is

Each case is a directory holding:

1. **`payload.json`**, the bytes under test.
2. **`case.json`**, the record (§3.4).
3. **`corpus/`**, the tree the payload is about, for every case whose expected
   outcome depends on a recompute (§3.6). Present exactly when `needsCorpus`
   is true.

A case MUST be reproducible from its own directory plus a spec-spine binary,
with no reference to this repository's tree, its keys or its history. That is
the whole point of publishing it, and it is the requirement the recompute
contract makes expensive: see §3.6.

### 3.4 `case.json`, and the index

```json
{
  "schemaVersion": "0.1.0",
  "id": "unknown-member-nested",
  "payloadType": "spec-spine/corpus-attestation",
  "payloadSchemaVersion": "0.1.0",
  "bytes": "mutated",
  "derivedFrom": "control-untampered",
  "mutation": "added member `extra` to the nested `tool` object",
  "subject": { "kind": "corpus", "attestationHash": "sha256:..." },
  "needsCorpus": false,
  "toolVersion": "0.21.0",
  "expect": { "outcome": "refused", "reason": "unknown-member", "exit": 3 }
}
```

- **`payloadType`** MUST be a stable name this repository owns and MUST NOT be
  a file path or a verb name. Note 04 S4 gives the composition envelope to
  Statecraft and reserves the type names for spec-spine; this is where they are
  written down. Two names exist: `spec-spine/corpus-attestation` and
  `spec-spine/spec-attestation`.
- **`payloadSchemaVersion`** is the version the payload itself declares, which
  for a version case is deliberately not this build's.
- **`bytes`**, **`derivedFrom`** and **`mutation`** are §3.2's. `derivedFrom`
  and `mutation` are required when `bytes` is `mutated` and MUST be absent
  otherwise.
- **`subject`** MUST identify what the payload is about by digest, never by a
  local path: the corpus `attestationHash`, or a spec attestation's hash
  together with its `specSourceHash`. For an `authored` payload that has no
  computable hash, `subject` is `{ "kind": "none" }`, which is the honest
  answer and is not the same as omitting the member.
- **`toolVersion`** is the `tool.version` the payload carries. It drives
  §3.7's version rule.
- **`expect.outcome`** MUST be `accepted` or `refused`; `expect.reason` a
  member of §3.5's closed set, present exactly when the outcome is `refused`;
  `expect.exit` this repository's CLI exit code, recorded so a verifier can
  compare its own mapping without being required to adopt it.

**The index**, `fixtures/verifier/index.json`:

```json
{
  "schemaVersion": "0.1.0",
  "payloadTypes": ["spec-spine/corpus-attestation", "spec-spine/spec-attestation"],
  "reasons": ["unreadable-json", "missing-schema-version", "unsupported-major",
              "unknown-member", "duplicate-key", "non-canonical-bytes",
              "content-mismatch", "version-mismatch"],
  "cases": ["control-untampered", "..."]
}
```

`reasons` is the closed set, declared in the index rather than left implicit,
so a consumer reads the vocabulary instead of inferring it from the cases that
happen to be present. `cases` lists every case directory; a directory not
listed, or a listed directory that does not exist, fails the test (§3.8).

**Compatibility.** The set is versioned as a whole by the index's
`schemaVersion`, independently of the attestation schemas. Adding a case,
adding a reason to the closed set, or adding an optional member to `case.json`
is a MINOR. Removing or renaming a reason, removing a case, or changing a
case's expected outcome is a MAJOR, and changing an expectation MUST NOT happen
without the spec that changes the verifier: a fixture is this repository's
written claim about its own behavior, and quietly editing the claim to match a
regression is the failure the whole set exists to prevent.

### 3.5 The reason classes, measured against the shipped verifier

Every row below was measured on 2026-09-21 with `spec-spine 0.21.0`, by
generating an attestation over a one-spec corpus and running
`verify-attestation --recompute --json` against each payload. None of it is
inferred from reading the code.

| `reason` | Where the verifier decides | Exit | `--json` |
|---|---|---|---|
| `unreadable-json` | `serde_json` before anything else | `3` | `error.kind: "parse"` |
| `missing-schema-version` | the loose `schemaVersion` read, before the strict parse | `3` | `error.kind: "schema"` |
| `unsupported-major` | the MAJOR gate, before the strict parse | `3` | `error.kind: "schema"` |
| `unknown-member` | the strict parse (`deny_unknown_fields`) | `3` | `error.kind: "parse"` |
| `duplicate-key` | the strict parse: a repeated struct field is a parse error | `3` | `error.kind: "parse"` |
| `non-canonical-bytes` | the byte comparison, after a value match | `1` | `outcome: "contentMismatch"`, `differences: ["bytes are not the canonical serialization"]` |
| `content-mismatch` | the recompute, field by field | `1` | `outcome: "contentMismatch"`, `differences` naming each field |
| `version-mismatch` | the `tool.version` comparison, before the recompute | `1` | `outcome: "versionMismatch"` with `expected` and `actual` |

An `accepted` case is exit `0` with `outcome: "match"`.

**The two the draft got wrong, corrected here:**

- A payload whose MAJOR is current and whose **MINOR is ahead** does **not**
  verify. The draft asserted it must, on the reasoning that a MINOR is additive
  and a verifier refusing one would refuse every future release. That reasoning
  holds for the **MAJOR gate**, and the MAJOR gate does admit it. It does not
  hold for the verb: `verify_recompute` compares `schemaVersion` like every
  other field, deliberately (spec 068 §3.3: skipping it let an attestation
  claiming a schema this build never emitted recompute as a match), so a
  MINOR-ahead payload is a `content-mismatch` on the `schemaVersion` field.
  Measured: `differences: ["schemaVersion (0.9.0 -> 0.1.0)"]`, exit `1`.
  The set therefore carries the case with its **measured** expectation, and
  §3.9 records the contradiction rather than burying it.
- **`duplicate-key` is refused at the parse**, not at the byte comparison as a
  non-canonical form. Measured: exit `3`, `error.kind: "parse"`. Both are
  refusals, and a fixture that recorded the wrong layer would teach a consumer
  to implement the check in the wrong place.

### 3.6 Recompute cases carry their corpus

Spec 068's verifier is a **recompute** verifier: `--recompute` re-reads the
corpus and compares. A payload alone therefore cannot produce `accepted`,
`content-mismatch` or `version-mismatch`; only the refusals that happen before
the recompute are decidable from the bytes.

A case whose expected outcome needs a recompute MUST set `needsCorpus: true`
and MUST carry, under `corpus/`, the complete tree the payload attests: every
`spec.md` and the `spec-spine.toml`, if any, and nothing else. The corpus is
kept to one spec so a case directory stays a few kilobytes and a consumer can
read it.

This is what "carries or precisely references all inputs needed to reproduce
its verdict independently" costs, and it is not optional: a fixture whose
expected outcome cannot be reproduced from the fixture is a claim, not a test.

### 3.7 The version rule, so the set does not rot

`verify_recompute` compares the payload's `tool.version` against the verifying
build and returns `version-mismatch` when they differ, before any content
comparison (spec 021 FR-005). A committed positive control therefore stops
producing `accepted` the moment this repository releases again.

The harness MUST resolve this by asserting, never by skipping:

- when `case.toolVersion` equals the verifying build, the recorded
  `expect` MUST hold;
- when it differs, the outcome MUST be exactly `version-mismatch`, with
  `expected` equal to `case.toolVersion` and `actual` equal to the verifying
  build.

Both branches assert. The second is not a weakened form of the first: it is
FR-005's own contract, that a version difference is a named outcome and never a
false content mismatch and never a skip-as-pass.

Cases whose refusal happens before the recompute (`needsCorpus: false`) are
version-independent and take the first branch always.

### 3.8 The set describes the shipped verifier, and is proven to

A committed fixture is a claim about behavior, so it MUST be executed here.
`crates/spec-spine-cli/tests/verifier_fixtures.rs` walks the set, runs this
repository's own verifier over each payload, and asserts the recorded outcome
and reason under §3.7's rule. A case whose expectation no longer matches the
shipped verifier fails the build.

**Before running any case**, the test MUST assert all of:

1. the index parses and its `schemaVersion` MAJOR is understood;
2. the case list is **non-empty**;
3. the case list and the directories on disk are the **same set**, so a case
   added without being listed, or listed without existing, is a failure rather
   than a silent omission;
4. the control case is present and is a `producer` case;
5. every reason in the index's closed set is exercised by at least one case,
   and every case's reason is in the closed set;
6. the number of cases actually executed equals the number listed.

An empty or partial walk that asserts nothing is the failure mode this
repository has already met more than once, and clauses 2, 3 and 6 are there
because "the loop ran zero times" and "the loop ran and passed" are
indistinguishable without them.

### 3.9 Where the bytes come from, and what is not promised

Fixtures MUST be generated by a documented command and committed, not produced
at test time. A fixture regenerated on each run would re-serialize the bytes
whose exact spelling is the point, and would make the non-canonical-bytes case
unable to fail.

**Distribution.** The set is committed in-tree and is therefore already
obtainable by anyone who can read the repository. For a consumer who cannot,
the route is the published crate: `crates/spec-spine-core/fixtures/` ships
inside the `spec-spine-core` `.crate` archive, so

```sh
cargo package -p spec-spine-core --locked
tar xzf target/package/spec-spine-core-<version>.crate
# fixtures are at spec-spine-core-<version>/fixtures/verifier/
```

is a deterministic extraction route with a digest (the `.crate` SHA-256).
**Committing the fixtures is not publishing them**, and this spec MUST NOT be
read as a publication claim: the archive above exists locally until a release
uploads it, and spec 104's record is where a packaged artifact's digest is
written down.

Nothing here promises that a consumer's verifier is correct. It promises that a
consumer's verifier can be **tested against the same bytes, the same corpora
and the same outcomes this one is**, which is the whole of what a producer can
offer.

## 4. Out of scope

- **The neutral verifier's home** (note 04 D-6). A family decision; this spec
  is deliberately only the fixtures, which is what D-6 says spec-spine's part
  is.
- **Evidence-bundle composition**, receipt shapes, the in-toto mapping, and any
  consumer trust policy (note 04 R2, S4). §3.1.
- **Signing keys and key distribution.** A fixture carries no seal; signature
  verification is a separate axis with its own key-management question, and a
  committed scratch key is a liability rather than a fixture.
- **Changing the verifier.** Every expectation in the set is measured from the
  verifier as it stands, including the two the draft predicted wrongly. If a
  measured behavior is judged a defect, that is a new spec, and the fixture's
  expectation moves with it and not before (§3.4's compatibility rule).
- **Obligation, closure and scope records.** A separate, unadopted increment.
- **Publishing the set to a registry.** §3.9.

## 5. Resolved decisions

**D-1 (2026-09-21, this is buildable without a named consumer, unlike spec
102).** Three consumers are already named in note 04 §7 and all three were
offered these cases, so the need is recorded rather than hypothetical. What is
not decided here is where a neutral verifier lives, and §4 keeps it that way.

**D-2 (2026-09-21, owner correction: the ownership claim on the two source
files is dropped).** Packaging fixtures does not change verifier behavior, and
nothing in this spec edits `attest.rs` or `verify_attestation.rs`. They are
`references`, which is non-owning.

**D-3 (2026-09-21, owner correction: producer bytes and mutated bytes are
different things).** §3.2. The draft described every payload as unmodified
`attest` output, which is false for every tamper case and would have taught a
consumer that a mutated payload is what the producer emits.

**D-4 (2026-09-21, owner correction: expectations are measured, not
predicted).** §3.5 and §3.9. The draft asserted that a MINOR-ahead payload must
verify. It does not, and the reason is a deliberate decision in spec 068. The
set records the measured behavior and §3.5 names the contradiction between the
MAJOR gate's admission and the recompute's comparison, so a later reader
decides it on purpose rather than discovering it.

## Verification` block".

That offer cannot be taken up. Those commands run against **this** repository's
tree, this repository's scratch key and a binary built here. A verifier written
in TypeScript against a stored payload has no way to execute them, and the
table in `docs/authority-evidence.md` §8 is a measurement report, not an input.
So the four contract gaps spec 068 closed are asserted for this repository's
own verifier and for nobody else's, and a consumer that gets them wrong finds
out from a tampered payload rather than from a test.

The gap is narrow and so is this spec. spec-spine emits the **bytes, the corpus
they are about, and the expected outcomes**. Everything a consumer builds
around them stays with the consumer.

## 2. Territory

A fixture directory under `crates/spec-spine-core/fixtures/verifier/`, the
documented generator inside it, and the test that proves its cases still
describe the shipped verifier.

The test lives in `crates/spec-spine-cli/tests/`, not beside the fixtures in
core, because it must invoke the **shipped verifier**: the decision sequence
under test (the loose `schemaVersion` read, the MAJOR gate, the strict parse,
the recompute, the byte comparison, and the exit code each produces) is
assembled in the CLI. A core test would have to reassemble that sequence from
library calls and would then be asserting its own reimplementation, which is
exactly the second-implementation failure this repository refuses elsewhere.

Both paths are declared `planned: true` (spec 063): they are this spec's
territory and they are not written yet, so the claim is a declaration rather
than an unresolved unit.

It claims **no source file**. The draft this spec replaces claimed
`attest.rs` and `verify_attestation.rs` through an `extends` edge. Nothing here
changes either: the fixtures describe the verifier as it stands, and a claim
that buys nothing makes two files look like this spec's territory to everyone
who reads the frontmatter afterwards. They are carried as `references`, which
is non-owning.

## 3. Behavior

### 3.1 The boundary, stated first

| Owned here | Owned by the consumer |
|---|---|
| the fixture bytes, byte for byte | the evidence bundle that carries them |
| the payload type name and its version | the envelope, its media type, any in-toto predicate |
| the subject each case is about | which subject a bundle is willing to trust |
| the outcome each case must reach | verification orchestration and scheduling |
| the reason class of a refusal | the trust decision taken on a refusal |

This spec MUST NOT emit an envelope, a bundle manifest, a trust policy, a key,
a signature over anything but a fixture's own payload, or any statement that a
fixture's subject is trustworthy. Spec 021 §7's vocabulary holds: these are
attestations, recomputable records, and nothing here is testimony. It MUST NOT
invent a consumer-side evidence bundle or trust policy of any kind.

It MUST NOT reintroduce a distributed harness in any form. A fixture is data a
verifier reads; it is not an installed file, not a template and not a
generator.

### 3.2 Three kinds of bytes, never conflated

The draft this spec replaces described every `payload.json` as bytes "written
by `attest` and never re-serialized". That is true of some cases and false of
most, because a tamper case is by construction not producer output. A fixture
set that describes a mutated payload as unmodified producer output is lying
about its own evidence.

Every case MUST declare which of exactly three kinds its `payload.json` is:

| `bytes` | Meaning |
|---|---|
| `producer` | verbatim output of `spec-spine attest` over the case's corpus, byte for byte, never re-serialized |
| `mutated` | `producer` bytes with one named, mechanical change applied; `mutation` says what |
| `authored` | hand-written bytes that `attest` could not produce at all, such as a payload that is not valid JSON |

A `producer` case MUST record the command that produced it. A `mutated` case
MUST record the `producer` case it derives from and the single change applied.
Re-serializing a `mutated` payload through a JSON library after the change is
permitted and is what most of the mutations are; what is forbidden is calling
the result producer output.

### 3.3 What a fixture case is

Each case is a directory holding:

1. **`payload.json`**, the bytes under test.
2. **`case.json`**, the record (§3.4).
3. **`corpus/`**, the tree the payload is about, for every case whose expected
   outcome depends on a recompute (§3.6). Present exactly when `needsCorpus`
   is true.

A case MUST be reproducible from its own directory plus a spec-spine binary,
with no reference to this repository's tree, its keys or its history. That is
the whole point of publishing it, and it is the requirement the recompute
contract makes expensive: see §3.6.

### 3.4 `case.json`, and the index

```json
{
  "schemaVersion": "0.1.0",
  "id": "unknown-member-nested",
  "payloadType": "spec-spine/corpus-attestation",
  "payloadSchemaVersion": "0.1.0",
  "bytes": "mutated",
  "derivedFrom": "control-untampered",
  "mutation": "added member `extra` to the nested `tool` object",
  "subject": { "kind": "corpus", "attestationHash": "sha256:..." },
  "needsCorpus": false,
  "toolVersion": "0.21.0",
  "expect": { "outcome": "refused", "reason": "unknown-member", "exit": 3 }
}
```

- **`payloadType`** MUST be a stable name this repository owns and MUST NOT be
  a file path or a verb name. Note 04 S4 gives the composition envelope to
  Statecraft and reserves the type names for spec-spine; this is where they are
  written down. Two names exist: `spec-spine/corpus-attestation` and
  `spec-spine/spec-attestation`.
- **`payloadSchemaVersion`** is the version the payload itself declares, which
  for a version case is deliberately not this build's.
- **`bytes`**, **`derivedFrom`** and **`mutation`** are §3.2's. `derivedFrom`
  and `mutation` are required when `bytes` is `mutated` and MUST be absent
  otherwise.
- **`subject`** MUST identify what the payload is about by digest, never by a
  local path: the corpus `attestationHash`, or a spec attestation's hash
  together with its `specSourceHash`. For an `authored` payload that has no
  computable hash, `subject` is `{ "kind": "none" }`, which is the honest
  answer and is not the same as omitting the member.
- **`toolVersion`** is the `tool.version` the payload carries. It drives
  §3.7's version rule.
- **`expect.outcome`** MUST be `accepted` or `refused`; `expect.reason` a
  member of §3.5's closed set, present exactly when the outcome is `refused`;
  `expect.exit` this repository's CLI exit code, recorded so a verifier can
  compare its own mapping without being required to adopt it.

**The index**, `fixtures/verifier/index.json`:

```json
{
  "schemaVersion": "0.1.0",
  "payloadTypes": ["spec-spine/corpus-attestation", "spec-spine/spec-attestation"],
  "reasons": ["unreadable-json", "missing-schema-version", "unsupported-major",
              "unknown-member", "duplicate-key", "non-canonical-bytes",
              "content-mismatch", "version-mismatch"],
  "cases": ["control-untampered", "..."]
}
```

`reasons` is the closed set, declared in the index rather than left implicit,
so a consumer reads the vocabulary instead of inferring it from the cases that
happen to be present. `cases` lists every case directory; a directory not
listed, or a listed directory that does not exist, fails the test (§3.8).

**Compatibility.** The set is versioned as a whole by the index's
`schemaVersion`, independently of the attestation schemas. Adding a case,
adding a reason to the closed set, or adding an optional member to `case.json`
is a MINOR. Removing or renaming a reason, removing a case, or changing a
case's expected outcome is a MAJOR, and changing an expectation MUST NOT happen
without the spec that changes the verifier: a fixture is this repository's
written claim about its own behavior, and quietly editing the claim to match a
regression is the failure the whole set exists to prevent.

### 3.5 The reason classes, measured against the shipped verifier

Every row below was measured on 2026-09-21 with `spec-spine 0.21.0`, by
generating an attestation over a one-spec corpus and running
`verify-attestation --recompute --json` against each payload. None of it is
inferred from reading the code.

| `reason` | Where the verifier decides | Exit | `--json` |
|---|---|---|---|
| `unreadable-json` | `serde_json` before anything else | `3` | `error.kind: "parse"` |
| `missing-schema-version` | the loose `schemaVersion` read, before the strict parse | `3` | `error.kind: "schema"` |
| `unsupported-major` | the MAJOR gate, before the strict parse | `3` | `error.kind: "schema"` |
| `unknown-member` | the strict parse (`deny_unknown_fields`) | `3` | `error.kind: "parse"` |
| `duplicate-key` | the strict parse: a repeated struct field is a parse error | `3` | `error.kind: "parse"` |
| `non-canonical-bytes` | the byte comparison, after a value match | `1` | `outcome: "contentMismatch"`, `differences: ["bytes are not the canonical serialization"]` |
| `content-mismatch` | the recompute, field by field | `1` | `outcome: "contentMismatch"`, `differences` naming each field |
| `version-mismatch` | the `tool.version` comparison, before the recompute | `1` | `outcome: "versionMismatch"` with `expected` and `actual` |

An `accepted` case is exit `0` with `outcome: "match"`.

**The two the draft got wrong, corrected here:**

- A payload whose MAJOR is current and whose **MINOR is ahead** does **not**
  verify. The draft asserted it must, on the reasoning that a MINOR is additive
  and a verifier refusing one would refuse every future release. That reasoning
  holds for the **MAJOR gate**, and the MAJOR gate does admit it. It does not
  hold for the verb: `verify_recompute` compares `schemaVersion` like every
  other field, deliberately (spec 068 §3.3: skipping it let an attestation
  claiming a schema this build never emitted recompute as a match), so a
  MINOR-ahead payload is a `content-mismatch` on the `schemaVersion` field.
  Measured: `differences: ["schemaVersion (0.9.0 -> 0.1.0)"]`, exit `1`.
  The set therefore carries the case with its **measured** expectation, and
  §3.9 records the contradiction rather than burying it.
- **`duplicate-key` is refused at the parse**, not at the byte comparison as a
  non-canonical form. Measured: exit `3`, `error.kind: "parse"`. Both are
  refusals, and a fixture that recorded the wrong layer would teach a consumer
  to implement the check in the wrong place.

### 3.6 Recompute cases carry their corpus

Spec 068's verifier is a **recompute** verifier: `--recompute` re-reads the
corpus and compares. A payload alone therefore cannot produce `accepted`,
`content-mismatch` or `version-mismatch`; only the refusals that happen before
the recompute are decidable from the bytes.

A case whose expected outcome needs a recompute MUST set `needsCorpus: true`
and MUST carry, under `corpus/`, the complete tree the payload attests: every
`spec.md` and the `spec-spine.toml`, if any, and nothing else. The corpus is
kept to one spec so a case directory stays a few kilobytes and a consumer can
read it.

This is what "carries or precisely references all inputs needed to reproduce
its verdict independently" costs, and it is not optional: a fixture whose
expected outcome cannot be reproduced from the fixture is a claim, not a test.

### 3.7 The version rule, so the set does not rot

`verify_recompute` compares the payload's `tool.version` against the verifying
build and returns `version-mismatch` when they differ, before any content
comparison (spec 021 FR-005). A committed positive control therefore stops
producing `accepted` the moment this repository releases again.

The harness MUST resolve this by asserting, never by skipping:

- when `case.toolVersion` equals the verifying build, the recorded
  `expect` MUST hold;
- when it differs, the outcome MUST be exactly `version-mismatch`, with
  `expected` equal to `case.toolVersion` and `actual` equal to the verifying
  build.

Both branches assert. The second is not a weakened form of the first: it is
FR-005's own contract, that a version difference is a named outcome and never a
false content mismatch and never a skip-as-pass.

Cases whose refusal happens before the recompute (`needsCorpus: false`) are
version-independent and take the first branch always.

### 3.8 The set describes the shipped verifier, and is proven to

A committed fixture is a claim about behavior, so it MUST be executed here.
`crates/spec-spine-cli/tests/verifier_fixtures.rs` walks the set, runs this
repository's own verifier over each payload, and asserts the recorded outcome
and reason under §3.7's rule. A case whose expectation no longer matches the
shipped verifier fails the build.

**Before running any case**, the test MUST assert all of:

1. the index parses and its `schemaVersion` MAJOR is understood;
2. the case list is **non-empty**;
3. the case list and the directories on disk are the **same set**, so a case
   added without being listed, or listed without existing, is a failure rather
   than a silent omission;
4. the control case is present and is a `producer` case;
5. every reason in the index's closed set is exercised by at least one case,
   and every case's reason is in the closed set;
6. the number of cases actually executed equals the number listed.

An empty or partial walk that asserts nothing is the failure mode this
repository has already met more than once, and clauses 2, 3 and 6 are there
because "the loop ran zero times" and "the loop ran and passed" are
indistinguishable without them.

### 3.9 Where the bytes come from, and what is not promised

Fixtures MUST be generated by a documented command and committed, not produced
at test time. A fixture regenerated on each run would re-serialize the bytes
whose exact spelling is the point, and would make the non-canonical-bytes case
unable to fail.

**Distribution.** The set is committed in-tree and is therefore already
obtainable by anyone who can read the repository. For a consumer who cannot,
the route is the published crate: `crates/spec-spine-core/fixtures/` ships
inside the `spec-spine-core` `.crate` archive, so

```sh
cargo package -p spec-spine-core --locked
tar xzf target/package/spec-spine-core-<version>.crate
# fixtures are at spec-spine-core-<version>/fixtures/verifier/
```

is a deterministic extraction route with a digest (the `.crate` SHA-256).
**Committing the fixtures is not publishing them**, and this spec MUST NOT be
read as a publication claim: the archive above exists locally until a release
uploads it, and spec 104's record is where a packaged artifact's digest is
written down.

Nothing here promises that a consumer's verifier is correct. It promises that a
consumer's verifier can be **tested against the same bytes, the same corpora
and the same outcomes this one is**, which is the whole of what a producer can
offer.

## 4. Out of scope

- **The neutral verifier's home** (note 04 D-6). A family decision; this spec
  is deliberately only the fixtures, which is what D-6 says spec-spine's part
  is.
- **Evidence-bundle composition**, receipt shapes, the in-toto mapping, and any
  consumer trust policy (note 04 R2, S4). §3.1.
- **Signing keys and key distribution.** A fixture carries no seal; signature
  verification is a separate axis with its own key-management question, and a
  committed scratch key is a liability rather than a fixture.
- **Changing the verifier.** Every expectation in the set is measured from the
  verifier as it stands, including the two the draft predicted wrongly. If a
  measured behavior is judged a defect, that is a new spec, and the fixture's
  expectation moves with it and not before (§3.4's compatibility rule).
- **Obligation, closure and scope records.** A separate, unadopted increment.
- **Publishing the set to a registry.** §3.9.

## 5. Resolved decisions

**D-1 (2026-09-21, this is buildable without a named consumer, unlike spec
102).** Three consumers are already named in note 04 §7 and all three were
offered these cases, so the need is recorded rather than hypothetical. What is
not decided here is where a neutral verifier lives, and §4 keeps it that way.

**D-2 (2026-09-21, owner correction: the ownership claim on the two source
files is dropped).** Packaging fixtures does not change verifier behavior, and
nothing in this spec edits `attest.rs` or `verify_attestation.rs`. They are
`references`, which is non-owning.

**D-3 (2026-09-21, owner correction: producer bytes and mutated bytes are
different things).** §3.2. The draft described every payload as unmodified
`attest` output, which is false for every tamper case and would have taught a
consumer that a mutated payload is what the producer emits.

**D-4 (2026-09-21, owner correction: expectations are measured, not
predicted).** §3.5 and §3.9. The draft asserted that a MINOR-ahead payload must
verify. It does not, and the reason is a deliberate decision in spec 068. The
set records the measured behavior and §3.5 names the contradiction between the
MAJOR gate's admission and the recompute's comparison, so a later reader
decides it on purpose rather than discovering it.

**D-5 (2026-09-21, build: the harness is a CLI test, not a core one).** See
§2. The fixtures record the answer the *verb* gives, and that answer is
assembled in the CLI: the loose `schemaVersion` read, the MAJOR gate, the
strict parse, the recompute, the byte comparison, and an exit code. A core test
would have to reassemble that sequence from library calls and would then be
asserting a second implementation of it.

**D-6 (2026-09-21, build: the generator is committed inside the fixture
directory).** §3.9 requires the bytes to come from a documented command, and a
command documented only in prose is one nobody can run. `generate.py` sits in
the fixture directory, covered by the directory unit this spec already claims.
Re-running it over an unchanged tree rewrites the same bytes.

**D-7 (2026-09-21, build: the harness is mutation-tested, three ways).** A
guard no fixture can reach is not a guard. Each was made to fire and then
reverted: removing a case from the index's `cases` list fails the same-set
assertion; changing one case's `expect.reason` fails with the observed reason
named in the message; removing the control's `corpus/` fails the `needsCorpus`
consistency assertion. Eleven cases, and every declared reason class is
exercised, which the harness asserts rather than leaving to inspection.

**D-8 (2026-09-21, build: the case the draft named that cannot exist).** The
draft's acceptance named `reformatted-same-values` and `minor-ahead-verifies`.
The first exists. The second does not and cannot: the measured behavior is a
content mismatch, so the case is `minor-ahead-content-mismatch`. Naming a case
after a behavior the verifier does not have would have baked the draft's wrong
prediction into the artifact three consumers are asked to test against.

**D-9 (2026-09-21, build: the `planned` flags come off with the completion).**
See §2.

## Verification

Behavioral. The test executes every fixture against the shipped verifier under
§3.7's rule and refuses an empty, partial or unlisted run.

Written to fail against the tree this spec is filed on: no fixture set and no
test file exist.

```verify:cli
# 3.3, 3.4: the set and its index exist.
test -d crates/spec-spine-core/fixtures/verifier
test -f crates/spec-spine-core/fixtures/verifier/index.json
# 3.8.4: the control case, without which a suite of refusals asserts nothing.
test -f crates/spec-spine-core/fixtures/verifier/control-untampered/payload.json
test -f crates/spec-spine-core/fixtures/verifier/control-untampered/case.json
# 3.6: the control is a recompute case, so it carries the corpus it is about.
test -f crates/spec-spine-core/fixtures/verifier/control-untampered/corpus/specs/000-bootstrap/spec.md
# 3.5: the two cases the draft predicted wrongly are present under their
# measured names, so a regression to the predicted behavior fails.
test -d crates/spec-spine-core/fixtures/verifier/minor-ahead-content-mismatch
test -d crates/spec-spine-core/fixtures/verifier/duplicate-key
# 3.4: the payload type name is owned here and is not a path or a verb.
grep -q 'spec-spine/corpus-attestation' crates/spec-spine-core/fixtures/verifier/index.json
# 3.8: the whole set is executed against the shipped verifier, with the
# non-empty, same-set, closed-reason and executed-count guards.
cargo test -p spec-spine-cli --test verifier_fixtures --locked
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
