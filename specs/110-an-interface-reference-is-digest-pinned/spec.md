---
id: "110-an-interface-reference-is-digest-pinned"
title: "An interface reference is digest-pinned"
status: approved
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "070-an-authority-snapshot-says-what-it-read"
  - "106-obligations-are-declared-constraints"
summary: >
  A spec can cite another repository's spec only in prose, so the citation
  cannot go stale visibly. This declares a cross-corpus interface reference
  pinned to the cited spec's content hash, and optionally to the digests of
  the sections it relies on, and adds one verifier that recomputes those
  digests from a corpus the caller supplies locally. A moved interface becomes
  a named, exit-coded fact. Discovery, fetching and trust in the exporter stay
  with the caller; nothing is ever pinned from what was first observed.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/interface.rs" }
  - { kind: file, path: "crates/spec-spine-core/src/interface.rs" }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_interface.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/interface.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/interface.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/interface_composed.rs" }
extends:
  # 3.1: the frontmatter grammar (`interface_references`).
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  # 3.2: the registry record and the schema MINOR.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry.schema.json" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json" }, nature: additive }
  # 3.2: validation at compile, V-032 to V-038.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
  # 3.4: the facade and the library exports.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  # 3.3: the verb, and spec 067's census of spec-id arguments (D-2).
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "067-a-short-id-names-the-same-spec-at-every-verb", unit: { kind: file, path: "crates/spec-spine-cli/tests/spec_id.rs" }, nature: additive }
  # 3.2: the pins every registry MINOR moves (106 and 109 did the same).
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  - { spec: "106-obligations-are-declared-constraints", unit: { kind: file, path: "crates/spec-spine-core/tests/obligations.rs" }, nature: additive }
  - { spec: "109-impact-and-conflict-are-declared", unit: { kind: file, path: "crates/spec-spine-core/tests/impacts.rs" }, nature: additive }
  # 3.3: the read axis moves for the new document, and its pin moves with it.
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  # 3.1: the template documents the key.
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
  # 3.3, 3.4: the documentation an author and a consumer read.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  # D-2: the registry MINOR moves the fixture corpus's registry hash, so this
  # spec regenerates spec 103's set, on 103 §3.9's authority path.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: corrective }
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "A reference names a corpus, a full spec id, a sha256 digest and an authored obtained date, and every malformed member is a compile error naming the declaring spec."
    anchor: "3-2-malformed-references-are-refused-at-compile"
  - id: "R-2"
    kind: requirement
    text: "interface verify recomputes each pin from a caller-supplied local export and answers current, sections-current, stale, missing or unverified, holding only on the first two."
    anchor: "3-3-verification-recomputes-from-a-corpus-the-caller-supplies"
  - id: "R-3"
    kind: requirement
    text: "A stale committed registry is refused (exit 2) before any export is read."
    anchor: "3-3-verification-recomputes-from-a-corpus-the-caller-supplies"
  - id: "I-1"
    kind: invariant
    text: "Nothing fetches, discovers or trusts a corpus, and no verb writes or fills in a pin from what it observed."
    anchor: "3-6-a-reference-is-updated-deliberately"
  - id: "I-2"
    kind: invariant
    text: "compile, check, lint, index and couple reach the same verdicts whether or not references are declared."
    anchor: "3-7-nothing-else-gates-on-a-reference"
  - id: "V-1"
    kind: verification
    text: "Every compile rule and every verifier outcome is exercised against pins copied from a compiled exporter, through the library, the facade and the shipped binary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/interface.rs"
      - "crates/spec-spine-cli/tests/interface.rs"
      - "crates/spec-spine-cli/tests/interface_composed.rs"
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

### 1.1 The dependencies

- **Spec 070** (the authority snapshot) is where this corpus already writes
  down what it read and what it hashed to. A pin is the same idea pointed at
  another corpus.
- **Spec 106** gives every section a digest. A reference that relies on two
  sections of a long spec can pin those two, and survive an edit elsewhere in
  the cited spec without losing the ability to say whether what it relies on
  moved.

### 1.2 What it enables (D-1)

- **Not possible today:** a spec here can say "this bridge line implements
  statecraft-cli's §3.13", and nothing will notice when §3.13 is rewritten.
