---
id: "206-a-stopped-merge-regenerates-its-derived-tree"
title: "A stopped merge regenerates its derived tree, and refuses anything else that conflicts"
status: draft
kind: "tooling"
created: "2026-10-27"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  # 001, 162 and 170 own the files the build extends (section 2): the verb's
  # registration in `main.rs`, the capability catalog, and the verb that lists it.
  - "001-compile-registry"
  # The two writers 3.5 step 1 runs: the compiler (001) and the indexer.
  - "004-codebase-index"
  - "022-index-sharding"
  # The core-plans, CLI-executes seam (3.1).
  - "043-verify-declared-acceptance"
  - "062-one-name-one-freshness-verb"
  # The configured derived root (3.1).
  - "092-the-engine-ships-governance-not-an-environment"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "122-a-commit-refuses-an-unresolved-merge"
  - "132-one-exit-contract-for-the-family"
  - "162-capability-catalog"
  - "170-a-consumer-is-served-answers-not-access"
establishes:
  # The verb's module and its end-to-end tests are new files. Both are
  # planned (spec 063): nothing else touches them, so the draft blocks no
  # other change. The edges onto files other specs own (the verb's
  # registration in `main.rs`, the capability catalog) are added by the build,
  # after ratification, and are named in section 2.
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_resolve_derived.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/resolve_derived.rs", planned: true }
summary: >
  A rebase or merge that stops on a conflict in the committed derived tree is
  resolved by hand today: take either side, run `compile` and `index`, stage
  the tree, and hope nothing else was conflicted or lying untracked in the
  working tree when the indexer read it. Every adopter that restacks pull
  requests has scripted that sequence, and spec 094's merge driver, which
  covers it inside git, documents that it can emit clean but stale shards
  because it runs before the merged tree is complete. This spec adds
  `spec-spine resolve-derived`, run after git has stopped, generalizing
  statecraft-cli's `scripts/resolve-shard-conflicts.sh`. It refuses when any
  path outside the derived root is still unmerged, or when the working tree
  holds anything the compile would read that is not staged; otherwise it
  regenerates both shard trees from the staged merge, stages exactly the
  derived root, and then checks what the merge did to authored text: no
  conflict marker staged, and the carried change's added and removed lines
  unchanged by the merge. It confirms freshness the way `check` does and
  reports in the family envelope. It never continues, commits or aborts the
  operation, and it writes nothing outside the derived root.
intent:
  goal: "After a merge, rebase or cherry-pick stops, one verb settles every derived-tree conflict from the complete merged tree, or refuses and says why."
  non_goals:
    - "Resolving a conflict in any authored file, a spec included"
    - "Continuing, committing or aborting the git operation"
    - "Replacing or retiring spec 094's merge driver"
---

# 206: A stopped merge regenerates its derived tree, and refuses anything else that conflicts

## 1. Purpose

The committed derived tree is a pure function of the corpus and the source
tree. A conflict in it carries no information: neither side's bytes are the
answer, and the answer is whatever the compiler and the indexer produce over
the merged inputs. Git does not know that, so a rebase onto a moved base stops
on a shard conflict, and someone has to resolve it.

How that is done today:

- **By hand, in every adopter.** The fallback the merge driver prints is
  `spec-spine compile && spec-spine index && git add .statecraft/derived/`.
  Adopters that restack stacked pull requests have scripted it around
  `git checkout --theirs` on the derived paths. One adopter, measured on
  2026-10-27, carried 48 copies of the same restack function across session
  scratch directories, each with this sequence inlined, and two variants that
  folded unrelated conflicts into the same blind resolution.
- **Inside git, in this repository.** Spec 094 4.3's
  `.githooks/merge-derived-index.sh` regenerates during the merge. Its header
  records the limitation: regeneration "is only as complete as the working
  tree at git's driver-invocation moment, so on some rebases the auto-heal can
  still emit a clean-but-stale hash (it cannot self-detect)". It is also
  opt-in per clone and not distributed to adopters (spec 092).
- **As a script, in statecraft-cli.** `scripts/resolve-shard-conflicts.sh`
  (statecraft-cli commit `765aab40`) is the reference implementation this spec
  generalizes. During a merge it refuses while any authored path is conflicted,
  takes the incoming copy of each conflicted shard, runs the repository's
  refresh, stages the shard tree, and exits 1 when a conflict marker remains or
  when the branch's own authored change, compared as the lines each file adds
  and removes without context, differs after the merge. It was written after a
  mechanical "take theirs" over a conflicted merge deleted 2,700 lines of a
  spec there. This verb keeps every one of its checks, so statecraft-cli can
  replace the script with the verb.

