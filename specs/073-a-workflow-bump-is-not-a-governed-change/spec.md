---
id: "073-a-workflow-bump-is-not-a-governed-change"
title: "A workflow bump is not a governed change"
status: draft
kind: "tooling"
created: "2026-09-08"
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
  - "024-index-sharding"
  - "030-cargo-workflow-dependency-waiver"
  - "069-the-shipped-default-hashes-what-it-names"
amends:
  # 004 3.5 defines the content hash over raw file bytes for every
  # `extra_hashed_inputs` match, with projections named only for npm and cargo
  # manifests. 3.1 below adds a third, which changes what enters the hash.
  - "004-codebase-index"
  # 069 3.6 requires the corrected test to assert that a workflow `uses:` bump
  # STALES the index, a re-index restores it, and only then does couple waive.
  # 3.4 below requires the bump to leave the index fresh, which contradicts it.
  - "069-the-shipped-default-hashes-what-it-names"
extends:
  # 3.1 the projection itself, beside the npm and cargo projections 030 added.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/manifest.rs", nature: additive }
  # 3.1 the fold site: the global-inputs scalar every shard hash carries.
  - { spec: "024-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: additive }
  # 3.2 the waiver classifier the projection must agree with.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/dep_only.rs", nature: additive }
  # 3.4 030's workflow auto-waive test, whose premise 069 corrected and this
  # spec restores on a different footing.
  - { spec: "030-cargo-workflow-dependency-waiver", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
  # 3.5 the freshness guard.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
summary: >
  A GitHub Actions workflow enters the index content hash as raw bytes, so a
  Dependabot `uses:` bump moves the global-inputs scalar and stales every
  shard in the repository. The bot can neither re-index nor add a waiver to
  its own PR body, so the PR walls on exit 2 and needs a maintainer to push a
  re-index commit. Spec 030 built the coupling half of this for workflows and
  read the freshness half as already fine; it was not, and spec 069 made that
  visible by fixing the default glob, then deferred the real fix as a hashing
  change with a migration of its own. This is that spec. A workflow folds as
  its governance projection, the parsed document with the pinned ref of each
  `uses:` reference removed and the action path kept, so a version or SHA bump
  is invisible to the ledger while a changed action, a `run:` or `with:` edit,
  an added step or an unpin all still stale it. The projection and 030's
  waiver classifier are required to state one rule, and the upgrade restales
  every workflow-bearing repository once.
---

# 073: A workflow bump is not a governed change

## 1. Purpose

Stop a routine action bump from invalidating the entire ledger.

Every shard hash folds a **global-inputs scalar** covering `spec-spine.toml`
and every `[index] extra_hashed_inputs` match (`shard.rs::global_inputs_hash`).
Those matches enter as raw file bytes. `.github/workflows/**/*` is half the
shipped default and is in this repository's own config, so a one-character
change to a pinned action ref changes the scalar, and the scalar is in every
shard, so **every shard in the repository goes stale at once**.

The change that triggers it is the one nobody reviews: Dependabot bumping
`actions/checkout@v4` to `@v5`. That PR cannot repair itself. The bot has no
toolchain to run `spec-spine index`, no write path to commit the regenerated
shards, and no way to put a `Spec-Drift-Waiver:` line in a body it does not
author. So `index check` exits 2 and the PR waits for a human to push a commit
whose entire content is a mechanical re-hash.

Spec 030 built exactly this mechanism for the coupling half: a workflow whose
only change is the `@ref` of `uses:` references clears the gate mechanically.
Its section 3.6 also asserted that such a bump "stays index-fresh (file unit,
no span)", which was true only because the shipped default glob matched no
files. Spec 069 fixed the glob, made the staleness real for every adopter, and
recorded in its own section 3.6 that the test asserting otherwise had to be
corrected rather than preserved. Its section 4 named the projection as the real
fix and deferred it here, on the grounds that a hashing-semantics change
carries a migration and should not ride along inside a one-line default
correction.

The result today is a mechanism that is half built: the gate forgives the bump
and the ledger does not.

## 2. Territory

No new files. Five units, all `extends`-ed:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/manifest.rs` | 004 | the projection, beside the npm and cargo ones |
| `crates/spec-spine-core/src/shard.rs` | 024 | the fold site applies it |
| `crates/spec-spine-core/src/dep_only.rs` | 005 | the shared predicate and ref semantics |
| `crates/spec-spine-cli/tests/couple.rs` | 030 | the workflow auto-waive test's premise |
| `crates/spec-spine-core/tests/index.rs` | 004 | the freshness guard |

**This spec `amends` 004.** Section 3.5 of spec 004 defines the content hash
over the normalized bytes of every hashed input, and names a governance
projection for npm and (via 030) cargo manifests only. Section 3.1 below adds a
third class, which changes what enters the hash for every adopter. That is a
change to what 004 requires, and 030 set the precedent for declaring it: it
amended 004 for exactly this reason when it added the cargo projection.

**This spec `amends` 069.** Section 3.6 of spec 069 requires the corrected test
to assert "the real sequence: the bump stales the index (exit 2), a re-index
restores it, and `couple` then auto-waives." Section 3.4 below requires the
bump to leave the index fresh, so that sequence stops being the specified
behavior. Per spec 040 both edges are declared here only; neither amended
spec's `spec.md` is edited to mention this one.

## 3. Behavior

### 3.1 A workflow folds as its governance projection (amends 004 3.5)

A hashed input whose path satisfies `dep_only::is_workflow_yaml` (a `.yml` or
`.yaml` file at any depth under `.github/workflows/`) MUST fold into the
global-inputs scalar as its **governance projection** rather than as raw bytes:
the parsed YAML document in which the pinned ref of every `uses:` scalar has
been removed, rendered through a canonical, sorted-key serializer so the result
is byte-identical across the release matrix.

The rule for a single `uses:` value:

- `owner/action@<ref>` projects to `owner/action`, at any nesting depth and
  including an action subpath (`owner/repo/path@ref`). The **action path MUST
  be preserved**: swapping which action runs is a governed change and MUST
  still stale the ledger. Only the ref is dropped.
- A `uses:` with no `@` (an unpinned action, or a local `./path` reference)
  MUST be preserved verbatim. Unpinning is a change to the security posture,
  not a version bump.
- A non-string `uses:` value MUST be preserved verbatim.

Everything else in the document MUST be preserved: `run:`, `with:`, `env:`,
`if:`, job and step structure, names, triggers, permissions, and any key the
projection does not recognize. The projection removes one thing, and a key it
has never heard of survives it unchanged.

An unparseable or non-mapping document MUST fall back to its raw bytes.
Over-hashing is the fail-closed direction, exactly as the npm and cargo
projections already do: a file the projection cannot read stales the ledger on
any edit rather than silently on none.

The projection applies wherever the file is folded as a hashed input. It does
not change which files are hashed, which is spec 069's question and is settled.

### 3.2 The projection and the waiver state one rule

Spec 030 already decides what a dependency-only workflow change is, in
`dep_only::workflow_dependency_only_change`, by comparing two parsed documents
while permitting each `uses:` ref to differ. Section 3.1 answers the same
question by projecting one document. The two MUST agree:

- a change `workflow_dependency_only_change` waives MUST leave the projection,
  and therefore the content hash, unchanged;
- a change that alters the projection MUST refuse the waiver.

This MUST be asserted by test over a shared matrix rather than assumed from the
two implementations looking similar. The failure this prevents is the one the
gate is worst at surfacing: a bump that self-clears the coupling gate while
still staling the ledger is precisely the wall this spec exists to remove, and
it would reappear the moment the two rules drift apart.

The two MAY continue to be separate implementations. Deriving one from the
other is section 4's question.

### 3.3 The upgrade restales every workflow-bearing repository once

Adopting this change recomputes the global-inputs scalar wherever a workflow is
a hashed input, so every shard's hash moves once. The behavior MUST match the
npm and cargo precedents: re-run `spec-spine index` and commit the regenerated
shards when upgrading.

No schema version changes. The shape of the emitted JSON is untouched, and only
a hash value moves, so `INDEX_SCHEMA_VERSION` MUST NOT be bumped. The migration
is a re-index, not a loader change.

**The implementing change MUST be sequenced against every other branch in
flight.** Because the restale moves every shard, a branch that regenerated its
shards before this one merged carries values computed under the previous rule.
Merging it afterwards leaves `index check` on the default branch stale, and it
does so **without a textual conflict**: the two branches touch different shard
files, so nothing collides and the merge driver, which regenerates only on a
same-shard conflict, never fires. The failure is silent at merge time and
surfaces as a red gate on the next commit.

Two orders are safe: land this change when no other implementation branch is
outstanding, or land it first and require every outstanding branch to rebase
and re-run `spec-spine index` before merging. The spec does not choose between
them, because which is cheaper depends on what is in flight; it requires that
one of them be chosen deliberately rather than discovered.

This constraint belongs to the implementation, not to the draft. Filing this
spec adds one shard and changes no existing one, so the draft merges in any
order.

The migration note MUST state the consequence for spec 023: an attestation
sealed before this change verifies its signature but no longer verifies by
recompute, because the ledger it attests to is hashed under the previous rule.
That is inherent to any change in hashing semantics, it applied equally to the
npm and cargo projections, and the fix is to re-attest after re-indexing. It is
stated because an attestation that fails recompute looks like tampering, and a
maintainer meeting that for the first time deserves to find it written down.

### 3.4 The workflow auto-waive test asserts freshness again (amends 069 3.6)

`crates/spec-spine-cli/tests/couple.rs` currently asserts, per 069 3.6, that a
`uses:` bump stales the index, that a re-index restores it, and that `couple`
then auto-waives. The corrected sequence MUST be: the bump leaves the index
**fresh**, and `couple` auto-waives with no re-index in between. The test MUST
keep its actual subject, which is that the bump self-clears the coupling gate.

A companion case MUST assert the other direction on the same fixture: a `run:`
edit in the same workflow stales the index and refuses the waiver. Without it
the suite proves only that the projection is permissive, not that it is
correct, which is the shape of assertion 069 3.6 was written to end.

### 3.5 Tests (minimum)

- A `uses:` tag bump (`@v4` to `@v5`) and a SHA-pin bump: hash unchanged.
- A changed action path with the same ref (`actions/checkout` to
  `other/checkout`): hash changes.
- An action unpinned (`@v4` removed): hash changes.
- A `run:`, `with:`, `env:` or `if:` edit, an added step, an added job, a
  changed trigger: each changes the hash.
- A comment-only or reformat-only edit: hash unchanged where the parsed
  document is unchanged, which is the projection's defined behavior and MUST be
  asserted rather than left to chance.
- An unparseable workflow: falls back to raw bytes, so any edit changes the
  hash.
- The 3.2 agreement matrix, both directions.
- End to end, on this repository's own corpus: a `uses:` bump leaves
  `spec-spine index check` at exit 0.

## 4. Out of scope

**Deriving the waiver from the projection.** Two documents are dependency-only
equal exactly when their projections are equal, so `workflow_dependency_only_change`
could be reimplemented as a projection comparison and the duplication in 3.2
would go away by construction. It is attractive and it is not this change: it
rewrites a classifier on spec 005's coupling path, where a subtle behavior
change means a gate that waives something it should refuse. The consistency
requirement in 3.2 buys the same guarantee under test. If it is done, it is
done deliberately, in its own spec, with the waiver's matrix as the evidence.

**A configurable projection.** A general `[index] projections` surface letting
an adopter declare which keys of which YAML files are ungoverned is the
generalisation of this spec and of the npm and cargo projections before it.
It needs a configuration design, a validation story for a projection an adopter
writes wrong, and an answer for what happens when a projection hides a change
the coupling gate refuses. None of that belongs inside a fix for the one
ecosystem that is currently walling PRs.

**The pip, docker and go ecosystems.** Spec 030 section 4 deferred these
because no triggering bump existed, and that is still true: this repository's
Dependabot configuration covers github-actions, cargo, npm and pip, and the pip
entry governs `py/` whose manifest is not a hashed input.

**Which files are hashed.** Spec 069 settled the default glob and this spec
does not revisit it. A repository that does not hash its workflows is
unaffected by this change in either direction.

**Whether a bare `file` unit should contribute its bytes.** Spec 057 section 4
left that open and it stays open. It is a different question about a different
class of input, and answering it here would change staleness for 74 claimed
paths in this repository alone.

## 5. Resolved decisions

**2026-09-08: `risk: high`, above the `medium` its neighbours carry.** Spec 069
changed which files are hashed and took `medium`; this changes how a file is
hashed, for every adopter, with a migration that silently invalidates
recompute-verification of any existing attestation. The blast radius of a wrong
projection is also worse than a wrong glob: a glob that matches too little
under-hashes visibly, while a projection that strips too much under-hashes
invisibly and forever. The rating is the reason 3.1 enumerates what is
preserved rather than what is removed.

**2026-09-08: only the ref is stripped, never the action path.** The tempting
simplification is to drop the whole `uses:` value, which makes the projection
one line. It also makes swapping `actions/checkout` for an attacker's fork
invisible to the ledger. Spec 030's waiver already draws the line in the same
place, refusing a changed action path and an unpin while permitting a ref
change, so preserving the path keeps 3.2's agreement achievable and keeps the
security property the waiver already relies on.

**2026-09-08: the fallback is raw bytes, not exclusion.** A workflow the parser
rejects could be dropped from the hash, which would be the permissive reading
and would make a syntactically broken workflow invisible to staleness. Falling
back to raw bytes means such a file stales on every edit, which is noisier and
correct. This matches npm and cargo, and the consistency is worth more than the
noise.

**2026-09-08: `amends` on both 004 and 069, following 030's precedent.** Spec
030 amended 004 when it added the cargo projection, for the same reason this
spec does: the content-hash definition names its projections, and adding one
changes it. The 069 edge is the less obvious of the two and is the reason the
frontmatter carries a comment: 069 3.6 does not merely observe that a bump
stales the index, it requires the test to assert it, so making the bump
harmless contradicts a requirement rather than correcting an implementation.

## Verification

Each line below is one command: spec 049 3.2 makes a fence's body line a
command, so no line may depend on a variable another line set.

Every assertion fails against pre-073 code. The end-to-end case is the one that
matters: on this repository's own corpus a `uses:` bump stales all 73 shards
today, which is the wall a Dependabot PR meets.

```verify:cli
cargo build --release --locked
# 3.1 the projection exists and is applied at the fold site.
grep -qF 'workflow_hash_projection' crates/spec-spine-core/src/manifest.rs
grep -qF 'workflow_hash_projection' crates/spec-spine-core/src/shard.rs
# 3.5 the projection's own matrix, including the preserved action path.
cargo test -p spec-spine-core --test index --locked
# 3.2 the projection and 030's waiver classifier agree.
cargo test -p spec-spine-core dep_only --locked
# 3.4 the corrected auto-waive sequence, with no re-index in between.
cargo test -p spec-spine-cli --test couple --locked
# 3.3 the emitted shape is untouched, so the schema conformance still holds.
cargo test -p spec-spine-core --test conformance --locked
```
