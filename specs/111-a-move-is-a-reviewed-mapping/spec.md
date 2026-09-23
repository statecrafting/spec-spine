---
id: "111-a-move-is-a-reviewed-mapping"
title: "A move is a reviewed mapping"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "100-a-deleted-path-is-judged-where-it-lived"
summary: >
  The gate passes `--no-renames`, so a move arrives as a deletion and an
  addition and each half is judged where it is true. That is correct and it
  is not the whole story: nothing records that the two halves are one act.
  The spec performing a move declares it in its own frontmatter as authored,
  reviewed data. The compiler validates the declaration's shape, the lint
  checks its paths against the tree, and a lookup derived from every
  declaration follows declared chains, reporting ambiguity and cycles instead
  of guessing through them. A move is informational: it transfers no
  ownership, authorizes no deletion, infers no similarity and changes no
  verdict.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/moves.rs" }
  - { kind: file, path: "crates/spec-spine-core/src/moves.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/moves.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/moves.rs" }
extends:
  # 3.1: the frontmatter key and the registry member.
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry.schema.json" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  # 3.2: shape validation at compile; 3.3: path checks in the lint.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/tests/conformance.rs" }, nature: additive }
  - { spec: "003-conformance-lint", unit: { kind: file, path: "crates/spec-spine-core/src/lint.rs" }, nature: additive }
  # 3.4: the lookup, through the facade and one CLI verb.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_registry.rs" }, nature: additive }
  # Registry/read MINOR pins this build moves, in territory other specs
  # established or already extend (109 D-7's precedent for moving a pin in a
  # sibling spec's territory): the shard-version and schema-version-axis
  # assertions these files pin move with every additive MINOR, this one
  # included.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "106-obligations-are-declared-constraints", unit: { kind: file, path: "crates/spec-spine-core/tests/obligations.rs" }, nature: additive }
  - { spec: "109-impact-and-conflict-are-declared", unit: { kind: file, path: "crates/spec-spine-core/tests/impacts.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-core/tests/scope.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-cli/tests/scope.rs" }, nature: additive }
  # Spec 088 §3.2: a key the parser accepts and the template omits is a key no
  # author can find; a dogfood test enforces it.
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
  # The registry MINOR moves the fixture set's attested registryHash, exactly
  # as 109 and 110 each recorded (109 D-7, D-4): regenerated with 103 §3.9's
  # documented, idempotent generator, through 103's own authority path.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: corrective }
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "docs/corpus-map.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "A move is declared only in the frontmatter of the spec performing it; its shape is validated at compile, and a dangling answered_by is a compile warning, as every other informational cross-reference is."
    anchor: "3-2-the-shape-is-validated-at-compile"
  - id: "R-2"
    kind: requirement
    text: "The lookup is derived from every declaration, follows declared chains, and reports ambiguity and cycles by name instead of choosing a path through them."
    anchor: "3-4-the-lookup-is-derived-and-never-guesses"
  - id: "I-1"
    kind: invariant
    text: "No verdict consults a move: couple reaches the same decision on the same diff with and without the declaration, including the refusal of an unauthorized deletion."
    anchor: "3-5-a-move-changes-no-verdict"
  - id: "I-2"
    kind: invariant
    text: "Nothing infers a move: no similarity, rename detection or content comparison, and --no-renames stays."
    anchor: "3-6-no-similarity-ever"
  - id: "V-1"
    kind: verification
    text: "Shape validation, path lints, chain, split, ambiguity and cycle lookups, and the coupling refusal with and without a mapping on a git fixture whose similarity index would pair the halves."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/moves.rs"
      - "crates/spec-spine-cli/tests/moves.rs"
---

# 111: A move is a reviewed mapping

## 1. Purpose

Design note 04 records A07: git similarity is a suggestion, never authority.
Spec 071 takes that position and spec 100 §3.6 keeps it, judging a rename's
delete half at the snapshot it lived in and its add half at head, with no
pairing by any heuristic.

The result is right and it loses something. After a move, nothing in the
corpus says the old path and the new path are the same thing. A reader asking
"where did this file go" gets an answer from git's similarity index, which is
a guess, or from a human's memory.

### 1.1 Why the dependency is spec 100

Spec 100 §3.6 is what makes a mapping additive rather than a replacement: it
fixes that the two halves are judged independently and forbids heuristic
pairing, so a declared mapping is a **record on top of** two correct verdicts
rather than a way to change either. It does not depend on obligations, scopes
or closures.

## 2. Territory

