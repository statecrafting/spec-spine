---
id: "093-a-governed-read-names-its-version"
title: "A governed read names its version"
status: draft
kind: "tooling"
created: "2026-09-14"
summary: >
  `docs/api.md` states that every JSON this tool emits is pretty-printed with
  sorted keys, and spec 037 gave every verdict verb a versioned envelope. The
  read verbs got neither. Measured at 0.19.0: thirteen read documents across
  ten verbs, of which twelve carry no version member at all and seven emit
  keys in struct declaration order, because they serialize with
  `serde_json::to_string_pretty` while every committed artifact goes through
  the canonical writer. A consumer therefore pins a document whose shape it
  cannot dispatch on and whose key order is an implementation detail of a Rust
  struct. This spec routes every read document through one emitter that sorts
  keys and stamps a new `READ_SCHEMA_VERSION`, and decides note 04's D7: the
  version is an additive member on an object document, and the documents that
  are not objects today become objects so they can carry one. Four documents
  across three verbs change shape breakingly, and two of the four are
  contracts approved specs state, so this spec carries `amends` edges to 010
  and 060 rather than editing them.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "010-registry-query-projection-flags"
  - "037-machine-readable-verdicts"
  - "054-effective-config-is-a-governed-read"
  - "055-the-ledger-answers-what-consumers-rebuild"
  - "060-plan-answers-the-whole-question"
amends:
  # 010 3.1: `registry list --ids-only --json` is "a JSON array of id strings".
  # 3.5 below wraps it so it can carry a version. Replacement text in 3.5.
  - "010-registry-query-projection-flags"
  # 060 3.2: `plan --next --json` is "the single spec object rather than an
  # array". 3.5 below moves the pick under a nullable member, so that an empty
  # ready set is a value rather than a missing key. Replacement text in 3.5.
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

Thirteen documents across ten verbs. A projection flag produces its own
document and is listed separately, because a consumer pins the document it
actually reads and `--ids-only` is not `--json`'s shape with fewer members:

| Document | Top-level key order | Version member |
|---|---|---|
| `registry list --json` | bare array of records | absent |
| `registry list --ids-only --json` | bare array of strings | absent |
| `registry show <id> --json` | sorted | absent |
| `registry status-report --json` | `total,draft,approved,superseded,retired` | absent |
| `registry status-report --nonzero-only --json` | same, zero counts omitted | absent |
| `registry relationships <id> --json` | `id,dependsOn,supersedes,amends,supersededBy,amendedBy,dependedOnBy` | absent |
| `registry plan --json` | `ready,blocked,notSchedulable,planned` | absent |
| `registry plan --next --json` | `id,title`, or the bare literal `null` | absent |
| `index owner <path> --json` | `path,owners` | absent |
| `index coverage --json` | `sourceFiles,claimedFiles,floorOnlyFiles,...` | absent |
| `index diagnostics --json` | bare array | absent |
| `index orphans --json` | `orphaned,inFlight` | absent |
| `config show --json` | `config_version,manifest,domains,...` | `config_version` |
| `lint`, `check`, `couple`, `attest`, `verify --plan` | sorted | `schemaVersion` |

`registry status-report` and `registry relationships` were absent from the first
draft of this spec's inventory, and the three projection documents were folded
into their parent verbs. Both omissions mattered: `--ids-only` is the one whose
shape an approved spec states (§3.6), and `--next`'s `null` is the one the
emitter as first drafted would have refused (§3.3).

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
rather than a verdict. The set is closed, and it is the thirteen documents §1.1
enumerates, produced by these ten verbs:

`registry list` (plain and `--ids-only`), `registry show`,
`registry status-report` (plain and `--nonzero-only`), `registry relationships`,
`registry plan` (plain and `--next`), `index owner`, `index coverage`,
`index diagnostics`, `index orphans`, and `config show`.

A projection flag produces a distinct document under this spec, not a variant of
one. Each MUST be emitted through §3.2's function and MUST carry its version,
because a consumer that pins `--ids-only` never parses the unprojected form and
gains nothing from the unprojected form being versioned.

The verdict verbs keep the 037 envelope unchanged; `index render` emits markdown
and is not JSON at all. A verb added after this spec that answers a question
rather than rendering a verdict joins this set, and §3.8's per-document assertion is
what makes the omission visible.

### 3.2 One emitter

Every read document MUST be written by one function in
`crates/spec-spine-core/src/read.rs`. It takes the value and a **versioning
mode**, and it MUST:

