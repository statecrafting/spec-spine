---
id: "032-ownership-coverage"
title: "Ownership coverage: `spec-spine index coverage` and the `C-002` ratchet"
status: approved
kind: "tooling"
created: "2026-08-28"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
  - "009-coupling-floor-claim-precedence"
establishes:
  - "crates/spec-spine-core/src/coverage.rs"
  - "crates/spec-spine-core/tests/coverage.rs"
  - "crates/spec-spine-types/src/coverage.rs"
extends:
  # The gate's second verdict (`C-002`), the `deleted` diff flag, and the
  # code-aware CLI summary.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
  # The shared source-extension list and the `index coverage` verb.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  # The config knob and the report DTO re-export (000 floors the types crate;
  # the same additive shape specs 012 and 013 used).
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # The core re-exports, the `coverage_json` facade, and the e2e exit-code
  # contract file (001's surface, as for specs 011/012/031).
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  spec-spine refuses drift but has never required coverage. The coupling gate
  skips a changed path that no spec claims ("not a coupling concern"), and the
  only coverage-shaped signal, `traceability.untracedCode`, is package-granular
  and read by nothing, so a crate with one governed file and two hundred
  ungoverned ones reports as traced. "This application is fully specified" is
  therefore not a state the tool can assert. This spec adds two additive
  pieces that share one definition of ownership. `spec-spine index coverage`
  reports, per source file, whether a spec *specifically* claims it (a resolved
  unit or a comment header), whether only a package's manifest floor covers it,
  or whether nothing does; `--fail-on-untraced` turns that into the whole-tree
  assertion. `[coupling] require_ownership` (default false) makes the gate ask
  the same question of every changed source file and refuse a floor-only or
  unowned one as `C-002`, the PR-time ratchet. Coverage is a read verb over the
  tree and the committed ledger, like `index check`: nothing new is committed,
  no schema version moves, and the report predicts the gate exactly because
  both call one classifier over one universe.
---

# 032: Ownership coverage

## 1. Purpose

spec-spine's claim is that an authority and its derivation move together. The
engine enforces one direction of that claim: code that a spec claims cannot
change without the spec (`C-001`, spec 005). It says nothing about whether the
corpus claims anything in particular. Two things keep unowned code invisible:

- **The gate skips it.** `couple.rs` resolves the owners of every changed path
  and, on an empty owner set, moves on. That is the right default (a gate that
  failed on unowned code would be unadoptable), but it is unconditional, so a
  repo that has reached full coverage has no way to defend it. Coverage, once
  earned, decays silently.
- **The metric cannot see it.** `traceability.untracedCode` lists packages with
  neither a `spec_ref` nor any implementing path inside them. One claimed path
  anywhere in a package marks the whole package traced, so for a monorepo the
  number is almost always zero and never actionable. Nothing in `lint`,
  `couple`, or the CLI reads it.

A finer metric has a trap of its own. The manifest floor
(`[package.metadata.<ns>].spec`, spec 005 §3.6) makes its spec an owner of
every file in the package, which is exactly right for drift and exactly wrong
for coverage: counted as coverage, it makes any repo that follows the adoption
guide read 100% by construction. This repo would read 61/61 that way. The
metric is only honest if the floor counts as **debt**, not as coverage.

The adopter this serves is the one specifying an application before writing
it (the workflow spec 025's `W-001` tier exists for). Its burn-down lists the
declared units that resolve nowhere; nothing lists the code that was never
declared. This spec supplies that second list and the gate that keeps it empty.

## 2. Territory

`spec-spine-core`'s new `coverage.rs` (the classifier, the universe predicate,
the enumeration, and the report; established here) and its tests; the report
DTOs in `spec-spine-types` (`coverage.rs`, established here); the `C-002` arm
and the `deleted` diff flag in `couple.rs`; the `require_ownership` knob in
`config.rs`; the `index coverage` subcommand in `cmd_index.rs`; the code-aware
summary in `cmd_couple.rs`; the shared source-extension list in `index.rs`; the
re-exports and `coverage_json` facade in `lib.rs`; the e2e contract tests.

All additive. No existing signature changes, no emitted-artifact change, no
schema version moves: coverage is a read verb, not a field of the index (§3.3).

## 3. Behavior

### 3.1 Ownership tiers

For one repo-relative source file, the classifier answers with the most
specific of three tiers, spans ignored (this is a whole-file question):

1. **Specific.** Some spec's *resolved, ownership-bearing* unit covers the
   file: a `file` unit (exact, or a trailing-slash subtree), a `section` or
   `symbol` unit located in it, or a `directory` / `crate` / `module` unit
   (spec 017) whose subtree contains it; **or** the file names its spec in a
   `// Spec:` comment header (spec 004 §3.2). An author decided which spec
   governs this file.
2. **Floor-only.** No specific claim, but a discovered package whose manifest
   names a spec contains the file. The floor is a safety net that covers a
   whole package regardless of what anyone has thought about, so it counts as
   ownership for drift (`C-001`, unchanged) and as debt here.
3. **Unowned.** Nothing covers it.

