---
id: "153-a-test-count-is-exact"
title: "A test count is exact"
status: draft
kind: "test"
created: "2026-09-25"
summary: >
  Thirty-six live acceptance lines, in seventeen blocks that eighty specs'
  plans resolve to, accept `test result: ok. [1-9][0-9]* passed`: they pass
  while any one test in the target runs, so deleting, renaming, ignoring or
  platform-gating most of a test file still passes. This spec states the rule
  (no live plan accepts a loose count) and asserts it corpus-wide through
  `verify <id> --plan`. It holds no block itself: each of the seventeen is
  tightened by its own spec, in the form spec 151 gave 144, so no single spec
  becomes the acceptance of eighty. This spec is built last, when its rule
  holds.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "151-carried-acceptance-tests-what-it-names"
constrains:
  # 3.1: the rule every live acceptance block must meet; each block is
  # tightened by its own spec (3.2).
  - { kind: acceptance-invariant, target_specs: ["082-an-amended-acceptance-is-the-one-that-runs", "084-an-acceptance-outlives-the-output-it-was-written-against", "085-a-version-pin-is-not-a-contract", "086-an-exact-key-set-refuses-what-the-rule-allows", "088-the-template-teaches-the-whole-grammar", "092-the-engine-ships-governance-not-an-environment", "093-the-harness-this-repository-runs", "096-compaction-is-a-verb-not-a-session", "097-a-path-leaves-the-corpus-the-way-a-spec-does", "098-a-citation-the-renumber-could-not-see", "099-a-merged-acceptance-is-asked-again", "102-a-ready-spec-carries-its-status", "137-the-legacy-ledger-is-paid-index-and-lint", "138-the-legacy-ledger-is-paid-coupling-and-freshness", "149-the-installer-is-tested", "151-carried-acceptance-tests-what-it-names", "156-statecraft-profile-10-governs-this-repository"], note: "no live plan accepts an open-ended positive test count" }
references:
  - { unit: { kind: file, path: "specs/151-carried-acceptance-tests-what-it-names/spec.md" }, role: "the exact form every tightened line takes" }
obligations:
  - { id: "R-1", kind: requirement, text: "No command that verify <id> --plan prints, for any id in the corpus, accepts an open-ended positive test count.", anchor: "3-1-the-rule" }
  - { id: "R-2", kind: requirement, text: "Each live block with a loose count is tightened by its own spec, which holds that block alone and names the tests each loose line ran.", anchor: "3-2-one-spec-per-block" }
intent:
  goal: "Make every live acceptance line that runs cargo test fail when the tests it names stop running, without making one spec the acceptance of eighty."
  non_goals:
    - "Narrowing what any filtered line selects (4.2)."
    - "Changing the sweep (4.3)."
---

# 153: A test count is exact

## 1. Purpose

Re-measured on `cbca1fa7` (`origin/main`, 0.28.0 plus 155 to 157 and 168's
draft), 2026-09-29, through the CLI only: `registry list --ids-only`,
`registry show <id> --json` for every holder, and `verify <id> --plan` for
each of the 158 ids. The first measurement (`4818eb50`, 2026-09-25) is
superseded where this section differs: since then #396 made 149 hold 139, and
156 holds 091, 094, 124, 134 and 135.

Spec 151 §1.2 made the argument for one line: an acceptance that accepts
`test result: ok. [1-9][0-9]* passed` passes while any single test in its
target runs, so a build that deletes, renames, `#[ignore]`s or gates to another
platform every test but one still passes it. 151 fixed 144's line and left the
rest to this spec (151 §4).

### 1.1 Where the pattern is

A line is **live** when it sits in the `verify:cli` block that
`verify <id> --plan` runs for some spec. A spec whose block another spec holds
(`amends_verification`) has a **dead** block: its lines run only where the
holder carries them.

- 36 lines are live, in 17 blocks (1.2).
- 7 lines are dead: 087 (held by 102), 091's three and 094's one (held by
  156, which carried 091's as four loose lines of its own and 094's as a
  bare `cargo test` of the target), 136 (held by 151, carried verbatim), 139 (held by 149, carried),
  144 (held by 151, replaced by an exact line).
- 151:61 and 151:131 are prose, outside any block.

### 1.2 The live lines

"Count" is `test result: ok. N passed` from running the line's command on
`4818eb50` (macOS, default features); the three rows new since then are
measured by the spec that tightens them. "Whole" means the line runs every
test in the target; otherwise the column names the substring filter.
"Plans" are the specs whose `verify --plan` runs the block, on `cbca1fa7`.