- **With this:** the citation carries the digest §3.13 had when it was read;
  `interface verify`, handed a local checkout of the cited corpus, recomputes
  it and answers `current`, `sections-current`, `stale`, `missing` or
  `unverified`, with exit code 0 or 1.
- **Scenario (hypothetical, no consumer has asked):** a Statecraft release job
  checks out the producer repositories it pins, runs `interface verify` with
  one `--export` per checkout, and blocks the release on any `stale` citation,
  printing which section moved.
- **Why it matters:** it is the cross-repository counterpart of the coupling
  gate, and it keeps the engine's invariant: a pure function of files the
  caller supplied, no network, no ambient trust.

## 2. Territory

- `crates/spec-spine-types/src/interface.rs`: the reference and report
  shapes.
- `crates/spec-spine-core/src/interface.rs`: the pure verifier and the local
  export reader.
- `crates/spec-spine-cli/src/cmd_interface.rs`: the `interface verify` verb.
- The two test files, and every other file the build touches as an `extends`
  edge on its owner.

## 3. Behavior

### 3.1 A reference names a corpus, a spec and a digest

```yaml
interface_references:
  - corpus: "statecraft-cli"
    spec: "002-environment-lifecycle"
    digest: "sha256:3f5c...64 hex..."
    sections:
      - anchor: "3-13-the-managed-instruction-file"
        digest: "sha256:9a01...64 hex..."
    obtained: "2026-09-21"
    rationale: "this repository inserts the bridge line that section specifies"
```

- **`corpus`** is a name, `^[a-z0-9][a-z0-9._-]*$`, not a URL and not a path.
  This spec defines no way to locate a corpus (§4).
- **`spec`** is the cited spec's **full** id in that corpus. A short id is
  refused: it can only be resolved against the other corpus, which is not
  here.
