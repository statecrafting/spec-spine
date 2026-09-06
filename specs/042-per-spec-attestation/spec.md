---
id: "042-per-spec-attestation"
title: "`attest --spec <id>`: a signed record scoped to one spec"
status: draft
kind: "tooling"
created: "2026-09-06"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "023-ledger-seal"
  - "024-index-sharding"
  - "041-completion-held-to-claims"
amends:
  # 3.1 states a MUST NOT about `attest`'s exit code that covers the
  # corpus-scoped verb too, which is 023's territory and a rule 023 never
  # carried. 023's own text is untouched (spec 040).
  - "023-ledger-seal"
extends:
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/src/attest.rs", nature: additive }
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/tests/attest.rs", nature: additive }
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # The schema constant and its pin test; 000 floors the types crate.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/attest.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
  - { unit: { kind: file, path: "specs/031-registry-freshness-check/spec.md" }, role: context }
summary: >
  Spec 023 attests the corpus: one payload, one hash, one optional Ed25519 seal
  over the whole ledger. The unit of work in a governed build is one spec, and a
  corpus-scoped attestation cannot answer "was this spec's territory sound when
  its work was declared done" without being reduced by hand. This spec adds
  `attest --spec <id>`, emitting a `SpecAttestation` scoped to a single spec: its
  own source hash, the resolved location and content hash of every owning unit it
  claims, and the verdicts restricted to it. It reuses 023's split exactly, the
  payload pure and reproducible with the wall clock and the signer identity in
  the detached seal, so the attested fact stays a function of `(config, file
  contents)` while the act of attesting is dated and attributed. It stays
  on-demand and gitignored like 023's, deliberately: the design note wanted a
  committed per-spec bundle, and committing one would restale on every edit to
  any claimed unit and would need its own freshness gate, which buys churn rather
  than assurance. A consumer that wants durable evidence receives the bundle; it
  does not need the repository to store it. One constraint is load-bearing and
  stated normatively: no lint rule may consume a `SpecAttestation`, because the
  payload records the lint verdict and a lint that read it would grade itself.
---

# 042: Per-spec attestation

Wave 2 of `docs/design/02-agentic-builder-substrate.md`, and the half of it that
survives contact with `.gitignore`. See §5.

## 1. Purpose

Spec 023 emits a `CorpusAttestation`: `inputsManifestHash`, `registryHash`, and
the `compile` / `lint` / optional `couple` verdicts, hashed and optionally sealed
with a detached Ed25519 signature. It answers "was the corpus sound at this
revision", reproducibly and offline.

It cannot answer the question a governed build actually asks, which is scoped to
one spec: *this* spec's territory, *this* spec's units, *this* spec's verdicts.
A corpus attestation covering forty specs says nothing retrievable about the one
that just had work done on it, and a consumer that wants a durable record per
unit of work has to either store forty-spec snapshots per spec or reduce the
payload itself, which means reimplementing the hashing it was supposed to trust.

The gap is scope, not mechanism. Every part of 023 that matters here (canonical
payload, pure hash, detached seal, `--recompute` verification) applies unchanged
to a narrower subject.

## 2. Territory

`attest.rs` in both crates (the DTO and the builder), `cmd_attest.rs` and
`verify_attestation.rs` for the flag and its verification path, the core facade,
and the schema constant with its pin test. No committed artifact changes and no
existing payload changes: `CorpusAttestation` is untouched, and a repo that never
passes `--spec` behaves exactly as it does today.

## 3. Behavior

### 3.1 The payload

```json
{
  "schemaVersion": "0.1.0",
  "tool": { "name": "spec-spine", "version": "0.13.0" },
  "specId": "036-configured-corpus-root",
  "specSourceHash": "<sha256 of the normalized spec.md bytes>",
  "lifecycle": { "status": "approved", "implementation": "complete" },
  "units": [
    { "unit": { "kind": "file", "path": "crates/spec-spine-core/src/couple.rs" },
      "contentHash": "<sha256 of the normalized file bytes>" }
  ],
  "verdicts": {
    "compile": { "ok": true },
    "resolution": { "ok": true },
    "lint": { "ok": true, "findingsHash": "<sha256 over this spec's canonical findings>" }
  }
}
```

- `specSourceHash` uses the project's standing normalization (BOM stripped,
  CRLF/CR to LF), so the payload is platform-independent like every other hash
  here.
- `units` lists **owning** units only, in the registry's canonical order, each
  with the content hash of what it resolved to. Non-owning `references` units are
  excluded: spec 034 settled that a cited file is not a claimed one, and an
  attestation of territory must not assert authority the gate does not.
