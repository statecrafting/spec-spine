---
id: "166-deterministic-reference-documentation"
title: "Generate one deterministic reference page per spec"
status: draft
kind: "governance"
created: "2026-09-27"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "155-selected-content-accessor"
  - "160-documentation-manifest-and-freshness"
  - "169-declared-obligation-traceability"
  - "162-capability-catalog"
summary: >
  Adds one bounded, fixed-export reference product: one Markdown page per
  exported spec, projected from a typed page model whose every field comes
  from a named ledger source (registry, index, selected content, declared
  traceability, capability catalog). Pages have fixed generated regions and
  one human-authored region, canonical bytes, declared-not-run executable
  examples, and a read-only regeneration check that exits 1 when committed
  pages differ. Narrative documentation stays outside the product.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/reference.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/reference-page.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/reference.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/reference.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-core/tests/fixtures/reference/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_reference.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/reference.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/reference_examples.rs", planned: true }
  - { kind: file, path: "docs/reference-documentation.md", planned: true }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap unit DG-01 and the documentation product boundary (13.5)" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/render.rs" }, role: "existing index projection, distinguished in D-2" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/verify.rs" }, role: "verify:cli block parser reused read-only" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "An export produces exactly one reference page per exported spec at a fixed path, and no other page kind."
    anchor: "3-1-one-export-one-page-kind"
  - id: "R-2"
    kind: requirement
    text: "Every page field is drawn from one named typed source, and an absent, unsupported, or unsupplied source is rendered explicitly, never inferred."
    anchor: "3-4-fields-and-their-sources"
  - id: "R-3"
    kind: requirement
    text: "Identical inputs produce byte-identical page models and Markdown pages on every supported platform."
    anchor: "3-5-markdown-projection-and-canonical-bytes"
  - id: "R-4"
    kind: requirement
    text: "The regeneration check never writes and exits 1 naming every missing, changed, orphaned, or malformed page."
    anchor: "3-10-the-regeneration-check"
  - id: "I-1"
    kind: invariant
    text: "Generation and checking read only declared repository inputs and never read environment, secrets, providers, network, Git history, or undeclared paths."
    anchor: "3-8-inputs-are-explicit-and-ambient-reads-are-forbidden"
  - id: "I-2"
    kind: invariant
    text: "Generation and checking never execute an example; examples run only in a separate acceptance step against the built binary."
    anchor: "3-7-executable-examples-are-declared-not-run"
  - id: "V-1"
    kind: verification
    text: "The implementation is checked for field sourcing, canonical bytes, region preservation, regeneration findings, containment, limits, and examples run against the built binary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/reference.rs"
      - "crates/spec-spine-cli/tests/reference.rs"
      - "crates/spec-spine-cli/tests/reference_examples.rs"
intent:
  goal: "publish a fixed, byte-reproducible reference of what each spec declares, owns, and relates to"
  non_goals:
    - "tutorials, explanations, navigation, migrations, or any other narrative documentation"
    - "claims of correctness, test success, acceptance, release, adoption, or observation"
    - "executing examples, generators, or tests during generation or checking"
    - "reading Git history, environment, secrets, providers, or the network"
---

# 166: Generate one deterministic reference page per spec

## 1. Purpose

The ledger already knows, per spec, its lifecycle, obligations, owned units,
typed edges, and declared acceptance. Specs 155, 169, and 162 add source
signatures, declared obligation relations, and an operation catalog. A reader
who wants that information today reads frontmatter, queries several verbs, and
reconciles the answers by hand; `docs/api.md` and `docs/cli-reference.md` are
authored and drift silently when the code moves.

This spec defines one bounded product: a Markdown page per exported spec,
generated regions a pure function of declared inputs, one authored region
kept byte for byte, and a read-only check against a fresh in-memory
generation. It states only what is declared and structurally resolved.

`index render` (spec 010) is a different product and is untouched (D-2).

## 2. Territory

- `crates/spec-spine-types/src/reference.rs` defines the export request, the
  page model, field records, region records, and check findings.
- `crates/spec-spine-types/schemas/reference-page.schema.json` defines the
  export request and page model on their own `reference-page` axis.
- `crates/spec-spine-core/src/reference.rs` builds page models from supplied
  inputs, projects them to Markdown, merges authored regions, and computes
  check findings. It returns files as data and writes nothing.
- `crates/spec-spine-core/src/lib.rs` exposes typed functions plus
  `reference_pages_json` and `reference_check_json` facade entries.
