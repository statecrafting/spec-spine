---
id: "100-a-deleted-path-is-judged-where-it-lived"
title: "A deleted path is judged where it lived"
status: draft
kind: "tooling"
created: "2026-09-21"
implementation: pending
owner: "The spec-spine Authors"
depends_on:
  - "005-coupling-gate"
  - "081-coupling-sees-the-change-being-committed"
summary: >
  The coupling gate resolves a deleted path's owners from the head index, where
  the claim the same change withdrew no longer exists, so a correct removal is
  refused as C-001 against a package floor that has nothing to say about it.
  A path the change deletes is judged against the base snapshot of the diff the
  gate already reads, which is the merge base and not the supplied base ref.
extends:
  - spec: "005-coupling-gate"
    paths:
      - "crates/spec-spine-core/src/couple.rs"
      - "crates/spec-spine-cli/src/cmd_couple.rs"
      - "crates/spec-spine-core/tests/couple.rs"
      - "crates/spec-spine-cli/tests/couple.rs"
    nature: additive
  - spec: "001-compile-registry"
    unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }
    nature: additive
references:
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 100: A deleted path is judged where it lived

## 1. Purpose

A removal that is correct in every way the corpus asks for is refused by the
gate that is supposed to bless it.

The shape is this. A file is owned by spec A. A change deletes the file and
withdraws A's claim in A's own frontmatter, in one commit, which is the only
grammar the corpus has for withdrawing a claim: no edge withdraws a unit, so
the predecessor's frontmatter is edited under a named authority (spec 037,
spec 092 §3.11). The gate then resolves the deleted path's owners from the
**head** index, where A's claim no longer exists, and the answer falls through
to the package manifest floor, spec F. F's `spec.md` is not in the diff, and F
has nothing true to say about a file it never claimed. The gate refuses.

Spec 092 §3.11.1 measured this against its own removal and D-11 records why it
was not fixed there: the fix needs the gate to hold two snapshots, which
`couple_with` does not take, and a realignment is not the place to open that
seam. This spec is the follow-up §4 named.

### 1.1 Reproduced, not predicted

Measured 2026-09-21 with `spec-spine 0.21.0` in a fixture corpus: one package
`pkg` whose manifest declares floor spec `002-floor`; spec `001-a` establishes
`pkg/src/doomed.rs` and `pkg/src/kept.rs`. The change deletes `doomed.rs` and
removes that one line from `001-a`'s `establishes` list.

```
C-001 'pkg/src/doomed.rs' changed without an authoring edit to any owning spec (002-floor)
```

`index owner pkg/src/doomed.rs` answers `001-a` (unit) and `002-floor` (floor)
at the base, and `(no spec owns this path)` at head. The withdrawal is in the
diff. The gate cannot see it.

### 1.2 The refusal's own advice does not work here

The `C-001` message offers three doors (spec 045). For this shape two of them
are unusable and the third is a human instrument:

- **Edit the owning spec.** The spec the message names is the floor, `002-floor`.
  Editing it to mention a file it never owned would be writing something untrue
  into an approved spec to make a gate pass.
- **Declare an `extends` edge naming the unit.** Measured in the same fixture:
  the unit is a path that no longer exists, so the next `index` raises
  `I-004`, and `check --fail-on-unresolved` exits 1 (spec 080). The remedy the
  gate prints provably cannot be followed.
- **A `Spec-Drift-Waiver:` line.** A human instrument, which an agent never
  writes on its own authority (spec 092 D-11).

A gate that refuses correct work and then names a remedy that cannot be
performed is worse than one that refuses silently, because it sends the author
to produce a second defect. That is why this is filed as a correctness defect
and not as an enhancement.

## 2. Territory

The coupling gate's ownership resolution (`couple.rs`), the CLI half that builds
the gate's inputs from git (`cmd_couple.rs`), the JSON facade entry in `lib.rs`,
and the acceptance in both `couple.rs` test files.

It claims no new file. Everything it changes is territory spec 005 established
and this spec extends.

## 3. Behavior

### 3.1 The base snapshot is the merge base, and the spec says so

The gate's text diff is `git diff --no-color -U0 --no-renames base...head`, a
**three-dot** range, so the "before" side of every hunk the gate reads is
`merge-base(base, head)` and **not** the commit `--base` names. The two differ
whenever the branch is behind its base, which on a busy default branch is most
of the time.

The base snapshot for ownership MUST therefore be `merge-base(base, head)`. It
MUST NOT be the supplied base ref. The CLI already resolves exactly this commit
for another purpose: `merge_base()` in `cmd_couple.rs` supplies the base-side
manifest bytes the dependency-only auto-waiver compares (spec 027), and
`cmd_delta.rs` calls the same helper for spec 071's `DeltaCommits.mergeBase`.
This spec adds a second consumer of a value the CLI already computes, not a new
notion of "base".

### 3.2 A deleted path is judged at the base snapshot

For a `DiffFile` whose `deleted` flag is set, **every ownership question the
gate asks about that path MUST be answered against the base snapshot**:

