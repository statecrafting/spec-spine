---
id: "032-ownership-coverage"
title: "Coverage: the unclaimed-path gate (`C-002`) and file-granular untraced reporting"
status: approved
kind: "tooling"
created: "2026-08-26"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"   # the traceability layer this widens
  - "005-coupling-gate"    # the gate this adds a second violation code to
  - "011-index-render-orphans"  # the projection that reports the coverage number
extends:
  - spec: "004-codebase-index"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/index.rs"        # untracedFiles computation
      - "crates/spec-spine-core/tests/index.rs"      # coverage acceptance fixtures
      - "crates/spec-spine-types/src/codebase.rs"    # Traceability gains one field
      - "crates/spec-spine-types/src/version.rs"     # INDEX_SCHEMA_VERSION minor bump
      - "crates/spec-spine-types/schemas/codebase-index.schema.json"  # the two optional properties
      - "crates/spec-spine-types/tests/dtos.rs"      # the schema-version pin test
  - spec: "005-coupling-gate"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/couple.rs"       # the C-002 arm
      - "crates/spec-spine-core/tests/couple.rs"     # C-002 acceptance fixtures
      - "crates/spec-spine-types/src/config.rs"      # [coupling] require_ownership
  - spec: "011-index-render-orphans"
    nature: additive
    paths:
      - "crates/spec-spine-core/src/render.rs"       # the coverage line
      - "crates/spec-spine-core/tests/render.rs"     # its projection test
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  spec-spine refuses drift but has never required coverage. The gate skips a
  changed path that no spec claims (`couple.rs`: "unclaimed path, not a coupling
  concern"), and the only coverage-ish signal, `traceability.untracedCode`, is
  computed at *package* granularity, so a crate with one governed file and two
  hundred ungoverned ones reports as fully traced. Nothing anywhere consumes it.
  The consequence is that "this application is fully specified" is not a state
  the tool can assert: a corpus can be complete, every gate green, and most of
  the codebase invisible to the ledger. This spec closes that in two additive
  pieces. (1) `[coupling] require_ownership` (default **false**) turns an
  unclaimed, non-bypassed, changed path into a first-class `C-002` violation, so
  coverage becomes a PR-time ratchet on the same diff the gate already walks.
  (2) `traceability.untracedFiles` reports every unclaimed source file, so the
  coverage number is honest and an adopter can see the distance to totality
  before opting in. Both are additive: `require_ownership` defaults off (turning
  it on by default would red every adopter, this repo included), and the new
  index field is an `INDEX_SCHEMA_VERSION` MINOR.
---

# 032: Ownership coverage

## 1. Purpose

spec-spine's claim is that an authority and its derivation must move together.
That claim has two directions and two failure modes, and the engine currently
implements one of the four:

|  | **counterpart missing entirely** | **one side changed without the other** |
|---|---|---|
| **upstream** (source → spec) | a spec cites no source — silent | the cited source was revised — no mechanism |
| **downstream** (spec → code) | code has no owning spec — **silent** | code changed without its spec — `C-001` |

This spec fills the bottom-left cell. The upstream row is out of scope (§4).

Two things make unowned code invisible today:

- **The gate skips it.** `couple.rs` walks every changed path, resolves its
  owners, and on an empty owner set does `continue` with the comment *"unclaimed
  path, not a coupling concern."* That is the correct default — a gate that
  failed on unowned code would be unadoptable — but it is unconditional, so
  there is no way for a repo that *has* achieved total coverage to defend it.
  Coverage, once earned, silently decays.
- **The metric can't see it.** `traceability.untracedCode` lists *"packages with
  neither a `spec_ref` nor any implementing path inside them."* One claimed path
  anywhere inside a package marks the whole package traced. For a monorepo of
  four crates this yields a number that is almost always zero and never
  actionable. Nothing in `lint`, `couple`, or the CLI reads it.

So the honest statement of today's guarantee is narrower than the README's: *code
that a spec claims cannot drift from it*. Nothing says the corpus claims
anything in particular. For an adopter specifying a whole application up front —
the workflow spec 025's `W-001` lifecycle tier exists to serve — the missing
half is exactly the half that matters: the burn-down tells you which declared
units are unresolved, and nothing tells you which code was never declared.

## 2. Territory

The `C-002` arm and the config field that gates it (`couple.rs`, `config.rs`);
the file-granular unclaimed-file computation (`index.rs`), the DTO field that
carries it (`codebase.rs`), and the schema-version bump (`version.rs`); the
coverage line in the projection (`render.rs`); the four test files.

All additive. No existing signature changes, no behavior change to any existing
verdict, and no new walking machinery: `untracedFiles` reuses `walk_source`
(already deterministic — sorted, `resolver_exclusions`-aware) over the same
package set and extension list the comment-header scan already enumerates.

## 3. Behavior

### 3.1 `C-002`: the unclaimed-path violation

When `[coupling] require_ownership = true`, a changed path that survives the
bypass filter and resolves to **no owning spec** is a violation:

```
C-002  '<path>' is not claimed by any spec (require_ownership is on)
```

Severity `Error`, reported in the same `violations` list as `C-001` and sorted
with it by path. Everything the gate already does upstream of the ownership
decision applies unchanged and in the same order:

1. **The bypass floor still exempts.** `DEFAULT_BYPASS_PREFIXES` ∪ the adopter's
   `bypass_prefixes` are filtered out *before* the ownership question is asked,
   so `docs/`, `.derived/`, lockfiles, and `.github/` never need an owner. This
   is what makes the flag adoptable at all: the floor is the built-in answer to
   "not all files are code."
2. **Explicit claims still beat the floor.** Spec 009's precedence is untouched:
   a path with a resolved unit claim is checked even if it is on the floor — and
   such a path has an owner by construction, so it can never raise `C-002`.
3. **The corpus is exempt.** A path under `layout.specs_dir` never raises
   `C-002`. A `spec.md` *is* the authority, so "which spec claims this spec?" is
   a category error rather than a coverage gap — without this, turning the flag
   on raises a violation for every spec in the corpus, which is what dogfooding
   this spec's own commit did before the exemption existed. It is deliberately
   **not** a new bypass-floor entry: the floor is spec 005's and changing it
   would alter `C-001` too. Corpus paths stay fully visible to `C-001`, so a
   corpus file that is explicitly claimed still drifts normally.
4. **Waivers still clear it.** `C-002` is an ordinary violation, so the PR-body
   `Spec-Drift-Waiver:` line and the spec 005 §3.5 dependency-only auto-waiver
   suppress the failure exactly as they do for `C-001`, with the violations
   still printed. No new waiver vocabulary.

`C-001` and `C-002` are **mutually exclusive by construction**: `C-001` fires
only when the owner set is non-empty, `C-002` only when it is empty. A path
raises at most one, which keeps a failure message single-subject.

### 3.2 `[coupling] require_ownership`, and why it defaults off

```toml
[coupling]
require_ownership = false   # default
```

Default-off is not timidity, it is correctness. Coverage is the one property in
this system that an adopter cannot inherit by installing the tool: `C-001`
becomes true of a repo the moment the corpus is written, whereas `C-002` becomes
true only after every governed file has been claimed. Shipping it on would fail
every existing adopter's next PR — this repo's included — for a condition none
of them has had the means to measure until this spec.

The intended adoption path is therefore the ratchet: read `untracedFiles`
(§3.3), drive it to empty, then set the flag to defend it. The flag is what
turns a one-time cleanup into an invariant.

### 3.3 `traceability.untracedFiles`

A new field beside `untracedCode`, which is **left exactly as it is** — its
package-granular semantics are a different question ("is this package governed
at all?") and changing them in place would silently alter an emitted field's
meaning for every existing consumer.

`untracedFiles` lists every **source file inside a discovered package that no
implementing path claims**, repo-relative POSIX, sorted, deduped. The file set
is `walk_source` over each package directory with the extension list already
used for comment-header scanning (`rs`, `ts`, `tsx`, `js`, `jsx`, `go`, `py`,
`sh`) and `config.index.resolver_exclusions` applied, so the enumeration is the
same one the resolver already trusts.

A file is **claimed** when some implementing path either equals it or is a
directory prefix of it (`claim == file || file.starts_with(claim + "/")`), which
is what makes a subtree claim (`crates/foo/src/`) cover its files without each
being enumerated.

A package-level `spec_ref` **does** claim the package's files. Manifest metadata
lands in `implementing_paths` as a directory claim (`source:
manifest-metadata`), and the coupling gate already treats such a claim as
ownership of every file beneath it (spec 005; `couple.rs` resolves a directory
prefix to its subtree). `untracedFiles` exists to predict `C-002`, so it must
read ownership from exactly the same set the gate does. A crate that names its
spec in `Cargo.toml` is covered, whole.

This is the one place the two untraced fields agree rather than diverge, and it
is load-bearing: any file this field calls claimed is a file the gate will find
an owner for, and vice versa. A coverage number that over-reported would send an
adopter chasing files `require_ownership` was never going to flag.

Alongside it, `traceability.sourceFileCount` carries the **denominator**: the
total source files enumerated across discovered packages. It is stored rather
than re-derived so a consumer can report coverage from the document alone,
without re-walking the tree with the same exclusions.

Both land in the **aggregate view**, which spec 024 recomputes from the shard
set on read and never commits, so no committed shard body changes.

**Known limit, measured on this repo.** spec-spine's own corpus reports
`61/61 files claimed (100.0%)`, because all four packages name their spec in
manifest metadata and a package-level claim covers its whole subtree. The number
is correct — `require_ownership` really would pass here, and really would catch a
new *package* added without a spec — but a repo that claims at crate granularity
gets full coverage by construction, and the field tells it little it did not
already know. The metric is only as fine as the claims the corpus makes.

Deliberately not fixed here: making the gate weigh a blanket manifest claim
differently from a specific unit claim would break the §3.3 correspondence
between this field and `C-002`, which is the property that makes the number
trustworthy at all. Ownership *granularity* is a separate question from
ownership *coverage*, and belongs in its own filing (§4).

### 3.4 The coverage line in `index render`

`render` gains one line under Traceability:

```
- coverage: 138/152 files claimed (90.8%), 14 untraced
```

Counts are derived from `untracedFiles` and the enumerated file set; the
percentage is rounded to one decimal. When there are no untraced files the line
reads `coverage: 152/152 files claimed (100.0%)` — the state `require_ownership`
is meant to defend. The existing orphans section is unchanged.

### 3.5 Schema version

`INDEX_SCHEMA_VERSION` goes `1.1.0 → 1.2.0`: two optional fields added to an
emitted DTO, none removed or retyped, loaders of `1.1.0` unaffected. Per
`docs/schema-versioning.md` that is a MINOR.

Unlike the registry side (spec 028), this **does** require a schema-file edit.
`codebase-index.schema.json` sets `additionalProperties: false` on
`traceability`, so an additive field is rejected by the conformance test until
the schema names it. Both new properties are declared **optional** (absent from
`required`), which is what keeps the bump a MINOR: a `1.1.0` document with
neither field still validates. The per-shard schemas are untouched — these
fields live only in the recomputed aggregate.

### 3.6 Determinism

Both additions are pure functions of `(config, file contents)`. `untracedFiles`
reads no clock and no environment: `walk_source` sorts every directory listing
before descending, the claim set is a `BTreeSet`, and the output is sorted and
deduped, so the field is byte-identical across the release matrix on one tree.
`C-002` is a pure function of `(config, registry, index, diff)` like every other
verdict `couple_with` produces.

### 3.7 Tests (minimum)

- With `require_ownership = false` (default), a changed unclaimed path yields no
  violation — the existing behavior, pinned against regression.
- With `require_ownership = true`, the same diff yields exactly one `C-002`.
- A bypassed path (floor and adopter-configured) yields no `C-002` even with the
  flag on.
- A spec-009 explicitly-claimed path on the bypass floor yields `C-001`, never
  `C-002`, when changed without its spec.
- A path with owners changed without its spec yields `C-001` only — the two
  codes never both fire for one path.
- A PR-body waiver suppresses a `C-002` failure exactly as it does `C-001`.
- A path under the corpus directory raises no `C-002`, and the exemption tracks
  a reconfigured `layout.specs_dir` rather than a hardcoded `specs/`.
- `untracedFiles` lists an unclaimed file inside a governed package, omits one
  covered by a subtree claim, and omits one matched by `resolver_exclusions`.
- A package with a `spec_ref` and no unit claims reports its files as **claimed**
  (§3.3), and the same file raises no `C-002` — the predictor and the gate pinned
  against each other.
- `untracedFiles` is sorted and deduped; two runs over one tree agree.
- `render` emits the coverage line, and the `100.0%` form when the list is empty.
- The `INDEX_SCHEMA_VERSION` pin test asserts `1.2.0`.

## 4. Out of scope

**The upstream row of the §1 table.** Hash-pinning a vendored external authority
(an RFC, a paper, a standard) so that revising it makes the deriving spec stale
is the natural sibling of this spec and is deliberately a separate filing: it
needs a `references`-side unit contract and a staleness input, neither of which
this spec touches.

**Making `require_ownership` the default.** A future MAJOR may revisit it; it
cannot change under a MINOR without breaking every adopter.

**Ownership granularity.** Whether a package-level manifest claim should count
as weaker evidence than a unit claim, and whether the gate should say so (§3.3).
It changes what `C-001` means, not just what `C-002` measures, so it is a
separate filing against spec 005.

**A lint-side whole-tree coverage gate.** `lint` judges corpus well-formedness,
not the code tree, and a repo-wide "you are 90% covered" failure has no
actionable diff attached. Coverage is enforced where it can name a changed path:
the coupling gate.

**Changing `untracedCode`'s package granularity** (§3.3), and teaching
`untracedFiles` about non-source assets — the extension list is the resolver's,
so a governed `.sql` or `.proto` is a resolver question, not a coverage one.