`references` units never count, in any tier: they are non-owning (spec 000,
constitution) and their locations are read from `resolvedUnits` with the
ownership flag, never from the path-level `implementingPaths` they also land in.
A file that a spec merely references is floor-only or unowned like any other.

**Untraced** means floor-only or unowned: the set `require_ownership` refuses.

### 3.2 The universe

Both the report and the gate ask the tier question of exactly one set of
paths, the **coverage universe**: a path is in it iff

- its extension is one the indexer treats as source (`rs`, `ts`, `tsx`, `js`,
  `jsx`, `go`, `py`, `sh`; the list the comment-header scan already uses, now
  shared so the two can never disagree about what a source file is);
- it lies inside a discovered package's directory (a root package contains
  everything; a file inside a nested package is attributed to the nested one);
- no path component is in `index.resolver_exclusions`;
- the gate would not bypass it (the floor plus `coupling.bypass_prefixes`,
  claim-aware per spec 009).

Prose, manifests, workflows, config (`spec-spine.toml` included), the corpus,
and scripts outside every package are not code the corpus can be expected to
claim, so they are outside the universe and can never raise `C-002`. This is
what makes the flag adoptable without a new bypass vocabulary, and it is what
makes the report an exact predictor: a file the report calls claimed is one the
gate will never refuse for lack of an owner, and a file it lists is one the
gate will refuse the next time it changes.

### 3.3 `spec-spine index coverage`

A read verb over the working tree and the committed index:

```
coverage: 56/64 source files specifically claimed (87.5%); 8 floor-only, 0 unclaimed
  crates/spec-spine-cli (floor 001-compile-registry): 13/13 claimed, 0 floor-only, 0 unclaimed
  crates/spec-spine-core (floor 001-compile-registry): 25/28 claimed, 3 floor-only, 0 unclaimed
  ...

floor-only (owned only by a package floor; claim in a spec):
  crates/spec-spine-core/tests/query.rs
  ...
```

- It is **freshness-guarded** exactly like `couple`: a stale committed index is
  exit 2, never a report over the wrong ledger. An unbuilt index is the same
  I/O error (3) `render` gives.
