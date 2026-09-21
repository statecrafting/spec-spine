---
id: "084-an-acceptance-outlives-the-output-it-was-written-against"
title: "An acceptance outlives the output it was written against"
status: approved
kind: "core"
created: "2026-09-17"
summary: >
  Spec 044's `## Verification` block has three assertions that no longer hold,
  and none of the three describes a defect. It compares the whole of `index
  check`'s stdout against one line, which spec 050 legitimately added a second
  line to; it takes `len()` of `index diagnostics --json` as a bare array, which
  spec 074 legitimately wrapped in an `items` envelope; and it pins
  `VERDICT_SCHEMA_VERSION` at the literal `0.2.0`, which spec 071 legitimately
  moved to `0.4.0`. The third is the sharpest: 044 3.6 rules that the envelope
  version MUST NOT move for a payload addition, and a literal pin cannot assert
  that, because it also fails for every move the rule permits. None of the three
  amending specs declared `amends_verification`, and all three predate the spec
  that built it. This spec declares 044's acceptance replaced and carries the
  corrected block, without editing 044's file.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "044-index-diagnostics-reach-a-gate"
  - "050-claimed-but-unwitnessed"
  - "071-a-change-is-classified-under-the-bases-rules"
  - "074-a-governed-read-names-its-version"
  - "082-an-amended-acceptance-is-the-one-that-runs"
