---
id: "155-selected-content-accessor"
title: "Select bounded content from one repository snapshot"
status: draft
kind: "governance"
created: "2026-09-26"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "106-obligations-are-declared-constraints"
  - "152-a-refusal-says-what-it-is-in-every-form"
summary: >
  Adds a deterministic, read-only selected-content request and response over
  one explicitly identified repository snapshot. Callers may select a whole
  spec, spec section, obligation, owned source unit, explicit file, explicit
  directory member, symbol, module, or deterministically supported test, with
  bounded projections, stable spans and digests, canonical ordering,
  continuation, explicit omissions, and no execution or authority effect.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/content.rs" }
  - { kind: file, path: "crates/spec-spine-core/src/content.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/content.rs" }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_content.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/content.rs" }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-core/tests/scope.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-cli/tests/scope.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/sections.rs" }, role: "existing section resolver" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/symbols.rs" }, role: "existing symbol and module resolver" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every returned item identifies one repository snapshot, selector, projection, path, inclusive span, digest, resolution state, completeness state, and content."
    anchor: "3-6-every-returned-item-is-self-identifying"
  - id: "R-2"
    kind: requirement
    text: "Selection order, serialization, budgets, omissions, and continuation are deterministic for the same request and snapshot."
    anchor: "3-9-ordering-budgets-and-continuation"
  - id: "R-3"
    kind: requirement
    text: "An ambiguous, stale, escaping, changed, or unsupported selection is explicit and never silently resolves to different content."
    anchor: "3-11-refusals-and-explicit-omissions"
  - id: "I-1"
    kind: invariant
    text: "Selected content is untrusted data and selection never executes it, fetches a repository, composes repositories, edits a file, or grants authority."
    anchor: "3-13-authority-security-and-privacy-boundary"
  - id: "V-1"
    kind: verification
    text: "The implemented accessor is checked for stable ordering, canonical JSON, snapshot changes, ambiguity, budgets, continuation, unsupported projections, spans, path containment, and dirty-tree refusal."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/content.rs"
      - "crates/spec-spine-cli/tests/content.rs"
intent:
  goal: "return bounded deterministic source and specification content from one immutable repository snapshot"
  non_goals:
    - "putting arbitrary content inside ContextClosure"
    - "semantic program understanding or inferred dependencies"
    - "cross-repository snapshot composition"
    - "remote fetching, source execution, editing, admission, or authority"
---

# 155: Select bounded content from one repository snapshot

## 1. Purpose

The registry, index, obligation read, and ContextClosure answer which governed
things exist and how they relate. They do not return the bounded source or spec
content a documentation generator, review tool, or agent needs to read. Each
consumer currently reopens files, reimplements section and symbol resolution,
and invents its own ordering, spans, truncation, and snapshot binding.

This spec defines one narrow accessor. It selects declared content from one
repository snapshot and returns a canonical read document whose omissions are
as explicit as its items. It is the first unit in the refreshed documentation
and agent-context roadmap. Context packets compose this accessor later; they do
not change its repository-scoped boundary.

## 2. Territory

- `crates/spec-spine-types/src/content.rs` defines the request, selector,
  projection, snapshot identity, item, omission, continuation, and response
  types.
- `crates/spec-spine-core/src/content.rs` resolves selectors and projections
  against one supplied repository tree without Git, network, execution, or
  writes.
- `crates/spec-spine-core/src/lib.rs` exposes typed functions and
  `selected_content_json(config_json, repo_root, request_json,
  snapshot_json)` through the existing JSON facade.
- `crates/spec-spine-cli/src/cmd_content.rs` and `main.rs` expose
  `content select --request <path> --repository <identity>
  [--revision <rev>] --json`.
- The planned core and CLI test files establish the negative and deterministic
  cases. The API, CLI, and schema-versioning documents explain the public
  contract.

The existing registry, index, section, symbol, module, obligation, canonical
JSON, hash, and error implementations remain owned by their current specs.
This unit consumes them. It does not alter their answers.

## 3. Behavior

### 3.1 One request names one repository

A request MUST contain:

```json
{
  "selectors": [],
  "defaultProjection": "full",
  "maxBytes": 262144,
  "maxItems": 64,
  "continuation": null
}
```