- **`digest`** is `sha256:` followed by the cited spec's content hash as the
  exporting corpus's registry records it (`contentHash` in `registry show
  --json`, the shard's `shardHash`), 64 lowercase hex digits. The algorithm is
  written out so a later construction cannot be mistaken for this one.
- **`sections`**, optional, pins anchors of the cited spec, each with the
  section digest the exporter's registry records under `sectionDigests`
  (spec 106), in the same `sha256:` form. Anchors are unique within one
  reference.
- **`obtained`** is an authored `YYYY-MM-DD` date recording when the digests
  were read. It is authored, never taken from a clock, and never rewritten.
- **`rationale`** is optional free text.

To obtain a pin, run `spec-spine registry show <id> --json` in the cited
corpus and copy `contentHash` and the needed `sectionDigests` entries, each
prefixed `sha256:`.

### 3.2 Malformed references are refused at compile

A reference with a missing or malformed member, an unsupported digest
algorithm, a short or malformed `spec`, a malformed `obtained`, a duplicate
anchor, or a second reference to the same `(corpus, spec)` pair in one spec is
a compile error naming the declaring spec and the reference.

A reference cannot be written without a digest. There is no placeholder form
and no command that fills one in: a pin that is filled in from whatever the
verifier first observed is trust on first use, which is exactly what a pin
exists to prevent (§3.6).

All of these are a pure function of one spec and are stored on its shard. The
registry record carries the references verbatim, omitted when empty; the
registry schema takes a MINOR.

### 3.3 Verification recomputes from a corpus the caller supplies

`spec-spine interface verify [--export <corpus>=<dir>]... [--spec <id>] [--json]`

- Each `--export` names a corpus and a local directory the caller asserts is
  that corpus's repository root. The verifier reads the exporter's
  `spec-spine.toml` there, when present, only for its specs directory, and
  reads `<specs-dir>/<spec>/spec.md` for each referenced spec. It reads
  nothing else, follows no link out of the named directory, and opens no
  network connection.
- For each reference (every reference in this corpus, or those declared by
  `--spec`), it recomputes the cited spec's content hash over
  `<repo-relative path>\0<normalized bytes>` (spec 077's one construction, the
  exporter's `shardHash`) and each pinned section's digest (spec 106 §3.5), and
  reports one outcome:

| Outcome | When | Exit |
|---|---|---|
| `current` | the recomputed content hash equals `digest` | 0 |
| `sections-current` | the content hash differs, the reference pins sections, and every pinned section's recomputed digest equals its pin | 0 |
| `stale` | the content hash differs and the reference pins no sections, or a pinned section's digest differs, or a pinned anchor no longer exists | 1 |
| `missing` | an export for the corpus was supplied and the spec is not in it | 1 |
| `unverified` | no export was supplied for the corpus | 1 |

- A reference that pins sections is a claim about those sections, which is
  why `sections-current` passes: the cited content is unchanged. It is still
  reported distinctly, with both digests, so a reader sees that the spec moved.
  A reference that pins no sections is a claim about the whole spec, and any
  change to it is `stale`.
- `unverified` refuses. A citation nobody could check is not a citation that
  held, and treating it as a pass would let a missing `--export` turn a gate
  green.
- A malformed `--export` (no `=`, an empty name, a name given twice), an
  unreadable export directory, an exporter `spec-spine.toml` that does not
  parse, or a `--spec` that names no spec here is a usage, I/O or config error
  (exit 3, or exit 1 for an unknown `--spec`).
- The verifier reads the references from the committed registry and refuses a
  stale one (exit 2) before reading any export, as the closure resolver does
  (spec 107 §3.5): checking the committed references while `spec.md` says
  something else would verify a statement nobody is currently making.
- A corpus with no references answers with an empty report at exit 0.

The answer, with `--json`, is a read document (spec 074, a MINOR of the read
axis): `references`, each with `declaredBy`, `corpus`, `spec`, `digest`,
`observedDigest` when the spec was found, `outcome`, and `sections` (each with
`anchor`, `digest`, `observedDigest` when present, and its own `current`,
`stale` or `missing`); and `summary`, a count per outcome. Entries are sorted
by declaring spec, corpus and spec, so the same inputs give the same bytes.
The text form names every non-`current` reference and what moved.

### 3.4 The library and the facade

The verifier is a pure function in core over the references and the exported
spec texts, taking no path and reading no file: the CLI reads the export
directories and passes their contents in. `interface_verify_json` takes
`{ "registry": <registry text>, "exports": { "<corpus>": { "<spec-id>":
{ "path": "<repo-relative path>", "text": "<spec.md>" } } }, "spec"?: <id> }`
and returns the same read document, so a binding can verify without a
filesystem.

### 3.5 What a reference establishes, and what it does not

It establishes that this spec **cites** that document **as it was at that
digest**, and, after `interface verify`, whether the supplied copy still says
it.

It does **not** establish:

- **that the supplied directory is the corpus it is named for.** The caller
  asserts that by naming it. Which checkout is authoritative is the caller's
  decision, and §4 keeps it there.
- **that the exporting corpus is trustworthy.** A `current` answer means the
  bytes match the pin, not that the bytes are right.
- **that the cited corpus is reachable.** Nothing here fetches anything.

### 3.6 A reference is updated deliberately

Changing a `digest` is an edit to a spec, so it goes through the ordinary
route: the spec that owns the reference is edited, under an authority that
covers it. No verb rewrites a reference, and `interface verify` never writes.
"The citation was updated and nobody read the new text" is exactly the
failure a pin exists to prevent, so the observed digest is printed for a human
to copy, never applied.

### 3.7 Nothing else gates on a reference

`compile`, `check`, `lint`, `index` and `couple` reach the same verdicts, with
the same bytes apart from the registry records that carry the references,
whether or not references are declared, including references to corpora that
exist nowhere. Only `interface verify` reads them, and only when asked.

## 4. Out of scope

- **Discovery.** How a caller finds the corpus named in `corpus` is not
  specified here: a registry of corpora is a product decision with an owner,
  and it is not this engine.
- **Fetching.** §3.3 reads local directories only.
- **Trust in the exporter** and **choosing which checkout is authoritative.**
  §3.5.
- **Pinning an obligation** by id rather than its section. Spec 106's
  obligations are anchored to sections, so pinning the section pins what the
  obligation says; an obligation-id pin is a later increment if a second
  corpus in this family declares obligations.
- **Automatic repair** of a stale reference. §3.6.
- **Recording references in the authority snapshot** (spec 070). A reference
  is a statement this corpus makes, carried in the registry record the
  snapshot already hashes.

## 5. Resolved decisions

**D-1 (2026-09-22, built under note 09 D-7, the contract corrected before the
build).** Filed on 2026-09-21 as a deferred contract on an unmerged branch;
carried here and corrected before any code:

- the draft recorded references and checked nothing. It now adds the
  verifier (§3.3) as a pure function of caller-supplied files, because
  "detectable" without a detector is an assertion nobody can run;
- the draft's `sections` were bare anchors. They carry digests now, on spec
  106's section digests, which is what makes `sections-current` possible;
- the draft allowed a malformed digest through as a lint warning. It is a
  compile error: a reference that cannot be checked is not a pin, and a
  placeholder digest is the first step to filling it from an observation;
- open question 1 (obligation pins) is out of scope with a stated reason
  (§4); open question 2 (what `corpus` is a name in) stays the caller's (§4);
  open question 3 (the snapshot) is answered in §4.

**D-2 (2026-09-22, the build).** What the contract was silent on:

- **Codes.** `V-032` corpus name, `V-033` spec id (short or malformed, with
  distinct messages), `V-034` reference digest, `V-035` section digest,
  `V-036` `obtained`, `V-037` duplicate anchor, `V-038` duplicate
  `(corpus, spec)`. A missing member or an unknown one is `V-002` (malformed
  frontmatter), because the reference is `deny_unknown_fields`: a misspelt
  `digest` must not parse as "no digest", which §3.2 forbids.
- **A digest message says which mistake was made**: an unsupported algorithm,
  no algorithm prefix, or a malformed `sha256:` value.
- **`--export <name>=` with an empty directory is a usage error** (exit 3),
  like the empty name §3.3 lists, rather than the working directory by
  default.
- **A referenced `spec.md` that resolves outside its export root** (a symlink
  out) is refused at exit 3. §3.3 says the verifier follows no link out of the
  named directory; refusing is how that is kept, rather than silently
  treating the spec as `missing`.
- **An export text with no frontmatter fence** recomputes no section digests,
  so every pinned anchor is `missing`, rather than one malformed export
  aborting the whole report.
- **`interface verify --spec` is a seventh spec-id argument.** It resolves
  through spec 067's one resolver, and spec 067's census and matrix are
  extended to seven, which is the route that census exists to force.
- **The registry MINOR (`1.6.0`) moves the fixture corpus's registry hash**,
  so spec 103's set is regenerated with its own generator. Only
  `registryHash` and `attestationHash` move; every recorded outcome,
  including `version-mismatch`, is unchanged.
- **The embedded schemas carry compile's grammar for every member**,
  including `spec` (the full-id pattern the record's own `id` uses), so a
  shard that could only come from a hand edit is refused by schema
  validation too, not only by compile. A test mutates each constrained member
  of an emitted shard and registry and asserts refusal (review of #313).
- **Composition is asserted, not assumed.** `interface_composed.rs` runs 102,
  106, 107, 109 and this spec over one exporter corpus: the section digest
  `registry obligation`, `registry show` and a pin carry is one value; an
  edit to an obligation's section moves its closure and stales both pins
  while the declared impact set stays byte-identical; a status flip stales a
  whole-spec pin and leaves a section pin and the obligation's closure alone;
  and a stale exporter ledger cannot hide a change, because the verifier
  recomputes from `spec.md`.

## Verification

Written to fail against the tree this spec is filed on: neither the key, the
verb nor the facade exists.

```verify:cli
cargo build --release --locked
# 3.1, 3.2: every compile rule; 3.3, 3.4: every outcome through the library.
cargo test -p spec-spine-core --test interface --locked
# 3.3: the verb through the shipped binary, every exit code, no writes.
cargo test -p spec-spine-cli --test interface --locked
# 3.2: the registry shards conform to the moved schema.
cargo test --workspace emitted_registry_conforms --locked
# 3.3: a corpus with no references answers empty, at exit 0.
./target/release/spec-spine interface verify --json | grep -q '"schemaVersion"'
sh -c './target/release/spec-spine interface verify --export "=x"; test $? -eq 3'
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