A new types module for the declaration (`crates/spec-spine-types/src/moves.rs`)
and a new core module for the lookup (`crates/spec-spine-core/src/moves.rs`);
additive edits to the frontmatter and registry DTOs, the types export list,
shape validation in `compile.rs`, path checks in `lint.rs`, the facade in
`lib.rs` and the `registry` CLI; two new acceptance files. The registry and
read schemas, `version.rs`, the conformance pins, the verifier fixtures and
the reference documentation move with their MINORs under the specs that own
them, as `extends` edges the build declares.

## 3. Behavior

### 3.1 A move is declared by the spec that performs it

```yaml
moves:
  - from: "kit/skills/build/SKILL.md"
    to: ".claude/skills/build/SKILL.md"
    kind: relocated
  - from: "crates/spec-spine-core/src/kit_embedded.rs"
    to: null
    kind: removed
    answered_by: "092-the-engine-ships-governance-not-an-environment"
```

- **`from`**: the path as it was. It need not exist at head.
- **`to`**: the path as it is, or `null` for a removal.
- **`kind`**: `relocated` (one path to one path), `split` (one `from`, `to` a
  list of two or more), `merged` (`from` a list of two or more, one `to`), or
  `removed` (`to` is `null`).
- **`answered_by`** (optional, `removed` only): the spec that answers for the
  path's former responsibility now.

The declaration lives with the moving spec, which is the spec that knows and
the one a reviewer is reading when the move happens. There is no second,
authored, corpus-wide path map; the corpus-wide view is derived (§3.4).
Paths are repo-relative POSIX, with no `..` segment and no leading `/`.

The compiler records the declarations on the spec's registry record as
`moves`, omitted when there are none, so a spec without the key has an
unchanged record. This is a registry schema MINOR.

### 3.2 The shape is validated at compile

Validation errors on the declaring spec:

- a `kind` other than the four, or a `from`/`to` whose arity does not match
  its `kind` (§3.1);
- a path that is empty, absolute, or has a `..` segment;
- the same path on both sides of one entry;
- `answered_by` on a kind other than `removed`.

A compile **warning** on the declaring spec:

- an `answered_by` that names no spec in the corpus.

That settles an inconsistency in the reserved draft, which listed this at
warning tier in its §3.5 and as a validation error in its acceptance. The
corpus's precedent decides it. A dangling reference to another spec is a
warning wherever the reference is informational: `depends_on` (`V-010`) and
every edge target (`L-004`). It is an error only where the reference moves
authority: `superseded_by` (`V-008`), because a superseded spec's authority
has to land somewhere. A move moves no authority (§3.5), so `answered_by` is
informational and takes the informational tier. It is judged from the corpus
alone, so it belongs in `compile`; compile warnings reach this repository's
gate through `check --fail-on-warn` (spec 064), so a dangling reference still
cannot merge here.

### 3.3 The paths are checked against the tree, as warnings

What a declaration says about the working tree is checked by `lint`, which is
where the corpus already compares claims with the tree (for example
`L-012`). At warning tier, so `lint --fail-on-warn` refuses them:

- a `relocated`, `split` or `merged` entry whose `to` path does not exist,
  naming the declaring spec and both paths;
- a `removed` entry whose `from` path still exists.

These are warnings rather than errors because they describe the tree at a
point in time, not the declaration's own well-formedness. They MUST NOT
attempt to verify that content moved (§3.6).

### 3.4 The lookup is derived and never guesses

The lookup takes a path and answers from the declarations in the committed
registry, across every spec:

- **unmapped**: no declaration has this path as a `from`.
- **resolved**: every declared step was followed to its end. A `relocated` or
  `merged` step continues from its `to`; a `split` continues from each `to`
  (a declared branch, not a guess); a `removed` step ends with its
  `answered_by`, if any. The answer lists every hop with the spec that
  declared it, and every terminal path.
- **ambiguous**: at some step, more than one declaration has the same `from`
  with different outcomes. The lookup stops there and names every candidate
  and its declaring spec. It MUST NOT pick one, by ordinal, date or any other
  rule: a path reused after a move is a real history that the declarations do
  not order.
- **cycle**: following declared steps returns to a path already visited. The
  lookup stops and names the cycle.

`unmapped` and `resolved` exit 0; `ambiguous` and `cycle` exit 1, because the
lookup refused to give an answer. The declarations themselves are not wrong
when they are ambiguous or cyclic (a file can move away and back), so neither
is a compile error or a lint warning.