`selectors` MUST contain at least one selector. `defaultProjection` defaults to
`full`. `maxBytes` defaults to 262144 and MUST be from 1 through 1048576.
`maxItems` defaults to 64 and MUST be from 1 through 256. Unknown members are a
usage error. These bounds apply to returned item content bytes, after LF
normalization, and to item count. Envelope metadata and omissions do not spend
the byte budget.

Repository identity is supplied separately because it is an opaque identity
the caller owns, not a remote URL spec-spine should infer. One request MUST NOT
name or open a second repository.

### 3.2 Snapshot identity is explicit

Every successful response and every item MUST carry:

- `repository`: the caller's non-empty opaque repository identity;
- `revision`: the full commit identity selected by the CLI;
- `tree`: that commit's full tree identity; and
- `dirtyState`: `clean-working-tree` or `clean-export`.

With no `--revision`, the CLI resolves `HEAD` and its tree, requires the
working tree and index to be clean, reads the requested content, then repeats
the HEAD, tree, and cleanliness reads. A dirty tree is refused before content
is returned. A state that changes during selection is refused as
`snapshot-changed`, even if every selected file happens to have the same
bytes.

With `--revision`, the CLI resolves the argument to one commit, records its
tree, exports that exact tree to a temporary directory by the same bounded Git
mechanism used by `delta`, and calls core on the export. The result records
`clean-export`. An unavailable revision, failed export, or tree mismatch is a
failure, never a fallback to HEAD.

Core and the JSON facade remain Git-free. Their `snapshot_json` input carries
the four fields above plus `binding: "caller-supplied"`; the caller is
responsible for supplying a clean export with that identity. The core hashes
the returned content but does not claim that an arbitrary directory came from
the asserted Git object. The CLI is the first-party binding that verifies it.

### 3.3 Selector vocabulary

Each selector has exactly one `kind`, its kind-specific identity, an optional
`projection`, and optional `required` (default `true`). The supported kinds
are:

| Kind | Identity and resolution |
|---|---|
| `spec` | Full or uniquely resolvable short spec id. Selects its `spec.md`. |
| `spec-section` | Spec id plus one exact section anchor from that spec. |
| `obligation` | Qualified `<spec-id>#<obligation-id>` under spec 106. |
| `owned-unit` | Spec id plus one unit identity exactly as its compiled record declares it. The unit MUST be owned by that spec and resolve through the committed, fresh index. |
| `file` | One repository-relative file path. |
| `directory-member` | One repository-relative directory plus one relative member path. It selects that member only and never enumerates the directory. |
| `symbol` | One exact symbol id supported by the existing structural resolver. |
| `module` | One exact module id supported by the existing structural resolver. |
| `test` | One exact Rust test function symbol carrying `#[test]` or `#[tokio::test]`, when the structural resolver can bind the attribute and function to one span. |

A test generated by a macro, a parameterized case without one stable function
identity, a doctest fragment, or a string-named JavaScript test is
`unsupported-selector`. Later resolver specs may add deterministic test kinds;
this accessor MUST NOT guess them.

Paths use the repository path type and containment rules of spec 144. Absolute
paths, parent traversal, Windows drive or stream forms, symlink escape, and a
directory member outside its named directory are refused.

### 3.4 Resolution uses current typed contracts

Spec and obligation selectors resolve through the fresh committed registry.
Owned units, symbols, modules, sections, and tests resolve through the fresh
committed index and its existing resolvers. Explicit file and directory-member
selectors read the named path directly after the same exclusion, containment,
and symlink checks.

An owned-unit selector MUST NOT degrade into a bare path selection if its unit
does not resolve. A symbol or module MUST NOT degrade into its containing file.
A short spec id MUST follow the one spec-id policy and ambiguous identity is a
refusal, not first-match selection.

### 3.5 Projections are closed and conservative

The projection enum is `full`, `declaration`, `signature`, `documentation`, or
`body`. Support is deliberately narrower than the selector vocabulary:

| Projection | Deterministic support |
|---|---|
| `full` | Every selector that resolves to one textual span. |
| `declaration` | An obligation's exact frontmatter entry; a symbol, module, or supported test declaration span produced by the structural grammar. |
| `signature` | A Rust or TypeScript symbol or supported Rust test for which the grammar returns one contiguous signature span. |
| `documentation` | Contiguous leading Rust or TypeScript documentation comments structurally attached to a symbol, module, or supported test. Absence is explicit. |
| `body` | A spec body without frontmatter; one spec section; or a symbol, module, or supported test body span returned by the grammar. |

