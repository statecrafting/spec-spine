---
id: "164-incremental-context-packets"
title: "A packet delta reproduces the packet it names"
status: draft
kind: "governance"
created: "2026-09-27"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "107-a-context-closure-is-declared"
  - "110-an-interface-reference-is-digest-pinned"
  - "155-selected-content-accessor"
  - "159-repository-scoped-context-packet"
  - "160-documentation-manifest-and-freshness"
summary: >
  A deterministic delta from caller-supplied predecessor packet pages to spec
  159's pages on a target snapshot, in six closed change kinds, that
  reproduces the target byte for byte or falls back explicitly, plus pure
  invalidation candidates over spec-160 manifests. No session state.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/context_packet_delta.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/context-packet-delta.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/context_packet_delta.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/context_packet_delta.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-core/tests/fixtures/context-packet-delta/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_context_delta.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/context_packet_delta.rs", planned: true }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "034-machine-readable-verdicts", unit: { kind: file, path: "crates/spec-spine-types/src/verdict.rs" }, nature: additive }
  - { spec: "152-a-refusal-says-what-it-is-in-every-form", unit: { kind: file, path: "crates/spec-spine-cli/tests/refusal_envelopes.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap rows L and M" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/delta.rs" }, role: "path-level change classifier, distinguished in 3.11" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/interface.rs" }, role: "digest-pinned interface identities" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/closure.rs" }, role: "closure member digests" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every member difference between a verified predecessor page set and its target is exactly one of six closed change kinds, decided by one total transition table."
    anchor: "3-4-member-states-and-the-closed-change-vocabulary"
  - id: "R-2"
    kind: requirement
    text: "Applying a delta to its named predecessor reproduces every named target page byte for byte, including packetDigest, or refuses and emits nothing."
    anchor: "3-7-the-reproduction-law-and-continuation"
  - id: "R-3"
    kind: requirement
    text: "A predecessor from another repository, request, budget, or schema, or one that cannot be verified, yields an explicit full-packet fallback, never a partial delta."
    anchor: "3-3-predecessor-verification-and-fallback"
  - id: "R-4"
    kind: requirement
    text: "Where interface or manifest dependency information is incomplete, the delta over-selects and records the basis of each over-selection."
    anchor: "3-8-conservative-selection"
  - id: "R-5"
    kind: requirement
    text: "Invalidation candidates are a deterministic list of manifest-input identities and never carry disposition, admission, policy, or lifecycle."
    anchor: "3-9-invalidation-candidates"
  - id: "I-1"
    kind: invariant
    text: "spec-spine keeps no session state, trusts no predecessor content, and never emits content of a member the target did not select."
    anchor: "3-13-authority-security-and-disclosure-boundary"
  - id: "V-1"
    kind: verification
    text: "The implemented delta is checked for the transition table, the reproduction law over chained deltas, every fallback reason, forged predecessors, conservative bases, candidate order, and non-disclosure."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/context_packet_delta.rs"
      - "crates/spec-spine-cli/tests/context_packet_delta.rs"
intent:
  goal: "let a long-running consumer refresh a context packet from bytes it already holds with a provably exact, content-minimal delta"
  non_goals:
    - "session state, caches, or packet storage inside spec-spine"
    - "evidence admission, invalidation policy, lifecycle grading, or disposition"
    - "path-level review classification, which spec 071 owns"
---

# 164: A packet delta reproduces the packet it names

## 1. Purpose

A consumer working across revisions, such as a long agent session, must today
re-request the whole spec-159 page set after every commit and diff it itself.
Each consumer invents a diff, and none can prove its patched copy is exact.

This spec defines one deterministic delta between two complete page sets of the
same request: the predecessor the caller holds and the target spec 159 would
assemble on a named snapshot. The delta names every member change in a closed
vocabulary, carries content only for members the target selects, reproduces
the target exactly when applied, and names as candidates the spec-160 manifest
inputs whose declared identities changed. What a candidate means for evidence,
documentation, or acceptance stays with Statecraft.

## 2. Territory

- `crates/spec-spine-types/src/context_packet_delta.rs` and
  `schemas/context-packet-delta.schema.json`: request, delta, fallback, change,
  layout, and candidate DTOs on their own schema axis.
- `crates/spec-spine-core/src/context_packet_delta.rs`: the pure compute and
  apply functions. Neither reads Git, the clock, or the environment.
- Spec 159's packet files (types, schema, core, `cmd_context.rs`, and
  `docs/context-packets.md`): the additive `interface` member of 3.5 and the
  two subcommands. The build declares each as an `extends` edge on 159 once
  159 has written it (D-9).
