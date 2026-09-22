---
id: "100-a-deleted-path-is-judged-where-it-lived"
title: "A deleted path is judged where it lived"
status: approved
kind: "tooling"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "005-coupling-gate"
  - "081-coupling-sees-the-change-being-committed"
summary: >
  The coupling gate resolves a deleted path's owners from the head index, where
  the claim the same change withdrew no longer exists, so a correct removal is
  refused as C-001 against a package floor that has nothing to say about it.
  A deleted path is judged against the snapshot that precedes the segment its
  deletion belongs to, reconstructed from that snapshot's own source bytes, and
  a snapshot the gate cannot reconstruct is an explicit non-success rather than
  a head-only pass.
amends:
  - "005-coupling-gate"   # 3.1's single loaded index; the gate may now refuse (exit 3)
  - "081-coupling-sees-the-change-being-committed"  # 3.1's union says nothing about which snapshot answers
amends_sections: ["3.1", "3.2"]
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
  - spec: "034-machine-readable-verdicts"
    paths:
      - "crates/spec-spine-types/src/version.rs"
      # 3.7's additive block moves the verdict schema to a new MINOR, and two
      # acceptance assertions pin the constant by literal. Updating a pin the
      # bump invalidates is a crossing, declared rather than waived.
      - "crates/spec-spine-cli/tests/cli.rs"
      - "crates/spec-spine-types/tests/dtos.rs"
    nature: additive
  - spec: "057-the-docs-name-what-adopters-derived"
    paths:
      - "docs/api.md"
      - "docs/schema-versioning.md"
    nature: additive
  # 3.2: the tree export spec 071 built for `delta` is the mechanism this gate
  # reuses. Reusing it means widening one function's visibility, which is a
  # crossing into 071's file and is declared rather than duplicated: a second
  # exporter that agreed today and drifted next quarter is the worse outcome.
  - spec: "071-a-change-is-classified-under-the-bases-rules"
    unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_delta.rs" }
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
the verdict schema constant, the two documentation files that record the request
member and the schema bump, and the acceptance in both `couple.rs` test files.

It claims no new file. Everything it changes is territory another spec
established and this spec extends or amends.

## 3. Behavior

### 3.1 Two snapshots, and which one answers

Three trees are in play and they are not interchangeable:

| Name | What it is |
|---|---|
| **merge base** | `merge-base(base, head)`, the "before" side of the three-dot diff the gate already reads |
| **HEAD** | the last commit, the "before" side of the working-tree diff |
| **working tree** | the state under judgment |

The gate's text diff is `git diff --no-color -U0 --no-renames base...head`, a
**three-dot** range, so the before side of every hunk is `merge-base(base,
head)` and **not** the commit `--base` names. The two differ whenever the
branch is behind its base, which on a busy default branch is most of the time.

**A deleted path is judged at the snapshot that immediately precedes the
segment in which its deletion is recorded.** There are exactly two segments and
therefore exactly two prior snapshots:

| Deletion recorded in | Prior snapshot | Why |
|---|---|---|
| the committed range `merge-base...head` | the **merge base** | the claim that authorizes the removal is the one the commits withdrew |
| the working-tree diff `git diff HEAD` (spec 081) | **HEAD** | the file existed at HEAD and its claim may have been withdrawn in the working tree; judging it at the merge base would ask about a commit that never held it |

Every other path keeps resolving at head, unchanged:

| Change | Snapshot | Why |
|---|---|---|
| deletion | the prior snapshot of its segment | the claim that authorizes a removal is the one the removal withdraws |
| addition | head | the claim that authorizes an addition is the one the change adds |
| modification | head | the owner set that must have authored the edit is the one the change leaves behind |

A deletion's owner set is the prior-snapshot answer **alone**, never the union
of prior and head. The head answer for a deleted path is by construction a
lexical match: a subtree or floor claim whose prefix still spells the vanished
path. Unioning it back in would re-admit exactly the false positive this spec
removes.

### 3.2 A prior snapshot is reconstructed from source, not read from a ledger