1. serialize the value to a JSON value;
2. bring the document to object form under §3.3;
3. apply the versioning mode:
   - **`Stamp`**: insert `schemaVersion` as a top-level member. Every document
     in §3.1 except `config show` uses this mode.
   - **`Preexisting`**: insert nothing, because the document already names a
     version its own spec declares. `config show` is the only caller, for the
     reason in §3.7. The emitter MUST refuse, as an internal error, a
     `Preexisting` document that carries no version member, so the exemption
     cannot be claimed by a document that has nothing to exempt.
4. write the result through the canonical writer already used for artifacts, so
   keys are sorted, the indent is two spaces, line endings are LF and there is
   exactly one trailing newline.

The mode is an argument, never a decision the emitter makes by inspecting member
names. A name-sniffing emitter would silently change behavior the first time a
read document happened to contain a member called `version`, which is the class
of accident this spec exists to close.

The value the emitter receives is the document's own shape. Where a document
names its answer with a member of its own (§3.3's `next`), the **caller**
constructs that object, for the populated answer and the empty one alike,
before calling the emitter. A `(value, mode)` signature cannot know that a
populated `{ "id", "title" }` needs wrapping while an object that is already
the document does not, so an emitter left to guess would stamp `schemaVersion`
beside `id` and `title` on the populated case and emit a different shape from
the empty one.

The function MUST NOT read a clock, the environment or the filesystem: it is a
pure function of `(value, mode)`, like every other emission path here.

### 3.3 Every read document is an object

A version member needs somewhere to sit, so no read document may reach the
canonical writer as anything but an object. Of today's three non-object shapes,
two are the emitter's to convert and one is the caller's to prevent:

| Shape today | Object form | Whose work | Members |
|---|---|---|---|
| a bare array | wrapped | the emitter | `items` carries the array, in the same order |
| the literal `null` (`plan --next`, empty ready set) | wrapped before the call | the caller | `next` is present and `null` |
| a scalar | refused | the emitter | internal error; no read emits one, and none should start |

An array can be wrapped generically, because `items` says nothing about the
document beyond "these are the entries". A named answer cannot: only the caller
knows the member is called `next`, and it MUST build `{ "next": ... }` for the
**populated** answer as well as the empty one, since the emitter sees a
populated `{ "id", "title" }` as an ordinary object and would otherwise stamp
it in place, leaving the two answers different shapes. The emitter MUST
therefore refuse a top-level `null` as an internal error, on the same footing
as a scalar: an absent answer is a shape the caller names, not one the emitter
invents a member name for.

No read document may represent an absent answer by omitting a member. A consumer
that must test whether `id` is present to learn whether anything is ready is
sniffing for members, which is precisely what §1.2 records as the cost this spec
removes; a member that is present and `null` is a value it can dispatch on.

The wrapping member name is part of each document's contract and is fixed here:
`items` for the three arrays, `next` for `plan --next`. §3.6 names what that
breaks.

The facade functions that return these documents (`coverage_json`,
`orphans_json`, `query_json`) MUST go through the same function, so the facade
and the CLI cannot emit different shapes. Spec 057 §3.3 established that pairing
and `tests/cli.rs` already pins the halves against each other.

### 3.4 The version axis

`READ_SCHEMA_VERSION` is a new compile-time constant in
`crates/spec-spine-types/src/version.rs`, starting at `0.1.0`, following the
precedent `DELTA_SCHEMA_VERSION` set: a new record type starts at `0.x` and the
axis moves on its own. It versions **the shape of the read document**, not the
artifacts the read is about, so it is independent of
`REGISTRY_SCHEMA_VERSION` and `INDEX_SCHEMA_VERSION` and does not move when
either of those does.

One constant covers every read document. Per-verb axes were rejected: ten
constants that always move together are ten chances to forget one, and a
consumer dispatching on `(verb, schemaVersion)` gets the same information from
one.

MINOR is additive (a new member), MAJOR is breaking (a member removed, renamed,
or changed in meaning). The rule is the one `docs/schema-versioning.md` already
states for the other axes, and that document MUST gain this axis in the same
change, since note 04's F10 recorded it as already understating the axes that
exist.

### 3.5 Sorted keys, stated once and true

After this spec, `docs/api.md`'s sentence is true of every document this tool
emits. The emitter is the only place that decides key order for a read, so a
future field added to `Plan`, `CoverageReport` or `OwnerReport` lands in sorted
position without anyone remembering to put it there.

### 3.6 The breaking changes, named

Four documents, across three verbs, change shape. All four are breaking for a
consumer that parses them today:

| Document | Today | After | Breaking |
|---|---|---|---|
| `registry list --json` | `[ {...}, ... ]` | `{ "items": [ {...}, ... ], "schemaVersion": ... }` | yes |
| `registry list --ids-only --json` | `[ "000-...", ... ]` | `{ "items": [ "000-...", ... ], "schemaVersion": ... }` | yes |
| `index diagnostics --json` | `[ {...}, ... ]` | `{ "items": [ {...}, ... ], "schemaVersion": ... }` | yes |
| `registry plan --next --json` | `{ "id": ..., "title": ... }`, or `null` | `{ "next": { "id": ..., "title": ... } \| null, "schemaVersion": ... }` | yes |

Every other document in §3.1 gains one member and keeps every member it had, in
sorted position. That is additive for a consumer reading by key, and not
nothing for the rest: a strict decoder that rejects unknown members fails on
the new one (`deny_unknown_fields` is the setting this repository's own config
loader uses, so the shape is not hypothetical), and a consumer that hashes,
diffs or golden-files the emitted bytes sees a change both from the new member
and from every existing member whose sorted position moves. Those consumers are
affected by any MINOR on this axis, which is what §3.4's version member exists
to let them detect.

Two of the four are shapes an **approved** spec states, so under spec 040 this
spec carries an `amends` edge to each and the replacement text lives here. The
amended documents are not edited.

Replacing spec 010 §3.1's second bullet:

> With `--json`: a JSON object carrying `items`, an array of id strings (not
> record objects), in the same order as the text form, alongside the read
> document's `schemaVersion`. The projection is still a projection: `items`
> holds ids and nothing else.

Replacing spec 060 §3.2's second paragraph:

> With `--json`, a read document carrying `next`: the single spec object when
> the ready set is non-empty, and `null` when it is empty. It is a named member
> rather than a one-element array, so a consumer still does not index into a
> list to reach the thing it asked for, and it is present and `null` rather
> than absent, so "nothing is ready" is a value the consumer reads rather than
> a missing key it infers.

`plan --next`'s empty case is the one this spec could not have left alone. It
emits the bare literal `null` today, which §3.3 cannot stamp and which the first
draft of §3.2 would have refused as an internal error, turning a true answer
into a crash. Naming it here is the correction.

All four MUST be announced in the release notes of the release that carries
them, in the terms note 04 §7 R1 uses for a consumer reading prose, and the
announcement MUST cover **both** surfaces the change reaches: the CLI documents
a consumer migrates, and the JSON-in/JSON-out facade functions §3.3 routes
through the same emitter (`query_json`, `coverage_json`, `orphans_json`), which
a binding consumes without ever running the CLI. A facade caller reads the same
wrapped shapes and has no `--json` flag to notice them by, so a note written
only as CLI migration reaches the wrong half of the consumers. The text forms
are unaffected throughout, which is what the ten skills and the `/spec` and
`/next` flows consume.

The alternative, leaving these unversioned, was rejected: the spec's claim is
that a governed read names its version, and documents exempted from it would
leave a consumer writing the sniffing code this spec exists to retire.

### 3.7 `config show` keeps its own version member

`config show --json` already names a version, as `config_version`, which tracks
`CONFIG_VERSION` and is the version of the configuration record rather than of
the read document. It MUST be routed through the emitter for key order, and its
existing member MUST NOT be renamed: the casing is inconsistent with every other
emitted member, and a rename is a breaking change to spec 054's contract that
buys a consumer nothing. `schemaVersion` MUST NOT be added beside it, because a
document with two version members cannot be dispatched on without a rule about
which one wins.

### 3.8 The tests

`crates/spec-spine-core/tests/read.rs` MUST assert the emitter's properties:
sorted keys for a value whose struct order is not sorted, the array wrapping,
the trailing newline, and the refusals: a scalar document and a top-level
`null` (§3.3) are both internal errors, and a `Preexisting` document with no
version member is a third (§3.2).

`crates/spec-spine-cli/tests/cli.rs` MUST assert, for **each of the thirteen
documents** in §3.1, that the emitted document parses as an object, carries its
version member, and has top-level keys in sorted order. Asserting it per
document rather than once on the emitter is deliberate: the defect this spec
fixes was never in a shared function, it was a set of call sites that did not
use one, and a projection flag is a call site.

Two cases carry their own assertion beyond that:

- `registry plan --next --json` **on a corpus with an empty ready set**, which
  MUST emit `next: null` at exit 0. Constructing that corpus is the test's work;
  without it the `null` path is unexercised, which is how it survived the first
  draft of this spec.
- `registry list --ids-only --json`, whose `items` MUST hold strings rather than
  record objects, so the amendment in §3.6 does not quietly become a change to
  what the projection projects.

## 4. Out of scope

**The 037 envelope for reads.** §1.3 records why: `ok` and `exitCode` cannot be
populated honestly by a read, and moving every existing member under `report`
breaks every consumer for no information.

**Renaming `config_version`** (§3.7).

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
the reason in §1.3. The cost is §3.6: the documents that are not objects today
must become objects, which is breaking for them, and that is accepted rather
than exempting them.

**D-2 (2026-09-14). One axis, not eight.** `READ_SCHEMA_VERSION` covers every
read document (§3.4).

**D-3 (2026-09-14). No DTO carries the version.** The member is added where the
document is written, following spec 055 §3.3, so no committed shard and no
schema file changes and `compile --check` stays fresh across this spec.

**D-4 (2026-09-15). A projection flag produces its own read document.**
`--ids-only`, `--nonzero-only` and `--next` each emit a document a consumer pins
on its own, so each is versioned, each is asserted in §3.8, and `--ids-only`'s
shape change is amended rather than assumed. The first draft of this spec folded
them into their parent verbs and consequently missed that an approved spec
states one of their shapes. Measured on 2026-09-15 at `0.19.0`: thirteen
documents, not six.

**D-5 (2026-09-15). An absent answer is a present `null`, never a missing
member.** §3.3. `plan --next --json` emits the bare literal `null` on an empty
ready set at exit 0, which the first draft of §3.2 would have refused as an
internal error. The fix is not to special-case the verb but to state what an
absent answer looks like, so the next read verb that can answer "nothing" has a
shape to use. The cost is the `amends` edge to spec 060.

**D-7 (2026-09-15). Generic wrapping is the emitter's, named wrapping is the
caller's.** §3.2 and §3.3. The emitter wraps an array under `items` and refuses
a top-level `null` or scalar; the caller builds `{ "next": ... }` for both the
populated and the empty answer. The split is forced by the signature: a
`(value, mode)` emitter cannot tell a populated `plan --next` object from a
document that is already in its final shape, so leaving the wrap to the emitter
would have made the populated and empty answers differ in shape, which is the
defect §3.3 exists to close.

**D-6 (2026-09-15). The `config show` exemption is a mode, not a name check.**
§3.2 and §3.7 read as a contradiction unless the emitter is told which document
is already versioned. It is told, by an argument; it never infers it from the
member names present. An emitter that looked for a member matching `version`
would silently exempt the first read document that grew an unrelated field by
that name.

## Verification

Each line is one command. Every line asserting a version member, an `items`
wrapper or a `next` member fails against pre-093 code, because no read document
carries any of them; those are the fail-first evidence. The sorted-key
assertions on `registry plan`, `registry status-report`, `registry
relationships`, `index owner`, `index coverage` and `index orphans` also fail
against pre-093 code (measured 2026-09-15 at `0.19.0`; §1.1 lists the orders).
The `cargo test` lines are **not** fail-first: the assertions they carry do not
exist at the parent commit, so the suites pass vacuously.

The empty-ready-set case for `plan --next` (§3.8) is **not** here. It needs a
corpus this repository is not, and `verify` runs against this tree; it lives in
`tests/cli.rs`, which the `cargo test` line below runs.

```verify:cli
# 3.4: the axis exists and starts where the note says.
grep -qF 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
# 3.2, 3.5: every object read is sorted and versioned.
target/release/spec-spine registry plan --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry show 093 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --nonzero-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry relationships 093 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index owner Cargo.toml --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index orphans --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
# 3.3, 3.6: the three array reads carry their items under a versioned object,
# and the ids-only projection still projects ids.
target/release/spec-spine registry list --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
target/release/spec-spine registry list --ids-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert all(isinstance(x, str) for x in d["items"]); assert d["schemaVersion"]'
target/release/spec-spine index diagnostics --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
# 3.3, 3.6: the pick is a named member, and it is populated on this corpus.
target/release/spec-spine registry plan --next --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]; assert d["next"]["id"]'
# 3.7: `config show` is sorted and keeps 054's version member, with no second one.
target/release/spec-spine config show --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert "config_version" in d; assert "schemaVersion" not in d'
# 3.8: the emitter's own properties, and the per-document assertions.
cargo test -p spec-spine-core --test read --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.4: the axis is documented where the others are.
grep -qF 'READ_SCHEMA_VERSION' docs/schema-versioning.md
```