- `crates/spec-spine-cli/src/cmd_context_delta.rs`, wired through 159's
  `context` group and `main.rs`: `context delta` and `context apply`.
- Both `lib.rs` files: typed functions and the facades
  `context_packet_delta_json` and `context_packet_apply_json`. `verdict.rs`
  and the refusal-envelope inventory: `context.delta` and `context.apply`.
- The planned tests and fixtures; `docs/context-packets.md` and the API, CLI,
  and schema-history documents.

## 3. Behavior

### 3.1 A delta relates two complete page sets of one request

A delta has one predecessor and one target, each a complete spec-159 page set:
an ordered, non-empty list of pages sharing one `requestDigest`, snapshot, and
`closureDigest`, every page but the last carrying a continuation and the last
carrying none, each page's first member key sorting strictly after the previous
page's last (D-12). One page with no continuation is a complete set. The target is the set spec 159 assembles for the predecessor's
request, on the named target snapshot, by the running producer.

Predecessor and target MAY name any two revisions of one repository; ancestry
is not checked, because the law of 3.7 is about bytes. A delta never starts or
ends at one page of a multi-page set.

### 3.2 The delta request

The request is a closed object with members `request`, `predecessorPages`,
`manifests`, and `consumerSchemaVersion`. `request` is the spec-159 request with `continuation` null. `predecessorPages`
holds 1 through 512 predecessor pages as the exact byte strings the caller
holds, so digests are recomputed over those bytes, not a re-rendering.
`manifests` holds 0 through 64 spec-160 manifests, also as exact strings.
`consumerSchemaVersion` names the delta axis the caller reads; an unsupported
MAJOR is refused. Unknown members are usage errors.

The CLI form is `context delta --request <path> --predecessor <page>...
[--manifest <path>]... --repository <identity> [--revision <rev>] --json`. It
binds the target snapshot exactly as spec 159 3.1 binds `context packet`. Core
and the facade take a caller-supplied snapshot binding and stay Git-free.

### 3.3 Predecessor verification and fallback

The producer MUST check, in order, and stop at the first row that holds:

| Row | Condition | Outcome |
|---|---|---|
| 1 | a page does not parse under a context-packet MAJOR the producer reads | usage error |
| 2 | a page's recomputed `packetDigest` differs from its recorded one | fallback `predecessor-unverifiable` |
| 3 | the pages are not one complete set under 3.1 | fallback `predecessor-incomplete` |
| 4 | predecessor `repository` differs from the target's | fallback `repository-changed` |
| 5 | predecessor context-packet `schemaVersion` differs from the one the producer emits | fallback `schema-version-changed` |
| 6 | predecessor `maxBytes` or `maxItems` differs from the request's | fallback `budget-changed` |
| 7 | predecessor `requestDigest` differs from the request's | fallback `request-changed` |
| 8 | the canonical delta would exceed the 3.10 limit | fallback `delta-too-large` |

Otherwise the producer returns a delta. A fallback carries `outcome:
"fallback"`, the reason, the target snapshot and `requestDigest`, the supplied
predecessor page digests, and the candidates of 3.9, with no content and no
changes. The caller obtains the target by running spec 159 with the same
request and snapshot.