A prior snapshot MUST be built by compiling and indexing the **exported tree of
that commit**: the `spec.md` bytes, the manifests and the source files as that
commit recorded them. It MUST NOT be read from the committed derived shards of
that commit.

The reason is that the committed ledger of a historical commit is evidence
somebody wrote, and it can be stale, absent, or wrong, whereas the corpus
source at that commit is immutable and is what the ledger was a function of.
Reconstructing from source is the only route that yields ownership the gate can
stand behind, and this engine already has it: `compile` and `index` are pure
functions of `(Config, file contents)` over a root, and `spec-spine delta`
(spec 071) already exports a commit's tree through a private index file
(`read-tree`, then `checkout-index`) and runs them over it.

Consequences, stated so they are not rediscovered:

- **There is no "stale base index" state.** Nothing reads a historical shard,
  so nothing can find one stale.
- **The configuration is the snapshot's own.** A prior snapshot is compiled
  under the `spec-spine.toml` of the commit it describes, exactly as spec 071
  §3.2 requires for `delta`. A change cannot re-own a path it is deleting by
  editing the configuration in the same commit.
- **`git archive` is not the export mechanism.** It honors `export-ignore`, and
  a candidate could then hide a path from the tree its removal is judged
  against. The `read-tree` / `checkout-index` route spec 071 established is the
  one to use.

### 3.3 What the snapshot read does not change

This spec changes **which snapshot answers the ownership question**. It does not
change what an answer means. In particular:

- **Surviving co-owners still refuse.** If specs A and B both own the path in
  the prior snapshot and the change edits neither `spec.md`, `C-001` still
  fires and names both. The clearance rule is unchanged: any one owner's
  `spec.md` in the diff clears the path (spec 005).
- **Ownership transfer still applies.** Supersedes transfer and the
  amends-awareness that follows from it are computed against the prior
  snapshot's registry, so a successor that inherited the predecessor's
  authority before the deletion is still an owner of the deleted path.
- **A floor-only deletion still refuses.** A file that no spec specifically
  owned in the prior snapshot, inside a package whose manifest named a floor
  there, is still `C-001` against that floor when no owning spec is edited.
  Whether removing an unowned file should be permitted is a separate question
  about what `C-001` means, and this spec does not answer it (§4).
- **The explicit-claim override still lifts a path out of a bypass**, and is
  now evaluated in the same snapshot as the owner set for a deleted path. One
  rule, both questions, so there is no state in which the gate decides a path
  is governed under one snapshot and resolves its owners under another.
- **The ownership ratchet is untouched.** `C-002` already exempts a deleted
  path (spec 029), so no prior read is needed for it and none is added.
- **The bypass floor is untouched.** Its prefixes, their sources and their
  precedence are exactly spec 005 §3.5 and spec 092 §3.8. Only the snapshot the
  spec 008 claim override is read from moves, and only for a deleted path.

### 3.4 A prior snapshot is built only when a deletion needs one

The gate MUST NOT reconstruct a snapshot when the diff carries no deleted path,
and MUST NOT reconstruct the HEAD snapshot when no deletion is recorded in the
working-tree segment. A change with no deletion asks no question a prior
snapshot could answer, and a run that exported a tree to answer nothing would
spend the cost on every pull request to serve the minority that need it.

This is also what keeps a shallow clone working for the ordinary case: a
pull request that deletes nothing never reaches §3.5.

### 3.5 Required prior evidence that cannot be obtained is a refusal

When a deletion is present and its prior snapshot is **required** (§3.4), the
gate MUST obtain it or MUST NOT return a verdict.

A historical snapshot is not the current working tree, and inability to judge
it is still inability to judge. The gate MUST NOT emit a successful head-only
verdict in place of an answer it could not compute.

| State | Cause | Behavior |
|---|---|---|
| **no merge base** | unrelated histories, or a shallow clone whose graft point is above the real merge base | refuse, exit `3` |
| **commit not present** | the object is not in this clone, typically `--depth 1` | refuse, exit `3` |
| **export failed** | `read-tree` or `checkout-index` failed, or the destination could not be written | refuse, exit `3` |
| **snapshot corpus does not validate** | the exported tree's `spec.md` set fails compilation or validation | refuse, exit `3` |

