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
  cannot run either. This emits them as a named, versioned fixture set: bytes
  the producer owns, with the outcome each case must reach.
extends:
  - spec: "068-a-verifier-checks-the-bytes-it-was-given"
    paths:
      - "crates/spec-spine-core/src/attest.rs"
      - "crates/spec-spine-cli/src/verify_attestation.rs"
    nature: additive
establishes:
  - { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/verifier_fixtures.rs", planned: true }
references:
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
in TypeScript against a stored evidence bundle has no way to execute them, and
the table in `docs/authority-evidence.md` §8 is a measurement report, not an
input. So the four contract gaps spec 068 closed are asserted for this
repository's own verifier and for nobody else's, and a consumer that gets them
wrong finds out from a tampered payload rather than from a test.

The gap is narrow and so is this spec. spec-spine emits the **bytes and the
expected outcomes**. Everything a bundle needs around them stays with the
consumer.

## 2. Territory

A fixture directory under `crates/spec-spine-core/fixtures/verifier/` and the
test that proves its cases still describe the shipped verifier, plus the two
files that already own strict payload validation.

Both new paths are declared `planned: true` (spec 063): they are this spec's
territory and they are not written yet, so the claim is a declaration rather
than an unresolved unit.

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
a signature over anything but the fixture's own payload, or any statement that
a fixture's subject is trustworthy. Spec 021 §7's vocabulary holds: these are
attestations, recomputable records, and nothing here is testimony.

It MUST NOT reintroduce a distributed harness in any form. A fixture is data a
verifier reads; it is not an installed file, not a template and not a generator.

### 3.2 What a fixture case is

Each case is a directory holding exactly:

1. **`payload.json`**, the stored bytes, written by `attest` and never
   re-serialized. Spec 068 makes a re-canonicalized copy fail verification, so
   a fixture that round-tripped its own bytes would assert the opposite of the
   contract it exists to pin.
2. **`case.json`**, a record with a fixed shape:

```json
{
  "schemaVersion": "0.1.0",
  "id": "unknown-member-nested",
  "payloadType": "spec-spine/corpus-attestation",
  "payloadSchemaVersion": "1.0.0",
  "subject": { "kind": "corpus", "attestationHash": "sha256:..." },
  "mutation": "added member `prCouple` to a nested object",
  "expect": { "outcome": "refused", "reason": "unknown-member", "exit": 3 }
}
```

- **`payloadType`** MUST be a stable name this repository owns and MUST NOT be
  a file path or a verb name. Note 04 S4 gives the composition envelope to
  Statecraft and reserves the type names for spec-spine; this is where they are
  written down.
- **`subject`** MUST identify what the payload is about by digest, never by a
  local path: the corpus `attestationHash`, or a spec's `SpecAttestation` hash
  together with its `specSourceHash`.
- **`expect.outcome`** MUST be one of `match`, `mismatch` or `refused`, and
  `expect.reason` a member of a closed set the fixture index declares. `exit`
  is this repository's CLI exit code, recorded so a verifier can compare its own
  mapping without being required to adopt it.

### 3.3 The cases the set MUST contain

Derived from the measured table in `docs/authority-evidence.md` §8 and from
spec 068's acceptance, not invented here:

**Tamper, at minimum one case each.** An untampered control that verifies; an
unknown member at top level; an unknown member nested; an unknown member in a
per-spec attestation; a reformatted copy with identical values; a flipped
verdict; a changed `tool.version`; a duplicate key.

**Cross-version, at minimum one case each.** A corpus `schemaVersion` with an
unsupported MAJOR; a per-spec `schemaVersion` with an unsupported MAJOR; a
payload whose MAJOR is current and whose MINOR is ahead, which MUST verify,
because a MINOR is additive and a verifier that refuses one would refuse every
future release.

The control case is not optional. A suite of refusals alone passes for a
verifier that refuses everything.

### 3.4 The set describes the shipped verifier, and is proven to

A committed fixture is a claim about behavior, so it MUST be executed here:
`crates/spec-spine-core/tests/verifier_fixtures.rs` walks the set, runs this
repository's own verifier over each `payload.json`, and asserts the recorded
outcome and reason. A case whose expectation no longer matches the shipped
verifier fails the build.

The test MUST assert, before running any case, that the set is **non-empty and
contains the control case**. An empty walk that asserts nothing is the failure
mode this repository has already met more than once.

### 3.5 Where the bytes come from, and what is not promised

Fixtures MUST be generated by a documented command and committed, not produced
at test time. A fixture regenerated on each run would re-serialize the bytes
whose exact spelling is the point.

The set is versioned as a whole by `schemaVersion` on its index. Adding a case
is additive. Changing a case's expected outcome is a change to what this
repository claims its verifier does, and MUST NOT happen without the spec that
changes the verifier.

Nothing here promises that a consumer's verifier is correct. It promises that a
consumer's verifier can be **tested against the same bytes and outcomes this
one is**, which is the whole of what a producer can offer.

## 4. Out of scope

- **The neutral verifier's home** (note 04 D-6). A family decision; this spec is
  deliberately only the fixtures, which is what D-6 says spec-spine's part is.
- **Evidence-bundle composition**, receipt shapes, and the in-toto mapping (note
  04 R2, S4).
- **Signing keys and key distribution.** A fixture carries a seal made with a
  scratch key or none; trust in a key is the consumer's.
- **Obligation, closure and scope records.** They reference a snapshot digest
  and are a separate, unadopted increment.
- **Publishing the set to a registry.** Committed in-tree is the whole delivery;
  where a release ships it is a packaging question for the release runbook.

## 5. Resolved decisions

*(Filed as a draft. Decisions taken during the build are appended here.)*

**D-1 (2026-09-21, this is buildable without a named consumer, unlike spec
102).** Three consumers are already named in note 04 §7 and all three were
offered these cases, so the need is recorded rather than hypothetical. What is
not decided here is where a neutral verifier lives, and §4 keeps it that way.

## Verification

Written to fail against the tree this spec is filed on: no fixture set exists.

```verify:cli
# 3.2: the set exists and every case carries both files.
test -d crates/spec-spine-core/fixtures/verifier
test -f crates/spec-spine-core/fixtures/verifier/index.json
# 3.3: the control case, without which a refusal suite asserts nothing.
test -d crates/spec-spine-core/fixtures/verifier/control-untampered
test -f crates/spec-spine-core/fixtures/verifier/control-untampered/payload.json
test -f crates/spec-spine-core/fixtures/verifier/control-untampered/case.json
# 3.3: the three cases most easily left out, by name.
test -d crates/spec-spine-core/fixtures/verifier/unknown-member-nested
test -d crates/spec-spine-core/fixtures/verifier/reformatted-same-values
test -d crates/spec-spine-core/fixtures/verifier/minor-ahead-verifies
# 3.2: a payload type name exists and is not a path or a verb.
grep -q 'spec-spine/' crates/spec-spine-core/fixtures/verifier/index.json
# 3.4: the set is executed against the shipped verifier, and the empty-walk
# guard is present by name.
test -f crates/spec-spine-core/tests/verifier_fixtures.rs
grep -q 'control' crates/spec-spine-core/tests/verifier_fixtures.rs
cargo test -p spec-spine-core --test verifier_fixtures --locked
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
