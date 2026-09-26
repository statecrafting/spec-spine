---
id: "153-a-test-count-is-exact"
title: "A test count is exact"
status: draft
kind: "test"
created: "2026-09-25"
summary: >
  Thirty-six live acceptance lines, in eighteen approved specs' blocks that
  seventy-four specs' plans resolve to, accept `test result: ok. [1-9][0-9]*
  passed`: they pass while any one test in the target runs, so deleting,
  renaming, ignoring or platform-gating most of a test file still passes. This
  spec holds those eighteen blocks under `amends_verification`, carries each in
  full, and replaces every loose line with the form spec 151 gave 144: the
  tests the line ran, named with `--exact`, and `N passed; 0 failed` where N is
  the number of names. Its block asserts the rule corpus-wide, through
  `verify <id> --plan` for every spec, so a loose count that comes back is
  refused wherever it lands.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
  - "085-a-version-pin-is-not-a-contract"
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
  - "088-the-template-teaches-the-whole-grammar"
  - "091-an-unclassified-review-failure-blocks-the-merge"
  - "092-the-engine-ships-governance-not-an-environment"
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "096-compaction-is-a-verb-not-a-session"
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
  - "098-a-citation-the-renumber-could-not-see"
  - "099-a-merged-acceptance-is-asked-again"
  - "102-a-ready-spec-carries-its-status"
  - "137-the-legacy-ledger-is-paid-index-and-lint"
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
  - "085-a-version-pin-is-not-a-contract"
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
  - "088-the-template-teaches-the-whole-grammar"
  - "091-an-unclassified-review-failure-blocks-the-merge"
  - "092-the-engine-ships-governance-not-an-environment"
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "096-compaction-is-a-verb-not-a-session"
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
  - "098-a-citation-the-renumber-could-not-see"
  - "099-a-merged-acceptance-is-asked-again"
  - "102-a-ready-spec-carries-its-status"
  - "137-the-legacy-ledger-is-paid-index-and-lint"
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
  - "151-carried-acceptance-tests-what-it-names"
amends:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
  - "085-a-version-pin-is-not-a-contract"
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
  - "088-the-template-teaches-the-whole-grammar"
  - "091-an-unclassified-review-failure-blocks-the-merge"
  - "092-the-engine-ships-governance-not-an-environment"
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "096-compaction-is-a-verb-not-a-session"
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
  - "098-a-citation-the-renumber-could-not-see"
  - "099-a-merged-acceptance-is-asked-again"
  - "102-a-ready-spec-carries-its-status"
  - "137-the-legacy-ledger-is-paid-index-and-lint"
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
  - "151-carried-acceptance-tests-what-it-names"
---

# 153: A test count is exact

## 1. Purpose

Measured on `4818eb50` (`origin/main`, 0.27.0 plus 148), 2026-09-25, through
the CLI only: `registry list --ids-only`, then `verify <id> --plan --json` for
each of the 153 ids, and `cargo test` of every target a live line runs.

Spec 151 §1.2 made the argument for one line: an acceptance that accepts
`test result: ok. [1-9][0-9]* passed` passes while any single test in its
target runs, so a build that deletes, renames, `#[ignore]`s or gates to another
platform every test but one still passes it. 151 fixed 144's line and left the
rest to this spec (151 §4).

### 1.1 Where the pattern is

42 lines in 21 spec files match `[1-9][0-9]* passed`. A line is **live** when
it sits in the `verify:cli` block that `verify <id> --plan` runs for some spec.
A spec whose block another spec holds (`amends_verification`) has a **dead**
block: its lines run only where the holder carries them.

- 36 lines are live, in 18 blocks (1.2).
- 3 lines are dead (1.3).
- 3 lines are prose, outside any block: 087:397, 151:61, 151:131.

### 1.2 The live lines

"Count" is `test result: ok. N passed` from running the line's command on
`4818eb50` (macOS, default features, 0 ignored, 0 failed in every target).
"Whole" means the line runs every test in the target; otherwise the column
names the substring filter it passes. "Plans" are the specs whose `verify
--plan` runs the block.