- `--json` emits the `CoverageReport` DTO: totals, the two sorted debt lists,
  and per-package counts (with the package's floor spec, if any). The same
  shape comes back from the `coverage_json` facade.
- `--fail-on-untraced` exits 1 unless every source file is specifically
  claimed. This is the whole-tree assertion "fully specified", runnable in CI
  once a repo has reached it, and the natural companion to `lint
  --fail-on-warn`.
- It writes nothing and changes no committed artifact.

Coverage is deliberately **not** a field of the index. Storing it would mean
either walking the tree inside the committed-index loader (so `render` stops
being a projection of the shards, spec 011, and every `couple` run pays a full
source walk) or committing it per package and folding it into the shard hash
(so every file added to any crate stales the index and rewrites that crate's
shard, eroding spec 024's conflict-free property for the most common kind of
concurrent PR). A verb over `(tree, committed shards)` has neither cost and is
the same shape as `index check`.

`traceability.untracedCode` is left exactly as it is: its package-granular
question ("is this package governed at all?") is a different one, and changing
an emitted field's meaning in place would silently alter it for every consumer.

### 3.4 `C-002`: the ownership ratchet

When `[coupling] require_ownership = true`, the gate asks the tier question of
every changed path in the universe, **before** the drift question, and refuses
a floor-only or unowned one:

```
C-002  '<path>' has no specific owning spec; only the package floor of <ids> covers it (require_ownership is on)
C-002  '<path>' is not claimed by any spec (require_ownership is on)
```

Severity `Error`, in the same `violations` list as `C-001`, sorted with it by
path. Everything upstream of the ownership question applies unchanged:

1. **The bypass set still exempts** (floor plus adopter additions), and an
   explicit claim still overrides it (spec 009); such a path has a specific
   owner by construction and can only ever be `C-001`.
2. **Waivers still clear it.** `C-002` is an ordinary violation, so the PR-body
   `Spec-Drift-Waiver:` line suppresses the failure exit with the violation
   retained for review. The spec 005 §3.5 dependency-only auto-waiver is
   unaffected: manifests are outside the universe.
3. **One path, one code.** `C-002` takes precedence: a floor-only file that
   changed without its floor spec would otherwise also be `C-001`, but claiming
   the file in a spec resolves both, so the single-subject message is the
   useful one. A specifically claimed file is judged for drift exactly as
   before.
4. **Editing the floor spec does not clear `C-002`.** The ratchet is a claim
   requirement, not a clearance rule; only a specific claim satisfies it.

The CLI summary names both codes when both are present ("2 violation(s): 1
drift (C-001), 1 unclaimed (C-002, require_ownership is on)") and its resolve
hint adds "claiming the path in a spec's owning edge". Every violation line is
now prefixed with its code (`C-001 '<path>' changed without ...`), so a reader
can tell the two apart without the summary; with no `C-002` present the
summary and resolve lines keep their pre-032 wording.

### 3.5 Why `require_ownership` defaults off

`C-001` becomes true of a repo the moment its corpus is written; `C-002`
becomes true only once every source file has a specific claim. Shipping the
flag on would fail every existing adopter's next PR, this repo's included, for
a condition none of them could measure until now. The intended path is the
ratchet: read the report, turn the flag on to stop new debt, drive the lists to
empty, then add `--fail-on-untraced` to CI to defend the state.

### 3.6 Deletions

The CLI marks a path whose diff header is `+++ /dev/null` as `deleted`
(`DiffFile.deleted`, additive: absent in an older `couple_json` request, so it
defaults off). `C-002` never fires on a deleted path: removing unowned code is
how coverage goes up, and a ratchet that refused it would be backwards. Drift
is unchanged: deleting a claimed file is still a whole-file change of an owned
path, judged as before. `--paths-from` carries no history, so a listed path is
judged as an edit.

### 3.7 Determinism

`coverage_with` is a pure function of `(config, index, path set)`: the listing
is sorted and deduplicated before classification, so order and duplicates in
the input cannot change the output. `enumerate_source_files` reads no clock and
no environment: the walk sorts every directory listing before descending, and
every emitted path is repo-relative POSIX. `C-002` is a pure function of
`(config, registry, index, diff)` like every other verdict `couple_with`
produces. Two runs over one tree agree, and the verdict is identical across the
release matrix.

### 3.8 Measured on this repo

At the commit that lands this spec, `index coverage` reports 59/64 source
files specifically claimed (92.2%). The five floor-only files are
`crates/spec-spine-core/tests/query.rs`, `crates/spec-spine-types/src/error.rs`,
`crates/spec-spine-types/tests/config.rs`, `crates/spec-spine-types/tests/dogfood.rs`
and `crates/spec-spine-types/tests/schema.rs` (all owned only by their crate's
floor: `001` for core, `000` for types). Under the pre-032 package-granular
metric the same tree read as fully traced. Claiming those files means editing
`002` and the tier-1 `000`, which is a governance decision for a human, so the
dogfood config ships with `require_ownership = false` and the debt named.

### 3.9 Tests (minimum)

- The classifier returns Specific for a file unit, a subtree unit, a
  directory-kind unit, a crate-kind unit, and a comment header; FloorOnly
  (naming the floor) for a file only a manifest covers; Unowned otherwise; and
  FloorOnly for a file a spec merely references.
- The universe excludes an excluded directory, a bypassed directory, a
  non-source file, a manifest, the corpus, and a script outside every package;
  every enumerated file satisfies the predicate (the report and the gate pinned
  to one set).
- The report is identical across two runs and across an unsorted, duplicated,
  noisy listing; a nested package's files are counted once and attributed to
  the nested package.
- `coverage` is an I/O error with no index, a report after `index`, and
  `Error::Stale` after a spec edit without re-indexing.
- With the flag off, an unowned changed path yields no violation (the pre-032
  contract, pinned). With it on: an unowned path is `C-002`; a floor-only path
  is `C-002` naming the floor, with or without the floor spec in the diff; a
  unit-claimed path is `C-001` and clears with its spec; a header-claimed path
  is specific; a referenced file is floor-only; a deleted path is never
  `C-002`; paths outside the universe and bypassed paths never are; an explicit
  claim under a bypass is `C-001`; a waiver clears `C-002`; one diff sorts
  `C-001` and `C-002` by path with one code per path.
- End to end: `index coverage` is 3 before `index`, reports text and JSON, is
  1 under `--fail-on-untraced` with debt and 0 without, and is 2 when stale;
  with `require_ownership` on, a real `git diff` adding an unowned source file
  exits 1 naming `C-002`, a waiver clears it, and deleting the file exits 0.
- `require_ownership` defaults to `false` and parses from TOML.

## 4. Out of scope

**Making `require_ownership` the default.** A future MAJOR may revisit it; it
cannot change under a MINOR without breaking every adopter.

**The upstream direction.** Hash-pinning a vendored external authority (an
RFC, a paper) so that revising it makes the deriving spec stale is the natural
sibling of this spec and needs a `references`-side unit contract and a
staleness input, neither of which this spec touches.

**A lint-side coverage gate.** `lint` judges corpus well-formedness, not the
code tree. The whole-tree assertion lives on the verb that reads the tree
(`index coverage --fail-on-untraced`); the per-change ratchet lives on the gate
that names a changed path (`C-002`).

**Non-source assets.** The extension list is the indexer's; a governed `.sql`
or `.proto` is a resolver question, not a coverage one.

**Span-granular coverage.** Whether a file with one claimed symbol and ten
unclaimed ones is "covered" is a finer question than a file listing can
answer, and answering it would make the report and the gate disagree (the gate
sees hunks, the report sees files). File granularity is the contract.

**Changing what `C-001` counts as an owner.** Today the drift gate reads every
`implementingPaths` entry as ownership, including the locations of `references`
units, which contradicts spec 004 §3.2 ("`references` is non-owning and
contributes no traceability"). This spec's classifier does not inherit that
reading, so `C-002` is correct; aligning `C-001` and the traceability lists is
a separate amendment against 004/005.
