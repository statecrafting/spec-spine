---
id: "105-an-amendment-carries-the-acceptance-it-replaces"
title: "An amendment carries the acceptance it replaces"
status: approved
kind: "core"
created: "2026-09-16"
summary: >
  Spec 101 amended spec 098 so that an unresolved claim exits 1 rather than 2,
  and migrated the four assertions that live in Rust. Spec 098's own
  `## Verification` block was left as it was ratified, and five of its
  assertions still require the pre-amendment code, so `spec-spine verify 098`
  has been red since 101 merged. A sixth line is red for an unrelated reason:
  098's AC-9 compares two `settings.json` files against a fixed commit to prove
  098 did not edit them, which specs 099 and 104 then legitimately did. Spec 103
  built the route an amendment needs to reach an acceptance block and landed
  after 101, naming the audit of the remaining blocks as its own work. This is
  that work for 098: this spec declares 098's acceptance replaced and carries
  the corrected block, without editing 098's file.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "098-a-blocking-claim-is-not-a-stale-shard"
  - "101-an-unresolved-claim-is-not-stale"
  - "103-an-amended-acceptance-is-the-one-that-runs"
amends: ["098-a-blocking-claim-is-not-a-stale-shard"]
# 3.1: this spec's `## Verification` block IS 098's acceptance from now on.
# 098's own file is not edited (spec 040 3.1), and 101's is not either: 101
# states a rule that is true and complete, and this spec changes none of it.
amends_verification: ["098-a-blocking-claim-is-not-a-stale-shard"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 105: An amendment carries the acceptance it replaces

## 1. Purpose

### 1.1 Two defects in one block, neither reachable by the gate

Measured on 2026-09-16 against `4d14cce` with the 0.20.0 binary:

```
$ spec-spine verify 098
verify: 098-a-blocking-claim-is-not-a-stale-shard: FAILED at command 9 (exit 1)
```

Command 9 is `check >/dev/null 2>&1; test $? -eq 2` against a fixture corpus
holding one spec that claims `src/nothing.rs` while declaring itself complete.
The verb answers 1, and answering 1 is what spec 101 requires:

```
$ spec-spine --repo "$TMPDIR/ss098" check ; echo $?
codebase-index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1 spec(s), which is not staleness
1
```

Five assertions in the block require the code 101 amended away. Measured one
state at a time, with the fixture built exactly as the block builds it:

| Fixture state | 098 requires | The verb answers |
|---|---|---|
| blocking claim, shards byte-exact, `check` | 2 | **1** |
| the same, `index check` | 2 | **1** |
| the same, `check --json` `exitCode` | 2 | **1** |
| blocking claim after regenerating | 2 | **1** |
| blocking claim and a stale shard together | 2 | **1** |
| a stale shard alone | 2 | 2 |
| the claim satisfied, shards fresh | 0 | 0 |

The last two rows are the reason this is a replacement and not a rewrite: exit
2 is still correct where the refusal is staleness, so a blanket substitution
would break the block in the other direction. Spec 101 D-2 decided that a
blocking claim outranks a stale shard, which is what the fifth row shows.

The sixth red line is a different defect with the same shape. 098's AC-9 is

```
git diff --quiet 3bc004bf5f0fb30b9c2127c1d2269face10fb37b -- kit/settings.json .claude/settings.json
```

and its comment explains the fixed base as protection against the assertion
weakening as 098's branch aged. It does protect that, and past the merge it
asserts something 098 never required: that **no commit after 098's branch
point** touches either file. Specs 099 and 104 both edited both files, and both
were ratified. The line has been red since 099 merged and can never be green
again.

### 1.2 Why spec 101 did not fix this

Spec 101 amended 098 and said so, and it migrated every assertion it could
reach: the four cases in `crates/spec-spine-cli/tests/cli.rs`, attributed to
098 in 101's own `extends` comment. The one it could not reach was 098's
`## Verification` block, because spec 040 forbids editing the amended file and
that block lives inside it. Spec 103 built the route (`amends_verification`)
and merged two pull requests after 101. This is a gap in sequence, not in
anyone's care.

### 1.3 Why this is 103's named follow-on

Spec 103 §4 puts it plainly: auditing the other blocks for the same defect is
"worth doing and is its own work, with its own findings". This is that work,
scoped to the one block now known to be red, and the finding is that 098's
block is red for two unrelated reasons rather than one.

Neither reason is reachable by the governance gate. `verify` is the one verb
that executes what the corpus declares, so it sits outside the gate chain
deliberately (`AGENTS.md`); nothing in CI runs it. A block can be red for
months and every gate stays green, which is exactly what happened.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 098's `## Verification` block is no longer the one
that runs. Nothing under `crates/` changes, no schema constant moves, and no
committed shard changes except the two this spec's own frontmatter produces.

## 3. Behavior

### 3.1 What spec 098's acceptance now is

This spec's `## Verification` block MUST replace spec 098's in full, through
`amends_verification` (spec 103 §3.1), and 098's file MUST NOT be edited. The
block is 098's, with the five assertions of §1.1 migrated from 2 to 1, with
AC-9 replaced per §3.3, and with this spec's own assertions (§3.5) ahead of
them under a heading that says whose is whose.

Replacing the block in full rather than the five lines follows spec 103 §3.5: a
reader asking what 098 accepts today should find one block that answers, not a
base document plus a patch to apply in their head.

### 3.2 The stale-only assertion is not migrated

The `AC-3: stale only` arm MUST keep `test $? -eq 2`. Spec 101 moved one
refusal between two rungs that both already meant "refused"; it did not touch
staleness, and exit 2 remains the staleness code. An acceptance that read
`-eq 1` there would assert the opposite of what 101 and 098 both require, and
would pass only against a build that had lost the distinction the two specs
exist to draw.

### 3.3 AC-9 keeps the half that can still be true

The fixed-base `git diff` line MUST be dropped, and the two pattern checks
beside it MUST be kept:

```
grep -q 'freshness. STALE' kit/settings.json
grep -q 'freshness. STALE' .claude/settings.json
```

098's D-6 recorded a scope claim: that spec edited neither file. That claim was
true and was settled when 098 merged; a diff against a fixed base re-litigates
it on every future run, against a tree that has moved for reasons 098 has no
authority over. What 098 actually depends on is the message shape those hooks
emit, and that is what survives here, asserted against the files as they stand.

An acceptance line MUST assert a property of the code, not a property of the
commit graph. This is the same family as spec 093's calendar assertion that
spec 103 corrected, and it is recorded here so the third instance is recognised
rather than rediscovered.

### 3.4 No code changes

No file under `crates/` is edited, no `*_SCHEMA_VERSION` constant moves, no
embedded schema changes, and no CLI surface is added or removed. The mechanism
this spec uses was built and shipped by spec 103 in v0.20.0; this spec is a
corpus change that uses it.

### 3.5 The acceptance

`registry show 105 --json` MUST carry `amendsVerification` and `amends`, both
naming 098 and nothing else: that is the declaration, and it is read through
the CLI rather than off the shard.

Spec 098's file MUST still carry the superseded form of the primitive-verb
assertion (`index check`, `-eq 2`). That is the spec 040 §3.1 half, and it is
the assertion that goes red if someone ever resolves this by editing 098
instead, which is the move `.claude/rules/adversarial-prompt-refusal.md`
refuses.

The resolution itself MUST keep passing through spec 103's own cases
(`spec103_` in `crates/spec-spine-core/tests/verify.rs`). No code changes here,
so the mechanism is what is asserted, not a new one.

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 098` runs, and `cmd_verify::run` refuses a nested call on
its own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so
even reading the plan from inside is a validation failure. The instance-level
fact, that `spec-spine verify 098` and `spec-spine verify 105` both exit 0 on
the merged tree, therefore cannot be asserted from inside the block that is
itself under test. It is what a reviewer runs, and what the release sweep runs.

## 4. Out of scope

- **Editing spec 098, or spec 101.** §1.2 and the frontmatter comment. 098
  keeps the block it was ratified with, which is the record spec 040 §3.2
  protects, and 101's text is true as written.
- **The remaining blocks.** Eighteen specs were run for the v0.20.0 release and
  098 was the only one red for a reason of its own; the other eighty-seven were
  not run. Spec 103 §4 named the full audit and it stays named.
- **Running `verify` from the gate.** It executes what the corpus declares, and
  the chain runs against branches whose contents are, in the general case, a
  stranger's. That reasoning is unchanged. The gap in §1.3 is real and its
  remedy is a periodic sweep a maintainer runs, not a gate step; naming where
  that sweep belongs is its own work.
- **A validation that an amendment must declare `amends_verification`.** Not
  decidable: whether an amendment invalidates the amended spec's block is a
  question about what the block asserts, which only running it answers. Spec
  103 §4 declined the neighbouring lint for the same reason.

## 5. Resolved decisions

D-1 (2026-09-16, why a replacement rather than five corrections). Spec 103 §3.5
settled that a block is the unit, and the reasons carry: a reader who finds
098's block in 098's file and a patch in this one has to apply the patch in
their head to know what 098 accepts. A block that is read in one place is worth
the duplication.

D-2 (2026-09-16, why AC-9 loses its diff rather than gaining a new fixed base).
Moving the base to this spec's branch point would buy silence until the next
spec touches `settings.json`, which specs 089, 099 and 104 show is a file this
corpus edits often. The line would go red again and the next session would file
this spec again. An assertion that must eventually fail for a legitimate reason
is not an assertion; the pattern checks beside it assert what 098 relied on and
can still go red for a real reason.

D-3 (2026-09-16, why this amends 098 and not 101). The edge records whose
document is changed, not whose change caused it. 101's text states a rule that
is true, complete and needs nothing added; what changes here is what 098
accepts. Spec 103 §3.1 requires every `amends_verification` entry to appear in
`amends`, so the two lists name 098 and stop there.

D-4 (2026-09-16, why no `verify` command appears in the block). The first draft
of this spec asserted the substitution by running `verify 098 --plan` and
grepping it, on the reasoning that `--plan` executes nothing and so cannot
recurse. Running it disproved that: `cmd_verify::run` builds the plan, tests
the spec id against `SPEC_SPINE_VERIFY_STACK` and returns a validation error
**before** the `plan_only` branch, so the nested call is refused whichever flag
it carries. The guard is right and the assertion was wrong. What replaces it
reads the declaration out of the registry and the non-edit out of 098's file,
and leaves the end-to-end run to a reviewer, where it was always going to have
to live: a block cannot be its own witness.

D-5 (2026-09-16, why the first draft's negative could not have failed). That
same draft also asserted that the plan does not contain `test $? -eq 2`. The
migrated block contains exactly that string, on the stale-only line §3.2 keeps,
so the assertion was false on the tree it was written for and would have been
caught only by running it. Recorded because it is the third instance in this
corpus of an acceptance line that could not do its job
(`verification-block-must-fail-before-the-build`, spec 103 §1.1, this one), and
the pattern in all three is an assertion written from the argument rather than
measured against the tree.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture corpus under `${TMPDIR:-/tmp}/ss098` carries the
state instead. This block is spec 098's acceptance (§3.1) as well as this
spec's own, so it is read in two halves and labelled as such.

**Fail-first evidence.** The five migrated lines fail at the parent commit,
where the block they live in requires the exit code the verb stopped spending
when spec 101 merged, and `registry show 105` is a not-found exit 1 there. The
AC-9 pattern checks are not fail-first: they are the half of 098's AC-9 that
was already green and is kept (§3.3). Neither is `cargo test ... spec103_`,
which asserts a mechanism this spec does not change.

```verify:cli
# --- spec 098's acceptance, which this block now holds (3.1) ---
cargo build --release --locked
cargo test -p spec-spine-core --test index_body --locked
cargo test -p spec-spine-cli --test cli --locked
# Fixture: committed shards byte-exact, one spec claiming a unit that is absent.
rm -rf "${TMPDIR:-/tmp}/ss098" && mkdir -p "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" init >/dev/null
sed -i.bak 's/REPLACE-WITH-DATE/2026-09-15/' "${TMPDIR:-/tmp}/ss098"/specs/000-bootstrap/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/000-bootstrap/spec.md.bak
printf '%s\n' '---' 'id: "001-missing-territory"' 'title: "A claim with no code"' 'status: draft' 'implementation: complete' 'created: "2026-09-15"' 'summary: >' '  Claims a file that does not exist while declaring itself complete.' 'establishes:' '  - "src/nothing.rs"' '---' '' '# 001: A claim with no code' > "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
# AC-1 / FR-001: the refusal and the exit code are unchanged.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 1
# AC-1 / FR-004: the code, the owning spec and the unit are named.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q '001-missing-territory'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'src/nothing.rs'
# AC-1 / FR-003: the index half reports no staleness and names no regeneration.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'spec-spine index'
# AC-1 / FR-005: it says regenerating will not clear this.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'regenerat'
# AC-6 / FR-007: the contradicted completion claim is named.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'complete'
# AC-7 / FR-007: no way out is offered. `planned: true` under `complete` is
# L-011 (spec 076 §3.3), and narrowing the claim is not the tool's call.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'planned: true'
# AC-1: the registry half is untouched.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-registry: fresh'
# AC-1 at the primitive verb.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check >/dev/null 2>&1; test $? -eq 1
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check 2>&1 | grep -q 'to refresh'
# AC-8 / FR-009: the --json envelope is unchanged in version, members and
# nesting, and already separates the two refusals structurally.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check --json > "${TMPDIR:-/tmp}/ss098"/check.json 2>/dev/null; test -s "${TMPDIR:-/tmp}/ss098"/check.json
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss098/check.json'));assert d['schemaVersion']=='0.4.0';assert sorted(d)==['exitCode','ok','report','schemaVersion','verb'];assert sorted(d['report'])==['index','registry'];assert sorted(d['report']['index'])==['actual','diagnostics','expected','fresh','unwitnessed'];assert d['report']['index']['diagnostics']['byCode']=={'I-004':1};assert d['exitCode']==1 and d['ok'] is False"
# AC-2: the regression. Regenerating exits 0, repairs nothing, and the verb
# still refuses with an accurate message.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 1
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
# AC-4: mixed. Move a committed shard's bytes while the blocking claim stands.
python3 -c "import json,glob;p=sorted(glob.glob('${TMPDIR:-/tmp}/ss098/.derived/codebase-index/by-spec/*.json'))[0];d=json.load(open(p));d['shardHash']='0'*64;json.dump(d,open(p,'w'),indent=2,sort_keys=True);open(p,'a').write('\n')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 1
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'STALE'
# AC-4 / FR-006: regeneration is attributed to the stale half only.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qiE 'only the stale|stale shard\(s\) only|not the unresolved'
# AC-4: after regenerating, the stale half is gone and the blocking half stands.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
# AC-6 in-flight arm (D-5): `draft` + `in-progress` is in flight under specs
# 025/041/044, so the unit is a `W-001` warning, nothing blocks, and `check`
# exits 0. Asserted for what it shows, which is why it is not the negative.
sed -i.bak 's/implementation: complete/implementation: in-progress/' "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md.bak
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 0
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
# AC-6 negative: a spec that blocks without declaring completion is refused for
# the same code, and is not accused of contradicting a claim it never made.
sed -i.bak -e 's/status: draft/status: approved/' -e 's/implementation: in-progress/implementation: deferred/' "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md.bak
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'complete'
# AC-3: stale only. Satisfy the claim, regenerate, then move a shard's bytes.
mkdir -p "${TMPDIR:-/tmp}/ss098"/src && printf '%s\n' '// placeholder' > "${TMPDIR:-/tmp}/ss098"/src/nothing.rs
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
python3 -c "import json,glob;p=sorted(glob.glob('${TMPDIR:-/tmp}/ss098/.derived/codebase-index/by-spec/*.json'))[0];d=json.load(open(p));d['shardHash']='0'*64;json.dump(d,open(p,'w'),indent=2,sort_keys=True);open(p,'a').write('\n')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 2
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'codebase-index: STALE'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-spine index'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
# AC-5: healthy. Regenerate; both halves fresh at exit 0.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-registry: fresh'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'codebase-index: fresh'
# AC-9 (D-6, amended by 105 3.3): the durable half only. 098's fixed-base
# `git diff` asserted that no commit after its branch point touches either
# settings.json, which specs 099 and 104 then legitimately did; the line could
# only ever go red. What 098 relied on is the message shape, and that is what
# is asserted here, on the files as they stand.
grep -q 'freshness. STALE' kit/settings.json
grep -q 'freshness. STALE' .claude/settings.json
rm -rf "${TMPDIR:-/tmp}/ss098"
# --- spec 105's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
target/release/spec-spine registry show 105 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["amendsVerification"] == ["098-a-blocking-claim-is-not-a-stale-shard"], d; assert d["amends"] == ["098-a-blocking-claim-is-not-a-stale-shard"], d'
# Spec 098's file is not edited (spec 040 3.1): its own block still carries the
# superseded form of the primitive-verb assertion, which is the half a reader
# compares against. This goes red the moment someone edits 098 to make it pass.
grep -qF 'index check >/dev/null 2>&1; test $? -eq 2' specs/098-a-blocking-claim-is-not-a-stale-shard/spec.md
# The resolution this spec relies on is spec 103's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 098` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (D-4).
cargo test -p spec-spine-core --test verify --locked spec103_
```