Surfaces: `spec-spine registry moves [<path>] [--json]` and the facade's
`query_json` with `op: "moves"`. Without a path it lists every declaration,
flattened and sorted, which is the derived path map. It is an inspection read
over the committed registry, like `registry show`, and adds a read document
at a read schema MINOR.

`index owner` is not changed. Its answer is the coupling gate's own owner
derivation (spec 048) and must stay exactly that, which is §3.5 from the
other side.

### 3.5 A move changes no verdict

`couple` MUST NOT consult a declaration. The delete half still needs the spec
that owned the old path to author the removal, and the add half still needs
the spec that owns the new path to claim it (spec 100 §3.6). A declared move
clears neither, transfers no ownership and authorizes no deletion.

If a mapping could clear a deletion, writing one would be a way to move a
governed file out from under its owner, with only review discipline in the
way.

### 3.6 No similarity, ever

This spec MUST NOT introduce rename detection, similarity scoring, content
comparison between `from` and `to`, or any inference that pairs a deletion
with an addition. It MUST NOT cause `--no-renames` to be dropped. A mapping
is true because an author wrote it and a reviewer read it.

## 4. Out of scope

- **Rename detection.** §3.6.
- **Changing any verdict, or `index owner`'s answer.** §3.4, §3.5.
- **Retroactive mappings** for moves already performed.
- **Spec-id mapping.** `docs/corpus-map.md` (spec 095) does that for ids.
- **Ordering a reused path's history.** §3.4 reports it as ambiguous.

## 5. Resolved decisions

**D-1 (2026-09-23, finalized before the build: where a mapping lives).** The
draft's open question 1 asked whether a mapping belongs in the moving spec or
in a corpus-wide map. Both: authored in the moving spec, and the corpus-wide
map derived from all of them by the lookup (§3.4). Nothing authors a second
map.

**D-2 (2026-09-23, chains and cycles).** The draft's open question 2 asked
how far a chain is followed. To its declared end, with ambiguity and cycles
reported by name and never resolved by a rule (§3.4).

**D-3 (2026-09-23, `answered_by` tier).** §3.2: a compile warning, the
tier every informational spec reference takes (`V-010`, `L-004`); only a
reference that moves authority (`superseded_by`, `V-008`) is an error. An
earlier finalization of this text claimed dangling references were errors
everywhere, which was wrong for `depends_on` and every edge; it was corrected
in review before the build.

**D-4 (2026-09-23, `reviewed_by` removed).** The draft's example carried a
`reviewed_by` member whose value was a placeholder sentence. The declaring
spec is the record of who performed the move, and its review is the pull
request that adds the declaration; a free-text field restating that would be
a second place to disagree.

**D-5 (2026-09-23, `index owner` is unchanged).** The draft listed
`index owner` reporting the mapping as a use. `index owner` is defined as the
gate's derivation (spec 048), and the lookup is a separate read, so an owner
answer and a move answer can never be confused for each other.

**D-6 (2026-09-23, build: codes).** §3.2's five bullets are covered by two
new codes: `V-040` (error, local to one spec: arity vs `kind`, path grammar,
the same path on both sides, `answered_by` on a kind other than `removed`)
and `V-041` (warning, corpus-wide: a dangling `answered_by`, per D-3's tier).
"A `kind` other than the four" is not a third code: `kind` is a closed, typed
enum (`MoveKind`), the way `ImpactNature` and `ObligationKind` already are in
this corpus, so an unknown value is refused at parse as malformed frontmatter
(`V-002`), consistent with that precedent, and `V-040` covers what a typed
enum cannot: arity and grammar. §3.3's two warnings are `L-015` (a
`relocated`/`split`/`merged` entry whose `to` does not exist) and `L-016` (a
`removed` entry whose `from` still exists), the next free codes after `L-014`.

**D-7 (2026-09-23, build: identical duplicate declarations).** Two specs
declaring the byte-identical step (same `from`, `to` set and `kind`) for the
same `from` collapse to one step in the lookup, naming every declaring spec,
rather than being reported as ambiguous: they are not a disagreement, they are
one fact two authors happened to both write down. Anything less than
byte-identical agreement (a different `to`, `kind` or `answered_by`) is
ambiguous, with every candidate reported and none picked. To carry this,
[`spec_spine_core::moves::Hop`] and `query_json`'s `moves` hop shape carry
`declaredBy` as a list, always, rather than a single id: the common case is a
one-element list, and the duplicate case needs no second shape.