A different producer build is not a fallback reason: reuse is by record
equality, so a differently rendered record is a change (`producerChanged`).

### 3.4 Member states and the closed change vocabulary

A key is spec 159's canonical member key: the spec-155 identity plus
projection. In each set a key is `present` (a member), `omitted` (an omission,
with its reason), or `absent`. Omission reasons are either resolution reasons
(`missing`, `removed`, `withdrawn`, `unresolved`, `ambiguous`) or selection
reasons (every other spec-159 reason).

Each key whose state or record differs yields exactly one entry, decided by
this table alone:

| Predecessor | Target | Change |
|---|---|---|
| present | present; interface differs, or unknown under 3.8 | `interface-changed` |
| present | present; record differs, interface equal or not applicable | `changed` |
| present or omitted | absent | `removed` |
| present, absent, or omitted with another record | omitted for a resolution reason | `newly-unresolved` |
| present, absent, or omitted with another record | omitted for a selection reason | `newly-omitted` |
| omitted or absent | present | `newly-available` |

Otherwise a key yields no entry. `removed` means the key left the request's resolved set; a
unit deleted from the repository while still requested is a spec-159
`removed` omission and therefore `newly-unresolved`.

Records are compared after restamping the four snapshot fields a spec-155 item
and a spec-159 member repeat (`repository`, `revision`, `tree`, `dirtyState`)
with the target's, as canonical bytes. A `changed` entry lists which of
`content`, `span`, `requirement`, and `origins` differ.

Every entry carries `key`, `change`, `basis` (3.8), the prior state, and the
prior item digest or omission reason. `changed`, `interface-changed`, and
`newly-available` carry the complete target member record; the other three
carry identities and reasons only.

### 3.5 Interface digests

A member's interface is what a dependent consumer pins. Each spec-159 member
gains an `interface` object, `{ "kind", "digest" }`, computed at assembly:

| Selector kind | Interface kind | Digest |
|---|---|---|
| `spec` | `spec-content-hash` | `sha256:` plus the registry content hash: the value a spec-110 reference pins as `digest` |
| `spec-section` | `section-digest` | the registry `sectionDigests` entry: the value a spec-110 reference pins under `sections` |
| `obligation` | `obligation-digest` | spec 107's obligation piece digest (107 D-8), which includes the anchor section digest |
| `symbol` | `symbol-signature` | spec 077 framing over the spec-155 `signature` projection content, span excluded |
| `symbol` without a supported signature | `unavailable` | null |
| any other kind | `not-applicable` | null |

Interface and item digests are independent: an obligation's anchor section or
a `body`-projected symbol's signature can change under an equal item
(`interface-changed`), and an edit above a selected section moves its span but
not its section digest (`changed`, `span`). An `interface-changed` entry names
both digests, so a spec-110 pin holder can match them directly.

### 3.6 The delta document

```json
{
  "schemaVersion": "1.0.0",
  "outcome": "delta",
  "producer": {},
  "producerChanged": false,
  "requestDigest": "sha256:...",
  "predecessor": { "snapshot": {}, "pageDigests": [] },
  "target": { "snapshot": {}, "pageDigests": [], "closure": null, "pages": [] },
  "changes": [],
  "counts": {},
  "invalidationCandidates": { "coverage": "packet-members", "items": [] },
  "deltaDigest": "sha256:..."
}
```