| Block:line | Target | Filter | Count | Plans |
|---|---|---|---|---|
| 082:414 | core `verify` | `superseded` | 1 | 082, 074 |
| 084:410 | core `verify` | `spec103_` | 5 | 084, 044 |
| 085:384 | core `verify` | `spec103_` | 5 | 085, 049 |
| 086:473 | core `verify` | `spec103_` | 5 | 086, 052 |
| 088:315 | types `dogfood` | `authoring_template` | 1 | 088 |
| 091:749 | core `ai_review_policy` | `inversion` | 1 | 091 |
| 091:750 | core `ai_review_policy` | `publication_failure` | 1 | 091 |
| 091:751 | core `ai_review_policy` | `post_step_condition` | 1 | 091 |
| 092:842 | core `scaffold` | whole | 21 | 092, 054, 055, 056, 058, 062, 064, 079, 083 |
| 092:884 | core `index` | `statecraft` | 2 | as 092:842 |
| 092:886 | core `couple` | `derived` | 3 | as 092:842 |
| 092:891 | cli `cli` | `statecraft_derived` | 1 | as 092:842 |
| 093:680 | core `harness_hooks` | whole | 55 | 093 |
| 093:684 | core `harness_skills` | whole | 21 | 093 |
| 093:690 | core `harness_hooks` | `pr_gate_` | 15 | 093 |
| 094:359 | core `gate` | whole | 27 (26 off Unix) | 094 |
| 096:325 | core `compact` | whole | 38 | 096 |
| 096:329 | core `compact` | `collision` | 1 | 096 |
| 096:333 | core `compact` | `short_id` | 4 | 096 |
| 096:336 | core `compact` | `idempotent` | 2 | 096 |
| 096:339 | core `compact` | `broken_across` | 2 | 096 |
| 096:342 | core `compact` | `more_than_one_ordinal` | 1 | 096 |
| 096:346 | core `compact` | `fence` | 2 | 096 |
| 096:353 | core `compact` | `report` | 2 | 096 |
| 097:661 | core `retire` | whole | 62 | 097 |
| 098:461 | core `compact` | whole | 38 | 098 |
| 098:464 | core `lint` | whole | 39 | 098 |
| 099:397 | core `gate` | `acceptance` | 5 | 099 |
| 102:308 | core `verify` | `spec103_` | 5 | 102, 087, 053 |
| 137:135 | core `lint` | whole | 39 | 137 and the 10 it holds |
| 137:137 | core `deferred_contract` | whole | 5 | as 137:135 |
| 137:140 | core `index` | whole | 57 (51 without `symbol-resolution`) | as 137:135 |
| 138:135 | core `couple` | whole | 53 | 138 and the 10 it holds |
| 138:148 | core `coverage` | whole | 30 (29 without `symbol-resolution`) | as 138:135 |
| 139:161 | core `attest` | whole | 30 (28 off Unix) | 139 and the 9 it holds |
| 151:157 | core `compile` | whole | 51 | 151, 136 and the 10 it holds, 144, 145 |