The first two are reached by the **three-dot range read itself**, which is the
gate's first act and fails before any snapshot is required. That read is
therefore covered by the same rule: its failure MUST carry the refusal's
language and remedy rather than only git's message, so a run that could not
read the history can never be mistaken for a run that found nothing. D-5
records why this is stated as a property of the range read rather than of the
snapshot builder.

A corpus that produces **validation violations** counts as unreadable. It does
not fail to parse at the `Err` boundary, so a snapshot built from it would be
silently missing exactly the specs that failed, and "nobody owned this path"
would be an artifact of the evidence not loading.

Exit `3` is the code spec 005 §3.7 already assigns to an "IO / parse / load
failure", so this adds a cause and no new code. The refusal MUST name the
cause and the remedy, because every one of these states has an operator fix:
fetch more history (`git fetch --deepen` or an unshallow clone), or repair the
historical commit's corpus and rebase.

Three things the gate MUST NOT do, each of which would turn "I cannot judge"
into a silent pass:

1. **Substitute another revision.** Not the supplied base ref, not the shallow
   graft point, not HEAD. A verdict computed against a tree nobody asked about
   is worse than no verdict.
2. **Ignore corrupt evidence.** A snapshot whose corpus does not compile has
   not answered. It is not empty.
3. **Treat a missing history as an empty diff.** A shallow clone that cannot
   reach the merge base has not told the gate that nothing changed.

**An absent path in a snapshot that was read is a different fact.** If the
prior snapshot was reconstructed successfully and the deleted path simply does
not appear in it, then the honest answer is that no spec owned it there: the
owner set is empty and `C-001` does not fire. That is a computed answer and not
a fallback, it MUST be recorded as such in the report (§3.7), and it MUST NOT
be resolved by falling back to head or to the working tree.

D-3 records why this differs from the draft this spec replaced, which allowed a
head-only fallback in four states.

### 3.6 Renames stay two facts

The gate passes `--no-renames`, so a rename reaches it as a deletion and an
addition. This spec MUST NOT introduce rename or similarity detection, and MUST
NOT pair a deletion with an addition by any heuristic.

Under §3.1 the two halves are judged where each one is true: the delete half at
the prior snapshot, so the spec that owned the old path must have authored the
move; the add half at head, so the spec that owns the new path must claim it.
That is the same position spec 071 takes (A07 in
`docs/design/04-authority-evidence-extension.md`: git similarity is a
suggestion, never authority), and it is what a reviewed move mapping would
later build on rather than replace.

### 3.7 The report records which snapshot answered

For every deleted path the gate examined, the report MUST record which snapshot
resolved its owners, from a closed set:

| Token | Meaning |
|---|---|
| `merge-base` | the committed segment's prior snapshot answered |
| `head-commit` | the working-tree segment's prior snapshot (HEAD) answered |
| `head-tree` | no prior snapshot was supplied; the tree under judgment answered (the compatibility path of §3.8 only) |

and, when the path was absent from the snapshot that answered, a flag saying
so. A reader of the report, prose or `--json`, MUST be able to tell a path
judged at a prior snapshot from one judged at head, because a verdict whose
provenance is invisible is a verdict whose correctness cannot be reviewed.

The block is **additive and omitted when empty**, so a run with no deletions,
and every call through the compatibility entry points of §3.8 that supplies no
snapshot and examines no deletion, emits exactly the bytes it emits today. That
is what reconciles a new report member with preserving legacy emitted bytes:
the member can appear, so the verdict schema takes a MINOR bump under
`docs/schema-versioning.md`; it does not appear for any input that produced a
report before this spec, so no existing consumer sees a byte move.

### 3.8 Compatibility is preserved, and is not the corrected behavior

The existing pure entry points MUST keep their exact signatures and their exact
behavior:

```rust
pub fn couple_with(cfg, registry, index, diff, waiver) -> Result<CoupleReport, Error>
pub fn couple_with_scope(cfg, registry, index, scope, diff, waiver) -> Result<CoupleReport, Error>
```

