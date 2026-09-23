---
id: "112-typed-overlays-ride-the-existing-seam"
title: "Typed overlays ride the existing seam"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "012-declared-extra-frontmatter-passthrough"
summary: >
  Four proposals (effect contracts, budgets, dependency rationale, contract
  adapters) all want to attach typed data to a spec's units without changing
  what a unit is. The corpus already has the seam: declared extra frontmatter
  (spec 012), carried into the registry as `extraFrontmatter`. This states the
  contract an overlay keeps on that seam, pins what the engine does and does
  not promise about it (canonical values, not original bytes; a hash scope
  that is the declaring spec's, not every shard), and asserts it with several
  independent overlays in one corpus. No new unit kind, no engine
  enforcement, and no domain schemas.
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/overlays.rs" }
  - { kind: file, path: "docs/overlay-contract.md" }
extends:
  # D-3: the claim puts the document into `[index] extra_hashed_inputs`.
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "specs/012-declared-extra-frontmatter-passthrough/spec.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "Each declared overlay key reaches its spec's registry record independently, with its value preserved under spec 012's canonical JSON normalization."
    anchor: "3-2-the-engine-transports-an-overlay-s-value-and-never-its-meaning"
  - id: "R-2"
    kind: requirement
    text: "An edit to an overlay value stales exactly the declaring spec's registry shard and the index shards that record that spec's content hash, as any edit to that spec.md does."
    anchor: "3-3-the-hash-scope-is-the-declaring-spec"
  - id: "R-3"
    kind: requirement
    text: "An undeclared key carrying a nested value is refused at compile time; declaring the key is what admits it."
    anchor: "3-4-declaring-is-what-admits-an-overlay"
  - id: "I-1"
    kind: invariant
    text: "No gate verdict changes because an overlay is present: compile, check, lint, coverage and couple reach the same answers on equivalent fresh trees with and without overlays."
    anchor: "3-6-no-verdict-reads-an-overlay"
  - id: "V-1"
    kind: verification
    text: "Several independent overlays, nested values, undeclared keys, freshness scope and gate neutrality, each on a scratch corpus."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/overlays.rs"
---

# 112: Typed overlays ride the existing seam

## 1. Purpose

Design note 04 §5 P2 carries four proposals that look unrelated and are the
same shape:

| Register | Proposal | What it attaches |
|---|---|---|
| B15 | effect contracts | what a unit may do: read, write, network, spawn |
| B18 | budgets | a cost or latency ceiling for a unit |
| B24 | dependency rationale | why a dependency exists, beyond that it does |
| A09 | contract adapters | how a unit's interface maps to another corpus's |

Each wants to hang typed data off a unit. None changes what a unit **is**:
identity stays `file`, `section`, `symbol`, `directory`, `crate`, `module`,
and resolution stays what it is. That is what makes all four overlay work
rather than engine work, and it is why they get one spec rather than four.

### 1.1 Why the dependency is spec 012

Spec 012 is the seam: `frontmatter.extra_known_keys` declares keys a corpus
recognizes, and their values reach the registry as `extraFrontmatter`.
`docs/overlay-contract.md` describes the library surface an overlay reads
through. What was missing is a contract that says what an overlay may rely on
when several share one corpus, and tests that hold the engine to it. Before
this spec the seam was asserted only for one declared key at a time.

### 1.2 What the reserved draft said that the engine does not do

The draft filed on 2026-09-21 made three claims this spec corrects rather
than builds (D-1):

- that values are preserved **byte-for-byte, including key order**. Spec 012
  §3.2 normalizes a declared value YAML to JSON once, with mapping keys sorted
  on emission. The engine parses and canonicalizes; it does not keep an
  author's formatting or key order, and a consumer that needs the original
  bytes reads `spec.md`.
- that editing an overlay value **stales every shard**. The content hash is
  per spec. An overlay edit is an edit to one `spec.md`, and it stales that
  spec's shards, not the whole tree (§3.3).
- that an **undeclared overlay key warns**. An undeclared key with a scalar or
  string-list value is accepted silently and counts toward the `V-007` cap;
  one with a nested value is refused as malformed frontmatter (`V-002`). There
  is no per-key warning (§3.4).

## 2. Territory

`docs/overlay-contract.md`, which this spec takes as its own (it had no owner),
and a new acceptance file, `crates/spec-spine-core/tests/overlays.rs`. No
engine source changes: the seam exists, and the build is the contract and its
evidence.

## 3. Behavior

### 3.1 An overlay is a declared extra frontmatter key, and nothing else

An overlay MUST be expressed as a key listed in
`frontmatter.extra_known_keys`. It MUST NOT introduce a new authority-unit
kind, a new edge type, a new verb, or a new field on any core DTO.

The test of whether a proposal is an overlay is exactly this: if it needs
core `Unit` identity to change, it is not one, and it belongs in a spec of its
own against the engine. All four of §1's proposals pass that test today.

### 3.2 The engine transports an overlay's value and never its meaning

The engine's obligations toward a declared overlay key are the ones spec 012
already provides, stated here so an overlay author can rely on them:

1. **Transport the value** into that spec's registry record under
   `extraFrontmatter.<key>`, as JSON: mappings to objects with keys sorted,
   sequences in order, scalars by YAML core-schema resolution (spec 012 §3.2).
   Two overlays in one spec are two independent members; neither is merged
   into, renamed by, or ordered relative to the other beyond canonical key
   order.
2. **Accept any JSON-representable value**, including arbitrary nesting, and
   refuse an unrepresentable one with `V-013` (spec 012 §3.3).
3. **Emit it deterministically**, through the canonical JSON the rest of the
   registry uses.

The engine MUST NOT validate an overlay's internal structure, interpret its
values, resolve anything named inside it, or change any verdict because of
it.

### 3.3 The hash scope is the declaring spec

An overlay value is part of `spec.md`, so it is inside that spec's
`contentHash`. Editing it changes that spec's registry shard and the index
shards that record that spec's content hash, and `check` reports exactly
those as stale until they are regenerated. Every other spec's shards are
unchanged.

The build measures this rather than assuming it: it records which shard files
change between two compiles and two index runs that differ in one overlay
value, and asserts the set.

### 3.4 Declaring is what admits an overlay

A nested value under an undeclared key is refused (`V-002`), and the same key
declared in `extra_known_keys` is accepted. So an overlay cannot arrive in a
corpus by accident, and a corpus admitting one has named it in its
configuration.

A scalar or string-list value under an undeclared key is accepted and counted
toward the `V-007` cap, exactly as before this spec. That is spec 012's
existing split, and this spec does not change it.

### 3.5 Overlays coexist by namespacing, and the consumer owns the meaning

Where a corpus carries more than one overlay, each MUST occupy its own
top-level key; two overlays MUST NOT share a key and partition it by
convention. `extraFrontmatter` is unvalidated, so a collision inside one key
is invisible to everything: no error, no warning, one value silently winning.

An overlay's semantics live with the consumer that reads it. A corpus
declaring an overlay key MUST write down, in its own standards documents,
what the key means, because the configuration records only the name. That is
spec 012's existing requirement; several overlays in one corpus make it
load-bearing.

Per-unit data is expressed by keying the overlay's own structure on a unit
spelling inside its value:

```yaml
effects:
  "crates/spec-spine-core/src/couple.rs":
    reads: ["the committed index"]
    network: false
```

The engine does not resolve those spellings. An overlay can name a path that
does not exist and nothing in spec-spine says so; resolving them is the
consumer's work, against the index it can already read.

### 3.6 No verdict reads an overlay

On two trees that are each freshly compiled and indexed and differ only in
whether overlays are declared and present, `compile`, `check`, `lint
--fail-on-warn`, `index coverage --fail-on-untraced` and `couple` reach the
same answers. The comparison is between equivalent fresh states: a tree with
an overlay edited and not regenerated is stale, which is §3.3 working, not a
verdict reading the overlay.

### 3.7 What an overlay establishes, and what it does not

It establishes that an author **declared** this data against this spec. It
does not establish that the data is true, that it is enforced, or that
anything read it. An effect contract is a claim, not a sandbox; a budget is a
claim, not a limit.

## 4. Out of scope

- **Enforcing anything.** §3.7.
- **Validating an overlay's structure**, or resolving unit spellings inside
  one. §3.2, §3.5.
- **A new unit kind, edge type or DTO field.** §3.1.
- **The four overlays' own schemas.** Each is its consumer's.
- **Preserving original YAML bytes or key order.** §1.2.

## 5. Resolved decisions

**D-1 (2026-09-23, finalized before the build: three claims of the reserved
draft are corrected).** §1.2 lists them. Each was a statement about the
engine that the engine does not make, measured on a scratch corpus with the
0.23.0 binary: a declared nested map compiled with sorted keys; an undeclared
scalar key compiled with no warning; an undeclared nested map was refused
with `V-002`. The draft's hash claim was never measured; §3.3 now requires
the measurement.

**D-2 (2026-09-23, the draft's open questions are answered by the boundary).**
Whether the engine should resolve unit references inside an overlay, and
whether a per-unit overlay key would be a better seam, both require the engine
to know an overlay's shape. Both stay out (§3.2), and a proposal that needs
either is an engine change with its own spec.

**D-3 (2026-09-23, territory).** `docs/overlay-contract.md` is the surface an
overlay author reads, and had no owner. This spec establishes it, which puts
it under `L-008` and therefore into `[index] extra_hashed_inputs`.

**D-4 (2026-09-23, build: the measured hash scope).** Editing one value in
one overlay of one spec changed exactly two shard files across both trees:
that spec's registry shard and its index `by-spec` shard. The other spec's
shards and the package shard were byte-identical, and `check` reported both
trees stale until they were regenerated and fresh after. The test asserts the
changed set by name.

**D-5 (2026-09-23, build: evidence and its limits).** The engine was not
changed, so the tests pass on the engine as it stood; their fail-first is the
absent test target. Their power was checked by a mutant that drops one
declared key in transport (`compile.rs`, building the record), which fails the
independence case. The neutrality case compares two freshly regenerated
trees on a `C-001` refusal, a clearance and a `C-002` refusal, so it cannot
pass by comparing two empty answers.

## Verification

Written to fail against the tree it is filed on: the test target does not
exist.

```verify:cli
# 3.2 - 3.6: independent overlays, nested values, undeclared keys, the
# measured hash scope, and gate neutrality on equivalent fresh trees.
cargo test -p spec-spine-core --test overlays --locked
cargo build --release --locked -p spec-spine-cli
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