`target.pages` holds one layout per target page, in order: the page envelope
(every spec-159 top-level field except `members`, `omissions`, and `closure`,
byte-equal to the page's, `packetDigest` included), the ordered member keys,
and the complete ordered omission records. `target.closure` is the closure
document when `closureDigest` changed and null when the predecessor's is
reused. `counts` holds every change kind, zeros included. `deltaDigest` hashes
the canonical delta with that field absent.

### 3.7 The reproduction law and continuation

`apply(predecessorPages, delta)` MUST:

1. refuse unless the recomputed predecessor page digests equal
   `delta.predecessor.pageDigests`;
2. build a member map from the predecessor, restamped as in 3.4, and replace
   or insert every record a change entry carries;
3. assemble each target page from its layout, the map, and the closure;
4. serialize each page with spec 159's canonical writer and recompute its
   `packetDigest`; and
5. refuse unless each reproduced digest equals both the layout's recorded
   `packetDigest` and the matching `delta.target.pageDigests` entry.

For every input on which the producer returns a delta, applying it to those
predecessor bytes MUST yield, page by page, documents byte-identical to those
spec 159 emits for that request on that snapshot. The producer MUST check this
on its own output before emitting; a failure is exit 4, never a delta. A layout
key missing from the map, an entry no layout uses, or a duplicate key is a
malformed delta, and apply emits nothing unless every page reproduces.

A delta is atomic: it has no continuation and is never paginated; a set too
large for one delta falls back under row 8. The target pages' spec-159
continuation tokens travel inside their envelopes, serve only spec 159's own
paging, and are a usage error in a delta request. Because reproduced pages are
exact, they are a valid predecessor for the next delta: deltas chain with no
state held by spec-spine.

### 3.8 Conservative selection

Every entry and candidate carries a `basis`: `digest` when it follows from
compared digests or records, or a named basis when dependency information is
missing. The producer MUST over-select in exactly these cases:

- `interface-unknown`: a key present in both sets whose interface is
  `unavailable` on either side is `interface-changed`, unless its item digest
  is equal and its projection is `full` or `signature`, which contain the
  whole signature.
- `closure-changed`: when `closureDigest` differs, every manifest input whose
  key is omitted in both sets with a closure origin is a candidate, since such
  a member has no item digest to compare.
- `unbound-manifest`: a manifest whose packet identity is not the
  predecessor's `requestDigest` and page digests is a whole-manifest candidate.
- `fallback`: on any fallback, every manifest is a whole-manifest candidate.

No other inference (directory scan, similarity, reference edge, spec-071
class, spec-109 impact) adds or removes an entry. This is spec 150's rule on a
narrower dependency: member-to-bytes is complete by digest, so only unknown
interfaces and bindings need over-selection.

### 3.9 Invalidation candidates

Each manifest's shape and `manifestDigest` are validated under spec 160; a
failure is a usage error naming its position. For a manifest bound to the
predecessor, each repository-local input whose identity and projection equal a
change entry's key yields one candidate carrying `manifestDigest`, `inputId`,
`requirement`, `key`, `change`, `basis`, and the input's `uses`.
Whole-manifest candidates carry null `inputId`, `key`, and `change`. Candidates
sort by `manifestDigest`, then `inputId` with null first, then `basis`, and are
deduplicated.

Coverage is packet members only. Spec-160 interface inputs, external
references, generator identity, and output bytes lie outside a packet; spec
160's freshness read evaluates them. A candidate states that a declared
identity changed. It carries no severity, verdict, admission, lifecycle grade,
or disposition, and computing it writes, invalidates, or rejects nothing.

### 3.10 Ordering, limits, canonical bytes, and schema

Entries sort bytewise by key; layouts keep page order and spec 159's member
order. Manifest order does not affect bytes. The canonical delta MUST NOT exceed
4194304 bytes; the check runs after the target is assembled, and excess is
fallback `delta-too-large`, never truncation.

Documents are canonical JSON (sorted keys, two spaces, LF, final newline).
Request, delta, and fallback share
`CONTEXT_PACKET_DELTA_SCHEMA_VERSION`, starting at `1.0.0` and recorded in
`docs/schema-versioning.md`, distinct from spec 071's `DELTA_SCHEMA_VERSION`.
Additive optional members are MINOR; removing or reinterpreting a member,
change kind, basis, fallback reason, ordering rule, or digest input is MAJOR.
The 3.5 `interface` member is a context-packet MINOR. CLI and facades emit
byte-identical documents for equal inputs; human output is never digested.

### 3.11 Relation to the change classifier and declared impact

Spec 071's `delta` verb classifies changed paths between two Git trees under
the base's rules for review; it carries no content and decides nothing. This
spec compares packets member by member and carries replacement content. They
share only the CLI's bounded tree export: no path class becomes a member
change or the reverse, and the verb is `context delta`, never bare `delta`.
Spec 109's impacts are authored declarations; this spec reads none of them.

### 3.12 Exit and error behavior

Both verbs use spec 132's exit contract and spec 152's envelope. `context
delta` exits 0 with a delta whose target is complete or partial, or
with any fallback; 1 with a delta whose target is `incomplete` under spec 159
3.8, carried as the envelope report; 2 when the target snapshot is dirty,
changes during the read, or fails containment; 3 for a malformed request, a
continuation, a count out of range, an unparseable page, or an invalid
manifest; 4 for I/O failure or a delta that fails 3.7.

`context apply --predecessor <page>... --delta <path> --out-dir <dir>` exits 0
after writing `page-NNNN.json` files into an empty or new directory; 2 when
predecessor digests do not match or a page does not reproduce, having written
nothing; 3 for malformed input or a non-empty directory; 4 for I/O failure.
The facade returns the ordered page strings instead of writing. Diagnostics
name keys, digests, positions, and reasons, never content.

### 3.13 Authority, security, and disclosure boundary

spec-spine holds no session, cache, or predecessor between calls; the caller
supplies predecessor bytes each time.

Predecessor bytes are untrusted. `context delta` only compares them: no
predecessor byte appears in a delta, and none is taken as the target's.
`context apply` does carry predecessor records into its output, but only the
records the delta producer found equal to the target's, and only through
3.7's two checks: the predecessor pages must have the digests the delta names
(step 1), and every reproduced page must have the `packetDigest` the delta
recorded (step 5), or apply writes nothing. Every record the producer found
different travels in the delta, so applying a delta to a forged,
self-consistent predecessor still yields the true target, and applying it to
other bytes refuses. Digest recomputation is an integrity check, not
authentication.

Content and interfaces are compared and emitted only for keys the target
presents; other keys compare state and omission records only, so a forged
predecessor cannot probe unselected text. No diagnostic echoes content,
environment values, credentials, or remote URLs.

A delta establishes no correctness, evidence currency, acceptance, or
authority; candidates are findings for Statecraft to adjudicate.

## 4. Acceptance criteria

1. Every 3.4 row yields exactly its entry; unchanged keys yield none.
2. Deltas reproduce their targets byte for byte over single and multi-page
   sets, a changed closure, a changed producer build, and a three-delta chain.
3. Every 3.3 fallback row is reached with its reason and no content.
4. A changed anchor section under an obligation, a changed signature under a
   symbol `body` projection, and a span-only shift yield `interface-changed`,
   `interface-changed`, and `changed` with `span`.
5. A forged self-consistent predecessor yields a delta that reproduces the true
   target from the forged bytes and refuses on genuine bytes; no entry reveals
   content of an unselected key.
6. Bound, unbound, and closure-changed manifests yield candidates with the
   right basis, in an order invariant to manifest input order.
7. LF, CRLF, and CR fixtures yield equal deltas; CLI and facade bytes match;
   existing packet, content, closure, interface, manifest, and spec-071
   documents stay byte-compatible apart from the 3.5 MINOR.

## 5. Out of scope

- Storing, caching, or indexing predecessor packets or session identities.
- Cross-repository or cross-request deltas, ancestry proofs, or Git history.
- Evaluating spec-160 interface inputs, external references, outputs, or
  generator identity.
- Evidence admission, invalidation policy, lifecycle grading, disposition, or
  automatic rejection.

## 6. Resolved decisions

**D-1 (2026-09-27): the delta covers complete page sets.** Page boundaries move
whenever content sizes move, so a page-to-page diff has no stable meaning.
Layouts reproduce boundaries instead.

**D-2 (2026-09-27): reuse is decided by record equality, never trust.** Reuse
on a digest match alone would let a forged predecessor into the target.
Comparing restamped records makes the law hold for any predecessor bytes.

**D-3 (2026-09-27): a fallback is an answer.** Rows 2 through 8 occur in normal
use, so a fallback exits 0 and names its reason.

**D-4: not used.** The 2026-09-27 draft carried no D-4. The number is left
unused, not reassigned, so the identifiers other entries cite (D-5 in D-10)
keep their meaning (noted 2026-10-10, review).

**D-5 (draft, owner to confirm): any schema-version difference falls back.**
Row 5 includes MINOR differences because reused records must equal the
target's bytes. Tolerating a MINOR needs a per-member projection rule.

**D-6 (draft, owner to confirm): apply refusals exit 2.** A predecessor mismatch
and a non-reproducing page leave nothing done. Exit 1 would read as a finding
about the repository, exit 4 as an internal failure.

**D-7 (draft, owner to confirm): `interface` lands on 159's axis.** If spec 159
is unbuilt when this is approved, folding the member into its `1.0.0` avoids a
MINOR on day one. Superseded by D-10.

**D-8 (draft, owner to confirm): candidates cover packet members only.**
Covering spec-160 interface inputs would duplicate its freshness read; deferred
unless Statecraft needs one call.

**D-9 (2026-09-27): no edge on 159's files until they exist.** Compile refuses
an `extends` on a unit another spec only planned (V-017, V-015), so the build
adds the five edges after 159 ships.

**D-10 (2026-10-10, refile): `interface` is a context-packet MINOR.** D-7's
condition does not hold: spec 159 was filed on `main` and its build opened
without an `interface` member, so 159 ships `1.0.0` first and 3.5 lands as the
MINOR that 3.10 already names. Rows 5 and 7 are unaffected: a predecessor on
the earlier MINOR falls back under row 5 (D-5).

**D-11 (2026-10-10, refile): the two verbs are cataloged.** Spec 162 holds
every CLI form and facade function to one catalog record (162 R-1). Whichever of
164 and 162 is built second adds `context.delta` and `context.apply` and their
facades. Effects: `context delta` reads `caller-file`, `config`, `corpus`,
`derived-ledger` and `source-tree`, executes `git` for the snapshot binding
exactly as `content select` does, and reads `clock` if it shares that verb's
temporary export; `context apply` reads `caller-file` and writes
`caller-path`. Neither opens a connection.

**D-12 (2026-10-10, refile): page-set completeness is structural.** Spec 159
as filed corrected its D-4: a continuation token is integrity checked, not
authenticated, so no reader can prove from the pages that none was skipped.
3.1 therefore uses the same structural check as spec 160 D-8 (shared identity,
continuation on every page but the last, strictly increasing member keys
across pages). Row 3 falls back on a set that fails it. The 3.7 law is
unaffected: reuse is by record equality against the target spec 159 assembles,
so a predecessor that silently skipped a member yields that member as
`newly-available` and the target still reproduces.

**D-13 (2026-10-10, review): apply's reuse is stated as what it is.** Review
of the filing found that 3.13 said predecessor bytes are "never emitted",
while `context apply` builds its pages from predecessor records. 3.13 now
separates the verbs: `context delta` emits no predecessor byte, and `context
apply` carries forward only records the producer matched, under 3.7's
predecessor-digest and reproduction checks. Behavior is unchanged.

## Verification

Every line fails before the build: no named target or subcommand exists yet.

```verify:cli
cargo test -p spec-spine-core --test context_packet_delta --locked
cargo test -p spec-spine-cli --test context_packet_delta --locked
cargo run -q -p spec-spine-cli --locked -- context delta --help
cargo run -q -p spec-spine-cli --locked -- context apply --help
```