An explicit file or directory member has no separate declaration, signature,
documentation, or body. A spec has no signature. A projection not listed for a
selector MUST produce an `unsupported-projection` omission. It MUST NOT return
`full` content under a different requested projection.

### 3.6 Every returned item is self-identifying

Every item contains:

- `identity`: the canonical selector identity;
- the response-level `repository`, `revision`, `tree`, and `dirtyState`;
- `selector` and `requestedProjection`;
- `path`, as a repository-relative POSIX path;
- `span`, with inclusive, one-based `startLine` and `endLine`;
- `digest`, `sha256:` plus lowercase hex;
- `content`, as normalized UTF-8 text;
- `resolution`: `resolved`;
- `completeness`: `complete`; and
- `schemaVersion`, equal to the enclosing read document's version.

Items are never truncated inside a line or selector. Content is BOM-stripped
and CRLF or CR normalized to LF before span extraction and hashing. The digest
uses spec 077's framing over the canonical identity, projection, path, span,
and normalized content, so equal text selected through different identities
does not claim to be the same item. A non-UTF-8 target produces an explicit
`binary-content` omission; selected-content v1 is text only.

### 3.7 Spans are portable and inclusive

Spans use normalized text and count Unicode scalar-independent lines, not byte
columns. The first line is 1, and both ends are inclusive. A whole empty file
has span 1 through 1 with empty content. A final newline does not create an
extra selected line. Existing section, symbol, and module spans are reused
only when they obey this rule; an implementation MUST adapt at the accessor
boundary rather than change an established resolver's contract silently.

### 3.8 Canonical identities and deduplication

Canonical identities are kind-prefixed strings containing the fully resolved
spec id or normalized path and the exact subordinate identity. Examples are
`spec:106-obligations-are-declared-constraints`,
`section:106-obligations-are-declared-constraints#3-5-every-section-has-a-digest-beside-the-spec-s-identity`,
`obligation:106-obligations-are-declared-constraints#R-5`, and
`file:crates/spec-spine-core/src/query.rs`.

Two selectors resolving to the same canonical identity and projection produce
one item. The response records the duplicate selector positions in
`coalescedSelectors`; no request member silently disappears.

### 3.9 Ordering, budgets, and continuation

Resolution is completed before ordering. Items and omissions are sorted by
canonical identity, projection, then original selector position. The response
is emitted by the read-document canonical writer: sorted keys, two-space
indentation, LF, and one trailing newline.

Items are admitted in that order while both budgets permit. Pagination occurs
only between items. When at least one otherwise returnable item remains, the
response has `completeness: "partial"` and a continuation object containing:

- `requestDigest`: the digest of the canonical request without continuation;
- `snapshotDigest`: the digest of repository, revision, tree, and dirtyState;
- `nextIdentity`: the first unreturned canonical identity; and
- `nextOrdinal`: its zero-based position in the ordered resolved set.

Supplying the continuation with a different request or snapshot is refused as
`stale-continuation`. Repeating it on the same snapshot starts exactly at the
named identity and ordinal. A continuation after the ordered set changed is
refused, never rounded to the nearest member.

### 3.10 Completeness and omissions

The response completeness is:

- `complete` when every distinct selector returned one complete item;
- `partial` when only the item or byte budget stopped the page and a valid
  continuation exists; or
- `incomplete` when any selector is missing, unsupported, binary, or too large
  to fit as one item.

Every non-returned selector produces an omission with its canonicalizable
identity, requested projection, `required`, reason code, and human message.
Reasons include `missing-content`, `unsupported-selector`,
`unsupported-projection`, `binary-content`, and `item-exceeds-byte-budget`.
An oversized item is not repeatedly offered through continuation. Required and
optional omissions use the same explicit shape; `required` lets a later packet
contract decide whether the omission refuses that packet.

### 3.11 Refusals and explicit omissions

The whole request is refused when its shape or budgets are invalid, a selector
is ambiguous, a path escapes, the registry or index needed for resolution is
stale or unreadable, the snapshot is dirty or changes during the read, or a
continuation does not bind to the request and snapshot. Missing and unsupported
individual selectors are omissions so a batch can answer every selector in
one deterministic response.

