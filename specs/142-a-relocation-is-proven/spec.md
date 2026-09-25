---
id: "142-a-relocation-is-proven"
title: "A relocation is proven"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Splitting an approved spec had no honest path. A partial `supersedes` added the
  new spec as an owner of a unit and left the old one owning it too, so an edit
  to either spec cleared a change to the code. Moving a section of text out of
  an approved spec read as a requirement change, whatever the text said, because
  a section's digest is keyed by its spec's path. `amends_sections` accepted any
  string, and nothing said whether `amends` grants code ownership. This spec
  makes a partial `supersedes` an exclusive hand-off of its unit; adds a
  `relocates` declaration that `delta` proves against the merge base, classing
  a proven move `relocation` instead of `requirement`; validates
  `amends_sections` against the amended specs' headings (L-017); warns when two
  live specs establish one unit (L-018); and states the rule that `amends`
  changes what a spec requires and never who owns its code.
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "018-structured-partial-supersedes"
  - "071-a-change-is-classified-under-the-bases-rules"
  - "106-obligations-are-declared-constraints"
  - "111-a-move-is-a-reviewed-mapping"
amends:
  # 3.1: 018 §4.3 made partial transfer additive and deferred the exclusive
  # hand-off "until demand"; the demand is Statecraft's split of its spec 002.
  - "018-structured-partial-supersedes"
  # 3.3: 071's class table gains `relocation`.
  - "071-a-change-is-classified-under-the-bases-rules"