- the owner set `owners_for_path` resolves, including the supersedes transfer
  and the amends-awareness that follow from it;
- the explicit-claim override inside the bypass predicate (spec 008), so a path
  that a claim lifted out of a bypass prefix at the base is still examined when
  the change removes it.

One rule, both questions, so there is no state in which the gate judges a path
it decided was governed under one snapshot and resolves its owners under
another.

For every other path the gate MUST keep resolving ownership at head, unchanged:

| Change | Snapshot | Why |
|---|---|---|
| deletion | base | the claim that authorizes a removal is the one the removal withdraws |
| addition | head | the claim that authorizes an addition is the one the change adds |
| modification | head | the owner set that must have authored the edit is the one the change leaves behind |

A deletion's owner set is the base answer **alone**, not the union of base and
head. The head answer for a deleted path is by construction a lexical match: a
subtree or floor claim whose prefix still spells the vanished path. Unioning it
back in would re-admit exactly the false positive this spec removes.

### 3.3 What the base-side read does not change

This spec changes **which snapshot answers the ownership question**. It does not
change what an answer means. In particular:

- **Surviving co-owners still refuse.** If specs A and B both own the path at
  the base and the change edits neither `spec.md`, `C-001` still fires and
  names both. The clearance rule is unchanged: any one owner's `spec.md` in the
  diff clears the path (spec 005).
- **A floor-only deletion still refuses.** A file that no spec specifically
  owned at the base, inside a package whose manifest names a floor, is still
  `C-001` against that floor when no owning spec is edited. Whether removing an
  unowned file should be permitted is a separate question about what `C-001`
  means, and this spec does not answer it (§4).
- **The ownership ratchet is untouched.** `C-002` already exempts a deleted path
  (spec 029), so no base read is needed for it and none is added.
- **The bypass floor is untouched.** Its prefixes, their sources and their
  precedence are exactly spec 005 §3.5 and spec 092 §3.8. Only the snapshot the
  spec 008 claim override is read from moves, and only for a deleted path.

### 3.4 The base snapshot is resolved lazily

The gate MUST NOT read a base snapshot when the diff carries no deleted path. A
change with no deletion asks no question the base could answer, and a run that
exported a tree to answer nothing would spend the cost on every pull request to
serve the minority that need it.

### 3.5 Base metadata that cannot be read is reported, never fatal

The base snapshot is a historical commit. It is not the tree under judgment, it
cannot be repaired by the author of the change, and in four states it cannot be
read at all. In every one of them the gate MUST fall back to the head answer
for that path and MUST name the reason in its report. None of them is an error
exit.

| State | Cause | Verdict |
|---|---|---|
| **missing** | the merge base predates the committed index, or the repository did not commit one | fall back to head, reason `no-base-index` |
| **stale** | the base's committed shards do not match the base's corpus | fall back to head, reason `stale-base-index` |
| **invalid** | unparseable JSON, or an unknown schema MAJOR | fall back to head, reason `unreadable-base-index` |
| **unreachable** | `git` cannot resolve a merge base: unrelated histories, or a shallow clone without one | fall back to head, reason `no-merge-base` |

A fallback MUST be visible. A reader of the report, prose or `--json`, MUST be
able to tell a path judged at the base from one judged at head because the base
could not be read; a silent fallback would make the gate's verdict depend on
clone depth with nothing on the screen to say so.

This is a deliberate divergence from spec 071, where a stale base index is
`Error::Stale` and the verb refuses. `delta` is a record whose caller chose both
roots and can fix either; `couple` is a gate whose base is whatever ref CI
passes, and refusing a branch because of a defect in one of its ancestors
punishes the wrong change. D-2 records the reasoning.

### 3.6 Renames stay two facts

The gate passes `--no-renames`, so a rename reaches it as a deletion and an
addition. This spec MUST NOT introduce rename or similarity detection, and MUST
NOT pair a deletion with an addition by any heuristic.

Under §3.2 the two halves are judged where each one is true: the delete half at
the base, so the spec that owned the old path must have authored the move; the
add half at head, so the spec that owns the new path must claim it. That is the
same position spec 071 takes (A07 in `docs/design/04-authority-evidence-extension.md`:
git similarity is a suggestion, never authority), and it is what a reviewed move
mapping would later build on rather than replace.

### 3.7 Committed and working-tree operation

With `--include-uncommitted` (spec 081), `--head` is already pinned to `HEAD`,
and the examined set is the union of `merge-base(base, HEAD)...HEAD` with
`git diff HEAD`. The run has **one** base snapshot, `merge-base(base, HEAD)`,
and it answers for every deleted path in the union.

A path deleted in the working tree that was created after the merge base does
not exist in the base snapshot. That is not one of §3.5's four states: the
snapshot was read and the path is genuinely absent from it. The gate MUST fall
back to the head answer for that path, with reason `absent-at-base`.

`--paths-from` carries no history and sets `deleted: false` for every listed
path (spec 081 §3.3), so no base read is engaged. That MUST stay true: a path
list is not a diff and has no base side to consult.