| Block:line | Target | Filter | Count | Plans |
|---|---|---|---|---|
| 082:414 | core `verify` | `superseded` | 1 | 2 |
| 084:410 | core `verify` | `spec103_` | 5 | 2 |
| 085:384 | core `verify` | `spec103_` | 5 | 2 |
| 086:473 | core `verify` | `spec103_` | 5 | 2 |
| 088:315 | types `dogfood` | `authoring_template` | 1 | 1 |
| 092:842 | core `scaffold` | whole | 21 | 9 |
| 092:884 | core `index` | `statecraft` | 2 | as 092:842 |
| 092:886 | core `couple` | `derived` | 3 | as 092:842 |
| 092:891 | cli `cli` | `statecraft_derived` | 1 | as 092:842 |
| 093:680 | core `harness_hooks` | whole | 55 | 1 |
| 093:684 | core `harness_skills` | whole | 21 | 1 |
| 093:690 | core `harness_hooks` | `pr_gate_` | 15 | 1 |
| 096:325 | core `compact` | whole | 38 | 1 |
| 096:329 | core `compact` | `collision` | 1 | 1 |
| 096:333 | core `compact` | `short_id` | 4 | 1 |
| 096:336 | core `compact` | `idempotent` | 2 | 1 |
| 096:339 | core `compact` | `broken_across` | 2 | 1 |
| 096:342 | core `compact` | `more_than_one_ordinal` | 1 | 1 |
| 096:346 | core `compact` | `fence` | 2 | 1 |
| 096:353 | core `compact` | `report` | 2 | 1 |
| 097:661 | core `retire` | whole | 62 | 1 |
| 098:461 | core `compact` | whole | 38 | 1 |
| 098:464 | core `lint` | whole | 39 | 1 |
| 099:397 | core `gate` | `acceptance` | 5 | 1 |
| 102:308 | core `verify` | `spec103_` | 5 | 3 |
| 137:135 | core `lint` | whole | 39 | 11 |
| 137:137 | core `deferred_contract` | whole | 5 | as 137:135 |
| 137:140 | core `index` | whole | 57 (51 without `symbol-resolution`) | as 137:135 |
| 138:135 | core `couple` | whole | 53 | 11 |
| 138:148 | core `coverage` | whole | 30 (29 without `symbol-resolution`) | as 138:135 |
| 149:178 | core `attest` | whole | 30 (28 off Unix) | 12 |
| 151:157 | core `compile` | whole | 51 | 14 |
| 156:398 | core `ai_review_policy` | `access_refusal` | at its build | 6 |
| 156:399 | core `ai_review_policy` | `publication_failure` | at its build | as 156:398 |
| 156:400 | core `ai_review_policy` | `unclassified_reviewer_failure` | at its build | as 156:398 |
| 156:401 | core `ai_review_policy` | `empty_successful_review` | at its build | as 156:398 |

The 17 blocks reach 80 plans.

### 1.3 What one holder would cost

A spec that holds a block carries it whole, including the sections that block
already carries for the specs it holds, and `compile` refuses a second holder
of one target (`V-019`). One spec fixing all 36 lines would hold all 17
blocks, about 594 plan lines on `cbca1fa7`, and become the plan of 81 specs:
about 48,000 plan lines against the corpus's 4,353 today, about 11 times, and
about 7 minutes for every `verify <id>` among the 81. Every later change to
one of those 81 specs' acceptance would have to hold that spec and carry its
whole block. One spec per block adds about one block's size per block, about
600 plan lines in all (D-5).

## 2. Territory

This spec establishes nothing and holds nothing. It `constrains` the 17
specs whose blocks carry a live loose line, a spec-scoped invariant (spec 017)
that claims no code. Its only surface is its own Verification block, which
reads the corpus through the release build.

## 3. Behavior

### 3.1 The rule

No live plan accepts a loose test count. For every id `registry list
--ids-only` names, the commands `verify <id> --plan` prints MUST NOT contain
`[1-9][0-9]* passed`.

### 3.2 One spec per block

Each of the 17 live blocks in 1.2 MUST be tightened by its own spec, which:

- holds exactly that block under `amends_verification` (and `amends`), and no
  other block with a loose line;
