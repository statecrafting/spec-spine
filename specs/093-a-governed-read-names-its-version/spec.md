---
id: "093-a-governed-read-names-its-version"
title: "A governed read names its version"
status: draft
kind: "tooling"
created: "2026-09-14"
summary: >
  `docs/api.md` states that every JSON this tool emits is pretty-printed with
  sorted keys, and spec 037 gave every verdict verb a versioned envelope. The
  read verbs got neither. Measured at 0.19.0: `registry plan`, `index owner`,
  `index coverage`, `config show` and `index orphans` emit keys in struct
  declaration order, and `registry plan`, `registry show`, `registry list`,
  `index owner`, `index coverage`, `index diagnostics` and `index orphans`
  carry no version member at all, because they serialize with
  `serde_json::to_string_pretty` while every committed artifact goes through
  the canonical writer. A consumer therefore pins a document whose shape it
  cannot dispatch on and whose key order is an implementation detail of a Rust
  struct. This spec routes every read document through one emitter that sorts
  keys and stamps a new `READ_SCHEMA_VERSION`, and decides note 04's D7: the
  version is an additive member on an object document, and the two verbs that
  emit a bare array become objects so they can carry one.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "037-machine-readable-verdicts"
  - "054-effective-config-is-a-governed-read"
  - "055-the-ledger-answers-what-consumers-rebuild"
  - "060-plan-answers-the-whole-question"
extends:
  # 3.2 to 3.4: the one emitter, declared as a core module beside the
  # canonical writer it wraps.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.5: the registry reads (show, plan, list).
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: corrective }
  # 3.5: the index reads (owner, coverage, diagnostics, orphans).
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  # 3.3: the version constant, beside the axes 037 and 088 added.
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.6: `config show` is sorted through the same emitter; its version member
  # is 054's and is not renamed.
  - { spec: "054-effective-config-is-a-governed-read", unit: "crates/spec-spine-cli/src/cmd_config.rs", nature: corrective }
  # 3.7: the CLI-level assertions over the emitted documents.
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
establishes:
  # Planned (spec 076) until the build writes them; the build drops the flag.
  # 3.2: the emitter and its guards.
  - { kind: file, path: "crates/spec-spine-core/src/read.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/read.rs", planned: true }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
---

# 093: A governed read names its version

## 1. Purpose

### 1.1 Two guarantees that stop at the verdict verbs

Spec 037 gave every verb that renders a verdict a `--json` envelope carrying
`schemaVersion`, `verb`, `ok`, `exitCode` and `report`, and `docs/api.md` states
the byte-level guarantee for everything this tool emits: pretty-printed, sorted
keys, LF, one trailing newline. Both statements are true of the verdict verbs
and of every committed artifact. Neither is true of the reads.

Measured on 2026-09-14 at `0.19.0`:

| Verb | Top-level key order | Version member |
|---|---|---|
| `registry plan --json` | `ready,blocked,notSchedulable,planned` | absent |
| `registry show --json` | sorted | absent |
| `registry list --json` | bare array | absent |
| `index owner --json` | `path,owners` | absent |
| `index coverage --json` | `sourceFiles,claimedFiles,floorOnlyFiles,...` | absent |
| `index diagnostics --json` | bare array | absent |
| `index orphans --json` | `orphaned,inFlight` | absent |
| `config show --json` | `config_version,manifest,domains,...` | `config_version` |
| `lint`, `check`, `couple`, `attest`, `verify --plan` | sorted | `schemaVersion` |

The cause is one line repeated: the read paths call
`serde_json::to_string_pretty`, which emits struct fields in declaration order,
while the artifact paths call the canonical writer, which sorts. The key order
of a read is therefore the order someone happened to declare a Rust field in,
and reordering two fields for readability would change every consumer's bytes
without changing any answer.

### 1.2 What an unversioned read costs a consumer

Note 04 §4.7 states the rule a verifier of any spec-spine record must follow:
dispatch on type and version, and treat an unknown MAJOR as `unsupported`,
which is neither pass nor fail. A document with no version cannot be dispatched
on at all. The consumer's options are to pin a tool version (which spec 062's
`required_version` floor exists to make unnecessary) or to sniff for members,
which is the ad-hoc parsing `.claude/rules/governed-artifact-reads.md` was
written against, one layer up.