### 3.8 The library contract is additive

The existing pure entry points MUST keep their exact signatures and their exact
behavior:

```rust
pub fn couple_with(cfg, registry, index, diff, waiver) -> Result<CoupleReport, Error>
pub fn couple_with_scope(cfg, registry, index, scope, diff, waiver) -> Result<CoupleReport, Error>
```

One new entry point takes the base snapshot as an option, and the two above
delegate to it passing `None`. A caller that passes nothing gets today's answer,
byte for byte. This is the smallest truthful contract: the resolution algorithm
stays in the library, which is the surface bindings wrap, and no existing
caller, overlay or test changes.

Two alternatives were considered and are recorded in D-1 rather than left to be
rediscovered: changing the two signatures in place (breaks every overlay for a
case most callers do not have), and carrying base-side owners as an additive
`DiffFile` field computed by the CLI (no library change, but it duplicates
`owners_for_path` at the CLI boundary and makes the verdict depend on a field a
caller can fabricate).

`couple_json` gains one optional request member, `"baseRoot": string`, an
exported tree in exactly the sense `delta_json` already takes `baseRoot` and
`headRoot` (spec 071). Absent, the facade behaves as today. The member is
additive, so no schema MAJOR moves.

The report gains an additive block naming, per deleted path, which snapshot
answered and, where it was head, which of §3.5's or §3.7's reasons applied.
Adding it is a schema MINOR of the verdict document under
`docs/schema-versioning.md`, and `docs/api.md` records the new request member.

## 4. Out of scope

- **What `C-001` means for an unowned deletion.** §3.3 keeps today's answer. A
  change to it is a change to the gate's policy, not to which snapshot it reads,
  and belongs to its own spec.
- **Rename detection**, in any form (§3.6).
- **The base-side read for additions and modifications.** Nothing measured asks
  for it and §3.2 gives the reason it would be wrong.
- **`index coverage` and `C-002`.** Deletions are already exempt.
- **A second freshness contract.** §3.5 deliberately does not add a gate on the
  base tree; the head tree's freshness refusal is unchanged.

## 5. Resolved decisions

*(Filed as a draft. Decisions taken during the build are appended here.)*

**D-1 (2026-09-21, the compatibility shape is additive, and the two rejected
options are recorded).** See §3.8. The choice is stated in the spec rather than
left to the build because "does the public signature change" is the question an
adopter and a binding author both ask first, and a build that answered it
silently would be making an API decision under cover of a bug fix.

**D-2 (2026-09-21, an unreadable base is a fallback, not a refusal).** See §3.5.
The alternative, matching spec 071 and refusing, was rejected because it makes
the gate's verdict depend on the health of a commit nobody on the branch can
reach. A shallow CI clone is the common case, not the exotic one.

## Verification

Written to fail against the tree this spec is filed on. Lines 1 and 2 assert
the defect is still present and the contract absent; they go red in exactly the
change that closes them.

```verify:cli
# 3.2: the base-side read exists in the gate at all. Red before the build.
test -f crates/spec-spine-core/src/couple.rs
grep -q 'base snapshot' crates/spec-spine-core/src/couple.rs
# 3.8: the two existing pure entry points still have their exact arity, and the
# new one takes the base as an option. All three lines must hold together.
grep -q 'pub fn couple_with(' crates/spec-spine-core/src/couple.rs
grep -q 'pub fn couple_with_scope(' crates/spec-spine-core/src/couple.rs
grep -qE 'Option<&(CodebaseIndex|BaseSnapshot)>' crates/spec-spine-core/src/couple.rs
# 3.1: the CLI resolves the merge base for the gate, not the supplied base ref.
grep -q 'merge_base' crates/spec-spine-cli/src/cmd_couple.rs
# 3.5: all four unreadable-base reasons are named in the code, not just handled.
grep -q 'no-base-index' crates/spec-spine-core/src/couple.rs
grep -q 'stale-base-index' crates/spec-spine-core/src/couple.rs
grep -q 'unreadable-base-index' crates/spec-spine-core/src/couple.rs
grep -q 'no-merge-base' crates/spec-spine-core/src/couple.rs
# 3.7: the absent-at-base fallback is distinct from the four failure reasons.
grep -q 'absent-at-base' crates/spec-spine-core/src/couple.rs
# The regression cases, by name: the false refusal and the refusals that stay.
grep -q 'deleted_path_resolves_owners_at_the_base' crates/spec-spine-core/tests/couple.rs
grep -q 'deleted_path_with_no_owning_edit_still_refuses' crates/spec-spine-core/tests/couple.rs
grep -q 'deleted_path_absent_at_base_falls_back_to_head' crates/spec-spine-core/tests/couple.rs
grep -q 'paths_from_engages_no_base_read' crates/spec-spine-core/tests/couple.rs
# The whole gate suite, which is where "modifications are unaffected" is asserted.
cargo test -p spec-spine-core --test couple --locked
cargo test -p spec-spine-cli --test couple --locked
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