- carries it in full, byte-identical, except for one change per loose line:
  the line becomes one command in spec 151's form,
  `sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
  where `<names>` are the tests the original line ran, measured at that
  spec's base, and N is their number; a writer-and-`grep` pair collapses into
  that one command, and the pair's `rm -f` stays;
- names the original selection, not a narrowed one: a filtered line names
  what its filter selects, a whole-target line every test in the target (D-6);
- adds spec 082 §3.4's superseded-acceptance note above the held block, and
  keeps that block unchanged below it;
- is filed and built in one pull request (D-3).

Where a held block is itself a holder (092, 102, 137, 138, 149, 151, 156), the
tightening spec carries the sections it carries and updates the superseded
notes of the specs it holds, as 154 did for 146.

### 3.3 Platform-bound counts

Four targets hold tests behind `cfg`: `gate`, `attest`, and `index` and
`coverage` behind the default `symbol-resolution` feature. Each tightened line
is exact on a Unix sweep host with default features, which is the host
`verify` and the sweep run on, and says so in a comment, as 151 did for
`repo_path`. Spec 148 owns Windows.

### 3.4 This spec is built last

This spec's block asserts 3.1 alone. It fails until the last of the 17
tightening specs merges, so its build is the flip to `complete` after that
merge, with no other change.

## 4. Out of scope

### 4.1 Other weak count forms

A fixed count with a substring filter and no names (for example 122's
`test result: ok\. 6 passed`) is exact and stays. So is 149's
`install.sh: 7 of 7 cases passed`. Only the open-ended `[1-9][0-9]*` form is
this spec's.

### 4.2 Narrowing a selection

Whether a filtered line should name fewer tests than its filter selects (096's
`short_id`, 093's whole `harness_hooks` next to its `pr_gate_` subset) is a
question about what each spec requires, and is not this spec's to change.

### 4.3 The sweep

How often the sweep runs one block that several specs share is spec 089's,
150's and 157's; this spec changes no script.

## 5. Resolved decisions

**D-1 (2026-09-25): hold the physical holder, not the dead block.** A dead
block is held already, and a second holder is `V-019`. The tightening spec for
a dead block's lines holds the block that carries them (1.1).

**D-2 (2026-09-25): 151's `[1-9]` line is 136's.** 151:157 is 136's line,
carried verbatim. Its tightening spec holds 151 and tightens the line inside
136's section.

**D-3 (2026-09-29, owner): this draft merges alone; each tightening spec is
filed and built together.** This spec holds nothing, so merging it as a draft
replaces no plan. A tightening spec's `amends_verification` replaces its
block's plans the moment it merges, so it lands with its carried block, as 146
and 151 did.

**D-4 (2026-09-29, owner): hold the current holders.** #396 merged first, so
the tightening spec for 139's line holds 149 and carries 149's block (006's and
139's sections). 156 holds 091 and 094, so the tightening spec for 156's four
lines holds 156 and carries its whole block (091, 094, 124, 134, 135).

**D-5 (2026-09-29, owner): one spec per block, not one holder.** The first
draft proposed one spec holding every block (1.3). The owner chose one
tightening spec per live block: about 600 added plan lines instead of about
44,000, a `verify <id>` that stays the size of one block, and a held block that
a later spec can change without carrying sixteen others. The cost is 17 small
specs, which may land in a few pull requests grouped by subject.

**D-6 (2026-09-29, owner): names, not a bare count.** A bare exact count
(`ok. 62 passed`) also fails when a test is added, which turns a block's plans
red on every new test in a shared file. Names fail on deletion, rename,
`#[ignore]` and gating, and not on addition. Whole-target lines run up to 62
names.

**D-7 (2026-09-25): the rule reads the plan beside the running spec.** `verify`
refuses a nested call on an id already on `SPEC_SPINE_VERIFY_STACK` before it
honours `--plan` (spec 083 D-4). The rule's loop skips those ids; nothing is
lost, because this spec holds no block, so the only id on the stack is its
own, whose plan is this block.

**D-8 (2026-09-29): the rule fails on this tree.** With 36 live loose lines on
`cbca1fa7`, 3.1 fails for 80 ids before any tightening spec lands, so the block
is not vacuous.

## Verification

```verify:cli
# Self-contained: the rule reads plans through the release build.
cargo build --release --locked
# 3.1: no live plan accepts a loose count. Ids on the verify stack are skipped:
# `verify` refuses them before honouring --plan, and their plan is this block (D-7).
sh -c 'B="$PWD/target/release/spec-spine"; P="$(mktemp)" || exit 1; ids="$("$B" registry list --ids-only)" || { echo "registry list exited $?"; rm -f "$P"; exit 1; }; [ -n "$ids" ] || { echo "registry list named no ids"; rm -f "$P"; exit 1; }; r=0; for id in $ids; do case ",${SPEC_SPINE_VERIFY_STACK:-}," in *",$id,"*) continue ;; esac; "$B" verify "$id" --plan > "$P" 2>&1 || { echo "$id: verify --plan exited $?"; r=1; continue; }; if grep -q "[[]1-9[]][[]0-9[]][*] passed" "$P"; then echo "$id: its plan accepts any positive test count"; r=1; fi; done; rm -f "$P"; exit $r'
```