- `lifecycle` records the spec's own `status` and `implementation` as declared at
  attestation time, with `implementation` omitted when that key is absent from
  the spec's frontmatter. It is what makes
  `resolution.ok: false` interpretable: an external consumer must be able to tell
  a phantom unit in a spec that is openly being built from a phantom unit in a
  spec asserting it is finished, and those are the same `false` with very
  different meanings. Without it the discriminator lives only in the indexer's
  severity tier, which is not in this payload and should not be: tiers are a
  local policy that spec 041 is already changing, while `status` and
  `implementation` are the author's own words and are what a record should keep.
- `resolution.ok` is true when every owning unit this spec claims resolves to an
  existing location, and false otherwise. It records the **fact**, never the
  indexer's severity tier for it: an in-flight spec whose phantom unit is only a
  `W-001` still attests `resolution.ok: false`. Tying the flag to the tier would
  make an attestation report "resolution ok" for a unit that does not exist,
  which is the one thing this payload exists to make impossible, and it would
  also make the record depend on the subject's lifecycle rather than on the
  corpus.
- `lint.ok` and `findingsHash` cover the findings attributed to this spec, using
  023's existing findings-hash construction so a changed finding set is
  detectable even when `ok` is unchanged.

A failing verdict never suppresses the payload. `attest --spec <id>` emits a
complete, hashable, signable attestation whether the verdicts are true or false,
and exits `0` for having produced one: it is a record, not a gate. A consumer
decides what a `false` means to it, and an attestation that refused to exist when
the news was bad would be worth nothing as evidence.

**The exit code of `attest` therefore MUST NOT be read as a verdict.** `0` means
an attestation was written, and nothing about what it says. This is the one verb
in the tool where that is true, so it is stated rather than left to be
discovered: `lint`, `couple`, `index check` and `compile --check` all put their
verdict in the exit code, and a caller who wants a gate uses one of them and
reads this payload for the record. `attest --spec X && next_step` is a misuse,
and a caller that means "stop if the spec is unsound" should run the gate verb
that says so.

The behavior is inherited, not invented here: `attest` has always written its
payload and returned `0` irrespective of the verdicts inside it, and spec 023
never wrote that down. This spec does, for both scopes.

Stating it for the corpus-scoped verb is a change to spec 023's contract, since
023 owns `attest` and carried no such rule, so this spec declares
`amends: ["023-ledger-seal"]`. The alternative was to scope the MUST NOT to
`--spec` alone and leave the corpus verb undocumented, which would put two
different exit-code contracts on one binary for no reason other than which spec
happened to notice first. Per spec 040 the amendment is declared here and
`specs/023-ledger-seal/spec.md` is not edited; the inbound view is
`spec-spine registry relationships 023-ledger-seal`.

A `--fail-on-false` gating flag was considered and rejected. A record verb that
can be configured to refuse invites being used as a gate, which puts a second,
weaker copy of the gate chain behind a flag; the verbs that already refuse are
the ones to call.

There is no `couple` verdict. Coupling is a property of a diff between two
revisions, not of a spec at one revision, and 023 already carries the
corpus-scoped version for consumers that want it.

### 3.2 Purity, and where the clock lives

The payload MUST be a pure function of `(config, file contents)`: no clock, no
environment, no git. Re-running `attest --spec <id>` on an unchanged corpus at
the same `tool.version` MUST yield byte-identical output.

The wall-clock instant and the signer identity live in the detached `LedgerSeal`,
exactly as spec 023 established: `alg`, `keyId`, `signedAt`, `sig` over the
32-byte attestation hash. The attested fact stays reproducible while the act of
attesting is dated and attributed.

This is the constraint the design note names first, and it is the reason a build
record (which agent, which session, what it cost) can never be folded into this
payload. Such a record is a legitimate artifact for a consumer to keep; it is not
this one, and mixing them would make determinism, the project's central claim,
false by construction.

### 3.3 It is not committed, and that is a decision