**D-8 (2026-09-23, build: registry/read schema versions).** Spec 114 was
built concurrently on another branch and took registry `1.7.0`; this build's
registry MINOR is `1.8.0`, and its `moves` member is what moved it. The read
axis moves `0.7.0` to `0.8.0` for the move lookup and the flattened move-list
documents. Where this landed a MINOR bump in territory another spec
established or already extends (the schema-version pins in
`crates/spec-spine-types/tests/dtos.rs`, `crates/spec-spine-core/tests/read.rs`,
`tests/obligations.rs`, `tests/impacts.rs`, `crates/spec-spine-core/tests/scope.rs`
and `crates/spec-spine-cli/tests/scope.rs`, and the verifier fixture set's
attested `registryHash`), this spec declares its own `extends` edge on that
file or directory, following exactly the precedent specs 106, 109 and 110 each
recorded for the same situation. The fixture set was regenerated with spec
103 §3.9's documented, idempotent generator.

**D-9 (2026-09-23, build: fail-first).** `crates/spec-spine-core/tests/moves.rs`
and `crates/spec-spine-cli/tests/moves.rs` were written before
`crates/spec-spine-types/src/moves.rs` and `crates/spec-spine-core/src/moves.rs`
existed: the core suite did not compile (no `spec_spine_core::moves` module,
no `moves` field on `Frontmatter`/`SpecRecord`) and the CLI suite could not
compile for the same reason, transitively. Confirmed by running both `cargo
test` invocations against the unbuilt tree before writing `compile.rs`'s,
`lint.rs`'s or `lib.rs`'s changes.

**D-10 (2026-09-23, build: the coupling test is a control, and its mutation
check).** `crates/spec-spine-cli/tests/moves.rs`'s three git-fixture tests
assert `couple` is unchanged by this spec (§3.5), so they pass identically
before this build (when no `moves` key exists at all, so the mapping-declared
variant cannot even be authored) and after it: `couple.rs` itself is not
edited anywhere in this build. To prove the test can actually fail, a
throwaway local edit to `couple.rs` (never committed) made a deleted path
skip its C-001 check when any spec's `moves` named it as a `from`; with that
mutation in place, `a_declared_mapping_does_not_clear_the_unauthorized_deletion`
and `the_same_fixture_without_the_mapping_reaches_the_identical_verdict` both
failed (the couple exit code dropped from 1 to 0). The edit was reverted
before this commit and `couple.rs`'s tree is byte-identical to its
pre-build state.

**D-11 (2026-09-23, review: the walk is iterative and memoized).** Independent
review of the first build found `spec_spine_core::moves::walk` was native
recursion with ancestry-only cycle detection: a long declared chain (on the
order of 10,000-20,000 sequential relocations) could overflow the native call
stack (a process abort, which core's "panic-free on user input" invariant
forbids), and a legal, acyclic, reconverging `split` DAG was walked once per
root-to-leaf path rather than once per node, making the traversal exponential
in depth rather than linear in the declaration count. Fixed by replacing the
recursion with an explicit work stack and the classic DFS three-color
coloring: `on_stack` (gray, the current ancestry, unchanged in meaning) and a
new `done` set (black, a fully-expanded node, safe to reuse without
re-expanding). This is provably traversal-order-independent for cycle
detection (a reachable cycle is found by any complete DFS, regardless of
which branch of a `split` is walked first), so a cycle reachable through only
one branch of a `split` is still reported (added as a regression test). A
consequence worth recording because it changes the documented shape: `hops`
in a `resolved` answer are now, by construction (a node is expanded at most
once), the **distinct** declared steps reached, deduplicated on `(from, to,
kind)` and sorted by `(from, to)`, rather than one copy per root-to-leaf path
that crossed them; every existing test's expectations were already written
against the deduplicated form (none exercised a reconverging DAG), so no test
assertion changed. Two performance regression tests were added: a 5,000-hop
linear chain resolves and completes in well under a second, and a depth-16
two-branch reconverging `split` DAG (`2^16` root-to-leaf paths, ~32
declarations) resolves with a hop count linear in the declarations.

## Verification

Written to fail against the tree it is filed on: nothing named here exists.

```verify:cli
# 3.1 - 3.4: shape validation, path lints, and every lookup outcome.
cargo test -p spec-spine-core --test moves --locked
# 3.4 - 3.6: the shipped verb and its exit codes, and couple's refusal of an
# unauthorized deletion with and without the mapping, on a git fixture whose
# similarity index would pair the two halves.
cargo test -p spec-spine-cli --test moves --locked
cargo build --release --locked -p spec-spine-cli
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
