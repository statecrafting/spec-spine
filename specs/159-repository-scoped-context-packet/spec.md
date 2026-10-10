---
id: "159-repository-scoped-context-packet"
title: "Assemble one repository context packet"
status: draft
kind: "governance"
created: "2026-09-26"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "107-a-context-closure-is-declared"
  - "155-selected-content-accessor"
summary: >
  Adds a deterministic, read-only context-packet request and response for one
  immutable repository snapshot. A packet combines a declared context closure
  with required and optional selected-content members under explicit byte and
  item budgets, records every omission and warning, binds continuation to the
  request and snapshot, and carries a canonical packet digest without creating
  cross-repository authority.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/context_packet.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/context-packet.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/context_packet.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/context_packet.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-core/tests/fixtures/context-packet/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_context.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/context_packet.rs", planned: true }
  - { kind: file, path: "docs/context-packets.md", planned: true }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/closure.rs" }, role: "closure identities and digest" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "A context packet binds one closure and ordered selected-content members to one explicitly identified repository snapshot."
    anchor: "3-1-one-packet-one-repository-snapshot"
  - id: "R-2"
    kind: requirement
    text: "Required members never disappear silently, and every optional omission records its identity and reason."
    anchor: "3-8-required-members-optional-members-and-completeness"
  - id: "R-3"
    kind: requirement
    text: "Packet ordering, budgets, continuation, canonical bytes, and digest are deterministic for one request and snapshot."
    anchor: "3-10-ordering-budgets-continuation-and-digest"
  - id: "I-1"
    kind: invariant
    text: "A packet is untrusted repository data and never composes repositories, executes content, admits evidence, or grants authority."
    anchor: "3-13-authority-security-and-disclosure-boundary"
  - id: "V-1"
    kind: verification
    text: "The implemented packet contract is checked for identity binding, ordering, budgets, required omissions, continuation, stale snapshots, canonical JSON, and containment."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/context_packet.rs"
      - "crates/spec-spine-cli/tests/context_packet.rs"
intent:
  goal: "assemble bounded deterministic context from one immutable repository snapshot"
  non_goals:
    - "cross-repository snapshot composition or orchestration"
    - "remote fetching, mutable sessions, execution, editing, or evidence admission"
    - "consumer authorization, redaction, prompt policy, or correctness judgment"
---

# 159: Assemble one repository context packet

## 1. Purpose

ContextClosure identifies the governed specifications and source units relevant
to a root, while selected content returns bounded text for explicit selectors.
Neither contract says how a consumer requests one ordered, digestible bundle,
distinguishes mandatory inputs from useful extras, resumes after a budget, or
proves which omissions occurred. Consumers otherwise invent incompatible packet
formats and may silently drop material they needed.

This spec defines a repository-scoped context packet. It composes existing
closure and selected-content identities without changing either contract. The
packet is deterministic data for a caller to evaluate. It is not an execution
context, authorization token, evidence record, or multi-repository snapshot.

## 2. Territory

- `crates/spec-spine-types/src/context_packet.rs` defines packet requests,
  member requests, packet members, omissions, warnings, continuation, and the
  response.
- `crates/spec-spine-types/schemas/context-packet.schema.json` defines the
  versioned public JSON document.
- `crates/spec-spine-core/src/context_packet.rs` assembles a packet from a
  supplied closure and selected-content resolver over one repository export.
- `crates/spec-spine-core/src/lib.rs` exposes typed assembly and
  `context_packet_json(config_json, repo_root, request_json, snapshot_json)`.
- `crates/spec-spine-cli/src/cmd_context.rs` and `main.rs` expose
  `context packet --request <path> --repository <identity> [--revision <rev>]
  --json` through the same snapshot binding as selected content.
- The planned core and CLI tests plus the context-packet fixture directory own
  positive, refusal, pagination, and portability evidence.
- `docs/context-packets.md`, the API reference, CLI reference, and schema
  history describe the public contract and its trust boundary.

The closure and selected-content implementations remain owned by specs 107 and
155. This spec calls them and reuses their identities. It does not add packet
fields to `ContextClosure` or alter selected-content item digests.