The output goes to `<derived_dir>/attestation/by-spec/<id>.json`, under the
existing `.gitignore` entry for `.derived/attestation/` (spec 023's, unchanged).
It is on-demand: nothing in the gate chain produces it, and no CI job requires it.

The design note proposed the opposite, a committed per-spec bundle giving "build
history in the repo as committed truth". Checking that against the repo argues
against it:

- **It would restale constantly.** The payload hashes every claimed unit's
  content, so *any* edit to *any* owned file invalidates that spec's attestation.
  Where the index shard restales on structural change, this would restale on
  every line of code, in every PR, for every spec touched.
- **It would need its own freshness gate.** The note's own constraint says every
  committed artifact class needs one designed with it, which is the lesson spec
  031 exists to record. That is a fourth committed tree and a fifth gate verb.
- **It would buy little.** A committed attestation proves what the working tree
  already shows, and the signature (the part a compliance consumer actually
  wants) is worth no more for sitting in git than for being handed over.

So the artifact is produced for whoever asks and kept by whoever needs it. A
consumer that wants durable evidence receives the bundle and retains it; the
repository is not the right store for a record about the repository.

### 3.4 No lint rule may consume a `SpecAttestation`

This is normative and load-bearing. The payload records `lint.ok`, so a lint rule
conditioned on the presence or content of an attestation would be grading its own
output: the attestation is valid because lint passed, and lint passes because the
attestation is valid.

Spec 041 was designed around this and needs no attestation, which is why the
completion claim is held by a severity tier in the indexer rather than by
evidence in a bundle. Any future rule wanting attestation-backed acceptance must
either live outside `lint` or use a payload with no lint verdict in it, and must
say which.

This prohibition is a convention backed by review, not a mechanism: nothing stops
a contributor adding such a rule and passing CI, because "this lint rule reads an
attestation" is not a property the build can test for. It is written here, in the
spec that owns the payload, so that the argument is available at review time to
whoever proposes one. That is the same honesty spec 040 §4 records about its own
rule, and the same reason: a convention that pretends to be enforced is worse
than one that admits what it is.

### 3.5 Verification

`verify-attestation --spec <id>` extends the existing verbs to the narrower
subject, with both existing modes:

- `--recompute` re-reads the corpus and checks the payload reproduces, keyless.
  A `tool.version` mismatch stays a distinct, named outcome (023 FR-005), never a
  false content mismatch.
- `--signature` checks the detached seal against a supplied public key.

Exit codes follow the standing contract: `1` for a failed verification, `3` for
I/O, parse or schema trouble.

### 3.6 Versioning

`SPEC_ATTESTATION_SCHEMA_VERSION` joins the constants in `version.rs` at `0.1.0`,
pinned by a test in `types/tests/dtos.rs` like every other schema version, and is
independent of the registry, index and corpus-attestation versions. An external
consumer pins the shape of the evidence it verifies without pinning the ledger it
was derived from, which is the whole point of a bundle format crossing a trust
boundary.

### 3.7 Tests (minimum)

- The payload is byte-identical across two runs on an unchanged corpus, and
  across a `--repo` invoked from a different working directory.
- Editing an owning unit changes exactly that unit's `contentHash` and the
  attestation hash, and nothing else.
- Editing the spec's own `spec.md` changes `specSourceHash`.
- `lifecycle` mirrors the spec's declared `status` and `implementation`, and
  omits `implementation` when the key is absent (as `000-spec-spine-bootstrap`
  has it). Two specs differing only in lifecycle produce different payloads.
- A `references` unit appears in no `units` entry.
- An unresolved owning unit yields `resolution.ok: false` and still produces a
  payload, rather than refusing: the attestation records the verdict, it does not
  gate on it.
- `--recompute` succeeds on an unchanged corpus and fails with a named outcome
  after any owned file changes; a `tool.version` mismatch is distinguishable from
  a content mismatch.
- `--sign` then `--signature` round-trips; the seal's `signedAt` differs between
  two signings of a byte-identical payload, and the payload hash does not.
- `attest` with no `--spec` is unchanged, byte for byte, from its current output.
- Both scopes exit `0` on false verdicts. A corpus with a failing lint verdict
  and a spec with `resolution.ok: false` each write their attestation and return
  `0`. This is the test that holds §3.1's MUST NOT: the rule now covers the
  corpus-scoped verb by amendment to 023, and a normative claim about an exit
  path deserves a test that exercises it rather than an assumption that it still
  behaves as it did when nobody had written the rule down.

## 4. Out of scope

**Committing the artifact, and any freshness gate for it.** §3.3.

**Redaction.** The design note asks who owns the guarantee about what a bundle
may not contain. This payload carries hashes, paths and verdicts, never file
contents, so nothing here needs redacting; a bundle format that later carries
transcripts or diffs is a different artifact and must answer the question itself.
Naming the boundary is the most this spec can honestly do.

**Transport, retention, countersigning.** What a hosted consumer does with a
received bundle, including chaining bundles into a transparency log, is that
consumer's design and deliberately not spec-spine's.

**Attesting many specs in one call.** `--spec` takes one id. A caller wanting
several runs it several times, and a batch flag can be added additively if a real
consumer ever wants one.

## 5. Relationship to the design note

The note's wave 2 (G2) asked for a per-spec bundle, committed, with the
completion gate consuming it. Two findings from the code split that apart:

1. `.gitignore` already keeps `.derived/attestation/` out of the tree, and the
   reasons that made 023 on-demand apply with more force per spec (§3.3).
2. The completion claim needs no bundle to be held to its territory (spec 041),
   so the consumer the note paired this with does not exist, and §3.4 explains
   why that pairing could not have worked in `lint` anyway.

What remains is worth building on its own terms: a signed, reproducible,
offline-verifiable record scoped to one unit of work, for a consumer outside this
repository. That is a narrower claim than the note made, and one that survives
being checked.
