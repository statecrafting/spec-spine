---
id: "112-typed-overlays-ride-the-existing-seam"
title: "Typed overlays ride the existing seam"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "012-declared-extra-frontmatter-passthrough"
summary: >
  Four proposals (effect contracts, budgets, dependency rationale, contract
  adapters) all want to attach typed data to a unit without changing what a
  unit is. The corpus already has the seam for that: declared extra frontmatter,
  preserved verbatim into the registry. This states the contract an overlay
  keeps so four of them can coexist without any of them reaching the engine.
  Deferred: specified now, built when a consumer names the need.
references:
  - unit: { kind: file, path: "docs/overlay-contract.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 112: Typed overlays ride the existing seam

## 1. Purpose

Note 04 §5 P2 carries four proposals that look unrelated and are the same
shape:

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

### 1.2 Why the dependency is spec 012

Spec 012 is the seam: `frontmatter.extra_known_keys` declares keys a corpus
recognizes, and their values are preserved verbatim into the registry as
`extraFrontmatter`. Every overlay below rides it. `docs/overlay-contract.md`
describes the surface an overlay may depend on and is the other half of the
existing mechanism.

No obligations dependency, no closure dependency. An overlay attaches to a
unit, and units predate all of this.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 An overlay is declared frontmatter, and nothing else

An overlay MUST be expressed as a declared extra frontmatter key. It MUST NOT
introduce a new authority-unit kind, a new edge type, a new verb, or a new
field on any core DTO.

The test of whether a proposal is an overlay is exactly this: if it needs core
`Unit` identity to change, it is not one, and it belongs in a spec of its own
against the engine.

All four of §1's proposals pass that test today.

### 3.2 The engine reads an overlay's shape and never its meaning

The engine's obligations toward an overlay are three, and they are the ones
spec 012 already provides:

1. **Preserve it verbatim.** Byte-for-byte into `extraFrontmatter`, including
   key order within the value, so a consumer reads what an author wrote.
2. **Do not warn about it** once declared in `frontmatter.extra_known_keys`.
3. **Fold it into the content hash**, because it is part of `spec.md`, so an
   overlay edit stales the ledger like any other authored change.

The engine MUST NOT validate an overlay's internal structure, interpret its
values, or change any verdict because of them. An effect contract saying a
unit performs no network access is a claim the corpus records and does not
check; a budget is a number nobody enforces here.

### 3.3 The consumer owns the meaning, and says so

An overlay's semantics live with the consumer that reads it. A corpus
declaring an overlay key MUST write down, in its own standards documents, what
the key means, because the configuration records only the name.

This is spec 012's existing requirement and it is restated because four
overlays in one corpus make it load-bearing: four undocumented keys is a
corpus nobody can read.

### 3.4 Overlays coexist by namespacing, not by luck

Where a corpus carries more than one overlay, each MUST occupy its own
top-level key. Two overlays MUST NOT share a key and partition it by
convention.

The reason is that `extraFrontmatter` is preserved verbatim and unvalidated,
so a collision between two overlays is invisible to everything: no error, no
warning, one value silently winning.

### 3.5 What an overlay establishes, and what it does not

It establishes that an author **declared** this data against this spec.

It does not establish that the data is true, that it is enforced, or that
anything read it. An effect contract is a claim, not a sandbox; a budget is a
claim, not a limit. A consumer that enforces one is doing its own work and
takes its own responsibility for it.

### 3.6 Attaching to a unit, which is the one thing the seam does badly

`extraFrontmatter` attaches to a **spec**, not to a unit within it. Every one
of §1's four proposals wants per-unit granularity.

The honest statement of the current seam is therefore: an overlay expresses
per-unit data by keying its own structure on the unit, inside its own value,
and the engine neither validates that keying nor resolves those units.

```yaml
effects:
  "crates/spec-spine-core/src/couple.rs":
    reads: ["the committed index"]
    writes: []
    network: false
```

This works and it is weaker than it looks: the unit spelling inside an overlay
is a string the engine never resolves, so an overlay can name a path that does
not exist and nothing says so. §5 records that as the open question, because
fixing it means the engine reading an overlay's shape, which §3.2 forbids.

## 4. Out of scope

- **Enforcing anything.** §3.5.
- **Validating an overlay's structure.** §3.2.
- **A new unit kind or edge type.** §3.1: a proposal that needs one is not an
  overlay.
- **Specifying the four overlays' own schemas.** Each is its consumer's, and
  writing four schemas nobody has asked for is the opposite of "specify now,
  implement for a named consumer need".

## 5. Open design questions

1. **Should the engine resolve unit references inside an overlay?** It would
   catch the dead path in §3.6's example and it would require the engine to
   know an overlay's shape, which §3.2 forbids. A middle route, a declared
   "this key's top-level keys are unit spellings" hint, is the smallest thing
   that could work and is not designed here.
2. **Is a per-unit overlay key a better seam?** Attaching `extraFrontmatter`
   to a resolved unit rather than to a spec would answer §3.6 properly. It is
   an engine change, so by §3.1 it is a different spec.

## Acceptance when built

No `verify:cli` block; nothing here is implemented, and most of this spec
describes a seam that already exists.

The build, if any, MUST establish behaviorally:

- four distinct overlay keys declared in one corpus are each preserved
  verbatim and independently into `extraFrontmatter`, with no key lost and no
  value merged;
- an overlay whose value is deeply nested round-trips byte-identically;
- editing an overlay value stales every shard, which is the content-hash
  obligation of §3.2.3, asserted by comparing a shard hash before and after;
- no verb's verdict changes on the same tree with and without the four
  overlays present: `compile`, `index check`, `lint --fail-on-warn`,
  `couple` and `index coverage --fail-on-untraced` all reach the same answers;
- an undeclared overlay key still warns, so declaring is what silences it and
  not the mere presence of a plausible name.