extends:
  # 3.1 the hand-off
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.2 the key, beside `moves`
  - { spec: "111-a-move-is-a-reviewed-mapping", unit: "crates/spec-spine-types/src/frontmatter.rs", nature: additive }
  # 3.2 the record member
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/registry.rs", nature: additive }
  # 3.2, 3.3 exports
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # 3.2, 3.3 registry 1.9.0, delta 0.2.0
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.2, 3.3 the pinned versions and class tokens
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
  # 3.2 the member in the shard schema
  - { spec: "106-obligations-are-declared-constraints", unit: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json", nature: additive }
  # 3.2 the member in the aggregate schema
  - { spec: "106-obligations-are-declared-constraints", unit: "crates/spec-spine-types/schemas/registry.schema.json", nature: additive }
  # 3.2 V-042, V-043 and the normalized source id
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  # 3.3 the relocation module
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.3 the proof and the class
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/src/delta.rs", nature: additive }
  # 3.3 the class and the report member
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-types/src/delta.rs", nature: additive }
  # 3.3 section spans, public
  - { spec: "020-keypath-section-anchors", unit: "crates/spec-spine-core/src/sections.rs", nature: additive }
  # 3.4, 3.5 L-017, L-018
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  # 3.6 the footer states the rule
  - { spec: "045-couple-names-the-crossing", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.2 the restamped specVersion
  - { spec: "109-impact-and-conflict-are-declared", unit: "crates/spec-spine-core/tests/impacts.rs", nature: additive }
  # 3.2 the restamped specVersion
  - { spec: "106-obligations-are-declared-constraints", unit: "crates/spec-spine-core/tests/obligations.rs", nature: additive }
  # 3.2 the record literal gains the member
  - { spec: "111-a-move-is-a-reviewed-mapping", unit: "crates/spec-spine-core/tests/moves.rs", nature: additive }
  # 3.2 the verifier fixtures, regenerated: their attestation hashes cover the registry, whose specVersion moved.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: additive }
  # 3.2, 3.3 the template documents the key, and the schema history records both bumps.
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: "standards/spec/templates/spec-template.md", nature: additive }
  - { spec: "055-a-version-pin-the-cli-can-check", unit: "docs/schema-versioning.md", nature: additive }
establishes:
  - { kind: file, path: "crates/spec-spine-core/src/relocation.rs" }
  - { kind: file, path: "crates/spec-spine-types/src/relocation.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/relocation.rs" }
references:
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
intent:
  goal: "a consumer can split an approved spec into several, moving text and code claims, and have the tools prove the move changed nothing it requires"
  non_goals:
    - "inferring a relocation: a move not declared is not one, and similarity is never evidence (3.3)"
    - "moving code: a code path's move is spec 111's `moves`"
---

# 142: A relocation is proven

## 1. Purpose

Statecraft needs to split its approved spec 002 into several specs. Measured on
0.25.0 in a scratch corpus shaped like it (`002-env` owning `x.rs` and `y.rs`,
with sections "1. X" and "2. Y"; a new `008-y` taking `y.rs` and section 2),
every route failed somewhere:

| Route | What happened |
|---|---|
| `008-y` establishes `y.rs`, 002 untouched | both own it, and an edit to `y.rs` clears with only 002's `spec.md` edited |
| 002 drops the claim and the section in the same change | works, but it is an edit to an approved spec's frontmatter and body with no way to show it moved rather than changed |
| `008-y` supersedes 002 `partial` on `y.rs` | 008 gains `y.rs` and 002 keeps it: the transfer is additive by 018 §4.3 |
| `008-y` amends 002, then edits `y.rs` | `C-001`: `amends` confers nothing over the code, which nothing documents |
| the section copied into `008-y` | its `sectionDigests` entry differs from 002's, because the digest is keyed `<specPath>#<anchor>` (spec 106): the same text in two specs is two identities |
| `amends_sections: ["2", "7.4"]` | accepted, although 002 has no section 7.4 |

## 2. Territory

- `crates/spec-spine-core/src/index.rs`: the hand-off (3.1).
- `crates/spec-spine-types/src/relocation.rs`, `frontmatter.rs`, `registry.rs`,
  the registry schemas: the declaration (3.2).
- `crates/spec-spine-core/src/compile.rs`: `V-042`, `V-043` (3.2).
- `crates/spec-spine-core/src/relocation.rs`, `delta.rs`, and
  `crates/spec-spine-types/src/delta.rs`: the proof and the class (3.3).
- `crates/spec-spine-core/src/lint.rs`: `L-017`, `L-018` (3.4, 3.5).
- `crates/spec-spine-cli/src/cmd_couple.rs`: the rule in the `C-001` footer
  (3.6).
- The authoring template, `docs/cli-reference.md`, `docs/schema-versioning.md`.

## 3. Behavior

### 3.1 A partial supersedes hands its unit over (amends 018 §4.3)

When a live spec (any status but `superseded` or `retired`) declares
`supersedes: { spec: P, scope: partial, unit: U }`, the index MUST stop treating
P's own claim on U as ownership. The claim stays in P's mapping as a resolved
unit with `ownership: false`, so its history is visible, and it seeds no
implementing path. The coupling gate, `index owner` and coverage therefore name
the successor alone as U's owner, and an edit to U clears only through the
successor's `spec.md` (or an `extends` edge naming it).

A predecessor written as a short id matches by ordinal (spec 015). A superseded
or retired successor hands nothing over.

### 3.2 A relocation is declared

A spec that takes over a section of another spec's text MAY declare it:

```yaml
relocates:
  - { spec: "002-env", from: "2-y", to: "1-y" }
```

- `spec` is the source; a short id resolves, and compile records the full id.
- `from` is the section's anchor in the source as it was at the merge base.
- `to` is the receiving section's anchor in this spec, `from` when omitted.
- `V-042` (error): `from` is empty, or `to` is not exactly one heading in this
  spec's body.
- `V-043` (error, corpus-wide): `spec` resolves to no spec, or to this spec.
- The registry record carries `relocates` (registry schema `1.9.0`, additive).

### 3.3 `delta` proves it against the merge base (amends 071)

For every declared relocation whose source or receiving `spec.md` the change
touches, `delta` MUST compare the source section at the merge base with the
receiving section at head. The comparison is a relocation digest over the
section's text without its own heading line, ignoring the leading section
numbers of every heading inside it, heading levels inside it, trailing
whitespace and runs of blank lines. Every other byte counts.

- The report lists each such relocation under `relocations`, with `proven` and,
  when it is false, a `reason`.
- A source `spec.md` whose body outside `## Verification` changed is classed
  `relocation`, and not `requirement`, exactly when its head body is its base
  body with the proven-relocated sections (each with its subsections) removed,
  compared with heading numbers stripped and blank-line runs collapsed.
  Otherwise it is `requirement`, as before.
- `relocation` does not require prior policy: a proven move changes nothing any
  spec requires. Every other class the file carries (an edge removed is still
  `authority`) is reported as before.
- The delta report schema is `0.2.0` (additive: a class and a list).

Nothing is inferred: without a `relocates` entry, text that reappears
elsewhere is not a relocation.

### 3.4 `amends_sections` names real sections (L-017)

Each `amends_sections` entry MUST be the section number (`3.1` for a heading
`3.1 The rule`, `4` for `4. Out of scope`) or the anchor of a heading in at
least one spec the declaring spec amends. Otherwise `lint` warns `L-017`, which
`lint --fail-on-warn` refuses. The list stays one list shared by every target
(spec 132's note).

### 3.5 One live establisher per unit (L-018)

When two or more live specs `establishes` the same unit, `lint` MUST warn
`L-018` on each, unless a partial `supersedes` naming the unit joins them (3.1
has then made one the owner). This repository has no such pair.

### 3.6 `amends` does not grant code ownership

An `amends` edge changes what the amended spec requires. It MUST NOT make the
amending spec an owner of the amended spec's code; the gate's amends-awareness
applies only to the amended spec's own `spec.md` (spec 005, unchanged). The
`C-001` footer's second door says so beside `extends`, which is the edge that
does confer ownership.

## 4. Out of scope

**Frontmatter edits to the source.** Removing an edge from an approved spec is
still an `authority` change, classed as before. The hand-off in 3.1 exists so a
split needs no such edit.

**Relocation between repositories.** Both specs are in one corpus.

## 5. Resolved decisions

**D-1 (2026-09-25): a declared relocation, not a move-stable registry digest.**
A path-independent digest in the registry would let a reader match text, but
matching is inference: two sections can be equal by accident. The declaration
says which section went where, and the digest only checks it. The digest is
computed where it is used, in `delta`, which already reads both trees.

**D-2 (2026-09-25): numbers and levels do not count, words do.** A moved section
is renumbered and may sit one level deeper; those are properties of where it
sits. Everything else, blank-line runs aside, must be byte-identical, so an
edit made on the way is not a relocation.

**D-3 (2026-09-25): the hand-off is in the index, not the gate.** Stripping
the predecessor's ownership where claims are resolved makes the gate,
`index owner`, coverage and `delta`'s owner sets agree without a second rule.
The predecessor's index shard changes when a successor appears, which spec
069's byte comparison already accounts for.

**D-4 (2026-09-25): `relocation` is a new class, not a flag on
`requirement`.** A consumer deciding whether a change needs the owner's
approval reads the class set and `priorPolicy`. A flag on `requirement` would
leave every existing consumer requiring approval for a proven move, which is
the cost this spec removes.

## Verification

```verify:cli
# 3.1 to 3.5: hand-off, retired successor, proven and unproven relocations, an
# extra edit, an undeclared move, the grammar codes, L-017 and L-018. Each test
# fails under a mutation of the rule it checks (recorded in the PR).
sh -c 'cargo test -p spec-spine-core --locked --test relocation 2>&1 | grep -q "test result: ok. 9 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --lib -- --exact relocation::tests::numbers_levels_and_blank_runs_do_not_matter_and_words_do relocation::tests::a_relocation_only_body_is_the_base_less_the_sections relocation::tests::a_fenced_hash_line_is_not_a_heading 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.2, 3.3: the schema versions and the class token set.
sh -c 'cargo test -p spec-spine-types --locked --test dtos -- --exact schema_versions_are_pinned delta_class_tokens_and_prior_policy_set_are_pinned 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 3.2: the template documents the key.
sh -c 'cargo test -p spec-spine-types --locked --test dogfood -- --exact authoring_template_documents_every_frontmatter_key 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 3.6: the footer states the rule.
grep -q 'never who owns its code (spec 142)' crates/spec-spine-cli/src/cmd_couple.rs
# 3.4, 3.5: this corpus's seven amends_sections lists all resolve, and no unit has two live establishers.
cargo build --release --locked
target/release/spec-spine lint --fail-on-warn
```