- `crates/spec-spine-cli/src/cmd_reference.rs` and `main.rs` add
  `reference generate`, `reference check`, and `reference model --json`.
- The core and CLI tests, the fixture directory, and
  `crates/spec-spine-cli/tests/reference_examples.rs` own the acceptance.
- `docs/reference-documentation.md`, the API reference, CLI reference, and
  schema history describe the product and its limits.

Registry, index, selected content, traceability, catalog, and manifest
contracts remain owned by specs 001, 004, 155, 169, 162, and 160. This spec
consumes their public documents and changes none of them.

## 3. Behavior

### 3.1 One export, one page kind

An export is one request naming a set of specs and one output directory. It
produces exactly one page per exported spec at `<outputDir>/<spec-id>.md`,
where `<spec-id>` is the full registry id. V1 has one page kind, the spec
reference page. It has no index page, table of contents, cross-page
navigation, landing page, or per-operation page (D-1).

Superseded and retired specs export like any other; an unknown id refuses.

### 3.2 Export request

The request is a closed JSON document, normally committed beside the pages:

```json
{
  "schemaVersion": "1.0.0",
  "outputDir": "docs/reference",
  "specs": { "select": "all" },
  "traceability": true,
  "catalog": "docs/reference/catalog.json",
  "limits": { "maxPages": 1024, "maxPageBytes": 524288, "maxSignatures": 256 }
}
```

`specs` is either `{ "select": "all" }` (every registry spec) or
`{ "ids": [...] }`, an explicit list of full ids; short ids are refused.
`outputDir` is a normalized repository-relative path under the repository
path contract (spec 144). It MUST NOT resolve inside the configured
`derived_dir` or `state_dir`, escape the repository, or cross a symbolic link.
`traceability` enables the spec-169 field. `catalog` is null or the
repository-relative path of a committed spec-162 catalog document. `limits`
defaults as shown and may only lower them. Unknown members are usage errors.

### 3.3 Page model and output schema

Generation builds a typed `ReferencePage` per spec, then projects it to
Markdown, which carries no fact the model lacks. `reference model --json`
emits the ordered models as one canonical document for bindings and spec 167.

Each model carries `schemaVersion`, `specId`, `path`, the ordered field
records of §3.4, the authored region bytes, and `inputs`, the ordered list of
input identities and digests that §3.4 names. It carries no timestamp,
hostname, absolute path, Git revision, tree, generator version, or build
digest (D-3).

### 3.4 Fields and their sources

Every page has these regions, always present, in this order. An empty field
renders the fixed line `None declared.`, never an omitted heading.

| Region id | Field | Source |
|---|---|---|
| `header` | id, title, spec path, summary, `reference-page` schema version | registry record |
| `lifecycle` | `status`, `implementation`, `superseded_by`, `retirement_rationale`, `created` | registry record |
| `requirements` | every obligation: qualified id, kind, text, anchor, withdrawn flag | registry obligations (spec 106) |
| `ownership` | every owning unit by edge (`establishes`, `extends` with target spec, `refines`, `co_authority`, `constrains`), its `planned` flag, and its index resolution state; `references` listed separately as non-owning | registry edges, committed index shards |
| `signatures` | for each owned `symbol` unit, the spec-155 `signature` projection text, path, span, and digest, or its omission reason | spec 155 core resolver |
| `relations` | every spec-169 relation whose source obligation belongs to this spec, grouped by relation kind (`tested-by` first), with target identity and resolution state | spec 169 traceability read |
| `operations` | every spec-162 catalog operation whose `governedBy` names this spec, with its effects, and its declared examples (§3.7) | spec 162 catalog document |
| `evidence` | each verification obligation's declared `inputs`, and the commands of the spec's `verify:cli` block labeled "declared, not executed" | registry obligations; spec 043 block parser over `spec.md` bytes |
| `limitations` | `intent.non_goals`, then the fixed product limitation statement of §3.14 | registry intent (spec 114); fixed text |
| `changes` | outgoing `amends`, `amends_sections`, `amends_verification`, `supersedes`, `amendment_record`; incoming amends, supersedes, and extends computed from other registry records | registry records |
| `inputs` | the input identities and digests this page was generated from | computed |
| `notes` | human-authored text | the committed page (§3.6) |

Absence rules are exhaustive:

- `traceability: false` renders the `relations` field as `Not supplied.`
  A supplied read with no relation for this spec renders `None declared.`
  Neither is rendered as "untested", "undocumented", or "unimplemented".
- `catalog: null` renders `operations` as `Not supplied.` A catalog that
  names no operation for this spec renders `None declared.` Operations are
  never matched to a spec by path, name, or ownership; only the catalog's own
  declared governing-spec identity binds them.
- A symbol whose signature 155 cannot project renders a row with the 155
  omission reason (`unsupported-projection`, `unresolved`, and so on). No
  fallback to `full` or `declaration` content is permitted.
- A spec with no `verify:cli` block renders `No declared acceptance block.`
- Git history, pull requests, CI results, attestations, and release notes
  are not sources. `changes` is the typed-edge record only.

The `inputs` field is the page's freshness basis: the spec's registry
`shardHash`, the `shardHash` of every index shard for its owned units, each
selected-content item digest used, the canonical digest of the traceability
read restricted to this spec, and the catalog document digest. It states the
basis only; freshness is decided by §3.10 and by spec 160, never asserted by
the page itself.

### 3.5 Markdown projection and canonical bytes

Pages are UTF-8 without BOM, LF line endings, no trailing whitespace on any
line, and exactly one trailing newline. The projection is a fixed template;
no configuration alters heading text, field order, or table layout.