One new entry point takes the prior snapshots, and the two above delegate to it
supplying none. A caller that passes nothing gets today's answer, byte for
byte, and no existing caller, overlay or test changes.

**What must not be claimed for them.** A call with no prior snapshot resolves a
deleted path at head. That is the defect §1 describes, retained deliberately
for compatibility, and it MUST NOT be documented or reported as though it
carried the two-snapshot guarantee. The distinction MUST be visible in three
places:

1. **In the report**, as `head-tree` (§3.7).
2. **In the rustdoc** of both compatibility entry points, which MUST say that
   they resolve deletions at head and name the entry point that does not.
3. **In `docs/api.md`**, which MUST distinguish the compatibility calls from
   the CLI and the new facade path.

The CLI and the facade's new request member are the paths that promise the
correction. The compatibility calls are the paths that promise only that
nothing moved.

Two alternatives were considered and are recorded in D-1 rather than left to be
rediscovered: changing the two signatures in place (breaks every overlay for a
case most callers do not have), and carrying prior-side owners as an additive
`DiffFile` field computed by the CLI (no library change, but it duplicates
`owners_for_path` at the CLI boundary and makes the verdict depend on a field a
caller can fabricate).

`couple_json` gains one optional request member, `"priorRoots"`, naming the
exported trees in exactly the sense `delta_json` already takes `baseRoot` and
`headRoot` (spec 071). Absent, the facade behaves as today. The member is
additive, so no schema MAJOR moves. `docs/api.md` records it.

### 3.9 Combined histories

With `--include-uncommitted` (spec 081) the examined set is the union of
`merge-base(base, HEAD)...HEAD` with `git diff HEAD`, and `--head` is already
pinned to `HEAD`. The union rule is unchanged: a path both segments know takes
the **later** segment's deletion verdict. What this spec adds is that the
deletion carries the segment that decided it, and that segment selects the
snapshot.

The cases that must be specified because they are the ones a single-snapshot
design gets wrong:

| Case | Committed segment | Working tree | Verdict resolved at |
|---|---|---|---|
| **plain committed deletion** | deletes the path | untouched | merge base |
| **plain working-tree deletion** | untouched | deletes the path | HEAD |
| **add then delete** | adds the path (and claims it) | deletes the path | HEAD, where the claim added by the commits still stands |
| **modify then delete** | modifies the path | deletes the path | HEAD |
| **delete then restore** | deletes the path | restores the path | not a deletion; head, as an addition |
| **claim withdrawn in the working tree** | untouched | deletes the path and edits the owning `spec.md` | HEAD, where the claim still stands, and the `spec.md` edit is in the diff, so it clears |

The **add then delete** row is the one the draft this spec replaces got wrong.
The path does not exist at the merge base, so a single base snapshot would find
no owner and silently pass a deletion whose claim, added in the same branch, is
still standing at HEAD. Judging it at HEAD is what makes the withdrawal
obligatory there too.

The working tree is never the snapshot for a deletion. Its whole relevant
property is that the claim has disappeared from it.

`--paths-from` carries no history and sets `deleted: false` for every listed
path (spec 081 §3.3), so no snapshot is required and none is built. That MUST
stay true: a path list is not a diff and has no prior side to consult.

## 4. Out of scope

- **What `C-001` means for an unowned deletion.** §3.3 keeps today's answer. A
  change to it is a change to the gate's policy, not to which snapshot it reads,
  and belongs to its own spec.
- **Rename detection**, in any form (§3.6).
- **The prior-snapshot read for additions and modifications.** Nothing measured
  asks for it and §3.1 gives the reason it would be wrong.
- **`index coverage` and `C-002`.** Deletions are already exempt.
- **A freshness gate on the historical tree.** §3.2 removes the question by
  reconstructing rather than reading; the head tree's freshness refusal is
  unchanged.
- **Caching exported trees between runs.** A correctness fix does not get to
  introduce a cache, and the laziness of §3.4 is the performance answer this
  spec is allowed to give.

