---
id: "085-a-verifier-checks-the-bytes-it-was-given"
title: "A verifier checks the bytes it was given"
status: approved
kind: "tooling"
created: "2026-09-11"
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "023-ledger-seal"
  - "042-per-spec-attestation"
  - "067-the-docs-name-what-adopters-derived"
  - "083-an-attestation-covers-the-territory-it-claims"
establishes:
  # 3.6: the tamper and version matrix, end to end through the binary.
  - { kind: file, path: "crates/spec-spine-cli/tests/verify_attestation_bytes.rs" }
extends:
  # 3.2: every attestation and seal DTO refuses a member it does not know.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-types/src/attest.rs", nature: additive }
  # 3.1 and 3.3: the verifier reads the file once, gates the version, and hashes what it read.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  # 3.3: the corpus recompute compares schemaVersion like every other member.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/src/attest.rs", nature: additive }
  # 3.4: the facade parses strictly, gates the version, and accepts the text form.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.6: library-level guards beside the existing attestation tests.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/tests/attest.rs", nature: additive }
  # 3.5: the versions table and the unknown-member sentence.
  - { spec: "067-the-docs-name-what-adopters-derived", unit: "docs/schema-versioning.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/authority-evidence.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
  - { unit: { kind: file, path: "specs/084-a-short-id-names-the-same-spec-at-every-verb/spec.md" }, role: context }
summary: >
  `verify-attestation` decides on a different object from the one it was handed.
  It deserializes the attestation without refusing members it does not know, and
  it checks the seal against a hash of its own re-serialization of what it
  parsed, never against the file's bytes. Measured at 75181a5: a sealed
  attestation with an injected `"prCouple": {"ok": true}` verifies as `match`
  and `valid` at exit 0, in both the corpus and the per-spec scope, and so does
  a reformatted copy. Separately, the corpus recompute never compares
  `schemaVersion`, so an attestation claiming `9.0.0` recomputes as `match` at
  exit 0, while the per-spec path reports an unnamed content mismatch. Spec 023
  AC-4 already requires a single tampered payload byte to fail the signature,
  and the attestation module documents that loaders reject an unknown MAJOR, so
  this spec extends rather than amends: the verifier reads the file once, both
  modes decide on those bytes, every attestation and seal DTO refuses unknown
  members at exit 3, an unknown schema MAJOR is refused at exit 3 in the
  loaders' own words, and the corpus recompute compares the version. Nothing
  `attest` emits changes, and every attestation and seal it has ever written
  still verifies, provided the bytes were kept.
---

# 085: A verifier checks the bytes it was given

## 1. Purpose

Spec 023 made the corpus attestation "the most verifiable link in any audit
chain that contains it", and spec 042 extended the same record to one spec for
consumers outside this repository. Both are checked by `verify-attestation`,
which a consumer runs precisely so that it does not have to reimplement the
check. Measured on 2026-09-11, and the seal row on 2026-09-12, against a
sealed pair written by `attest --sign` with a scratch key:

| What was done to the stored file | `--recompute` | `--signature` | Exit |
|---|---|---|---|
| nothing | match | valid | 0 |
| a top-level member added: `"prCouple": {"ok": true}` | match | valid | **0** |
| a nested member added under `verdicts.compile` | match | valid | **0** |
| a member added to a per-spec attestation's first unit | match | valid | **0** |
| reformatted: newlines removed, values unchanged | match | valid | **0** |
| an unknown member added to the seal: `"revokedAt"` | | valid | **0** |
| corpus `schemaVersion` changed to `9.0.0`, recompute only | **match** | not run | **0** |
| corpus `schemaVersion` changed to `0.2.0`, recompute only | **match** | not run | **0** |
| per-spec `schemaVersion` changed to `9.0.0` | mismatch, "tool.name or schemaVersion" | not run | 1 |
| `tool.version` changed | `versionMismatch` | | 1 |
| a verdict flipped | mismatch naming it | invalid | 1 |
| a key duplicated | parse error | | 3 |

The consumer that motivated per-spec attestations reads the file with its own
parser after spec-spine says it is valid. In the bold rows that consumer reads
members spec-spine never verified. A verdict about one object has been
reported as a verdict about another.

Three mechanisms produce the rows:

- **Unknown members are dropped.** None of `CorpusAttestation`,
  `SpecAttestation`, their nested types or `LedgerSeal` refuses an unknown
  field, so serde discards it. `docs/schema-versioning.md` says the artifact
  DTOs refuse unknown fields; only `Config`, the edge items and `Unit` do.
- **The seal is checked over a re-serialization.** `verify_attestation.rs`
  computes `attestation_hash(&loaded)`, the SHA-256 of the canonical JSON of the
  parsed value, and checks the seal against that. For a file `attest` wrote the
  two are the same bytes; for any other file they are not, and the difference is
  exactly what the bold rows exploit.
- **The corpus recompute compares member by member and skips one.**
  `verify_recompute` compares `tool.name`, both input hashes and each verdict,
  and never `schemaVersion`. The per-spec path compares the whole value, so it
  notices, but its report cannot say what it noticed.

Spec 023 AC-4 requires "a single tampered payload byte fails the signature",
and FR-004 has `--recompute` check "the emitted hashes against the
attestation". The attestation DTO module documents that loaders reject an
unknown MAJOR. The code does none of the three for these inputs.

## 2. Territory

- `crates/spec-spine-types/src/attest.rs`: `deny_unknown_fields` on every
  attestation and seal type.
- `crates/spec-spine-cli/src/verify_attestation.rs`: one read of the file, the
  version gate, the seal over the bytes read, and the byte comparison in
  `--recompute`.
- `crates/spec-spine-core/src/attest.rs`: `verify_recompute` compares
  `schemaVersion`; both recompute paths name it.
- `crates/spec-spine-core/src/lib.rs`: the two verification facades.
- `crates/spec-spine-core/tests/attest.rs` and the new
  `crates/spec-spine-cli/tests/verify_attestation_bytes.rs`: the guards.
- `docs/schema-versioning.md`: the versions table and the sentence about which
  DTOs refuse unknown fields.

Drafts 084 and 085 both extend `verify_attestation.rs`, `core/src/attest.rs`
and `core/src/lib.rs`. Neither needs the other's behavior, so there is no
`depends_on` edge, but they are not safe to build in parallel: build them one
after the other, and the second rebases.

## 3. Behavior

### 3.1 The bytes are read once, and both modes decide on them

`verify-attestation` MUST read the attestation file once and hold its exact
bytes. Every mode MUST decide on those bytes:

- `--signature` MUST verify the seal over SHA-256 of the bytes read, not over a
  re-serialization of their parse. For every file `attest` has written, the
  file is the canonical JSON, so the two digests are equal and every existing
  seal verifies exactly as before.
- `--recompute` MUST compare the recomputed canonical bytes with the bytes
  read. When the parsed values are equal and the bytes are not, the outcome is
  `contentMismatch` with the difference `bytes are not the canonical
  serialization`, at exit 1.

The rule is the one a consumer can check without trusting anything: the digest
a record is referenced by is SHA-256 over the stored bytes, and a verifier that
approves a record approves those bytes.

### 3.2 An unknown member is refused

`CorpusAttestation`, `SpecAttestation`, `ToolStamp`, `CompileVerdict`,
`LintVerdict`, `CoupleVerdict`, `Verdicts`, `SpecVerdicts`, `ResolutionVerdict`,
`AttestedUnit`, `AttestedLifecycle` and `LedgerSeal` MUST refuse unknown
fields. A file carrying a member this build does not know MUST fail to load, at
exit 3 as a parse error naming the member, before either mode runs.

An unknown member is a claim this build cannot evaluate. A verifier that drops
it has verified a smaller object than the one it was handed. The refusal is
also the honest answer across versions: a newer producer's additive member,
met by an older verifier, becomes "cannot read" rather than today's false
`valid`, or the false `invalid` that 3.1 alone would give.

### 3.3 An unknown schema MAJOR is refused, and the corpus recompute compares the version

Before either mode runs, the verifier MUST parse the payload's `schemaVersion`
as `MAJOR.MINOR.PATCH` and refuse a MAJOR other than this build's for that
payload type, at exit 3, with the message the registry and index loaders use
(`attestation schema MAJOR 9 is unsupported (this build understands 0.x)`). A
value that is not semver is refused at exit 3 too.

The corpus recompute MUST compare `schemaVersion` like every other member, and
both recompute paths MUST name it (`schemaVersion (0.2.0 -> 0.1.0)`) rather
than report `tool.name or schemaVersion`. The order of checks is: the MAJOR
gate, then spec 023 FR-005's `tool.version` comparison, then content, so an
unreadable payload is never reported as a version or content difference.

### 3.4 The facade holds the same rules

`verify_attestation_json` and `verify_spec_attestation_json` receive the
attestation inside a JSON request, so the file's bytes never reach them. They
MUST apply 3.2 and 3.3 to the value they receive. They MUST also accept
`attestationText`, a string holding the attestation's exact bytes, as an
alternative to `attestation`, and apply 3.1 to it. A request carrying both, or
neither, is refused as a parse error. The response shape is unchanged, which
keeps the CLI's `--json` report and the facade byte-identical for the same
inputs (spec 037 3.1).

### 3.5 The documentation says what is true

`docs/schema-versioning.md` MUST list every versioned artifact at its current
constant (registry `1.2.0`, index `1.1.0`, corpus attestation `0.1.0`, per-spec
attestation `0.1.0`, verdict envelope `0.3.0`, `build-meta` `0.1.0`, config
`0.1.0`), and MUST say which DTOs refuse unknown fields: `Config`, the edge
items, `Unit`, and after this spec every attestation and seal type. The
registry and index DTOs do not, and their per-shard MAJOR gate is their guard.
It MUST tell a consumer to store the bytes `attest` wrote, since a
re-serialized copy no longer verifies under 3.1.

The parenthetical is the set as it stands today; the requirement is each
artifact's current constant, not these literals. Draft 088 moves the verdict
envelope to its next MINOR when it adds the `delta` verb token, so whichever of
the two builds second writes the value the constant holds by then.

### 3.6 What must keep working

- `attest` emits byte-identical output in both scopes, and
  `ATTESTATION_SCHEMA_VERSION` and `SPEC_ATTESTATION_SCHEMA_VERSION` do not
  change. This spec changes what a verifier accepts, never what a producer
  writes.
- Every attestation and seal `attest` has written, stored unmodified, verifies
  with the outcome and exit code it has today. That includes corpus
  attestations and file-unit per-spec attestations written by `v0.18.0`, which
  recompute as `match` under the current build (measured).
- The rows of 1 that are not bold keep their outcomes and exit codes.
- Spec 083's fixtures keep passing.

## 4. Out of scope

**A build identity.** Two binaries that print `0.18.0` can behave differently,
and `tool.version` cannot tell them apart. That is a release-process question
recorded as design/04 D1, not a verifier rule.

**A framed digest construction.** `hash::content_hash` does not frame content,
so two trees can fold to one unit hash. Changing it would change every
historical digest; draft 087 proposes a framed construction for new record
types only.

**Refusing unknown members in the registry and index DTOs.** Their loaders gate
the MAJOR per shard, and a committed shard is regenerated rather than handed
between parties. Whether they should also refuse unknown members is a separate
question with a restale cost.

**Trust in the signer.** Which keys a consumer accepts stays the consumer's
policy (spec 023 6).

**The short id at `verify-attestation --spec`.** Draft 084.

## 5. Resolved decisions

**D-1 (2026-09-11): `extends`, not `amends`.** Spec 023 AC-4 already requires a
tampered payload byte to fail the signature, FR-004 already has the recompute
check the attestation's hashes, and the DTO module already documents the MAJOR
rule. This spec makes the code do what those say. It changes a verdict only for
inputs spec-spine never emitted. The reasoning is spec 083 D-1's: declaring an
amendment would record a change of requirement that did not happen.

**D-2 (2026-09-11): exit 3 for an unknown member or MAJOR, exit 1 for bytes that
are not canonical.** A member or a MAJOR this build does not know makes the
file one it cannot read, which is the standing "3 for I/O, parse or schema
trouble" (spec 042 3.5). A file whose values are right and whose bytes are not
the ones attested has been read and has failed verification, which is 1. The
alternative, one `unsupported` outcome at exit 1 for all three, is recorded as
design/04 D2 for the reviewer.

**D-3 (2026-09-11): no schema version moves.** Nothing emitted changes, so no
payload gains or loses a member. A version bump would tell consumers the shape
changed when only the reader did.

**D-4 (2026-09-12): the MAJOR gate runs before the strict parse.** 3.2 and 3.3
both refuse at exit 3 and both are ordered "before either mode runs", but not
against each other, and a payload from a later MAJOR line typically trips both.
The gate reads `schemaVersion` out of the raw bytes first, so such a payload is
refused as `attestation schema MAJOR 1 is unsupported` rather than as whichever
additive member the strict parse happened to reach first. The alternative,
parsing strictly and gating afterwards, is a few lines shorter and answers
"unknown field `obligations`" to the question "why will this not verify". No
line of the matrix distinguishes the two, so nothing here is a requirement
change: it is the choice the spec left open, made where it shows.

## Verification

Each line below is one command, run in its own `sh -c` from the repository root
(spec 049 3.5), so no line depends on a variable another line set. The scratch
files live at a fixed path for that reason. The key is a fixed test seed; the
public key is read back from the seal's default `keyId`, which is the hex public
key.

Against pre-085 code the lines split three ways:

- **Fail first.** The two injected-member lines, the nested-member line, the
  per-spec member line, the seal member line, both unknown-MAJOR lines, the
  same-MAJOR version line and both reformatted-bytes lines. They exit 0 or 1
  today where 3 or 1 is required, and they are the evidence that the defect
  is fixed.
- **Pass before and after.** The baseline, the `tool.version` line, the verdict
  flip, the duplicate key, the per-spec recompute of a directory-unit spec and a
  file-unit spec, the line that the written bytes hash to `attestationHash`, and
  the closing `check`. They guard 3.6.
- **Setup and cleanup.** The build, the scratch directory, the key, the two
  `attest` runs that write the pairs, the public-key read, and the final
  `rm -rf`.

Two vacuous passes are closed. Each refusal line checks the exit code **and**
that stderr names the member or the MAJOR, so a line cannot pass because a file
was missing (also exit 3). Both reformatted-bytes lines assert exit 1
exactly, so neither can pass on a usage error, and the recompute one also
asserts the outcome and the difference 3.1 spells out, so it cannot pass on a
mismatch found for some other reason.

```verify:cli
cargo build --release --locked
rm -rf "${TMPDIR:-/tmp}/ss085" && mkdir -p "${TMPDIR:-/tmp}/ss085"
printf '0707070707070707070707070707070707070707070707070707070707070707' > "${TMPDIR:-/tmp}/ss085/k"
target/release/spec-spine attest --sign --key "${TMPDIR:-/tmp}/ss085/k" >/dev/null
target/release/spec-spine attest --spec 083-an-attestation-covers-the-territory-it-claims --sign --key "${TMPDIR:-/tmp}/ss085/k" >/dev/null
sed -n 's/.*"keyId": "\([0-9a-f]*\)".*/\1/p' .derived/attestation/attestation.sig > "${TMPDIR:-/tmp}/ss085/pub" && test -s "${TMPDIR:-/tmp}/ss085/pub"
target/release/spec-spine verify-attestation --recompute --signature --public-key "${TMPDIR:-/tmp}/ss085/pub"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"prCouple\": {\"ok\": true},"; next} {print}' .derived/attestation/attestation.json > "$D/t1.json"; target/release/spec-spine verify-attestation --attestation "$D/t1.json" --seal .derived/attestation/attestation.sig --signature --public-key "$D/pub" 2> "$D/t1.err"; test $? -eq 3 && grep -q prCouple "$D/t1.err"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"prCouple\": {\"ok\": true},"; next} {print}' .derived/attestation/attestation.json > "$D/t1r.json"; target/release/spec-spine verify-attestation --attestation "$D/t1r.json" --recompute 2> "$D/t1r.err"; test $? -eq 3 && grep -q prCouple "$D/t1r.err"
D="${TMPDIR:-/tmp}/ss085"; awk '/"compile": \{/{print; print "      \"ignoredWarnings\": 12,"; next} {print}' .derived/attestation/attestation.json > "$D/t2.json"; target/release/spec-spine verify-attestation --attestation "$D/t2.json" --seal .derived/attestation/attestation.sig --recompute --signature --public-key "$D/pub" 2> "$D/t2.err"; test $? -eq 3 && grep -q ignoredWarnings "$D/t2.err"
D="${TMPDIR:-/tmp}/ss085"; A=.derived/attestation/by-spec/083-an-attestation-covers-the-territory-it-claims; awk '/"contentHash":/ && !done {print "        \"coveredBy\": \"review\","; done=1} {print}' "$A.json" > "$D/s1.json"; target/release/spec-spine verify-attestation --spec 083-an-attestation-covers-the-territory-it-claims --attestation "$D/s1.json" --seal "$A.sig" --recompute --signature --public-key "$D/pub" 2> "$D/s1.err"; test $? -eq 3 && grep -q coveredBy "$D/s1.err"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"revokedAt\": \"2026-01-01\","; next} {print}' .derived/attestation/attestation.sig > "$D/sl.sig"; target/release/spec-spine verify-attestation --seal "$D/sl.sig" --signature --public-key "$D/pub" 2> "$D/sl.err"; test $? -eq 3 && grep -q revokedAt "$D/sl.err"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "9.0.0"/' .derived/attestation/attestation.json > "$D/t3.json"; target/release/spec-spine verify-attestation --attestation "$D/t3.json" --recompute 2> "$D/t3.err"; test $? -eq 3 && grep -q "MAJOR 9" "$D/t3.err"
D="${TMPDIR:-/tmp}/ss085"; A=.derived/attestation/by-spec/083-an-attestation-covers-the-territory-it-claims.json; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "9.0.0"/' "$A" > "$D/s3.json"; target/release/spec-spine verify-attestation --spec 083-an-attestation-covers-the-territory-it-claims --attestation "$D/s3.json" --recompute 2> "$D/s3.err"; test $? -eq 3 && grep -q "MAJOR 9" "$D/s3.err"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "0.2.0"/' .derived/attestation/attestation.json > "$D/t4.json"; target/release/spec-spine verify-attestation --attestation "$D/t4.json" --recompute --json > "$D/t4.out"; test $? -eq 1 && grep -q '"schemaVersion (0.2.0' "$D/t4.out"
D="${TMPDIR:-/tmp}/ss085"; tr -d '\n' < .derived/attestation/attestation.json > "$D/t6.json"; target/release/spec-spine verify-attestation --attestation "$D/t6.json" --seal .derived/attestation/attestation.sig --signature --public-key "$D/pub"; test $? -eq 1
D="${TMPDIR:-/tmp}/ss085"; tr -d '\n' < .derived/attestation/attestation.json > "$D/t6r.json"; target/release/spec-spine verify-attestation --attestation "$D/t6r.json" --recompute --json > "$D/t6r.out"; test $? -eq 1 && grep -q contentMismatch "$D/t6r.out" && grep -q "bytes are not the canonical serialization" "$D/t6r.out"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"version": "[^"]*"/"version": "0.0.1"/' .derived/attestation/attestation.json > "$D/r1.json"; target/release/spec-spine verify-attestation --attestation "$D/r1.json" --recompute --json > "$D/r1.out"; test $? -eq 1 && grep -q versionMismatch "$D/r1.out"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"ok": true/"ok": false/' .derived/attestation/attestation.json > "$D/r2.json"; target/release/spec-spine verify-attestation --attestation "$D/r2.json" --recompute --json > "$D/r2.out"; test $? -eq 1 && grep -q contentMismatch "$D/r2.out"
D="${TMPDIR:-/tmp}/ss085"; awk '/"registryHash":/ && !done {print "  \"registryHash\": \"00\","; done=1} {print}' .derived/attestation/attestation.json > "$D/r3.json"; target/release/spec-spine verify-attestation --attestation "$D/r3.json" --recompute 2> "$D/r3.err"; test $? -eq 3 && grep -q registryHash "$D/r3.err"
target/release/spec-spine attest --spec 048-kit-ships-the-governed-loop-skills >/dev/null && target/release/spec-spine verify-attestation --spec 048-kit-ships-the-governed-loop-skills --recompute
target/release/spec-spine verify-attestation --spec 083-an-attestation-covers-the-territory-it-claims --recompute --signature --public-key "${TMPDIR:-/tmp}/ss085/pub"
H=$(target/release/spec-spine attest --json | sed -n 's/.*"attestationHash": "\([0-9a-f]*\)".*/\1/p'); F=.derived/attestation/attestation.json; if command -v sha256sum >/dev/null 2>&1; then G=$(sha256sum "$F" | cut -d' ' -f1); else G=$(shasum -a 256 "$F" | cut -d' ' -f1); fi; test -n "$H" && test "$H" = "$G"
target/release/spec-spine check
rm -rf "${TMPDIR:-/tmp}/ss085"
```