amends: ["044-index-diagnostics-reach-a-gate"]
# 3.1: this spec's `## Verification` block IS 044's acceptance from now on.
# 050's own file is not edited (spec 037 3.1), and neither are 057, 088 or 093:
# each states a rule that is true and complete, and this spec changes none of
# them. What changes is what 050 accepts.
amends_verification: ["044-index-diagnostics-reach-a-gate"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 084: An acceptance outlives the output it was written against

## 1. Purpose

### 1.1 Three red assertions, no defect behind any of them

Measured on 2026-09-17 against `4ce7085` with the 0.20.0 binary:

```
$ spec-spine verify 050
verify: 044-index-diagnostics-reach-a-gate: FAILED at command 5 (exit 1)
```

`verify` stops at the first failure, so command 5 is all a reader sees. Running
the block to the end, one command at a time, finds three:

| Cmd | Assertion | Actual | Amended by |
|-----|-----------|--------|------------|
| 5 | `"$(index check)" = "index is fresh"` | a second line, `unwitnessed claims: 87 (87 allowed by [lint] unwitnessed_allowed)` | 057 |
| 7 | `len(index diagnostics --json) = 0` over a bare array | `{"items": [], "schemaVersion": "0.1.0"}` | 093 |
| 8 | `index check --json` `schemaVersion` `= "0.2.0"` | `0.4.0` | 088 |

Every rule 050 states still holds. 3.1 requires that a corpus with no
diagnostics keep printing the bare `index is fresh`, and it does: what command 5
trips over is spec 050's unwitnessed-claims line, which is a second line about a
different subject and not a suffix on the verdict. 3.4 requires that
`index diagnostics` list what the shards record, and it lists nothing, correctly,
because this corpus records nothing. 3.6 requires that the envelope version not
move for a payload addition, and no payload addition has moved it.

### 1.2 A literal pin cannot assert that a literal does not move

Command 8 is the one worth naming as a class. Spec 044 3.6 rules:

> Adding a member to one verb's `report` payload is additive and MUST NOT change
> `VERDICT_SCHEMA_VERSION`.

The block asserts that rule by pinning the constant at the value it held on
2026-09-06. That assertion fails in exactly two cases: when a payload addition
moves the version, which is the defect 3.6 exists to catch, and when anything
else moves it, which 3.6 expressly permits. Spec 071 moved it for an
envelope-level reason of its own, which is the permitted case, and the line went
red. A pin that fails for the permitted case is not a weak assertion of the rule;
it is an assertion of the calendar, and spec 082 3.5 corrected the same shape in
spec 074's block.

The same paragraph is why command 8's replacement cannot be a stronger pin. What
3.6 forbids is a causal link between one kind of change and one constant, and no
command run against today's tree can observe a link to a change that happened in
the past. 3.3 says what is observable instead, and says plainly what is not.

### 1.3 The crossings were silent, and nothing reruns a merged block

Specs 050, 088 and 093 each changed an output that 050's acceptance asserts on.
None declared `amends_verification`, and none could have: spec 082 built that key
and shipped it in v0.20.0, after all three had merged. The gate chain does not run
`verify`, deliberately (spec 043), so each block went red at merge time with
nothing to report it. Spec 083 gave one such block its replacement; spec 082 4
named the corpus-wide audit and left it open. This spec is that audit's first
entry, and four more blocks are red for the same family of reason.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 044's `## Verification` block is no longer the one that
runs. Nothing under `crates/` changes, no schema constant moves, no CLI surface
is added or removed, and no committed shard changes except the two this spec's
own frontmatter produces.

## 3. Behavior

### 3.1 What spec 044's acceptance now is

This spec's `## Verification` block MUST replace spec 044's in full, through
`amends_verification` (spec 082 3.1), and 050's file MUST NOT be edited. The
block is 044's, with commands 5, 7 and 8 corrected per 3.2 to 3.4, with one
assertion added per 3.4, and with this spec's own assertions (3.5) after them
under a heading that says whose is whose.

Replacing the block in full rather than the three lines follows spec 082 3.5: a
reader asking what 050 accepts today should find one block that answers, not a
base document plus a patch to apply in their head.

### 3.2 The verdict line is read as a line, and the verb's exit status is kept

The bare-verdict assertion MUST read the first line of `index check`'s stdout
rather than the whole of it, because 044 3.1 governs the verdict line and spec
050's line is a second line about unwitnessed claims.

It MUST be split across two commands: one that runs the verb with its stdout
redirected to a file, and one that compares the first line of that file. A
pipeline ending in `head` reports `head`'s exit status, so a verb that failed
outright would be read as a verb that printed something else, and the block
would report the wrong defect. The redirect leaves the verb's own status as the
line's status, which is what makes a failure of the verb a failure of the block.

### 3.3 The envelope comparison is a consistency check, and says so

The version assertion MUST NOT pin a literal. It MUST instead compare the
`schemaVersion` of two verbs whose `report` payloads differ, assert the value is
non-empty, and assert that the two are in fact different verbs with different
payloads.

That difference MUST be asserted as a positive claim about a named member and a
named verb token, never as an inequality of key sets. `sorted(a) != sorted(b)`
over two payloads goes red whenever either one legitimately changes shape, which
is the same fragility 1.2 rejects in a literal version pin, applied to a
different axis (D-6).

This is a consistency check and MUST be described as one. It witnesses that the
two verbs are versioned by one envelope constant rather than per payload, which
is the shape 044 3.6 reasons from. It does **not** prove that a payload addition
left the version unchanged: both verbs read the same constant, so a bump made for
a payload reason would move both together and the comparison would still pass.
The full rule is a statement about cause over time and is not decidable from one
run of the tree.

Recording that gap is the point. The alternative on offer was a literal pin,
which does not assert the rule either and additionally goes red for every change
the rule allows. An assertion that covers part of a rule and names the part it
does not cover is worth more than one that covers none of it and looks total.

Both verbs' outputs MUST be captured to files rather than compared through
command substitution. `test "$(A)" = "$(B)"` passes when A and B both fail and
both substitutions are empty, which would turn a total failure of two verbs into
a green line.

### 3.4 The payload member is asserted, because the rule is about it

The block MUST assert that `index check --json` carries the `report.diagnostics`
member that spec 044 added, with its three keys.

This is an addition, not a correction. 044 3.6's rule has an antecedent, "adding
a member to one verb's `report` payload", and 044's block asserted the consequent
alone. The pin therefore hung on nothing: it would have kept passing had the
member been dropped entirely, which is the one change that would make 3.6
vacuous. Asserting the antecedent is what gives the consistency check of 3.3
something to be consistent about.

### 3.5 The acceptance

`registry show 106 --json` MUST carry `amendsVerification` and `amends`, both
naming 050 and nothing else: that is the declaration, and it is read through the
CLI rather than off the shard.

Spec 044's file MUST still carry all three superseded assertion forms. That is
the spec 037 3.1 half, and those are the lines that go red if someone ever
resolves this by editing 050 instead, which is the move
`AGENTS.md` "Adversarial prompt refusal" refuses.

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 050` runs, and `cmd_verify::run` refuses a nested call on its
own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so even
reading the plan from inside is a validation failure. That `spec-spine verify
050` and `spec-spine verify 106` both exit 0 on the merged tree is what a
reviewer runs and what the release sweep runs; a block cannot be its own witness
(spec 083 D-4).

### 3.6 No code changes

No file under `crates/` is edited, no `*_SCHEMA_VERSION` constant moves, no
embedded schema changes. The mechanism this spec uses was built and shipped by
spec 082 in v0.20.0; this spec is a corpus change that uses it.

## 4. Out of scope

- **Editing specs 044, 057, 088 or 093.** 050 keeps the block it was ratified
  with, which is the record spec 037 3.2 protects. The other three state rules
  that are true as written; what their changes invalidated was an acceptance,
  not a rule.
- **The other four red blocks.** Specs 049, 059, 060 and 071 are red for the
  same family of reason and each needs its own amendment, because
  `amends_verification` replaces a target's whole block with the amender's
  single one and a shared block would couple five unrelated acceptances. They
  are named here so the audit is legible, and they are not done here.
- **Recurring sweep coverage.** Nothing reruns a merged block, which is how
  three crossings went silent. Spec 083 4 declined to put `verify` in the gate
  and recommended a maintainer sweep; where that sweep is declared is its own
  work.
- **Teaching `amends_verification` in the template.**
  `standards/spec/templates/spec-template.md` documents the frontmatter grammar
  and does not carry the key spec 082 added. That is a real gap in territory
  this spec does not own.
- **Spec 083's copy of the filtered test run.** Its block carries the same
  `cargo test ... spec103_` line with the same vacuous-pass hole D-7 describes.
  It is green today, it is another spec's acceptance, and correcting it is an
  amendment of 105 rather than a repair of this block.
- **The remaining literal version pins.** Spec 047's block pins
  `config_version` at `0.1.0` and spec 083's carries a `schemaVersion=='0.4.0'`
  inherited from 098. Both are green today and both are the shape 1.2 names. A
  green line is not this spec's to change.

## 5. Resolved decisions

D-1 (2026-09-17, why a replacement rather than three corrections). Spec 082 3.5
settled that a block is the unit, and spec 083 D-1 carried it: a reader who finds
044's block in 044's file and a patch in this one has to apply the patch in their
head to know what 050 accepts. A block read in one place is worth the
duplication.

D-2 (2026-09-17, why the version line becomes a consistency check and not a
better pin). Every candidate pin asserts the calendar. Pinning `0.4.0` moves the
red to the next legitimate bump and this spec gets filed again, which spec 083
D-2 named as the test an assertion has to pass: an assertion that must eventually
fail for a legitimate reason is not an assertion. What 3.6 forbids is a causal
link, and no command run against one tree can observe causation in the past. 3.3
takes the observable half and states the boundary in the spec rather than leaving
a reader to infer that the line proves more than it does.

D-3 (2026-09-17, why the payload assertion is added rather than left out). Adding
to a block being repaired is scope this spec had to justify. The reason it is in
is that without it the block asserts a consequent whose antecedent nothing
checks: drop `report.diagnostics` entirely and 044's original command 8 and the
replacement's consistency check both still pass, while 3.6 has nothing left to
govern. One line closes that, and it is an assertion about spec 044's own
territory, not new territory.

D-4 (2026-09-17, why `head` is not left in a pipeline). `test "$(cmd | head -1)"`
reports `head`'s status, and `head` exits 0 on empty input, so a verb that failed
and printed nothing reads as a verb whose first line was empty. The block would
then fail, but for the wrong reason and with a message naming the wrong defect.
The same hazard is worse in a two-substitution comparison, where two failures
compare equal and the line passes; 3.3 requires files there for that reason.
Recorded because the failure is silent in the passing direction, which is the
only direction that matters.

D-5 (2026-09-17, what the fail-first evidence is and is not). This spec has
almost none, and that is correct rather than a gap. A correction to an acceptance
changes no code, so the corrected lines pass at the parent commit: measured at
`4ce7085`, the verdict line, the `items` count and the version comparison all
exit 0 against the same binary whose output made 044's originals red. What was
red at the parent was 044's block, and this spec does not make anything go green
that was legitimately red.

The first draft of this entry claimed the three corrected lines were fail-first
and then described the three **original** lines failing, which is a different
claim about different commands. Running them is what caught it. That is the
fourth instance in this corpus of an acceptance claim written from the argument
rather than measured against the tree
(`verification-block-must-fail-before-the-build`, spec 082 1.1, spec 083 D-5,
this one), and the first where the unmeasured claim was about the fix rather than
the defect.

What was measured instead is failability: each corrected line was run against the
state it exists to catch, and each failed. The verdict line fails when the
`(N warning(s), ...)` suffix 044 3.1 forbids is present. The `items` count fails
against the pre-093 bare array, with a `TypeError` rather than a false pass, and
against a non-empty `items`. The version comparison fails when the two values
diverge, and the payload assertion of 3.4 fails when `report.diagnostics` is
absent. Failability against the condition is what an assertion owes; being red at
one particular parent commit is evidence only when the spec changes code.

The one line that is fail-first at the parent in the ordinary sense is
`registry show 106`, a not-found exit 1 there, which is 3.5's half.

D-6 (2026-09-17, why the payload difference is a named member and not a key-set
inequality). The first draft asserted `sorted(a['report']) != sorted(b['report'])`.
Review pointed out that this is the shape 1.2 condemns: it is a claim about the
relative shape of two payloads on one day, and it goes red if `compile`'s report
ever gains the members `index check`'s carries, which is a change no rule here
forbids. The objection is correct and it lands on this spec's own argument, so
the assertion changed rather than the argument. What replaces it is two positive
claims: the verb tokens differ (`index.check` against `compile.spec`, which spec
034 makes part of the envelope), and `diagnostics` is present in one payload and
absent from the other. The first is a statement about what the verbs are. The second is
not, and D-8 records what it is instead.

D-7 (2026-09-17, why the filtered test run asserts its own match count).
`cargo test --test verify spec103_` exits 0 when the filter matches nothing:
measured, `zzz_no_such_test_` gives `0 passed; 31 filtered out` and status 0. The
line was copied from spec 083's block, where it has the same hole. A filter that
stops matching after a rename would leave the line green while asserting nothing,
which is the vacuous-pass family this corpus has already paid for once. The run
is captured and its summary asserted to name a non-zero pass count, so the line
goes red when the filter matches nothing as well as when a match fails. Spec
083's copy is not edited here: a green line in another spec's block is not this
spec's to change, and it is named in 4.

D-8 (2026-09-17, what the second arm of the comparison actually rests on, and
why 000 is the anchor). Review of D-6 pointed out that `'diagnostics' not in
b['report']` is still a claim about what `compile`'s payload happens to contain,
so D-6's sentence claiming both arms describe what the verbs *are* was an
overclaim, in a spec whose subject is assertions that overclaim. It is corrected
above rather than defended.

The arm stays, because what it rests on is sound even though it is contingent.
The structural weight is carried by the verb tokens, which spec 034 makes part of
the envelope. The member arm adds that the two payloads differ in a named way,
which is what makes a shared version say anything about payload independence, and
its contingency is loud: if `compile`'s report ever gains `diagnostics` the line
goes red and a reader is sent to this entry, rather than passing while asserting
less than it appears to. The distinction from the literal pin 1.2 rejects is the
failure mode, not the contingency: a pin goes red on a routine and expected event
(088 performed one), while this goes red only if two verbs' payloads converge,
which no rule here forbids but nothing plans either.

The counter-verb is anchored on spec 000 rather than 024. `compile --spec` needs
an id that resolves, and any id can be deleted, so the anchor is a dependency
whichever one is chosen; 000 is the tier-1 bootstrap whose `unamendable` anchors
are non-overridable, and its removal would end the corpus rather than merely
break this line. 024 was the first draft's choice and it was arbitrary, which is
the part review was right to flag.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the scratch directory under `${TMPDIR:-/tmp}/ss106` carries the
state instead. This block is spec 044's acceptance (3.1) as well as this spec's
own, so it is read in two halves and labelled as such.

**Fail-first evidence.** Only `registry show 106` is red at the parent commit, a
not-found exit 1. The corrected lines are not, and measurably so: this spec
changes no code, so they exit 0 against the parent's binary. What each corrected
line was measured against is the condition it exists to catch, and each failed
there; D-5 records the probes and why that, not redness at a parent, is what
these lines owe.

```verify:cli
# --- spec 044's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test diagnostics --locked
cargo test -p spec-spine-cli --locked
# This corpus carries no unresolved units, so the strict flag passes here.
target/release/spec-spine index check --fail-on-unresolved
# Scratch: each line is its own shell, so a verb's stdout is carried in a file
# rather than a variable. The redirect leaves the verb's own exit status as the
# line's status, which a pipeline into `head` or `python3` would not (3.2, D-4).
rm -rf "${TMPDIR:-/tmp}/ss106" && mkdir -p "${TMPDIR:-/tmp}/ss106"
# 3.1 (amended by 3.2): a clean corpus keeps the bare verdict line. Spec 050's
# unwitnessed-claims line is a second line about a different subject, so the
# assertion reads the verdict line rather than the whole of stdout.
target/release/spec-spine index check > "${TMPDIR:-/tmp}/ss106/check.txt"
test "$(head -1 "${TMPDIR:-/tmp}/ss106/check.txt")" = "index is fresh"
# 3.4: the read verb answers, and answers as JSON. Spec 074 wrapped the payload
# in an `items` envelope, so the count is of `items` and not of the document.
target/release/spec-spine index diagnostics
target/release/spec-spine index diagnostics --json > "${TMPDIR:-/tmp}/ss106/diagnostics.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss106/diagnostics.json')); assert len(d['items'])==0, d"
# 3.3: the envelope version is compared across two verbs whose payloads differ,
# not pinned to a literal. This is a consistency check: it witnesses one
# envelope constant rather than per-payload versioning, and it does NOT prove
# that a payload addition left the version unchanged, since a bump made for a
# payload reason would move both verbs together. The payload difference is
# asserted as a named verb token and a named member, not as a key-set
# inequality, which would be the same calendar shape on another axis (D-6).
# The counter-verb is anchored on spec 000, the tier-1 bootstrap spec, because
# `compile --spec` needs an id that exists and 000 is the one id whose removal
# would end the corpus rather than move this line (D-8).
target/release/spec-spine index check --json > "${TMPDIR:-/tmp}/ss106/check.json"
target/release/spec-spine compile --spec 000 --json > "${TMPDIR:-/tmp}/ss106/compile.json"
python3 -c "import json; a=json.load(open('${TMPDIR:-/tmp}/ss106/check.json')); b=json.load(open('${TMPDIR:-/tmp}/ss106/compile.json')); assert a['schemaVersion'], a; assert a['schemaVersion']==b['schemaVersion'], (a['schemaVersion'], b['schemaVersion']); assert a['verb'] != b['verb'], (a['verb'], b['verb']); assert 'diagnostics' in a['report'] and 'diagnostics' not in b['report'], (sorted(a['report']), sorted(b['report']))"
# 3.4: the member spec 044 added to `index check`'s payload is present. This is
# the antecedent 044 3.6's rule is about, and 044's block never asserted it.
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss106/check.json'))['report']; assert sorted(r['diagnostics'])==['byCode','errors','warnings'], r"
# --- spec 084's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
target/release/spec-spine registry show 084 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["amendsVerification"] == ["044-index-diagnostics-reach-a-gate"], d; assert d["amends"] == ["044-index-diagnostics-reach-a-gate"], d'
# Spec 044's file is not edited (spec 037 3.1): its own block still carries all
# three superseded assertion forms. These go red the moment someone resolves
# this by editing 050 instead.
grep -qF 'index check)" = "index is fresh"' specs/044-index-diagnostics-reach-a-gate/spec.md
grep -qF 'print(len(json.load(sys.stdin)))' specs/044-index-diagnostics-reach-a-gate/spec.md
grep -qF '= "0.2.0"' specs/044-index-diagnostics-reach-a-gate/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 050` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count:
# a name filter that matches nothing exits 0 (measured: `0 passed; 31 filtered
# out`), so the bare line would stay green while asserting nothing (D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss106/spec103.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss106/spec103.txt"
rm -rf "${TMPDIR:-/tmp}/ss106"
```
