# Schema versioning

> The registry and index schemas are **library-owned** and decoupled from any
> consumer's history. They start fresh at `0.1.0` (not OAP's `2.2.0` / `3.0.0`
> lines). This document is the policy: what is MINOR vs MAJOR, how loaders react,
> the pre-1.0 caveat, the `deny_unknown_fields` forward-compat consequence, and
> how adopters pin.

## The versioned artifacts

| Artifact | Field | Current | Owner |
|---|---|---|---|
| registry shards (`spec-registry/by-spec/<id>.json`) | `specVersion` | `1.6.0` | library |
| index shards (`codebase-index/by-spec/<id>.json`, `by-package/<slug>.json`) | `schemaVersion` | `1.1.0` | library |
| corpus attestation (`attestation/attestation.json`) | `schemaVersion` | `0.1.0` | library |
| per-spec attestation (`attestation/by-spec/<id>.json`) | `schemaVersion` | `0.1.0` | library |
| authority snapshot (`attestation/snapshot.json`, spec 070) | `schemaVersion` | `0.1.0` | library |
| verdict envelope (any `--json` verdict verb) | `schemaVersion` | `0.5.0` | library |
| change-classification report (`delta --json`, spec 071) | `schemaVersion` | `0.1.0` | library |
| read documents (`--json` on the read verbs, and the facades behind them; spec 074) | `schemaVersion` | `0.6.0` | library |
| `build-meta.json` | `schemaVersion` | `0.1.0` | library (non-deterministic; excluded from goldens) |
| `spec-spine.toml` | `config_version` (optional) | `0.1.0` | library |

Every row is a compile-time `const` in `spec-spine-types/src/version.rs`
(`attest.rs` for the corpus attestation), and the conformance test asserts the
emitted JSON validates against the embedded schema of that version, so a value
here that has gone stale is a documentation bug rather than a behavior one. The
list is the whole set: an artifact that carries a version and is absent from
this table is a gap in the table.

Since spec 022 both artifacts are **sharded** (one committed file per authority
unit; no monolithic `registry.json` / `index.json`). Each shard carries its
artifact's schema version; a reader gates the MAJOR per shard. The in-memory
aggregate DTOs (`Registry`, `CodebaseIndex`) are unchanged and carry the same
version.

MINOR history:

- `index.json` `0.2.0` (spec 011): additive `build.sliceHashes` -- per-slice
  content hashes for `index check --slice <name>`.
- `registry.json` `0.2.0` (spec 012): `extraFrontmatter` values under
  **declared** keys (`frontmatter.extra_known_keys`) widen from
  `string | string[]` + scalars to arbitrary JSON. Generic readers
  (`serde_json::Value`) are untouched; readers that assumed the narrow shape
  were depending on an undocumented restriction. Undeclared keys keep the
  narrow shape.
- `index.json` `0.3.0` (spec 016): additive `directory` / `crate` / `module`
  resolved-unit kinds. The payload is schema-permissive, so the bump is the
  version const alone; readers that handled only `file`/`section`/`symbol`
  see new `kind` values in `resolvedUnits[].unit`.
- `registry.json` `0.3.0` (spec 018): a `supersedes` item may be a structured
  object (`{ spec, scope, unit? }`) as well as a bare id. Full supersession
  still emits a bare string, so a full-only corpus is byte-identical; only a
  `partial` item emits an object. Readers that assumed `supersedes: string[]`
  must accept `string | object` entries.
