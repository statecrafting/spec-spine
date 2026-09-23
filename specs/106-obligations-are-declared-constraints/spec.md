---
id: "106-obligations-are-declared-constraints"
title: "Obligations are declared constraints"
status: approved
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "070-an-authority-snapshot-says-what-it-read"
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
summary: >
  A spec's normative content is prose, so nothing downstream can name a single
  requirement, cite it, or tell whether the sentence it cited has changed. This
  adds an optional `obligations` frontmatter key: three kinds, ids stable
  within a spec, an anchor validated against the spec's own headings, explicit
  inputs for a verification, and withdrawal in place rather than deletion. The
  registry gains a digest for every section of every spec, beside the spec's
  full content hash, and a read resolves a qualified `<spec-id>#<obligation-id>`
  reference. No gate reads any of it.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/obligation.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/obligations.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/obligations.rs" }
extends:
  # 3.1 - 3.3: the frontmatter grammar and the registry record.
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  # 3.9: the registry schema MINOR.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry.schema.json" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json" }, nature: additive }
  # 3.3 - 3.5, 3.7: validation and section digests at compile.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
  - { spec: "020-keypath-section-anchors", unit: { kind: file, path: "crates/spec-spine-core/src/sections.rs" }, nature: additive }
  # 3.6: the read that resolves a qualified reference.
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-core/src/query.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_registry.rs" }, nature: additive }
  # 3.10: the documentation an author and a consumer read.
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  # D-10: this spec changes what the producer emits for the fixture corpus, so
  # it regenerates spec 103's set, on 103 §3.9's authority path.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: corrective }
  # D-12: the read axis moves for the new document, and its pin moves with it.
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  # 3.9: the pin every registry MINOR moves (026, 063 and 082 did the same).
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  # D-8: one existing test pinned the head registry version.
  - { spec: "082-an-amended-acceptance-is-the-one-that-runs", unit: { kind: file, path: "crates/spec-spine-cli/tests/cli.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: context }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Obligations are declared under one frontmatter key, `obligations`, and nowhere else."
    anchor: "3-1-declarations-live-in-frontmatter"
  - id: "R-2"
    kind: requirement
    text: "An obligation's kind is exactly one of requirement, invariant or verification."
    anchor: "3-2-three-kinds-and-no-fourth"
  - id: "R-3"
    kind: requirement
    text: "An obligation id is unique within its spec, and an obligation is withdrawn in place, keeping its id, never deleted."
    anchor: "3-3-ids-are-stable-within-a-spec-and-withdrawal-is-in-place"
  - id: "R-4"
    kind: requirement
    text: "An obligation's anchor resolves to exactly one heading in its own spec's body, or compile refuses it."
    anchor: "3-4-an-anchor-is-validated-not-asserted"
  - id: "R-5"
    kind: requirement
    text: "The registry carries a digest for every section of every spec, beside and never instead of the spec's full content hash."
    anchor: "3-5-every-section-has-a-digest-beside-the-spec-s-identity"
  - id: "R-6"
    kind: requirement
    text: "A reference to an obligation is qualified as `<spec-id>#<obligation-id>`, and an unqualified one is refused, never resolved locally."
    anchor: "3-6-a-reference-is-qualified-and-a-read-resolves-it"
  - id: "R-7"
    kind: requirement
    text: "A verification obligation declares its inputs explicitly, and no other kind declares any."
    anchor: "3-7-a-verification-declares-its-inputs"
  - id: "I-1"
    kind: invariant
    text: "No gate verdict reads an obligation or a section digest."
    anchor: "3-9-compatibility"
  - id: "V-1"
    kind: verification
    text: "Each obligation rule is refused at compile with its code, section digests move only with their section, and a corpus without the key keeps every shardHash."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/obligations.rs"
      - "crates/spec-spine-cli/tests/obligations.rs"
---

# 106: Obligations are declared constraints

## 1. Purpose

Everything a spec requires is prose. The corpus can say which spec owns a file
and which specs an edge connects; it cannot say which **requirement** a change
answers, which requirement an acceptance command tests, or which requirement a
successor amends. A reviewer reads "§3.5" in a commit message, and the number
moves the next time a section is inserted above it. A machine reads nothing.

That is the gap note 04 §4.4 records as A01, A02 and A08. Its prerequisite, an
authority snapshot that names what it read, has had an owning spec since 070.

### 1.1 Why it is built now (design note 09, D-7)

This was filed on 2026-09-21 as a deferred contract under "specify now,
implement only for a named consumer need". The owner replaced that rule on
2026-09-22 with an opportunity-led evaluation. Recorded here as the evaluation
requires:

- **What becomes possible.** A requirement, invariant or verification is an
  addressable thing, `<spec-id>#<obligation-id>`, with its text, the heading
  that states it in full, and a digest of that heading's section. Commit
  messages, acceptance lines, review comments and orchestrators can cite a
  requirement instead of a paragraph number, and can tell when the sentence
  they cited changed.
- **Why it is distinctive.** It is the substrate every later contract in note
  09 section 1.3 builds on: context closures (107), impact and conflict sets
  (109), digest-pinned interface references (110) and waiver lifecycles (113)
  all name obligations or sections. No tool in this family gives a requirement
  a stable, hash-checkable identity tied to the prose it came from.
- **One integration scenario.** A Statecraft work order records
  `100-a-deleted-path-is-judged-where-it-lived#R-1` together with the section
  digest it was issued against. At review the platform reads the digest again;
  if it moved, the order is flagged for re-reading before its result is
  accepted.
- **What remains hypothetical.** No consumer reads obligations today, and no
  corpus but this one declares any. This spec declares its own, so the feature
  is exercised by the corpus that ships it.
- **The smallest coherent contract.** Section 3: one optional key, three
  kinds, validated anchors, section digests, one read. No gate.
- **Dependencies, compatibility, evidence.** Registry schema MINOR, no
  `shardHash` moves, no verdict moves; the acceptance below.

## 2. Territory

- `crates/spec-spine-types/src/obligation.rs` (established): the authored and
  compiled shape.
- The frontmatter grammar, the registry record, the registry schema version and
  both registry JSON Schemas (`extends`).
- `compile.rs` for validation and section digests, `sections.rs` for the one
  section-text rule (`extends`).
- `query.rs`, the `query_json` facade and `cmd_registry.rs` for the read
  (`extends`).
- The authoring template, `docs/api.md` and `docs/schema-versioning.md`
  (`extends`).
- `crates/spec-spine-core/tests/obligations.rs` and
  `crates/spec-spine-cli/tests/obligations.rs` (established).

## 3. Behavior

### 3.1 Declarations live in frontmatter

Obligations MUST be declared in a spec's YAML frontmatter under the one key
`obligations`, a list. There MUST NOT be a second declaration syntax, in
particular no fenced block in the body. The corpus already has exactly one
place a machine reads a spec's structure; a second grammar would have its own
parser, its own errors, and its own way of disagreeing with the first.

```yaml
obligations:
  - id: "R-1"
    kind: requirement
    text: "A deleted path is judged at the snapshot preceding its segment."
    anchor: "3-1-two-snapshots-and-which-one-answers"
  - id: "V-1"
    kind: verification
    text: "A shallow clone that cannot read its range exits 3."
    anchor: "3-5-required-prior-evidence-is-a-refusal"
    inputs: ["crates/spec-spine-cli/tests/couple.rs"]
  - id: "R-2"
    kind: requirement
    text: "The superseded wording, kept so its citations still resolve."
    anchor: "3-2-the-old-rule"
    withdrawn: true
```

Each entry has exactly the members `id`, `kind`, `text`, `anchor`, and
optionally `inputs` and `withdrawn`. An unknown member, a missing required
member, or a value of the wrong type MUST be refused as malformed frontmatter
(`V-002`), the way every other frontmatter defect is.

### 3.2 Three kinds and no fourth

`kind` MUST be one of exactly three:

| `kind` | What it asserts |
|---|---|
| `requirement` | something the implementation MUST do |
| `invariant` | something that MUST remain true across every state |
| `verification` | something an executable check MUST establish |

A fourth kind is not adopted (note 09 D-5). One whose boundary with these three
is unclear produces miscategorisation rather than information, and a new kind
is a MINOR addition later if someone can say what it carries that these cannot.

### 3.3 Ids are stable within a spec, and withdrawal is in place

- An `id` MUST match `^[A-Za-z][A-Za-z0-9]*(-[A-Za-z0-9]+)*$`, so it can never
  contain the `#` a qualified reference uses, and MUST be unique within its
  spec. It is not globally unique and MUST NOT be read as such. A violation is
  `V-021`.
- `text` MUST be non-empty after trimming (`V-024`).
- An obligation MUST NOT be deleted to retire it. It is **withdrawn in place**:
  `withdrawn: true`, keeping its id, kind, text and anchor. A withdrawn
  obligation still occupies its id, so reusing the id for a different
  obligation is a duplicate and is refused by the uniqueness rule above.

The last rule is how "an id is never reused" is enforced without history.
Compile is a pure function of the files in front of it and reads no git, so it
cannot see an obligation that was deleted. Deleting a tombstone is therefore an
edit compile cannot refuse; it is visible to anyone who compares two registries,
because the ids are in the shards, and that comparison is where it belongs.

### 3.4 An anchor is validated, not asserted

`anchor` is REQUIRED and names a heading in the declaring spec's body, in the
slug form the corpus already computes for markdown section units
(`sections::anchor_of`, spec 020). It MUST resolve to **exactly one** heading;
a missing, dangling or ambiguous anchor MUST be refused at compile (`V-022`),
naming the anchor. The body is the text after the frontmatter, so a `#` comment
in the frontmatter can never be mistaken for a heading.

The anchor ties a machine-readable obligation to the prose that states it in
full. An anchor nobody checks is a comment, and a spec whose obligations point
at headings it has since renamed is worse than one with no anchors, because a
reader trusts it.

### 3.5 Every section has a digest, beside the spec's identity

A spec's identity stays what it is: the content hash over its whole `spec.md`,
which the registry shard records as `shardHash` (spec 022) and `registry show`
reports as `contentHash` (spec 048).

Alongside it, every spec's registry record MUST carry `sectionDigests`: for
every heading in its body, the heading's anchor mapped to a digest of that
section's text. A section runs from its heading line to the line before the
next heading of the same or a shallower level (spec 020's markdown rule), so a
section includes its subsections. When two headings share an anchor, the first
one's section is the one digested, which is the one `resolve_section` answers.

The digest is the corpus's one hash construction (spec 077): SHA-256 over
`<specPath>#<anchor>`, a NUL byte, and the section's lines joined with LF and
ending in LF, normalized as every hashed input is.

Both identities are required and neither substitutes for the other. The full
hash is what makes a ledger reproducible; a section digest is the finer grain a
consumer needs to tell "the sentence my obligation points at changed" from
"some other part of this spec changed". Every section is digested, not only
anchored ones, so a later reader of a section (107, 110) reads a digest from the
ledger instead of recomputing one the ledger does not hold.

### 3.6 A reference is qualified, and a read resolves it

A reference to an obligation MUST be **qualified**: `<spec-id>#<obligation-id>`.
The spec part follows the corpus's one spec-id policy (spec 084): a full id, or
a short form that resolves to exactly one spec.

`spec-spine registry obligation <reference> [--json]` MUST resolve one:

- an unqualified reference (no `#`, or an empty half) MUST be refused as a usage
  error (exit 3) whose message says a qualified form is required. It MUST NOT be
  resolved against any spec, because a reference that means different things
  depending on where it is read is worse than one that fails;
- a spec or obligation that does not exist MUST be `NotFound` (exit 1);
- otherwise the answer is a read document (spec 074) with the spec's `id` and
  `specPath`, the `obligation` as compiled, the `sectionDigest` of its anchor,
  and the spec's `contentHash` read from its committed shard, never recomputed
  (spec 048 §3.3). As with `registry show`, a spec whose shard cannot be found
  is reported without `contentHash` rather than refused; spec 048 §3.3 chose
  that degrade so the rest of the record stays readable, and this read keeps
  the same boundary.

A withdrawn obligation resolves, with `withdrawn: true`, so a citation of it
still says what it cited and that it no longer holds.

The `query_json` facade MUST answer the same question as op `obligation` with
`id` set to the reference, over the registry text it is given. It carries no
`contentHash`, because the registry document does not; that is the same
boundary `show` keeps.

### 3.7 A verification declares its inputs

A `verification` obligation MUST declare at least one input, and an obligation
of any other kind MUST NOT declare any (`V-023`). Inputs are repo-relative paths
or commands, written as strings. They MUST NOT be inferred from the spec's
`## Verification` block, its territory, or anything else, and compile does not
check that a path exists: an input is a declaration of what the evidence is,
not a claim that it is present.

Inference is how a declaration becomes untrue without anyone editing it.

### 3.8 What an obligation establishes, and what it does not

An obligation record establishes:

- that a spec **declares** this requirement, invariant or verification;
- that the declaration is **related** to named evidence: a verification names
  inputs, and every obligation names the section that states it.

It does **not** establish:

- **that a verification result is true.** A verification naming a test says
  the corpus declares that test as the evidence. It says nothing about whether
  the test passed, can fail, or tests what it claims.
- **that the declarations enumerate every requirement.** The set is what the
  author wrote down. Nothing may read "no obligation says otherwise" as
  "nothing else is required".

### 3.9 Compatibility

The `obligations` key is optional; a spec without it is valid, which is every
spec but this one on the day it is built.

`obligations` and `sectionDigests` on the registry record are additive, so the
registry schema takes a MINOR, `1.3.0` to `1.4.0`, under
`docs/schema-versioning.md`. Every shard restamps its `specVersion` and gains
`sectionDigests`; no `shardHash` moves, because the hash is over `spec.md`
source bytes. Both JSON Schemas declare the two members, and the conformance
test holds emitted shards to them.

No gate verdict changes. `couple`, `check`, `lint` and `index coverage` MUST NOT
read obligations or section digests. Clearance stays "an owning spec's
`spec.md` is in the diff"; a gate that also demanded the right obligation be
cited would refuse most legitimate work on the day it shipped.

### 3.10 Documentation

The authoring template MUST document `obligations` with its members, the
three kinds, withdrawal in place, and the four codes. `docs/api.md` MUST
document the facade op; `docs/schema-versioning.md` MUST record the registry
MINOR and the two new members.

## 4. Out of scope

- **Obligation-aware coupling or any other gate.** 3.9.
- **A completeness check** of any kind. 3.8.
- **Detecting a deleted tombstone.** 3.3: it needs two registries, and the
  comparison belongs to whoever holds both.
- **Obligation granularity on `amends`.** A successor naming
  `<spec>#<obligation>` rather than a section anchor is the obvious next
  increment. `amends_sections` already exists, and two overlapping
  granularities need a decision between them first.
- **ContextClosure** (107), **impact and conflict sets** (109).
- **Migrating existing prose into obligations.** A corpus adopts the key spec
  by spec, or never.

## 5. Resolved decisions

**D-1 (2026-09-22, built under D-7, not deferred).** Filed on 2026-09-21 as a
deferred contract claiming no territory. The owner's D-7 (design note 09
section 10) authorizes it; 1.1 records the evaluation. It now claims the
territory it builds, `implementation` is set, and the filing's "no territory"
rule, which existed to keep an unbuilt contract from making the gate red, no
longer applies to it.

**D-2 (2026-09-22, correction: "no reuse after removal" becomes withdrawal in
place).** The draft required a validation error when an id is reused for a
different obligation "after the original is removed". Compile is a pure
function of the files it reads (the engine's first invariant), so it cannot see
a removed obligation. The rule as written could not be built. It is replaced by
3.3: removal is by `withdrawn: true`, which keeps the id occupied, so reuse is
an ordinary duplicate. This also decides the draft's open question 2 (is an
obligation removable): not by deletion.

**D-3 (2026-09-22, correction: the anchor is required).** The draft required an
anchor to resolve when present and never said it could be absent. An
obligation without an anchor has no prose it answers to and no section digest,
which is most of what makes it worth declaring. Required, and ambiguity (two
headings with one slug) is refused along with absence.

**D-4 (2026-09-22, correction: a digest for every section).** The draft asked
for a digest per **anchored** section. Spec 107 digests arbitrary sections a
closure names, and spec 110 pins sections of another corpus; both would have to
recompute digests the ledger does not hold, which is the "recomputed, not read"
failure spec 048 §3.3 refuses for content hashes. So every section is
digested. It is the draft's open question 3 decided: the digest lives in the
registry record.

**D-5 (2026-09-22, correction: the reference rule needs a reader).** The draft
said a cross-spec reference must be qualified and named nothing that reads
one, so the rule could not be exercised. 3.6 adds the one read that does,
through the CLI and the facade. Contexts that carry references (107, 109) reuse
its parser.

**D-6 (2026-09-22, correction: the draft's "a fourth kind is a MINOR later").**
Kept, and bounded: 3.2 fixes the closed set, and a later kind is additive to it.

**D-7 (2026-09-22, build: the read axis does not move; its second half superseded by D-12).** `registry list` and
`show` now carry `obligations` and `sectionDigests`, because they emit the
registry record. `docs/schema-versioning.md` already says the read axis
"does not move when `REGISTRY_SCHEMA_VERSION` ... does": a record's members are
versioned by the registry schema, which takes the MINOR here. The
`obligation` answer is a new document, not a member added to an existing one,
so it does not move the read axis either. Spec 102 moves that axis separately
and for its own reason.

**D-8 (2026-09-22, build: a pin on the head registry version).**
`cli.rs`'s `spec103_registry_show_carries_amends_verification` asserted a
shard's `specVersion` equals `1.3.0`. That is spec 082's MINOR, and every later
MINOR would break it without 082's contract changing. It now asserts MAJOR 1
and at least MINOR 3, which is what 082's test was about. Spec 082's own
acceptance does not pin the value and is unaffected. `cli.rs` is 082's
territory, declared here as an `extends` edge.

**D-9 (2026-09-22, build: the guards were made to fire).** With the ambiguous
arm of the anchor check folded into the valid one, the ambiguity test fails.
With a section digest taken over the whole body instead of the section, the
two digest tests fail. Both restored, all thirteen core tests pass. This spec
declares its own nine obligations, so every compile of this repository
exercises the grammar, the anchors and the digests on real prose.

**D-10 (2026-09-22, integration with spec 103: the verifier fixtures are
regenerated).** Merged with spec 103, the fixture harness failed as `STALE
FIXTURES`: every registry record now carries `sectionDigests`, so the
attested `registryHash` of the fixture corpus moved, and so did every case's
`attestationHash`. That is the harness doing its job, not noise to adapt away.
The set was regenerated with 103 §3.9's documented command against this
build; the diff is those two digests in each case and nothing else, and a
second run rewrites nothing. The directory is declared as a `corrective`
`extends` edge on spec 103, the authority path 103 §3.9 names.

**D-11 (2026-09-22, review: schema and compile agree on `text`).** V-024
refuses an obligation whose `text` is empty after trimming, while both registry
schemas accepted any non-empty string, so a hand-built document with
`"text": "   "` validated. Both schemas now also require `"pattern": "\\S"`,
so the schema and the compiler refuse the same values.

**D-12 (2026-09-22, integration with spec 102: the read axis moves for the new
document).** D-7 held that a new read document does not move
`READ_SCHEMA_VERSION`. Once spec 102 merged, that left the `obligation` answer
stamped `0.2.0`, a version whose changelog describes only `status` on a ready
entry: a documentation and version disagreement produced by two independent
additive changes. The axis versions the shape of an answer (spec 074), and a
new answer is a shape added, so it takes a MINOR of its own: `0.3.0`. D-7's
first half stands (`list` and `show` gain members through the registry
schema). `read.rs`'s pin moves with it, declared as an `extends` edge on spec
074, as spec 102 did.

**D-13 (2026-09-22, recorded when spec 109 moved the registry to 1.5.0).**
The acceptance pinned `REGISTRY_SCHEMA_VERSION` to exactly `1.4.0`, so spec
109's additive MINOR failed it without anything this spec requires changing:
the version-pin trap spec 085 names. The line now asserts a floor, MAJOR 1
and MINOR at least 4, which still fails if the constant regresses below this
spec's MINOR. Measured: it passes at `1.5.0` and fails at `1.3.0`. No
requirement of this spec moved; this spec is `draft`.

## Verification

Written to fail against the tree it is filed on: nothing named here exists.
This spec's own `obligations` key is added by the build that makes it a known
frontmatter key; filed before that, it would be malformed frontmatter.

```verify:cli
cargo build --release --locked
# 3.1 - 3.7: every rule, refused at compile by code, over disposable corpora.
cargo test -p spec-spine-core --test obligations --locked
# 3.6: the read through the shipped CLI, including the unqualified refusal.
cargo test -p spec-spine-cli --test obligations --locked
# 3.5, 3.9: the registry schema moved, and the shards conform to it. A floor,
# not a value pin (spec 085): MAJOR 1 and at least this spec's MINOR 4, so a
# later additive MINOR passes and a regression below 1.4.0 fails (D-13).
grep -qE 'REGISTRY_SCHEMA_VERSION: &str = "1\.([4-9]|[1-9][0-9]+)\.[0-9]+"' crates/spec-spine-types/src/version.rs
cargo test --workspace emitted_registry_conforms --locked
# This spec's own obligations compile and resolve.
./target/release/spec-spine registry obligation 106-obligations-are-declared-constraints#R-5 --json | grep -q '"sectionDigest"'
sh -c './target/release/spec-spine registry obligation 106#R-5; test $? -eq 0'
sh -c './target/release/spec-spine registry obligation R-5; test $? -eq 3'
# 3.10: the template documents the key.
grep -q '^# obligations:' standards/spec/templates/spec-template.md
# 3.9: no gate verdict moved.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