## 5. Resolved decisions

*(Filed as a draft. Decisions taken during the build are appended here.)*

**D-1 (2026-09-21, the compatibility shape is additive, and the two rejected
options are recorded).** See §3.8. The choice is stated in the spec rather than
left to the build because "does the public signature change" is the question an
adopter and a binding author both ask first, and a build that answered it
silently would be making an API decision under cover of a bug fix.

**D-2 (2026-09-21, superseded by D-3; retained so the reasoning is not
rediscovered).** The first draft of this spec allowed a head-only fallback,
with a named reason, in four unreadable-base states, on the argument that
refusing punishes a branch for a defect in one of its ancestors and that a
shallow CI clone is the common case. The row is kept because the trade-off is
real and a later reader will re-derive it.

**D-3 (2026-09-21, owner ruling: unobtainable required evidence is a refusal,
not a fallback).** See §3.5. D-2 is withdrawn. A gate that cannot read the
evidence its verdict depends on has not judged the change, and a successful
verdict in that state makes the gate's answer depend on clone depth. The
objection D-2 raised is answered rather than dismissed, in three parts: §3.2
removes two of the four states outright by reconstructing from immutable source
instead of reading a historical ledger; §3.4 keeps every deletion-free pull
request off the path entirely, which is most of them; and the remaining states
all have an operator remedy the message names. The cost is that a shallow clone
must fetch history to merge a deletion, which is a CI configuration line, and
the benefit is that the gate never reports a pass it did not compute.

**D-4 (2026-09-21, owner ruling: a deletion is judged at the snapshot preceding
its own segment).** See §3.1 and §3.9. The first draft used one base snapshot
for the whole run and added an `absent-at-base` head fallback for working-tree
deletions of files added after the merge base. That is the add-then-delete hole:
the claim is standing at HEAD, and falling back to head asks the tree the claim
has already been removed from. Per-segment provenance answers it without a
fallback, and makes the working tree never a snapshot for anything.

**D-5 (2026-09-21, build: the two history states are caught by the range read,
not by the snapshot builder).** Measured while writing the acceptance: `git
diff base...head` refuses outright when there is no merge base (`fatal:
base...head: no merge base`) and when the clone lacks the commits (`unknown
revision`), so the gate never reaches the point where it would ask for a
snapshot. The requirement is still met, because the failure is an explicit
exit `3` and never a pass; what would not have been met is the requirement
that the refusal *say* the gate judged nothing. So the range read's error is
wrapped with the same language and remedy. The cost is that a deletion-free
change in a clone that cannot read its own range now also gets the fuller
message, which is correct: it judged nothing either.

**D-6 (2026-09-21, build: a snapshot whose corpus does not validate is
refused).** `compile` reports a malformed spec as a validation violation, not
as an `Err`, so `prior_ownership_from_root` would happily return a snapshot
with the broken specs missing. That is the "ignore corrupt evidence" failure
§3.5 forbids, wearing a different hat: the read succeeded and the evidence did
not. The validation verdict is checked explicitly and a failure is an `Err`.

**D-7 (2026-09-21, build: the segment set rides with the snapshots, not on
`DiffFile`).** §3.8's D-1 rejected computing prior-side *owners* at the CLI
boundary, and that reasoning does not extend to the segment, which is a fact
only the CLI can know and which no library resolution duplicates. Carrying it
on `DiffFile` would have moved that struct's serialized shape, which §3.7
promises not to do, and would have broken every existing struct literal in the
test suites for a field most callers never set. It is a field of
`PriorSnapshots` instead.

**D-8 (2026-09-21, build: `export_tree` is shared with `delta`, under a
declared edge).** Spec 071 already exports a commit's tree through a private
index file, deliberately not via `git archive` (which honors `export-ignore`
and would let a candidate hide a path from the tree its change is judged
against). Reimplementing that in the gate would be a second exporter agreeing
today and drifting later. Widening one function's visibility is a crossing
into 071's file, declared as an `extends` edge in this spec's frontmatter,
which is the route spec 005 offers and needs no waiver.