## 3. Behavior

### 3.1 One packet, one repository snapshot

Every request MUST resolve against exactly one snapshot identity from spec 155:
`repository`, `revision`, `tree`, and `dirtyState`. The response and every
member MUST repeat that identity. The closure and every selected-content item
MUST have been resolved from the same snapshot during the same stable read.

The CLI MUST perform the same before-and-after cleanliness and identity checks
as `content select`. Core stays Git-free and accepts the caller-supplied
snapshot binding. A dirty tree, changed snapshot, mismatched member identity,
or unavailable revision is refused before a packet is returned.

One request MUST NOT name a second repository, remote URL, checkout, or export.
Statecraft may compose several immutable packet identities in a separate
orchestration record. spec-spine MUST NOT claim they share a transaction,
revision, tree, cleanliness state, or authority.

### 3.2 Request document

A request has this closed shape:

```json
{
  "closure": { "root": "155-selected-content-accessor" },
  "members": [],
  "maxBytes": 524288,
  "maxItems": 96,
  "continuation": null,
  "rationale": null,
  "workIdentity": null,
  "consumerSchemaVersion": "1.0.0"
}
```

`closure` MUST contain exactly one of `root` or `digest`. `root` is a spec id
accepted by the closure resolver. `digest` is a previously resolved closure
digest accompanied by the exact closure document in the request; the producer
MUST recompute and compare it and MUST NOT discover a closure by digest.

`members` is an ordered request set before canonical sorting. Each entry reuses
one selected-content selector and projection and adds `requirement`, either
`required` or `optional`. An empty explicit member list is valid because the
closure still contributes its own members.

`maxBytes` defaults to 524288 and MUST be 1 through 4194304. `maxItems`
defaults to 96 and MUST be 1 through 512. `rationale` and `workIdentity` are
optional opaque strings of at most 1024 bytes each. They are carried for audit
context, spend no content budget, and have no authority effect.

`consumerSchemaVersion` is required. The producer accepts the current major
and supported minors only. An unsupported major or a consumer minor requiring
fields the producer cannot emit is refused. Unknown request members are usage
errors.

### 3.3 Closure members are selected explicitly

The producer MUST first resolve the closure under spec 107. It then converts
each resolved closure member to the most specific selected-content selector
that preserves that member's declared identity. A closure member that has no
supported textual projection becomes an explicit packet omission; it MUST NOT
be converted to a bare file selector or inferred substitute.

The packet carries the complete closure document and `closureDigest`. The
closure bytes do not spend the content byte budget, but every selected content
item derived from it spends item and byte budgets. A caller MAY add selectors
outside the closure. Their packet members record `origin: "additional"`;
closure-derived members record `origin: "closure"`.

### 3.4 Member identity and deduplication

The canonical member key is the spec-155 selected-content identity plus its
requested projection. Required and optional requests for the same key collapse
to one member and `required` wins. Closure and additional origins collapse to
one member with ordered `origins: ["closure", "additional"]`.

Deduplication MUST NOT merge two selectors that happen to return equal bytes.
It is identity based, not content based. Equal content under distinct canonical
identities remains distinct and spends budget separately.

### 3.5 Response document

A successful response contains:

- `schemaVersion` and the producer package identity and version;
- the snapshot identity;
- the normalized request identity and `requestDigest`;
- the full closure and `closureDigest`;
- ordered `members`, each carrying the complete selected-content item plus its
  requirement and origins;
- ordered `omissions` and `warnings`;
- `completeness`;
- `continuation`; and
- `packetDigest`.

The producer identity MUST include the package version and an executable or
library build digest supplied by the binding. Core MUST NOT infer its own
binary identity. An unavailable build digest is represented as `unverified`,
never omitted or replaced with a package version.

### 3.6 Omission vocabulary

Every omission contains the canonical member key, requirement, origins, and
exactly one reason:

- `missing`;
- `removed`;
- `withdrawn`;
- `unresolved`;
- `ambiguous`;
- `unsupported-selector`;
- `unsupported-projection`;
- `non-text`;
- `oversized-member`;
- `item-budget`; or
- `byte-budget`.

Selector ambiguity, path escape, stale ledgers, snapshot change, malformed
input, and invalid continuation remain refusals under their owning contracts.
They are not softened into omissions. An omission MAY carry the underlying
family error code and safe detail, but MUST NOT copy untrusted content into the
error message.

### 3.7 Warnings

Warnings are ordered typed records. V1 supports `unverified-producer-build`,
`closure-member-unsupported`, and `optional-member-omitted`. Warnings MUST NOT
change `complete` to `incomplete` unless the same condition also creates a
required omission. Consumers MUST use `completeness`, omissions, and the exit
contract, not warning text, to decide whether a packet is usable.

### 3.8 Required members, optional members, and completeness

`completeness` is `complete` only when every required member is returned and no
further page is needed. It is `partial` when all required members selected for
the current page are present but continuation remains. It is `incomplete` when
any required member is omitted.

An optional omission is recorded and MAY coexist with a complete final packet.
A required omission MUST return the full deterministic response as a finding
with `completeness: "incomplete"` and exit 1. A malformed request, unsafe path,
stale snapshot, or invalid continuation is refused and returns no packet.

No required member may be downgraded to optional because of budget. If the
first remaining required member cannot fit in an otherwise empty page, it is
reported as `oversized-member` and the packet is incomplete rather than
issuing a continuation that can never advance.

### 3.9 Empty and missing results

A closure with no resolved textual member and no additional selector produces
an empty, complete packet if it also has no unresolved required member. A
missing closure root, closure digest mismatch, or stale committed registry or
index is a finding or refusal according to the underlying read contract and
MUST NOT become an empty packet.

Removed units and withdrawn obligations retain their requested identity in the
omission. Their former content is never recovered from Git history by this
operation.

### 3.10 Ordering, budgets, continuation, and digest

Members are ordered by canonical selector identity and projection using bytewise
UTF-8 ordering. Omissions use the same key, then reason. Warnings order by code,
then member key. Input order MUST NOT affect emitted order or digest.

Budgets count normalized UTF-8 bytes in member `content` and returned member
count. Envelope, closure, omission, warning, rationale, and identity bytes do
not spend the content budget. A member is atomic and never split.

Continuation is opaque canonical base64url carrying the schema major, snapshot
identity digest, request digest, closure digest, budgets, and last emitted
member key. It is authenticated by a SHA-256 checksum over its canonical
payload, not by a secret. Changed request, snapshot, closure, schema major, or
budgets is `stale-continuation`. A valid continuation resumes strictly after
the last key and MUST NOT repeat or skip a member.

`requestDigest` hashes the canonical normalized request without continuation.
`packetDigest` hashes the canonical response with `packetDigest` absent and
with continuation included. Both use `sha256:` plus lowercase hex. Two pages
have distinct packet digests. The final consumer may record the ordered page
digest list; spec-spine does not invent an aggregate session identity.

### 3.11 Canonical JSON and schema evolution

The context-packet schema begins at `1.0.0`, uses sorted-key canonical JSON,
LF, and a trailing newline. The schema has its own version axis documented in
`docs/schema-versioning.md`. Additive optional members move MINOR. Removing or
reinterpreting a member, ordering rule, digest input, or enum value moves
MAJOR. PATCH changes descriptions or constraints without changing accepted or
emitted instances.

The CLI and JSON facade MUST emit byte-identical packet documents for equal
typed inputs, snapshot, producer identity, and page. Human output is a
projection of the typed response and is never a digest input.

### 3.12 Refusal and error behavior

The operation uses the family exit and error-envelope contract. Usage errors
include unknown fields, invalid enum values, conflicting closure forms, zero or
excessive budgets, and malformed continuation. Findings include missing or
unresolved declared content and required omissions. Refusals include dirty or
changed snapshots, path containment failures, unsupported schema major, and a
continuation bound to other inputs. I/O and internal schema failures remain
failures.