The hand sequence has four failure modes seen in adopters:

1. **A real conflict rides along.** The script resolves the derived paths,
   then continues the rebase while an authored file is still conflicted, or
   resolves that file with the same "take one side".
2. **The indexer reads what is not staged.** `index` walks the working tree,
   so an untracked directory or an unstaged edit in the clone is indexed, and
   the committed shards describe a tree the commit does not hold. The
   governance job then reports them stale on the pull request.
3. **Nobody checks the result.** The sequence writes and stages; it does not
   ask whether what it staged is fresh.
4. **The merge rewrote authored text unnoticed.** An authored conflict
   resolved by taking one side, or a resolution that kept markers, still
   stages cleanly; only a comparison of the carried change before and after
   the merge shows it.

A verb the engine owns can refuse 1 and 2 before it writes, and answer 3 and
4 in the same call, as the reference implementation already does for 1, 3
and 4.

## 2. Territory

New files, both claimed as planned by this draft:

- `crates/spec-spine-cli/src/cmd_resolve_derived.rs`: the verb.
- `crates/spec-spine-cli/tests/resolve_derived.rs`: its end-to-end tests,
  each in a scratch repository with a real stopped merge, rebase and
  cherry-pick.

The build adds, after ratification, `extends` edges onto the files other
specs own: the verb's registration in `crates/spec-spine-cli/src/main.rs`
(spec 001), and its entry in the capability catalog (spec 162) so
`spec-spine capabilities` (spec 170) lists it. Library support, if the build
needs any, is an additive `extends` on the module that owns it. These edges
are left out of the draft on purpose: a merged draft that claims a shared
file refuses every change touching that file until it is ratified.

## 3. Behavior

### 3.1 The verb

```
spec-spine resolve-derived [--repo <DIR>] [--json]
```

It takes no path arguments. The derived root is the configured one
(`[layout] derived_dir`, through `Config::is_derived_path`, spec 092 3.8), never
a literal `.statecraft/derived` or `.derived`.

The engine core MUST NOT spawn a process. Core receives the unmerged and
working-tree state as data and returns a plan; the CLI reads that state from
git and carries out the plan. This is the seam `couple` and `verify` already
use (spec 043).

### 3.2 What it reads

Before writing anything, the verb MUST read:

1. **The unmerged entries**: every path git's index records at a stage other
   than 0 (`git ls-files --unmerged -z`).
2. **The working-tree state**: every tracked path whose working-tree content
   differs from its index entry, and every untracked path git does not ignore
   (`git status --porcelain=v1 -z --untracked-files=all`).
3. **The operation in progress**, from the head git records for it:
   `MERGE_HEAD` for a merge, `REBASE_HEAD` for a rebase, `CHERRY_PICK_HEAD`
   for a cherry-pick. It fixes the two sides 3.6 compares:
   - in a merge, the **carried change** is `HEAD`'s change since
     `git merge-base HEAD MERGE_HEAD`, and the **receiving side** is
     `MERGE_HEAD`;
   - in a rebase or a cherry-pick, the carried change is the commit being
     applied (`REBASE_HEAD` or `CHERRY_PICK_HEAD`) against its first parent,
     and the receiving side is `HEAD`.

A read that fails (not a git repository, git missing, a non-zero exit) is a
failure (exit 4, 3.7), never an absence of entries.

### 3.3 When there is nothing to resolve

With no unmerged entry under the derived root, the verb MUST write nothing
and exit 0, whatever else 3.2 read. It reports that nothing under the derived
root was conflicted, and lists any unmerged authored path and any path 3.4
item 2 would name, as information: with nothing to regenerate there is
nothing for those paths to corrupt. It does not regenerate a tree nobody
asked about: `compile` and `index` remain the verbs for that.

### 3.4 When it refuses

Otherwise, with at least one unmerged entry under the derived root, the verb
MUST refuse, write nothing, and leave the index unchanged when:

1. **An authored path is unmerged.** Any unmerged entry whose path is outside
   the derived root. The refusal names every such path. Resolving it is the
   author's job; this verb does not take a side on authored content.
2. **The compile would read what the commit will not hold.** Any path outside
   the derived root that is untracked and not ignored, or that differs between
   the working tree and the index. The refusal names every such path and says
   that the regenerated tree would describe it. A conflicted path already
   resolved and staged is not in this set: what it compares is the working
   tree against the index, and a staged resolution matches.
3. **A derived entry regeneration would not settle.** An unmerged path under
   the derived root that the compile and the index would neither write nor
   prune, for example a `build-meta.json` an adopter tracks against the
   default ignore rule. This is decided from the in-memory output of 3.5 step
   1, before anything is written, and the refusal names every such path.