The 18 blocks reach 74 specs' plans. 26 of the 36 lines are two commands, a
`cargo test ... > "${TMPDIR:-/tmp}/<file>" 2>&1` writer and a `grep` of that
file; the other 10 (091's three, 137's three, 138's two, 139's and 151's) run
`cargo test` inline.
No captured file is read by anything but its writer, its loose `grep` and an
`rm -f`.

The names each filtered line selects on `4818eb50` (`-- --list` with the
line's filter):

- `verify superseded`: `every_superseded_verification_block_says_so_in_its_own_document`.
- `verify spec103_`: `spec103_a_cycle_does_not_hang_verify`,
  `spec103_a_spec_with_no_amender_runs_its_own_block`,
  `spec103_a_withdrawn_amender_does_not_hold_the_acceptance`,
  `spec103_resolution_follows_the_chain`,
  `spec103_verify_runs_the_replacement_block`.
- `dogfood authoring_template`: `authoring_template_documents_every_frontmatter_key`.
- `ai_review_policy`: `inversion_of_the_refusal_branch_is_detected`;
  `publication_failure_fails_the_job_even_when_classification_succeeded`;
  `the_post_step_condition_refuses_an_empty_review_the_classifier_let_through`.
- `index statecraft`: `statecraft_derived_and_state_are_pruned_and_the_rest_is_governed`,
  `statecraft_derived_matching_is_separator_aware`.
- `couple derived`: `a_regenerated_shard_at_the_configured_derived_root_is_not_drift`,
  `the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym`,
  `the_effective_bypass_list_reports_the_configured_derived_root`.
- `cli statecraft_derived`: `statecraft_derived_layout_compiles_indexes_and_is_judged`.
- `harness_hooks pr_gate_`: the 15 tests whose names contain `pr_gate_`
  (`the_pr_gate_*` and `spec123_the_pr_gate_*`, `spec132_the_pr_gate_*`).
- `compact`: `collision` is `an_ordinal_collision_is_refused_and_names_both`;
  `short_id` is `a_bare_short_id_in_a_command_is_rewritten`,
  `a_short_id_after_a_flag_is_still_the_commands_argument`,
  `a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone`,
  `the_sweeps_only_argument_is_a_short_id`; `idempotent` is
  `applying_the_output_to_the_output_is_idempotent`,
  `the_new_forms_are_idempotent`; `broken_across` is
  `a_citation_broken_across_a_line_is_rewritten`,
  `a_citation_broken_across_two_blank_lines_is_not_a_citation`;
  `more_than_one_ordinal` is
  `a_citation_naming_more_than_one_ordinal_rewrites_every_one`; `fence` is
  `a_block_that_names_its_own_fence_is_located_whole`,
  `a_fence_inside_a_block_does_not_open_a_second_one`; `report` is
  `the_report_names_the_form_behind_every_rewrite_and_counts_per_form`,
  `the_report_of_a_plan_that_changes_nothing_is_empty_not_absent`.
- `gate acceptance`: the five `the_acceptance_workflow_*` tests.

A whole-target line's names are every test `cargo test -p <crate> --test
<target> -- --list` prints: 13 targets, 489 names on `4818eb50`. The build
re-measures them at its own base (3.2).

### 1.3 The dead lines

| Block:line | Why dead | Live copy |
|---|---|---|
| 087:516 | 102 holds 087 | 102:308, carried with the capture file renamed |
| 136:139 | 151 holds 136 | 151:157, carried verbatim |
| 144:154 | 151 holds 144 | none: 151 replaced it with an exact line (151 §3.2) |

### 1.4 What the fix costs

A spec that holds a block carries it whole, including the sections that
block already carries for the specs it holds, and `compile` refuses a second
holder of one target (`V-019`). So one spec fixing all 36 lines holds all 18
blocks, and its block becomes the plan of 75 specs: the 74 of 1.2 and itself.
The 18 blocks are 561 commands on `4818eb50`. On the pre-release sweep of
`d78fb09a` (before 151), 17 of them (527 commands) took 386 s together, and
the whole sweep ran 3,525 commands in 3,519 s. On `4818eb50` the 153 plans
total 3,777 commands, 2,447 of them in the 74 plans; after this spec each of
the 75 runs about 565, so a full sweep runs about 43,700 commands, about 11
times as many, or about 9 h at `d78fb09a`'s rate.

## 2. Territory

This spec establishes nothing. It edits each of the 18 amended specs only to
add spec 082 §3.4's superseded-acceptance note above that spec's block.

## 3. Behavior

### 3.1 The rule

No live plan accepts a loose test count. For every id `registry list
--ids-only` names, the commands `verify <id> --plan` prints MUST NOT contain
`[1-9][0-9]* passed`.

### 3.2 Every live line names its tests

`spec-spine verify` for each of the 18 amended specs, and for every spec one of
them holds, MUST run this spec's block. It carries each amended block in full,
in its own `# ---- carried for <id> (amends_verification) ----` section, with
exactly one change per loose line:

- The line becomes one command in spec 151's form:
  `sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
  where `<names>` are the tests the original line ran, measured at the build's
  base, and N is their number.
- A writer-and-`grep` pair collapses into that one command. The writer goes;
  the pair's `rm -f` stays, unchanged, so a carried section differs from its
  source only in the lines this section names.
- The names are the original selection, not a narrowed one: a filtered line
  names what its filter selects, and a whole-target line names every test in
  the target. 096's `short_id` line names all four tests its filter selects
  today (4.2).

Every other command, comment and section is carried byte-identical.

### 3.3 Platform-bound counts

Three targets hold tests behind `cfg`: `gate` (one `#[cfg(unix)]`, 094:359),
`attest` (two, 139:161) and `index`/`coverage` (six and one behind the default
`symbol-resolution` feature, 137:140 and 138:148). Each line is exact on a Unix
sweep host with default features, which is the host `verify` and the sweep run
on; each says so in a comment, as 151 did for `repo_path`. None cannot be made
exact. Spec 148 owns Windows.

### 3.4 The amended specs say so

Each of the 18 carries spec 082 §3.4's note naming this spec above its own
block, and keeps its block unchanged below it, loose lines included.

### 3.5 The rule's own block

This spec's own lines (the Verification block below) come first, before the
carried sections: the rule (3.1), then a check that every amended block is
carried in full with one exact replacement per loose line, that each
replacement's N equals its number of names, that each amended file still holds
its superseded line, and that each carries the note (3.2, 3.4).

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

How often the sweep runs one block that many specs share (1.4) is spec 089's
and 150's; this spec changes no script.

## 5. Resolved decisions

**D-1 (2026-09-25): hold the physical holder, not the dead block.** 087, 136
and 144 are not amended: their blocks are held already (1.3), and a second
holder is `V-019`. 102 and 151 are amended instead, and carry them.

**D-2 (2026-09-25): 151's `[1-9]` line is 136's.** 151:157 is 136's line,
carried verbatim. This spec holds 151, so it carries 151's block (136's, 144's
and 145's sections included) and tightens that line inside 136's section.

**D-3 (2026-09-25, open, owner): filed and built together.** A draft that
declares `amends_verification` replaces 74 plans the moment it merges, so this
draft's block, which carries no section yet, would leave those specs without
their acceptance. Recommended default: as 146 and 151 were, the spec is filed
and built in one pull request and this draft is never merged alone. The
alternative is to merge the draft without the three edges and add them in the
build.

**D-4 (2026-09-25, open, owner): spec 149 and block 139.** PR #396 (head
`a93e682a`, unmerged) makes 149 hold 139 and carry 139's block, attest line
included (149:178 on that branch). On `origin/main` 149 holds only 006, so this
draft holds 139. Whichever merges second retargets:

- 149 first (recommended default): the build replaces 139 by 149 in all three
  edge lists and carries 149's block (006's and 139's sections), tightening the
  attest line inside 139's section. Holding 139 as well would be `V-019`.
- 153 first: 149 can no longer hold 139; to change 139's acceptance it must
  hold 153 and carry this spec's whole block.

The block below reads the edge list from the registry, so it needs no edit
either way.

**D-5 (2026-09-25, open, owner): one block for 75 plans.** The owner adopted
one follow-up spec before the cost in 1.4 was measured: every one of 75 specs
would run about 565 commands (about 7 min at `d78fb09a`'s rate, against the
sweep's 900 s per-spec limit), and a full sweep would run about 11 times
today's commands, about 9 h (1.4). Options: (a) one spec, as adopted, cost accepted;
(b) one spec, plus a separate spec letting the sweep run each distinct
effective plan once, which already repeats 136 to 139's blocks ten times each
today; (c) one spec per holder group. Recommended default: (b), with the sweep
spec filed and merged first; (c) if the owner declines to change the sweep.

**D-6 (2026-09-25, open, owner): names, not a bare count, for whole-target
lines.** A bare exact count (`ok. 62 passed`) is shorter and also fails when a
test is added, which makes every new test in a shared file (`lint`, `index`,
`couple`, `compile`, `retire`, `gate`) a red plan for 75 specs until another
spec re-holds this block. Names fail on deletion, rename, `#[ignore]` and
gating, and not on addition. Recommended default: names everywhere (3.2), at
the cost of whole-target lines up to 62 names long.

**D-7 (2026-09-25): the rule reads the plan beside the running spec.** `verify`
refuses a nested call on an id already on `SPEC_SPINE_VERIFY_STACK` before it
honours `--plan` (spec 083 D-4). The rule's loop skips those ids; nothing is
lost, because the running spec's plan is this block, which it reads through
any other id that resolves here.

**D-8 (2026-09-25): the rule alone cannot fail the draft.** On a branch where
this draft's edges stand, every one of the 74 plans resolves to this short
block, so 3.1 holds vacuously. The carried-in-full check (3.5) is what fails
until the build.

## Verification

```verify:cli
# Self-contained: the rule reads plans through the release build.
cargo build --release --locked
# 3.1: no live plan accepts a loose count. Ids on the verify stack are skipped:
# `verify` refuses them before honouring --plan, and their plan is this block (D-7).
sh -c 'B="$PWD/target/release/spec-spine"; P="${TMPDIR:-/tmp}/ss153.plan"; r=0; for id in $("$B" registry list --ids-only); do case ",${SPEC_SPINE_VERIFY_STACK:-}," in *",$id,"*) continue ;; esac; "$B" verify "$id" --plan > "$P" 2>&1 || { echo "$id: verify --plan exited $?"; r=1; continue; }; if grep -q "[[]1-9[]][[]0-9[]][*] passed" "$P"; then echo "$id: its plan accepts any positive test count"; r=1; fi; done; rm -f "$P"; exit $r'
# 3.2, 3.4, 3.5: every amended block is carried in full, each loose line has
# its own exact replacement on the same target, every exact line's count is
# its number of names, each amended file keeps its superseded line and carries
# the note. The amended list is read from the registry (D-4).
python3 -c 'import re,json,subprocess,collections as C; me="153-a-test-count-is-exact"; L=re.compile(r"\[1-9\]\[0-9\]\* passed"); T=re.compile(r"cargo[ ]test[ ]-p[ ](\S+) .*?--test[ ](\S+)"); W=re.compile(r"^cargo[ ]test[ ]"); F=re.compile(r"TMPDIR:-/tmp\}/([^\x22]+)"); sec=lambda i: re.split(r"\n## (?:\d+\. )?Verification\n",open("specs/%s/spec.md"%i).read(),maxsplit=1)[1].split("\n## ",1)[0]; blk=lambda i: [t for f in re.findall(r"```verify:cli\n(.*?)\n```",sec(i),re.S) for t in (x.strip() for x in f.split("\n")) if t and not t.startswith("#")]; av=json.loads(subprocess.run(["target/release/spec-spine","registry","show",me,"--json"],capture_output=True,text=True,check=True).stdout)["amendsVerification"]; mine=blk(me); ex=lambda cs: [m for m in cs if T.search(m) and "-- --exact" in m]; tg=lambda m: T.search(m).groups(); held={h: blk(h) for h in av}; loose={h: [c for c in held[h] if L.search(c)] for h in av}; wr={h: [c for c in held[h] if W.search(c) and any(f in c for l in loose[h] for f in F.findall(l))] for h in av}; lt=[tg(c) if T.search(c) else tg([w for w in wr[h] if any(f in w for f in F.findall(c))][0]) for h in av for c in loose[h]]; missing=list((C.Counter(c for h in av for c in held[h] if c not in loose[h] and c not in wr[h])-C.Counter(mine)).items()); need=C.Counter([tg(m) for h in av for m in ex(held[h])]+lt); have=C.Counter(tg(m) for m in ex(mine)); short=[(t,n,have[t]) for t,n in need.items() if have[t]<n]; n=lambda m: re.search(r"ok\. (\d+) passed; 0 failed",m); bad=[m for m in ex(mine) if not n(m) or int(n(m).group(1))!=len(m.split("-- --exact",1)[1].split("2>&1")[0].split())]; kept=[h for h in av if not loose[h]]; notes=[h for h in av if ("> `%s` declares this spec in\n"%me) not in open("specs/%s/spec.md"%h).read()]; [print("not carried (%d short):"%k,c[:160]) for c,k in missing]; [print("no exact replacement:",t,"needs",k,"has",v) for t,k,v in short]; [print("count is not the number of names:",m[:160]) for m in bad]; [print("amended block lost its superseded line:",h) for h in kept]; [print("no superseded-acceptance note:",h) for h in notes]; raise SystemExit(1 if (missing or short or bad or kept or notes or not av) else 0)'
# 3.4: the corpus-wide note test holds with the 18 notes added.
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact every_superseded_verification_block_says_so_in_its_own_document 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
```