JSON mode writes the versioned verdict envelope to stdout on every non-success.
When a deterministic incomplete packet exists, the envelope report contains
that packet. Error detail MUST name identities and safe reasons, never selected
content, environment values, credentials, or remote URLs.

### 3.13 Authority, security, and disclosure boundary

Packet content, rationale, documentation, examples, and tool-like text are
untrusted data. The producer MUST NOT interpret embedded instructions, follow
links, load remote content, execute examples, or change selection because text
claims to be authoritative. Content remains explicitly delimited as data at
every binding boundary.

The operation reads only the named repository export through existing
containment and exclusion rules. It MUST NOT include ignored files, ambient
environment, Git remote configuration, credentials, or unselected content.
Budgets apply before final allocation, and error records never echo content.

A packet proves only deterministic assembly from declared inputs. It does not
prove correctness, approval, freshness in another repository, evidence
admission, acceptance, release, adoption, or qualification. Consumers own
redaction, disclosure, prompt-injection containment, and authorization to show
private repository text.

## 4. Acceptance criteria

1. A closure root or verified closure document plus additional selectors
   produces the exact ordered packet members for one clean snapshot.
2. Required and optional duplicate requests collapse deterministically, with
   required winning and origins preserved.
3. Missing, removed, withdrawn, unsupported, and oversized members are explicit;
   a required omission returns an incomplete finding and never disappears.
4. Byte and item budgets, page boundaries, continuations, request digests, and
   packet digests are byte-identical for repeated reads and invariant to input
   member order.
5. Dirty or changed snapshots, mismatched closure or member identities, unsafe
   paths, stale continuations, and unsupported schema majors refuse.
6. LF, CRLF, and CR fixtures produce equal normalized members and packet
   digests on supported platforms.
7. Existing closure, selected-content, registry, index, and interface documents
   remain byte-compatible.

## 5. Out of scope

- Cross-repository packet composition, distributed snapshot consistency,
  remote repository discovery, or mutable session state.
- Adding arbitrary content to `ContextClosure` or changing selected-content
  identity, projection, span, or digest semantics.
- Executing content, examples, tests, generators, or commands.
- Applying edits, formatting, accepting claims, admitting evidence, or granting
  authority.
- Choosing consumer disclosure, redaction, provider, prompt, or retention
  policy.
- Aggregating page digests into a task or session identity.

## 6. Resolved decisions

**D-1 (2026-09-26): packets remain repository-scoped.** Statecraft can compose
several immutable packet identities while preserving each repository's own
revision and tree. spec-spine does not invent a distributed Git transaction.

**D-2 (2026-09-26): required budget failures are incomplete findings.** A
caller needs the exact missing identity to decide whether to increase a budget
or stop. Silent omission and endless continuation are both forbidden.

**D-3 (2026-09-26): closure and selected-content identities are reused.** A
packet adds composition metadata only. It does not define competing selector,
span, content-digest, or closure-digest semantics.

**D-4 (2026-09-26): continuation is integrity checked but not secret.** It
prevents accidental or adversarial substitution from being accepted as another
request, but it is not an authorization token and carries no server state.

**D-5 (2026-09-26): producer build identity is explicit.** Package version
alone cannot distinguish local builds. A binding supplies a digest or records
the build as unverified; core does not inspect itself.

**D-6 (2026-09-26): consumers own disclosure.** spec-spine enforces selection,
containment, and budgets. It cannot decide whether a caller is permitted to
reveal the selected private text.

**D-7 (2026-09-27): renumbered.** Drafted as 156 on an unmerged branch whose
ordinals 156 through 158 were taken on `main` first; refiled as 159 with its
content unchanged except for spec 155 as merged and a Verification block that
fails before the build.

## Verification

Each named test target fails before the build, because it does not exist while
`implementation: pending`. `cargo test --workspace` is the regression floor and
passes either way.

```verify:cli
cargo test -p spec-spine-core --test context_packet --locked
cargo test -p spec-spine-cli --test context_packet --locked
cargo test --workspace --locked
```