**D-9 (2026-09-21, build: `couple`'s freshness guard fires before the gate for
an unwithdrawn working-tree deletion).** Found while writing the
add-then-delete acceptance. Deleting a file that a unit claim still names
makes the working tree's index unresolved, so `couple` exits 2 on staleness
before any ownership question is asked. That is pre-existing spec 005 /
spec 023 behavior and this spec does not change it; the acceptance therefore
exercises add-then-delete over a **floor-owned** path, where no unit claim is
left dangling and the gate is actually reached. The discriminating property is
unaffected: the path is absent at the merge base and owned at HEAD, so a
base-only design passes it and this one refuses.

**D-10 (2026-09-21, review: a partial snapshot set is the compatibility path,
not an unmet §3.5 refusal).** Raised on the pull request: `PriorSnapshots` lets
`merge_base` and `head_commit` be `None` independently, so a facade caller
supplying `priorRoots` with only `mergeBase` gets working-tree deletions
resolved at head. That is §3.8's compatibility path reaching one side of a run
rather than the whole of it, and it is not §3.5's case: §3.5 refuses evidence
the gate tried to obtain and could not, and the CLI is where that obtaining
happens. The CLI never constructs a partial set, because `PriorExports::build`
builds exactly the sides the run's deletions need. A caller that declares no
snapshot for a side has not failed to read one, and the run is not silent about
it: §3.7's provenance records `head-tree` for every path resolved that way, in
the verdict a reader reviews. The behavior therefore stands and the contract is
written down instead, in `PriorSnapshots`'s rustdoc and in `docs/api.md`, which
is what §3.8 already requires of every path that does not carry the
two-snapshot guarantee.

## Verification

Behavioral assertions. Every case below is a test that constructs a corpus,
runs the gate, and asserts the verdict; the string searches the first draft used
as its principal evidence are gone, because a `grep` for a reason token passes
against a comment and proves nothing about a decision.

Written to fail against the tree this spec is filed on: the named test files do
not exist yet and the named tests do not exist in the files that do, so both
`cargo test` lines are red before the build.

```verify:cli
# The gate's own suite, which after this build contains, by name:
#   - deleted_path_resolves_owners_at_the_merge_base
#       the reproduced package-floor false refusal, asserted as a PASS
#   - deleted_path_with_surviving_co_owner_still_refuses
#       a legitimate C-001 that must survive: two owners, neither edited
#   - deleted_path_floor_only_still_refuses
#       the other legitimate refusal: no specific owner at the prior snapshot
#   - deleted_path_explicit_claim_overrides_bypass_at_the_prior_snapshot
#       a claim lifted the path out of a bypass prefix before the deletion
#   - deleted_path_supersedes_transfer_applies_at_the_prior_snapshot
#       the successor inherited authority and clears the removal
#   - deleted_path_absent_from_a_read_snapshot_has_no_owner
#       absent-in-a-read-snapshot is a computed answer, not a fallback
#   - deletion_reports_which_snapshot_answered
#       the §3.7 provenance token is in the report
#   - legacy_couple_with_is_byte_identical_and_reports_head_tree
#       the compatibility path, unchanged, and honestly labelled
#   - modifications_and_additions_are_unaffected_by_a_prior_snapshot
#       the non-deletion behavior, asserted against a snapshot being present
cargo test -p spec-spine-core --test couple --locked
# The CLI suite, which after this build contains, by name:
#   - committed_deletion_is_judged_at_the_merge_base
#   - worktree_deletion_is_judged_at_head_commit
#   - add_then_delete_is_judged_at_head_commit
#   - delete_then_restore_is_not_a_deletion
#   - shallow_clone_is_not_treated_as_an_empty_diff
#   - unrelated_histories_refuse_exit_3
#   - corrupt_prior_corpus_refuses_exit_3
#   - no_deletion_builds_no_prior_snapshot
#   - paths_from_builds_no_prior_snapshot
cargo test -p spec-spine-cli --test couple --locked
# The whole workspace, because the verdict schema constant moves and the
# conformance test asserts every emitted document against its schema.
cargo test --workspace --locked
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