4. **No operation is recorded.** Unmerged derived entries with none of the
   three heads of 3.2 item 3 leave 3.6 with no carried change to compare. The
   refusal says so. A conflicted `git stash pop` is such a stop: it leaves
   stage 1, 2 and 3 entries and writes none of `MERGE_HEAD`, `REBASE_HEAD` or
   `CHERRY_PICK_HEAD` (checked with git 2.50.1 on 2026-10-27).

All four refusals are exit 2 (refused, nothing done, spec 132). When more
than one holds, the report names every set.

### 3.5 Regeneration

When 3.3 does not apply, the verb MUST do the following, in order. 3.4 items
1, 2 and 4 are decided before step 1, and item 3 after it; step 1 writes
nothing, so a refusal under 3.4 at either point leaves the tree and the index
as they were.

1. **Compile the registry and build the index once, in memory, over the
   working tree**, which 3.4 item 2 has established equals the staged merge
   outside the derived root. Each half is the existing compiler and indexer,
   unchanged: the same shard bytes `compile` and `index` write, the same
   pruning of shards no unit produces. This one result decides 3.4 item 3 and
   is what step 2 writes; there is no second compile.
2. **Write that result and stage exactly the derived root**: every
   regenerated shard added, every pruned shard removed, and therefore every
   unmerged entry under the derived root collapsed to stage 0. It MUST stage
   nothing outside the derived root, and nothing the derived root's own ignore
   rules exclude (`build-meta.json`).
3. **Check what the merge did to authored text** (3.6).
4. **Confirm freshness** with the reads `spec-spine check` performs, in
   memory, and fold the result into the verdict (3.7).

If step 1 reports a validation failure on the merged corpus, the verb MUST
NOT write either tree and MUST leave the index unchanged: an invalid corpus is
a finding about authored text (exit 1), and the conflicted shards stay
conflicted so the stop is still visible.

The verb MUST NOT run `git rebase --continue`, `git merge --continue`,
`git commit`, `git cherry-pick --continue` or any `--abort`. What happens to the
operation is the caller's decision.

### 3.6 What the merge did to authored text

After staging, the verb MUST make two checks over the index, both excluding
the derived root, and report each. Neither writes anything.

1. **No conflict marker is staged.** No staged line outside the derived root
   begins with `<<<<<<< ` or `>>>>>>> ` (`git grep --cached`). Each hit is
   named with its path and line. Spec 122's commit hook refuses the same thing
   later; this names it while the operation is still stopped.
2. **The carried change is preserved.** The carried change (3.2 item 3) and
   the index's difference from the receiving side are each reduced to the
   sorted list of lines they add and remove, each prefixed with its file and
   its sign, with positions and context dropped (context moves whenever the
   receiving side edited nearby). The two lists MUST be equal. When they are
   not, the verb reports, per file, the lines present in one list and not the
   other.

Either check failing is a finding (exit 1). The staged derived tree stays as
written, because it is correct for the staged authored tree whatever that
tree's merit; what needs a person is the authored tree. A difference under
item 2 is legitimate only where both sides edited the same lines, which is
for the person to judge, never the verb.

### 3.7 The verdict

The `--json` output is the family envelope (spec 132), verb
`resolve-derived`. Its `report` carries:

- `unmerged`: the unmerged paths it read, each marked `derived` or `authored`;
- `operation`: `merge`, `rebase` or `cherry-pick`, with the carried change's
  commits and the receiving side's commit (absent under 3.3);
- `unstaged`: the paths 3.4 item 2 names (empty when none);
- `regenerated`: the derived paths staged with new content;
- `removed`: the derived paths staged as deletions;
- `markers`: 3.6 item 1's hits (empty when none);
- `authoredChange`: `preserved` true or false, and under false the per-file
  lines 3.6 item 2 found in one list only;
- `freshness`: the two halves of `check`'s report, unchanged.

Exit codes, under spec 132's contract:

| Exit | When |
|---|---|
| 0 | nothing to resolve (3.3), or regenerated, staged, fresh, no marker staged and the carried change preserved |
| 1 | the merged corpus does not validate, a marker is staged, the carried change is not preserved, or the staged tree is not fresh after regeneration |
| 2 | refused under 3.4 |
| 3 | usage |
| 4 | a git read, a write or a stage failed |

A finding of "not fresh after regeneration" (exit 1) is a defect in the
engine, not the caller's: the report MUST say so and name the drifted shards.