This is design note 04's F8 and its D7. F8 records the finding; D7 records the
open choice between wrapping the reads in the 037 envelope and adding a version
field to each. §5 D-1 makes that choice.

### 1.3 A read is not a verdict

The reads answer questions; they render no verdict, and `ok` and `exitCode` mean
nothing on them. So the 037 envelope is the wrong shape here even though it is
the versioned one: it would put two members on every read document that no read
can populate honestly, and it would move every existing member one level down
into `report`, which breaks every consumer for the benefit of two fields that
carry no information.

## 2. Territory

This spec adds one core module (`read.rs`) holding the read-document emitter and
its guards, declares it in `lib.rs`, adds `READ_SCHEMA_VERSION` to
`version.rs`, and routes the read paths in `cmd_registry.rs`, `cmd_index.rs` and
`cmd_config.rs` through it. It adds `crates/spec-spine-core/tests/read.rs` for
the emitter's properties and extends `crates/spec-spine-cli/tests/cli.rs` for
the emitted documents themselves.

It changes **no DTO**. Spec 055 §3.3 established the rule this follows: a value
that belongs to the emitted document rather than to the record is added where
the document is written, not to the record type, because a member on the record
type is a member in every committed shard and a schema bump for a value the
shard does not need.

## 3. Behavior

### 3.1 What a read document is

A read document is the JSON a verb emits when its answer is a question's answer
rather than a verdict: `registry show`, `registry plan`, `registry list`,
`index owner`, `index coverage`, `index diagnostics`, `index orphans` and
`config show`. The verdict verbs keep the 037 envelope unchanged; `index
render` emits markdown and is not JSON at all.

### 3.2 One emitter

Every read document MUST be written by one function in
`crates/spec-spine-core/src/read.rs`. It MUST:

1. serialize the value to a JSON value;
2. insert `schemaVersion` as a top-level member when the document is an object,
   or wrap an array document as `{ "items": [...], "schemaVersion": ... }`;
3. write the result through the canonical writer already used for artifacts, so
   keys are sorted, the indent is two spaces, line endings are LF and there is
   exactly one trailing newline;
4. refuse, as an internal error, a document that is neither an object nor an
   array.

The function MUST NOT read a clock, the environment or the filesystem: it is a
pure function of `(verb, value)`, like every other emission path here.

The facade functions that return these documents (`coverage_json`,
`orphans_json`, `query_json`) MUST go through the same function, so the facade
and the CLI cannot emit different shapes. Spec 057 §3.3 established that pairing
and `tests/cli.rs` already pins the halves against each other.

### 3.3 The version axis

`READ_SCHEMA_VERSION` is a new compile-time constant in
`crates/spec-spine-types/src/version.rs`, starting at `0.1.0`, following the
precedent `DELTA_SCHEMA_VERSION` set: a new record type starts at `0.x` and the
axis moves on its own. It versions **the shape of the read document**, not the
artifacts the read is about, so it is independent of
`REGISTRY_SCHEMA_VERSION` and `INDEX_SCHEMA_VERSION` and does not move when
either of those does.

One constant covers every read document. Per-verb axes were rejected: eight
constants that always move together are eight chances to forget one, and a
consumer dispatching on `(verb, schemaVersion)` gets the same information from
one.

MINOR is additive (a new member), MAJOR is breaking (a member removed, renamed,
or changed in meaning). The rule is the one `docs/schema-versioning.md` already
states for the other axes, and that document MUST gain this axis in the same
change, since note 04's F10 recorded it as already understating the axes that
exist.

### 3.4 Sorted keys, stated once and true

After this spec, `docs/api.md`'s sentence is true of every document this tool
emits. The emitter is the only place that decides key order for a read, so a
future field added to `Plan`, `CoverageReport` or `OwnerReport` lands in sorted
position without anyone remembering to put it there.

### 3.5 The one breaking change, named

`registry list --json` and `index diagnostics --json` emit bare arrays today. An
array cannot carry a member, so they become objects: `{ "items": [...],
"schemaVersion": "0.1.0" }`. This is a **breaking** output change for those two
verbs and for nothing else.