Generated text is escaped deterministically. Every free-text value (titles,
summaries, obligation text, non-goals, rationale) normalizes CRLF and CR to
LF, replaces each internal newline with one space, trims, and escapes `\`,
`|`, `<`, `>`, and `&` so no generated value can form a table cell break, an
HTML comment, or a region delimiter. Identifiers render as code spans whose
backtick fence is one longer than the longest backtick run inside them.
Signatures and commands render as fenced blocks whose fence is at least three
backticks and one longer than the longest backtick run in the body; the info
string is `rust`, `typescript`, or `console` from the source kind, never
guessed.

Collections order bytewise by canonical identity: obligations by id, units by
canonical unit JSON within each edge kind, relations by spec 169's ordering,
operations by catalog operation `name`, incoming edges by source spec id. Authoring
order in frontmatter and map iteration order never change bytes.

Given equal request, config, registry, index, source bytes, traceability read,
catalog, and prior authored regions, the page models and Markdown bytes are
identical on every release triple.

### 3.6 Generated and authored regions

Each region is delimited by two full lines:

```text
<!-- spec-spine:begin generated id="lifecycle" -->
<!-- spec-spine:end generated id="lifecycle" -->
```

The authored region uses `authored` in place of `generated`. A region's span
runs from its begin line through its end line plus one following blank line,
except the last region, which has none. Every byte of a page therefore
belongs to exactly one region, which is the coverage rule spec 160 enforces
for its manifest outputs.

V1 has one authored region per page, `notes`, fixed as the last region
(D-4). On generation, the generator reads the committed page at the target
path, if any, and copies the bytes strictly between the `notes` delimiters
verbatim after LF normalization. A new page gets an empty `notes` region.
Generated regions are always rewritten from the model; hand edits inside them
do not survive.

A committed page is malformed, and generation refuses without writing any
page, when its authored region contains a line beginning with
`<!-- spec-spine:`, is not valid UTF-8, exceeds 65536 bytes, or appears more
or fewer than once. An existing file under `outputDir` that matches the page
name pattern but whose spec is no longer exported is an orphaned page; if its
`notes` region is non-empty, generation refuses rather than delete human text,
and if it is empty, generation removes it. The human decides by moving the
text or deleting the file.

### 3.7 Executable examples are declared, not run

Examples come only from the spec-162 catalog. Each rendered example carries
its catalog example `id`, `fixture`, `argv`, `stdin`, `exitCode`, and
`stdoutIncludes` substrings exactly as spec 162 §3.9 declares them, inside a generated `operations`
region. Authored regions may contain code blocks; they are prose, never
examples, and are never run.

Neither `reference generate`, `reference check`, nor `reference model` runs
an example, spawns a process for one, or records success. Running examples is
the separate acceptance step in `crates/spec-spine-cli/tests/reference_examples.rs`:
it runs every example of the fixture catalog against the built binary
(`CARGO_BIN_EXE_spec-spine`) inside a copied fixture corpus and compares exit
class and output with the declared expectation. `verify` remains the one
executing verb of the CLI (D-5).

### 3.8 Inputs are explicit and ambient reads are forbidden

Core generation is a pure function of `(Config, request, registry, index,
selected file bytes, traceability read, catalog, prior page bytes)`. It has no
clock, environment, `git`, network, or filesystem access.

The CLI reads only: the configuration; the committed registry and index
shards; the `spec.md` of each exported spec; the source files of owned symbol
units needed for signatures; the request; the catalog path it names; and
existing files directly under `outputDir`. Every path passes the existing
containment checks. The CLI reads no environment variable for content, no
credential store, no provider, no remote, no ignored file, and no Git object
or history. Generation reads the working-tree bytes as `compile` and `index`
do; it does not require a clean snapshot (D-6).

### 3.9 Generation

`spec-spine reference generate --request <path>` requires the ledger fresh
by `check`'s comparison; a stale ledger is a finding and nothing is written.
It builds every model and checks every limit before writing any page, and
never touches a file outside `outputDir`.

### 3.10 The regeneration check

`spec-spine reference check --request <path> [--json]` performs the same
generation in memory and compares it with the committed files. It never
writes. Findings, ordered by page path then kind, are:

- `page-missing`: an exported spec has no page;
- `page-changed`: the page bytes differ, naming each differing region id;
- `page-orphaned`: a page-pattern file exists for a spec not exported;
- `page-malformed`: region delimiters, coverage, or the authored region
  violate §3.6; and
- `ledger-stale`: the registry or index is not fresh.

Any finding exits 1. No finding exits 0. The authored `notes` region never
produces `page-changed`, because generation copies it. Findings name paths,
region ids, and digests, never page content. The check is not added to this
repository's gate chain by this spec (D-8).

### 3.11 Manifest binding

`reference generate --manifest <path>` also emits one spec-160 manifest:
outputs are the pages, regions the §3.6 regions, inputs the §3.4 identities,
examples the rendered catalog examples. It follows spec 160's clean-snapshot
and spec-159 packet rules; if they fail, nothing is written. Page bytes are
identical with or without `--manifest`.

### 3.12 Limits, errors, and exit codes

Limits are checked before unbounded allocation: at most `maxPages` pages, at
most `maxPageBytes` canonical bytes per page, and at most `maxSignatures`
signature rows per page. An excess refuses the whole export with
`limit-exceeded`, naming the limit and the spec; nothing is truncated (D-7).

Exit codes follow spec 132: 0 none; 1 a finding, stale ledger, or unknown
spec; 2 containment, orphaned notes, or a malformed page during generation;
3 a malformed request; 4 I/O or a tool-produced schema failure. JSON mode
writes the verdict envelope on every non-success, findings as its report.

### 3.13 Schema evolution

The `reference-page` axis starts at `1.0.0` in `docs/schema-versioning.md`.
The Markdown template is part of the axis: a new optional model field moves
MINOR; any change to region ids or order, heading text, escaping, delimiters,
or ordering rules moves MAJOR, because it changes committed page bytes.

### 3.14 Authority and disclosure boundary

Each page ends its `limitations` field with this fixed statement: "This page
restates declared and structurally resolved records. It does not state that
code is correct, that a test ran or passed, or that anything was accepted,
released, adopted, or observed." The product never uses the words `tested`,
`released`, `adopted`, or `observed` as a status of the spec.

All source text is untrusted data: generation never follows a link, obeys
embedded instructions, or widens its reads. Consumers own publication.

## 4. Acceptance criteria

1. A fixture corpus with every lifecycle state yields one page per exported
   spec and no other file; each of the twelve regions appears in order with
   its §3.4 source, and empty, `Not supplied.`, and omission rows are exact.
2. Reordering frontmatter, obligations, edges, and catalog entries leaves page
   bytes unchanged; LF, CRLF, and CR fixtures yield identical pages; the
   fixture page tree is byte-identical across the determinism workflow's
   release triples.
3. A spec summary containing `<!-- spec-spine:end generated id="header" -->`,
   `|`, and backtick runs renders escaped and does not break region parsing.
4. Authored `notes` bytes survive regeneration verbatim; a malformed or
   duplicated authored region, and an orphaned page with notes, refuse without
   writing.
5. `reference check` exits 0 on a fresh tree, and exits 1 with the exact
   finding for a missing, edited, orphaned, or malformed page and for a stale
   ledger; it writes nothing in any case.
6. Ownership alone never produces a relation or an operation row; a catalog
   operation binds only through its declared `governedBy`.
7. A test harness that denies environment reads and records opened paths
   shows generation reads only §3.8's paths and no Git object.
8. Every fixture catalog example runs against the built binary with its
   declared `exitCode` and `stdoutIncludes`; generation and check spawn no process.
9. Limit excess refuses atomically; existing registry, index, render,
    selected-content, traceability, and catalog outputs are byte-compatible.

## 5. Out of scope

- Tutorials, explanations, navigation, indexes, migrations, release notes,
  and all other narrative documentation.
- Per-operation pages or a generated `docs/cli-reference.md` (D-9).
- Changing `index render`, `index orphans`, or any existing projection.
- Git history, CI or attestation results as sources; evidence admission or
  the grades of design note 09 §13.5.
- Multiple authored regions, custom templates, HTML output, remote catalogs.

## 6. Resolved decisions

**D-1 (2026-09-27): one page kind, per spec.** The spec is the unit every
source already keys on, so each field has one unambiguous source. Navigation
and index pages are narrative and belong to authored documentation.

**D-2 (2026-09-27): distinct from `render.rs`.** `index render` is a stdout
view of one index artifact with no page files, regions, or check. Reusing it
would change a shipped projection's bytes for no gain. The new module shares
only its purity convention.

**D-3 (2026-09-27): pages carry content digests, not snapshot identity.** A
revision, tree, timestamp, or generator version in page bytes would stale
every page on every commit or release. Snapshot and generator identity live
in the spec-160 manifest.

**D-4 (2026-09-27): one authored region, fixed last.** It settles placement.

**D-5 (2026-09-27): examples run in acceptance, not in a verb.** Generation
and check stay safe on a stranger's branch; the built-binary test executes.

**D-6 (2026-09-27): generation reads the working tree.** A clean-snapshot
rule would force a commit before pages could regenerate with the change. The
manifest step keeps spec 160's clean-snapshot rule.

**D-7 (2026-09-27): limits refuse.** A truncated page would look complete.

**D-8 (2026-09-27): not in this repository's gate.** That is the owner's.

**D-9 (2026-09-27): owner decision, open.** Whether this repository commits
an export, and whether `docs/cli-reference.md` becomes generated output (which
would need a per-operation page kind derived from the spec-162 catalog), is
reserved to the owner. Default until ruled: both documents stay authored and
this spec's product is exercised only on fixtures.

**D-10 (2026-10-10, refile): traceability is spec 169, and the catalog's
names are spec 162's as filed.** The draft named spec 161, refiled as 169
before it reached `main`; every reference now names 169, whose §3.4 ordering
the `relations` field uses. Spec 162 as filed names an operation by `name`,
lists its governing specs in `governedBy`, and shapes examples as
`{ id, fixture, argv, stdin, exitCode, stdoutIncludes }` (162 §3.3, §3.9);
§3.4, §3.5 and §3.7 now use those members instead of the draft's "operation
id", "governing spec" and "exit class". Acceptance criteria were renumbered
from 1 to 9, closing a gap where criterion 2 was missing; no criterion was
added or removed.

**D-11 (2026-10-10, refile): the catalog is read from a committed file, never
from the running binary.** Spec 162 compiles its catalog into the binary and
serves it as the `catalog` member of `capabilities --json`. A page built from
the generator's own catalog would make page bytes depend on the generator
version, which D-3 forbids. `catalog` therefore names a committed copy of that
member, and its document digest enters `inputs` like every other source. A
caller that wants the current binary's catalog writes it to that path first,
as a separate step.

**D-12 (2026-10-10, refile): the three verbs are cataloged.** Spec 162 holds
every CLI form and facade function to one catalog record (162 R-1). Whichever
of 166 and 162 is built second adds `reference.generate`, `reference.check`
and `reference.model` and the two facades. Effects: all three read `config`,
`corpus`, `derived-ledger`, `source-tree` and `caller-file`; `generate` writes
`caller-path` (the output directory), and with `--manifest` also executes `git`
for spec 160's clean-snapshot rule; `check` and `model` write nothing. None
executes `declared-commands` or opens a connection.

## Verification

Each named test target fails before the build, because it does not exist while
`implementation: pending`. `cargo test --workspace` is the regression floor and
passes either way.

```verify:cli
cargo test -p spec-spine-core --test reference --locked
cargo test -p spec-spine-cli --test reference --locked
cargo test -p spec-spine-cli --test reference_examples --locked
cargo test --workspace --locked
```