- registry `1.4.0` (spec 106): additive `obligations` (a spec's declared
  requirements, invariants and verifications) and `sectionDigests` (every body
  section's anchor mapped to a SHA-256 over `<specPath>#<anchor>`, NUL, and the
  section's normalized lines). `sectionDigests` appears on **every** shard,
  because every spec has headings, so every shard's bytes change once; no
  `shardHash` moves, because it is over `spec.md`. (This table read `1.2.0`
  before this change; `1.3.0` was spec 082's `amendsVerification`.)
- registry `1.5.0` (spec 109): additive `impacts` and `conflicts` on a record,
  each entry's `obligation` normalized to its full qualified
  `<spec-id>#<obligation-id>` form. Absent on every existing spec (neither key
  existed before this spec), so only `specVersion` restamps and no
  `shardHash` moves.
- registry `1.6.0` (spec 110): additive `interfaceReferences` on a record, a
  spec's cross-corpus citations carried verbatim (`corpus`, `spec`, `digest`,
  `sections`?, `obtained`, `rationale`?). Absent on every existing spec, so only
  `specVersion` restamps and no `shardHash` moves. A binary predating it meets
  the member with a parse error (exit 3).

MAJOR history:

- registry + index `1.0.0` (spec 022): **breaking on-disk shape.** The committed
  artifacts move from one monolithic file behind a global content-hash line to a
  per-unit shard tree (`by-spec/<id>.json`, `by-package/<slug>.json`), so two PRs
  touching different units write disjoint files and never conflict. The aggregate
  view (validation, orphans, untraced code, content hash) is recomputed from the
  shard set on read, never committed. The in-memory DTOs are unchanged; only the
  storage shape and the version line move. A 0.x reader rejects a 1.x shard
  (unknown MAJOR), so adopters re-run `compile` + `index` once on upgrade. The
  spec 011 `[index.slices]` hashes move to a small `codebase-index/slices.json`
  sidecar.

Each is a **compile-time `const`** in `spec-spine-types`
(`REGISTRY_SCHEMA_VERSION`, `INDEX_SCHEMA_VERSION`, `BUILD_META_SCHEMA_VERSION`,
`CONFIG_VERSION`, and the record axes `VERDICT_SCHEMA_VERSION`,
`DELTA_SCHEMA_VERSION`, `READ_SCHEMA_VERSION` and `SNAPSHOT_SCHEMA_VERSION`). The conformance test asserts that emitted JSON validates
against the *embedded* JSON Schema of that version, so a schema/version mismatch
fails the **build**, not at runtime. The schemas live inside
`spec-spine-types/schemas/` and are `include_str!`'d, which makes the published
crate self-contained and the version a true compile-time constant.

---

## MINOR vs MAJOR

The version line is SemVer over the *schema*, independent of the toolchain's own
crate version (though in practice they move together pre-1.0).

**MINOR bump = additive only.** Old readers keep working.

- a new **optional** field
- a new enum variant (e.g. a new edge type)
- a new unit kind (`crate` / `module` are reserved for exactly this)

**MAJOR bump = breaking.** Old readers must refuse.

- a removed or renamed field
- a retyped field
- changed semantics of an existing field

---

## How loaders react

`load_registry` / `load_index` (the overlay seam; see
[overlay-contract.md](overlay-contract.md)) enforce the policy at read time. They
**reject an unknown MAJOR**:

```text
parse the artifact's specVersion / schemaVersion
  → not semver?           → Error::Schema("… is not semver")
  → MAJOR ≠ our MAJOR?     → Error::Schema("… schema MAJOR N is unsupported (this build understands M.x)")
  → otherwise             → load (the build understands its own MAJOR line)
```

So a build understands **its own MAJOR line only**. A `0.x` build reads any `0.y`
artifact; it refuses a `1.x` artifact. A `1.x` build refuses a `0.x` or `2.x`
artifact. This is what lets an overlay pinned to a MAJOR trust the bytes it reads.

Within a MAJOR, MINOR differences are *not* rejected by the loader; new optional
fields a newer producer added are simply ignored by an older reader (standard
additive evolution). The exception is the pre-1.0 caveat below.

---

## Pre-1.0 caveat (`0.x`)

Under SemVer `0.x`, **MINOR may break.** While the library is `0.x`, a MINOR
schema bump (`0.1 → 0.2`) is permitted to make a breaking change; there is no
stable MAJOR yet to absorb it. Because every MAJOR is `0`, the loader's
MAJOR-equality check does **not** catch a `0.1`-vs-`0.2` break; the guard against
it is:

1. **Pin the toolchain version** (see below) so producer and reader move together.
2. **`deny_unknown_fields`** (next section) turns any field the older reader does
   not know into a hard error rather than a silent misread.

Once the library reaches `1.0.0`, MINOR becomes strictly additive and the
MAJOR-rejection rule is the full forward-compatibility story.

---

## `deny_unknown_fields` and forward compatibility

Which types refuse a member they do not know, as of spec 068:

| Refuses an unknown member | Does not |
|---|---|
| `Config` (every `spec-spine.toml` table) | the registry DTOs |
| the typed-edge items and `Unit` | the index DTOs |
| every attestation type (corpus and per-spec, and their nested types) | |
| `LedgerSeal` | |

The registry and index DTOs are the deliberate exception. Their loaders gate the
MAJOR per shard, and a committed shard is regenerated by the reader's own build
rather than handed between parties, so there is no second party whose unknown
member could go unverified. Whether they should refuse one anyway is a separate
question with a whole-tree restale as its cost (spec 068 4).

For the types that do refuse, it is a deliberate trade:

- **Upside:** a misspelled config knob, or a field from a newer schema an older
  binary does not understand, is a **loud error**, never a silently-ignored
  setting or a silent misread. This is the exact failure class that sank a
  reference repo (a stale config silently producing wrong output).
- **Consequence: an older pinned binary errors on a newer config/artifact.** If
  you upgrade your `spec-spine.toml` to use a knob added in a newer release, an
  older pinned `spec-spine` binary will reject it with a clear `config error`
  rather than ignoring the knob. **This is correct behavior under the pre-1.0
  pin model**: the error tells you to upgrade the binary, instead of letting a
  newer config quietly do nothing on an old binary. Upgrade the binary and the
  config together.

The same applies to artifacts: an older reader given a newer-schema artifact with
an unknown field fails loudly, which is the intended safety net.

**Store the bytes `attest` wrote.** `verify-attestation` decides on the bytes it
is given (spec 068 3.1): the seal is checked over SHA-256 of those bytes, and
`--recompute` compares them against the canonical serialization of what they
parsed to. That is the rule a consumer can check without trusting the verifier,
since the digest a record is referenced by is over the stored bytes. It also
means a re-serialized copy no longer verifies, even when every value survived
the round trip. Keep the file, byte for byte; do not pass an attestation through
a formatter, a JSON library that reorders keys, or a database column that
normalizes whitespace. A consumer holding only a parsed value can still verify
it through the facade by sending `attestationText` (spec 068 3.4).

---

## How adopters pin

Determinism + pinning is how an adopter gets reproducible governance:

- **crates.io:** pin the CLI version, `cargo install spec-spine-cli --version
  =X.Y.Z`, or commit a `Cargo.lock`/`--locked` in CI.
- **Prebuilt binary:** pin the release tag, `SPEC_SPINE_VERSION=vX.Y.Z
  install.sh` (see [adoption-guide.md](adoption-guide.md)).
- The binary **embeds** the schema version; every emitted artifact **carries** it
  (`specVersion` / `schemaVersion`). So a committed `registry.json` /
  `index.json` records exactly which schema produced it, and a mismatched binary
  is caught at load.

Keep the binary version and the committed artifacts in lockstep; bump them
together, and let the loader's MAJOR check (post-1.0) or the pin + `deny_unknown_fields`
(pre-1.0) catch any drift.

---

## Reserved-for-MINOR extension points (additive by construction)

These are already shaped so they slot in as a MINOR (additive) bump later,
without breaking readers:

- **`Unit` kinds `crate` and `module`**: the enum is additive; today's readers
  ignore a unit kind they do not recognize within the same MAJOR.
- **New edge types**: additive enum variant.
- **`provenance.uri_schemes`**: an open map, so new schemes are config, not a
  schema change at all.
- **`frontmatter.extra_known_keys` / `extra_frontmatter`**: overlay-specific
  fields ride through without a schema bump (see
  [overlay-contract.md](overlay-contract.md) §5).

## The read-document axis: spec 074

`READ_SCHEMA_VERSION` versions **the shape of an answer**: the JSON a read verb
emits with `--json` (`registry list`, `show`, `status-report`, `relationships`,
`plan`, their projection flags, and `index owner`, `coverage`, `diagnostics`,
`orphans`), and what the facades `query_json`, `coverage_json` and
`orphans_json` return. It does not version the artifacts those answers are
about, so it does not move when `REGISTRY_SCHEMA_VERSION` or
`INDEX_SCHEMA_VERSION` does. MINOR adds a member; MAJOR removes, renames or
changes the meaning of one.

Every read document is an object with sorted keys and a top-level
`schemaVersion`. `config show --json` is the one exception to the stamp: it
already carries `config_version` (spec 047) and gains no second version member.
A read is not a verdict, so it is not wrapped in the spec 034 envelope.

**Breaking in the release that ships spec 074**, for the CLI and the facades
alike:

| Document | Before | After |
|---|---|---|
| `registry list --json` / `query_json` `list` | `[ {...} ]` | `{ "items": [ {...} ], "schemaVersion" }` |
| `registry list --ids-only --json` / `query_json` `list` + `idsOnly` | `[ "id" ]` | `{ "items": [ "id" ], "schemaVersion" }` |
| `index diagnostics --json` | `[ {...} ]` | `{ "items": [ {...} ], "schemaVersion" }` |
| `registry plan --next --json` | `{ "id", "title" }` or `null` | `{ "next": { "id", "title" } \| null, "schemaVersion" }` |
| `orphans_json` (facade) | `[ "id" ]` | `{ "items": [ "id" ], "schemaVersion" }` |

Every other read document gains `schemaVersion` and keeps its members, now in
sorted order. That is additive when reading by key; a strict decoder that
rejects unknown members, or a consumer that hashes or golden-files the bytes,
sees a change. The text forms are unchanged.

**`0.2.0` (spec 102), additive.** Each `registry plan` ready entry gains
`status`, the spec's `status` as the registry records it:
`{ "id", "status", "title" }`, and so does `plan --next`'s `next` object,
which is a ready entry. `blocked` entries do not gain it. Ready-set
membership and order are unchanged; `status` is reported, never consulted. The
registry schema does not move, and no `shardHash` changes. Every read document
carries the new `schemaVersion`, because the axis is one constant (spec 074).

**`0.3.0` (spec 106), additive.** A new read document: the `obligation` answer
(`registry obligation <spec>#<id> --json`, `query_json` `op: "obligation"`),
`{ "spec", "specPath", "obligation", "sectionDigest", "schemaVersion" }`, and
`contentHash` from the CLI when the committed shard is present. No member of
an existing document moved.

**`0.4.0` (spec 107), additive.** A new read document: the resolved closure
(`registry closure --request <file> --json`, `closure_json`),
`{ "digest", "members", "rationale"?, "schemaVersion" }`. No member of an
existing document moved.

**`0.5.0` (spec 109), additive.** A new read document: the impact set
(`registry impacts [--target] [--declared-by] --json`, `query_json`
`op: "impacts"`), `{ "impacts": [...], "conflicts": [...], "schemaVersion" }`.
No member of an existing document moved. (This table's summary row above had
drifted to `0.2.0` since spec 102; corrected here alongside the axis's own
move, since spec 106's and 107's entries below were already accurate and only
the summary line had fallen behind.)

**`0.6.0` (spec 110), additive.** A new read document: the interface report
(`interface verify --json`, `interface_verify_json`), `{ "references": [...],
"summary": {...}, "schemaVersion" }`. No member of an existing document moved.

## Migration note: spec 034, the verdict envelope

**If you regex a field out of a verdict verb's stdout, stop, and parse the
`--json` envelope instead.**

The case that broke: `spec-spine attest` printed its `attestationHash` in a
prose line, and claude-observatory matched that line with a regular expression
at two call sites. Prose is a rendering. It is allowed to change wording, and it
did. The envelope is the contract.

```sh
# Don't
spec-spine attest | grep -o 'attestationHash: .*' | cut -d' ' -f2

# Do
spec-spine attest --json | jq -r '.report.attestationHash'
```

Every verb that renders a verdict takes `--json`: `compile --check`,
`index check`, `lint`, `couple`, `attest`, `verify-attestation`, `verify`, and
`compile --spec`. Each writes one envelope with `schemaVersion`, `verb`, `ok`,
`exitCode`, and either `report` or `error`.

**Verdict `0.5.0` (spec 100).** `couple`'s report gained a `deletions` block
naming, for each deleted path examined, which snapshot resolved its owners
(`merge-base`, `head-commit` or `head-tree`) and whether the path was absent
from it. Additive in the strict sense this policy requires: the block is
**omitted when empty**, so every input that produced a verdict before spec 100
still produces the same payload bytes, and a consumer that does not read it is
unaffected.

**The guarantee that makes migrating safe:** `--json` changes what is written
and never what is decided. Every exit code is identical with and without it. A
consumer switching to the envelope is changing its parsing, not its control
flow, so the migration can be done one call site at a time with no behavioral
risk.

## Pinning the binary: `[meta] required_version`

The schema versions above tell a **loader** what it can read. They say nothing
about which **binary** should be reading it, and a repository is otherwise
governed by whichever `spec-spine` happens to be on the path. Spec 055 adds
`[meta] required_version` for that: a semver requirement the CLI checks before
doing any work, refusing with exit `3` and naming the requirement, the running
version, and where the pin lives.

Cargo's conventions: a bare `"0.15.0"` is a caret range, `"=0.15.0"` is exact,
`">=0.15, <0.16"` is a range, `"*"` constrains nothing. `--version`, `--help`
and `init` are exempt, because they are how an operator finds out what they are
running and what to do about it.

**A binary older than the pin refuses too, for the wrong reason.** Every
`Config` table carries `deny_unknown_fields`, so a binary released before spec
055 rejects the whole `[meta]` table as an unknown field:

```
spec-spine: config error: TOML parse error at line 1, column 2
unknown field `meta`, expected one of `manifest`, `domains`, ...
```

That is worth knowing rather than discovering. It is the property that makes the
pin usable at all: an adopter adding the key today is protected from every older
binary immediately, not only from future ones. The message is poor, naming a
parse error rather than a version mismatch, and it cannot be improved, because
the binary producing it was built before the key existed. **If you see "unknown
field `meta`", your binary is out of date; it is not a corrupt config.**

The two axes stay separate. `required_version` constrains the binary; a
repository can pin a binary without pinning a schema (the ordinary case), or
find its shards rejected by a binary that satisfies its pin, which is the pin
being too loose and the loader being right to refuse.