It MUST be announced in the release notes of the release that carries it, in the
terms note 04 §7 R1 uses for a consumer reading prose: a consumer of
`registry list --json` reads `.items`, and a consumer of
`registry list --ids-only` (the text form, which is what the skills and the
`/spec` flow use) is unaffected.

The alternative, leaving those two unversioned, was rejected: the spec's claim
is that a governed read names its version, and two verbs exempted from it would
leave a consumer writing the sniffing code this spec exists to retire.

### 3.6 `config show` keeps its own version member

`config show --json` already names a version, as `config_version`, which tracks
`CONFIG_VERSION` and is the version of the configuration record rather than of
the read document. It MUST be routed through the emitter for key order, and its
existing member MUST NOT be renamed: the casing is inconsistent with every other
emitted member, and a rename is a breaking change to spec 054's contract that
buys a consumer nothing. `schemaVersion` MUST NOT be added beside it, because a
document with two version members cannot be dispatched on without a rule about
which one wins.

### 3.7 The tests

`crates/spec-spine-core/tests/read.rs` MUST assert the emitter's properties:
sorted keys for a value whose struct order is not sorted, the array wrapping,
the trailing newline, and the refusal for a scalar document.

`crates/spec-spine-cli/tests/cli.rs` MUST assert, for each verb in §3.1, that
the emitted document parses, carries its version member, and has top-level keys
in sorted order. Asserting it per verb rather than once on the emitter is
deliberate: the defect this spec fixes was never in a shared function, it was
eight call sites that did not use one.

## 4. Out of scope

**The 037 envelope for reads.** §1.3 records why: `ok` and `exitCode` cannot be
populated honestly by a read, and moving every existing member under `report`
breaks every consumer for no information.

**Renaming `config_version`** (§3.6).

**A JSON Schema for read documents.** The three schema files this repository
embeds cover the committed artifacts, which the conformance test validates
because a drifting artifact schema is a build failure. A read document is not
committed and not validated against anything; adding a fourth schema file would
add a conformance surface without a consumer. A later spec may add one when a
consumer needs it.

**Versioning what the reads are about.** `REGISTRY_SCHEMA_VERSION` and
`INDEX_SCHEMA_VERSION` still answer "what shape are the shards"; this axis
answers "what shape is this answer". A read document does not restate the
artifact versions.

## 5. Resolved decisions

**D-1 (2026-09-14). Note 04's D7 is decided as an additive member, not an
envelope.** A read document gains `schemaVersion` in place (§3.2) rather than
being wrapped in the spec 037 verdict envelope. The envelope was rejected for
the reason in §1.3. The cost is §3.5: two verbs that emit arrays must become
objects, which is breaking for them, and that is accepted rather than exempting
them.

**D-2 (2026-09-14). One axis, not eight.** `READ_SCHEMA_VERSION` covers every
read document (§3.3).

**D-3 (2026-09-14). No DTO carries the version.** The member is added where the
document is written, following spec 055 §3.3, so no committed shard and no
schema file changes and `compile --check` stays fresh across this spec.

## Verification

Each line is one command. Every line asserting a version member or an `items`
wrapper fails against pre-093 code, because no read document carries either;
those are the fail-first evidence. The sorted-key assertions on `registry plan`,
`index owner` and `index coverage` also fail against pre-093 code (measured:
`ready,blocked,notSchedulable,planned`, `path,owners` and
`sourceFiles,claimedFiles,...`). The `cargo test` lines are **not** fail-first:
the assertions they carry do not exist at the parent commit, so the suites pass
vacuously.

```verify:cli
# 3.3: the axis exists and starts where the note says.
grep -qF 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
# 3.2, 3.4: every object read is sorted and versioned.
target/release/spec-spine registry plan --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry show 093 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index owner Cargo.toml --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index orphans --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
# 3.5: the two array reads carry their items under a versioned object.
target/release/spec-spine registry list --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
target/release/spec-spine index diagnostics --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
# 3.6: `config show` is sorted and keeps 054's version member, with no second one.
target/release/spec-spine config show --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert "config_version" in d; assert "schemaVersion" not in d'
# 3.7: the emitter's own properties, and the per-verb assertions.
cargo test -p spec-spine-core --test read --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.3: the axis is documented where the others are.
grep -qF 'READ_SCHEMA_VERSION' docs/schema-versioning.md
```