Without `--json`, the verb prints one line per regenerated or removed path,
then 3.6's two results, then the freshness lines `check` prints.

## 4. Relation to spec 094's merge driver

The driver is unchanged and stays opt-in. When it is registered and succeeds,
git does not stop, and this verb finds nothing to resolve (3.3). When it is
not registered, or it fails closed, git stops, and this verb settles the stop
from the complete tree the driver could not see. A later spec may make the
driver call this verb, or retire it; this spec does neither.

## 5. Acceptance criteria

1. In a scratch repository, a rebase that stops on a same-shard conflict and
   nothing else is resolved by the verb: exit 0, the derived root has no
   unmerged entry, `git diff --cached` touches nothing outside it, and
   `spec-spine check` exits 0 afterwards.
2. The same with a merge and with a cherry-pick.
3. A stop that also conflicts an authored file exits 2, names that file, and
   leaves every index entry, the derived ones included, exactly as it was.
4. An untracked, non-ignored file in the working tree exits 2, names it, and
   writes nothing; ignoring it makes the same call succeed.
5. An unstaged edit to a tracked authored file exits 2 and names it.
6. A modify/delete conflict on a shard whose spec one side removed is staged
   as a deletion and reported under `removed`.
7. A merged corpus that fails validation exits 1, writes no shard, and leaves
   the shard conflicts in place.
8. A configured derived root other than the default is honored: the
   default's path is never read or staged.
9. With nothing conflicted under the derived root, the verb exits 0 and writes
   nothing, also when an authored path is unmerged or the working tree is
   dirty, and the report lists those paths.
10. An unmerged derived path that regeneration neither writes nor prunes
    (a tracked `build-meta.json`) exits 2, names it, and writes nothing.
11. A stop with unmerged derived entries and no operation head (a conflicted
    `git stash pop`) exits 2 and says so.
12. A merge whose authored conflict was resolved by staging the markers exits
    1, names the path and line, and leaves the regenerated derived tree staged.
13. A merge whose authored conflict was resolved by taking the receiving side
    over the carried change exits 1 and names the carried lines that were
    lost; the same merge resolved by keeping both sides exits 0.
14. A rebase whose replayed commit is carried unchanged onto a base that edited
    nearby lines of the same file exits 0: context is not compared.
15. The verb never continues, commits or aborts the operation: after every
    test above, git still reports the operation in progress where it was.
16. `--json` emits the family envelope with the 3.7 report keys.
17. `spec-spine capabilities` lists the verb.

## 6. Out of scope

- Any resolution of an authored file. A spec text conflict is the author's.
- Driving the git operation in any way (3.5, last paragraph).
- Changing, distributing or retiring spec 094's merge driver.
- Regenerating outside a stopped operation; `compile` and `index` do that.
- Restacking pull requests. A tool that restacks calls this verb at the stop.
- Retiring statecraft-cli's `scripts/resolve-shard-conflicts.sh`. That is
  statecraft-cli's change to make once this verb ships.

## 7. Resolved decisions

**D-1 (2026-10-27): one top-level verb, not a flag on `compile` or `check`.**
`compile` writes one tree and `check` writes nothing; this verb writes both
trees and the index. A flag on either would give an existing verb a second
contract, and spec 062 3.2 kept each primitive's contract intact for exactly
that reason.

**D-2 (2026-10-27): refuse untracked and unstaged paths rather than stash
them.** Stashing would let the verb succeed and would change the caller's
working tree behind their back. Refusing names what the regenerated tree would
otherwise describe, and the caller decides.

**D-3 (2026-10-27): neither side is "taken".** Regeneration supersedes both
sides, so the verb never writes ours or theirs into the derived root, even as
an intermediate step. That also makes the result independent of which side a
rebase calls ours. The reference implementation takes the incoming copy before
refreshing; the result is the same, because the refresh overwrites it.

**D-4 (2026-10-27): the authored-change check is kept from the reference
implementation, and widened from merges to rebases and cherry-picks.** It is
the only check that catches failure mode 4, and it is cheap: two diffs of the
staged tree. A rebase or cherry-pick has a carried change as well defined as a
merge's (the commit being applied), so the same comparison applies with the
sides named in 3.2 item 3.

## Verification

Each line below is one command (spec 043 3.2). Until the build, these fail:
the verb and its test file do not exist yet.

```verify:cli
cargo build --release -p spec-spine-cli --locked
# 3.1 to 3.7: every acceptance criterion in a scratch repository.
cargo test -p spec-spine-cli --locked --test resolve_derived
# 17: the catalog lists the verb.
sh -c 'target/release/spec-spine capabilities | grep -qF resolve-derived'
```