Under `--json`, every refusal uses spec 152's error envelope and family exit
contract. A successful complete, partial, or incomplete selection is a read
document, not a verdict, and exits 0. Consumers MUST inspect `completeness`.

### 3.12 Schema and compatibility

Selected content joins the read schema axis under spec 074. The implementation
MUST move `READ_SCHEMA_VERSION` by MINOR, update the schema-version table, and
add the document to the closed read inventory. The request and response types
are public owned DTOs, and the JSON facade remains additive.

No committed registry or index schema changes. Existing resolver results,
ContextClosure, interface verification, gates, and attestation bytes remain
unchanged. A future projection or selector is an additive read-schema MINOR
only when old readers can retain their current interpretation; removing or
retyping one is MAJOR.

### 3.13 Authority, security, and privacy boundary

Returned content is untrusted data. The accessor MUST NOT interpret
instructions inside it, execute it, follow links, fetch remote content, or
admit any claim. It MUST NOT read outside the repository export, include Git
remote URLs, environment values, credentials, ignored secrets, or unselected
content. Budgets are enforced before allocating the final response, and every
filesystem walk remains within existing resolver exclusions and containment
rules.

Selection grants no permission to edit, run, publish, accept, or deploy. It
does not decide whether the returned content is correct, current in another
repository, approved, or safe to show without a consumer-side redaction and
prompt-injection boundary.

## 4. Acceptance criteria

1. Whole-spec, section, obligation, owned-unit, explicit file,
   directory-member, symbol, module, and supported Rust test selectors return
   the exact bounded content and identity described above.
2. Ordering, deduplication, canonical JSON, item digests, spans, pagination,
   and continuation are byte-identical for repeated reads of one snapshot.
3. Dirty trees, a snapshot changed during selection, stale continuations,
   ambiguous selectors, escaping paths, and stale required ledgers refuse with
   the family error contract.
4. Missing content, removed units, withdrawn obligations, unsupported tests or
   projections, binary targets, and oversized items are explicit and never
   silently substituted or dropped.
5. LF, CRLF, and CR fixtures yield equal normalized content, spans, and item
   digests on supported platforms.
6. Existing registry, index, closure, interface, gate, and attestation tests
   remain byte-compatible.

## 5. Out of scope

- Adding content fields to `ContextClosure` or changing its digest.
- Inferring semantic dependencies, call graphs, meaning, authority, or test
  coverage.
- Enumerating arbitrary directory trees or returning binary content.
- Cross-repository consistency, packet composition, remote repository fetches,
  or mutable snapshot storage.
- Applying edits, formatting, executing tests or source, generating
  documentation, admitting evidence, or choosing provider policy.
- Supporting macro-generated, parameterized, doctest, or string-named tests
  until a separate resolver spec gives them deterministic identity and spans.

## 6. Resolved decisions

**D-1 (2026-09-26): the CLI binds Git; core stays Git-free.** This preserves
the workspace purity rule while letting the first-party command prove
revision, tree, and cleanliness. A facade caller receives the same content
contract but owns the assertion that its root is the supplied snapshot.

**D-2 (2026-09-26): repository identity is opaque and caller-supplied.** A
remote URL may contain credentials, may be absent, and is not a stable product
identity. The accessor carries the identity without discovering or validating
it.

**D-3 (2026-09-26): v1 is text and line based.** Bytes remain available to
attestation. Selected content normalizes text so Windows and Unix checkouts
agree; binary access needs a separately justified contract.

**D-4 (2026-09-26): pagination never splits an item.** Split content would
make spans and per-item digests page-dependent. An oversized item is one
explicit omission.

**D-5 (2026-09-26): unsupported projections are data, not fallback.** Returning
full content for an unsupported signature or documentation request would make
the `requestedProjection` member false.

**D-6 (2026-09-26): tests are supported only through exact structural
identity.** Rust functions with recognized test attributes are the first
bounded case. Expanding this set belongs to the resolver roadmap, not to
heuristics in the accessor.

## Verification

```verify:cli
cargo test -p spec-spine-core --test content --locked
cargo test -p spec-spine-cli --test content --locked
cargo test --workspace --locked
```
