---
id: "086-an-exact-key-set-refuses-what-the-rule-allows"
title: "An exact key set refuses what the rule allows"
status: approved
kind: "core"
created: "2026-09-17"
summary: >
  Spec 052's `## Verification` block asserts that `index orphans --json` has
  exactly the keys `orphaned` and `inFlight`, and spec 074 legitimately added a
  `schemaVersion` member to every governed read, so `spec-spine verify 059` has
  been red since 093 merged. 059 3.1 requires the two named arrays to be present
  and always emitted; it never required them to be the only members, and an
  equality over the key set asserts a closed document the rule does not state.
  The same block is silent about the two decisions 059 actually recorded for
  that verb: that `--json` emits both arrays even when empty, and that the prose
  form stays silent on a corpus with nothing to report. This spec declares 059's
  acceptance replaced and carries the corrected block, without editing 059's
  file. It is the second of the four follow-ons spec 084 4 named.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "052-read-verbs-on-a-code-free-corpus"
  - "074-a-governed-read-names-its-version"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  # D-7: not a behavioural dependency. This spec cites 107's decision record as
  # precedent and states its own position in the series, so the order those
  # claims assume is declared rather than left to the merge queue.
  - "085-a-version-pin-is-not-a-contract"
amends: ["052-read-verbs-on-a-code-free-corpus"]
# 3.1: this spec's `## Verification` block IS 059's acceptance from now on.
# 059's own file is not edited (spec 037 3.1), and 093's is not either: 093
# states a rule that is true and complete, and this spec changes none of it.
# What changes is what 059 accepts.
amends_verification: ["052-read-verbs-on-a-code-free-corpus"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 108: An exact key set refuses what the rule allows

## 1. Purpose

### 1.1 One red assertion, no defect behind it

Measured on 2026-09-17 against `e4d7d28` with the 0.20.0 binary:

```
$ spec-spine verify 059
verify: 052-read-verbs-on-a-code-free-corpus: FAILED at command 4 (exit 1)
```

Command 4 is

```
index orphans --json | python3 -c '... assert set(o) == {"orphaned", "inFlight"}'
```

and the verb answers

```json
{ "inFlight": [...], "orphaned": [...], "schemaVersion": "0.1.0" }
```

Spec 074 routed every read document through one emitter that sorts keys and
stamps a `READ_SCHEMA_VERSION`, `index orphans` among the ten verbs it names,
and recorded the addition for this verb explicitly in its D-8. Every other
assertion in 059's block is green. Run one line at a time to the end of the
block, this is the only red one:

| Cmd | Assertion | Status |
|-----|-----------|--------|
| 4 | `set(o) == {"orphaned", "inFlight"}` | **red**, `schemaVersion` is a third member |
| 5 | the empty-universe fixture builds | green |
| 6 | `index coverage` exits 0 on it | green |
| 7 | `index coverage --fail-on-untraced` exits 1 | green |
| 8 | the message names `no package was discovered` | green |
| 10 | this repository's own coverage assertion | green |
| 11 | `compile --check` | green |

`verify` stops at the first failure, so a reader who runs the verb sees command
4 and learns nothing about the other six.

### 1.2 An equality over a key set asserts a closed document

059 3.1 says what `--json` must carry: "the flat array becomes an object with
the two arrays as named members", and the `--json` form "always emits both
arrays, since a consumer parsing an object should not have to distinguish
'absent' from 'empty'". Both sentences are about **presence**. Neither says the
object is closed, and 059 3.1's own paragraph on schema versions is careful
about the opposite question: it records that `INDEX_SCHEMA_VERSION` does not
move because no committed artifact changes, leaving the document's own
versioning unaddressed, which is the gap 093 later filled.

`set(o) == {...}` asserts more than presence. It asserts that nothing else may
ever appear, which is a closed-document rule 059 does not state and which this
project's schema policy contradicts: MINOR is the additive case, the embedded
`specRecord` schema sets no `additionalProperties` deliberately so "an additive
minor can extend it", and 093 then performed exactly that addition. The
assertion refuses what the rules it sits among permit.

It is also weaker than it looks in the direction that matters. `set(o)` says
nothing about the members' **types**: replace both arrays with strings and the
assertion still passes, while the shape 059 exists to produce is gone. A
consumer's breakage is a wrong type, not an extra key.

This is the fourth site in this corpus with an acceptance stricter than its
rule, and the second where the over-assertion is about shape rather than a
version literal: spec 074's calendar assertion (corrected by 103), spec 044's
version pin (corrected by 106), spec 049's version pin (corrected by 107), and
this. Spec 084 1.2 states the general form.

### 1.3 The block is silent about the decisions 059 recorded

059 3.1 carries two dated decisions about this verb, and its block asserts
neither.

The first is that the `--json` form emits both arrays whatever their contents,
so a consumer never distinguishes absent from empty. Command 4 runs against
**this** repository, where both groups are populated, so the empty case the
decision is about is never exercised.

The second is that a corpus with no orphans prints **nothing** in prose: "two
headers and two `(none)` lines would be noise on the answer 'nothing to
report', and an existing acceptance test pinned the silence." Nothing in 059's
block reads the prose form of `orphans` at all.

The block already builds an empty-universe fixture for 3.2 and 3.3, and that
fixture is exactly the corpus both decisions describe: measured, it answers
`{"inFlight": [], "orphaned": [], "schemaVersion": "0.1.0"}` and prints nothing.
The assertions are two lines against a fixture the block already has.

### 1.4 The crossing was silent

Spec 074 changed the shape of a document another approved spec's acceptance
pinned, named that verb in its own D-8, and nothing told 059. `verify` is the
one verb that executes what the corpus declares, so it sits outside the gate
chain deliberately (`AGENTS.md`); CI never runs it. Spec 084 1.3 records the
identical mechanism and names this spec's target among the four remaining.

093 did carry `amends` edges, to 010 and 060, for the two documents whose stated
contracts it broke. It carried none to 059, and 3.6 below records why that was
right: what 093 invalidated here was an acceptance, not a rule.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 059's `## Verification` block is no longer the one
that runs. Nothing under `crates/` changes, no schema constant moves, and no
committed shard changes except the two this spec's own frontmatter produces.

## 3. Behavior

### 3.1 What spec 052's acceptance now is

This spec's `## Verification` block MUST replace spec 052's in full, through
`amends_verification` (spec 082 3.1), and 059's file MUST NOT be edited. The
block is 059's, with command 4 replaced per 3.3, with the two assertions of 3.4
added against the fixture the block already builds, and with this spec's own
assertions (3.5) after them under a heading that says whose is whose.

Replacing the block in full rather than the one line follows spec 082 3.5 and
spec 083 D-1: a reader asking what 059 accepts today should find one block that
answers, not a base document plus a patch to apply in their head.

### 3.2 The verb's exit status is the line's status

Command 4 pipes the verb into `python3`, so the line reports `python3`'s status
and the verb's is discarded. The replacement MUST redirect the document to a
file and assert against the file, so a verb that fails takes the line down at
the redirect. The same applies to the prose assertion of 3.4, which MUST NOT be
written as `test -z "$(...)"`: a verb that failed and printed nothing would read
as a verb that was correctly silent.

The rule reaches every line whose verb must **succeed**, 3.5's `registry show`
included. That one is where the hazard is not hypothetical: at the parent commit
the verb exits 1 and prints nothing, so a pipeline reports a JSON decode error
for what is a not-found, naming the wrong defect to the first reviewer who runs
the block. Spec 085 D-4 records the same correction, made under review on its own
pull request.

It does **not** reach a line whose verb is required to exit non-zero, and 059's
block has one: the `index coverage --fail-on-untraced 2>&1 | grep -q 'no package
was discovered'` line, where exit 1 **is** the behaviour 059 3.2 requires and the
assertion is about the message the refusal carries. There the pipeline is
load-bearing, because `grep`'s status is the one that should decide the line; a
redirect would make the line fail on a correct refusal. That line is inherited
unchanged for that reason, not by oversight. D-8.

The fixture root inherited from 059's block stays at `${TMPDIR:-/tmp}/ss059`
unchanged, and the live-corpus document is captured beside it rather than inside
it, so the fixture the other assertions build is not disturbed.

### 3.3 The key set becomes presence, type, order and version

The equality MUST be replaced by four claims about the document:

- it is an object whose keys are **sorted**, which is spec 074 3.2's rule for
  every governed read and the half that fails against pre-093 output;
- it carries a non-empty `schemaVersion`, which is 093's addition;
- `orphaned` is present and is a **list**;
- `inFlight` is present and is a **list**.

Presence and type are what 059 3.1 requires. Nothing here forbids a further
member, because 059 forbids none and the schema policy permits them.

Against pre-059 output, a bare array of ids, the member reads raise rather than
pass: measured, `TypeError: list indices must be integers`. The distinction 059
was built to draw is still what goes red if it is lost.

### 3.4 The two recorded decisions are asserted, on the fixture that shows them

The replacement MUST assert, against the empty-universe fixture 059's block
already builds:

- that `--json` carries both arrays and both are **empty**, which is the decision
  that a consumer never distinguishes absent from empty, exercised on the corpus
  where the distinction could be made;
- that the prose form writes **nothing**, which is the decision that a corpus
  with no orphans stays silent.

Neither is vacuous. Measured: omitting an empty group from the document fails
the first, and the second fails against this repository's own corpus, which has
orphans and prints them.

These are assertions about spec 052's own territory, not new territory, and they
are added for the reason spec 084 D-3 gives: a block that asserts a consequent
whose antecedent nothing checks leaves the rule with nothing to govern.

### 3.5 The acceptance

`registry show 108 --json` MUST carry `amendsVerification` and `amends`, both
naming 059 and nothing else: that is the declaration, and it is read through the
CLI rather than off the shard.

Spec 052's file MUST still carry the superseded key-set equality. That is the
spec 037 3.1 half, and it is the assertion that goes red if someone ever
resolves this by editing 059 instead, which is the move
`AGENTS.md` "Adversarial prompt refusal" refuses.

The resolution itself MUST keep passing through spec 082's own cases (`spec103_`
in `crates/spec-spine-core/tests/verify.rs`). No code changes here, so the
mechanism is what is asserted, not a new one. The run MUST be captured and its
summary asserted to name a non-zero pass count: a name filter that matches
nothing exits 0, so the bare line would stay green while asserting nothing (spec
084 D-7).

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 059` runs, and `cmd_verify::run` refuses a nested call on its
own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so even
reading the plan from inside is a validation failure. The instance-level fact,
that `spec-spine verify 059` and `spec-spine verify 108` both exit 0 on the
merged tree, is what a reviewer runs and what the release sweep runs.

### 3.6 No `amends` edge was owed to 059 by spec 074, and none is claimed here

This spec carries `amends: ["059-..."]` because spec 082 3.1 requires every
`amends_verification` entry to appear there: replacing what a spec accepts is an
amendment of that spec. It does **not** assert that 093 owed 059 one.

093 amended 010 and 060 because each states a document's shape as a contract and
093 broke that stated shape. 059 states presence, and 093 preserved presence.
What 093 invalidated was an assertion stricter than 059's rule, and an
over-strict assertion is not a rule anyone amended. The distinction matters
because the opposite reading would make every additive change an amendment of
every spec whose block happened to be written tightly, which is a governance
burden nothing here wants.

### 3.7 No code changes

No file under `crates/` is edited, no `*_SCHEMA_VERSION` constant moves, no
embedded schema changes, and no CLI surface is added or removed. The mechanism
this spec uses was built and shipped by spec 082 in v0.20.0; this spec is a
corpus change that uses it.

## 4. Out of scope

- **Editing spec 052, or spec 074.** 1.2 and the frontmatter comment. 059 keeps
  the block it was ratified with, which is the record spec 037 3.2 protects, and
  093's text is true as written.
- **Asserting the id ordering inside each group.** 059 3.1 requires each group to
  stay id-sorted, per spec 010 3.3, and neither block asserts it. On this
  repository's corpus each group holds one or two entries, so the assertion would
  be very nearly vacuous, and the fixture's groups are empty, where it is
  entirely so. A fixture built to hold three orphans in a deliberate order would
  assert it, and that is a fixture this spec does not build. D-3.
- **The other two red blocks.** Specs 053 and 071 are red for the same family of
  reason and each needs its own amendment, because `amends_verification` replaces
  a target's whole block with the amender's single one. Spec 084 4 names all
  four; 107 took 056 and this is the second.
- **The remaining green literal pins.** Spec 047's block pins `config_version` at
  `0.1.0` and spec 083's carries a `schemaVersion=='0.4.0'` inherited from 098.
  A green line in another spec's block is not this spec's to change; 106 4
  already names them.
- **Teaching `amends_verification` in the template.**
  `standards/spec/templates/spec-template.md` does not carry the key spec 082
  added. 106 4 names it as a real gap in territory neither spec owns.

## 5. Resolved decisions

D-1 (2026-09-17, why presence and type rather than a looser key set). The
smallest repair is `{"orphaned", "inFlight"} <= set(o)`, a subset test. It fixes
the redness and keeps the defect 1.2 names in the other direction: a subset test
over keys still says nothing about what the members hold, so both arrays could
become strings and the line would pass. Reading the members by name and
asserting their type costs the same line and asserts the shape 059 exists to
produce. The key-order and version claims come along because 093 requires them
of every governed read, and because they are what fails against pre-093 output.

D-2 (2026-09-17, why the fixture assertions are added rather than left out).
Adding to a block being repaired is scope this spec had to justify, as spec 084
D-3 did for its one added line. The reason is that 059 3.1's two dated decisions
are the parts of that verb a future change is most likely to break quietly:
omitting an empty group reads as a tidier document, and printing two `(none)`
headers reads as a friendlier one. Both were decided against, on the record, and
neither was asserted. The fixture that shows both already exists in the block for
another purpose, so the cost is two lines and no new machinery.

D-3 (2026-09-17, why the id ordering is not asserted). 059 3.1 requires it and
nothing checks it, which looks like the same gap D-2 closes. It is not, because
the corpus cannot show it: this repository's groups hold one or two entries and
the fixture's hold none, so an ordering assertion would pass whatever the code
did. That is the vacuous-pass family this corpus has paid for before
(`verification-block-must-fail-before-the-build`, spec 084 D-7), and adding a
line that cannot fail is worse than leaving the gap visible. Asserting it needs a
fixture with three orphans in a deliberately unsorted filesystem order, which is
a fixture worth building and is not this spec's repair. Named in 4.

D-4 (2026-09-17, why the prose assertion is a file and not `test -z "$(...)"`).
Command substitution discards the verb's exit status, so a verb that failed and
printed nothing is indistinguishable from a verb that was correctly silent, and
the line passes in the wrong direction. This is spec 084 D-4's hazard with the
polarity that makes it silent: there, a failed verb made the line fail for the
wrong reason; here it would make the line **pass** for the wrong reason.
Redirecting to a file keeps the verb's status on its own line and leaves `test
! -s` to assert only emptiness.

D-5 (2026-09-17, what the fail-first evidence is and is not). This spec changes
no code, so the corrected lines pass at the parent commit `3173bb0` against the
same binary whose output made 059's original red. That is correct rather than a
gap, and it is spec 084 D-5's finding restated at a third site.

What an assertion owes instead is **failability against the condition it exists
to catch**, and each line was run against that condition. Measured on 2026-09-17
by substituting documents and corpora:

| Condition | Line's status |
|---|---|
| pre-059 output, a bare array of ids | fails (`TypeError`) |
| pre-093 output, no `schemaVersion` | fails |
| keys emitted in struct order rather than sorted | fails |
| a group omitted from the document | fails |
| an empty group reported as non-empty on the fixture | fails |
| the prose form run on a corpus that has orphans | fails |
| the tree as it stands | passes |

The one line that is fail-first at the parent in the ordinary sense is
`registry show 108`, a not-found exit 1 there, which is 3.5's half.

D-6 (2026-09-17, why the live-corpus document is captured outside the fixture
root). The block's fixture line begins `rm -rf "${TMPDIR:-/tmp}/ss059"`, so a
document written into that directory before it would be deleted, and one written
after it would sit at the root of a corpus the following lines pass to `--repo`.
Neither is harmful today and both invite a later reader to move a line and break
something quietly. A sibling path costs nothing and the fixture line is inherited
unchanged, which keeps the diff against 059's block to what 3.1 says it is.

D-7 (2026-09-17, why spec 085 is a declared dependency). This spec cites 107 in
1.2 as one of the corrected sites, in 3.2 as the precedent for redirecting the
`registry show` read, and in 4 as the first of the series this is the second of.
Review of the pull request pointed out that all three are past-tense claims about
a spec that was not in this branch's base, so either the ordering was real and
undeclared or the claims were false. The ordering is real: the four repairs were
built in one sitting and each carries the previous one's findings. Declaring it
in `depends_on` makes the order the corpus enforces rather than one a reader has
to trust, at the cost of marking this spec blocked in `registry plan` until 107
is complete, which is the truth of it. The dependency is on 107's **record**, not
on anything it does: nothing here executes, reads or relies on 107's behaviour,
and the two blocks share no state.

D-8 (2026-09-17, why one pipeline stays). Review of the pull request noted the
`index coverage --fail-on-untraced | grep -q` line as in tension with 3.2's rule,
and reading it that way is fair, because the first draft of 3.2 said the rule
reached every line. It does not, and the boundary is which status the line should
report. Where a verb must succeed, its status is the line's answer and a pipeline
throws it away. Where a verb must **refuse**, as `--fail-on-untraced` must on an
empty universe, its non-zero status is the expected outcome and the question is
what the refusal said; there `grep`'s status is the right one to surface, and a
redirect would turn a correct refusal into a failing line. 3.2 now states both
halves rather than a rule with a silent exception.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture under `${TMPDIR:-/tmp}/ss059` and the captured
documents beside it carry the state instead. This block is spec 052's acceptance
(3.1) as well as this spec's own, so it is read in two halves and labelled as
such.

**Fail-first evidence.** Only `registry show 108` is red at the parent commit, a
not-found exit 1. The corrected lines are not, and measurably so: this spec
changes no code, so they exit 0 against the parent's binary. D-5 records what
each line was measured against instead, and that it failed there.

```verify:cli
# --- spec 052's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test render --locked
cargo test -p spec-spine-core --test coverage --locked
# 3.1 of spec 052, as 3.3 here reads it: `orphans --json` is an object carrying
# both named arrays. The document is captured to a file so the verb's own exit
# status is this line's status, which a pipeline into `python3` would not be
# (3.2, D-4). It is captured beside the fixture root rather than inside it,
# because the next line begins by deleting that root (D-6).
target/release/spec-spine index orphans --json > "${TMPDIR:-/tmp}/ss059-live.json"
# Presence, type, sorted keys and the version member. Not a key-set equality:
# 059 requires the two arrays to be there, not to be the only members, and
# spec 074 added `schemaVersion` to every governed read (3.3).
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss059-live.json')); k=list(d); assert k==sorted(k), k; assert d['schemaVersion'], d; assert isinstance(d['orphaned'], list), d; assert isinstance(d['inFlight'], list), d"
rm -f "${TMPDIR:-/tmp}/ss059-live.json"
# 3.2 + 3.3 of spec 052: the empty-universe fixture. Built once at a fixed path,
# because each line here is its own shell and a `$(mktemp -d)` would not survive
# to the next assertion. Inherited from 059's block unchanged.
rm -rf "${TMPDIR:-/tmp}/ss059" && mkdir -p "${TMPDIR:-/tmp}/ss059/specs/001-x" && : > "${TMPDIR:-/tmp}/ss059/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n' > "${TMPDIR:-/tmp}/ss059/specs/001-x/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index >/dev/null
# 3.4: 059 3.1's first recorded decision, on the corpus where it can be seen.
# Both arrays are emitted even though both are empty, so a consumer never has to
# tell absent from empty. Omitting an empty group fails this line (D-2).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index orphans --json > "${TMPDIR:-/tmp}/ss059-empty.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss059-empty.json')); assert d['orphaned']==[], d; assert d['inFlight']==[], d"
rm -f "${TMPDIR:-/tmp}/ss059-empty.json"
# 3.4: 059 3.1's second recorded decision. With nothing to report the prose form
# prints nothing, rather than two headers and two `(none)` lines. The redirect
# keeps the verb's status on its own line: `test -z "$(...)"` would pass for a
# verb that failed and printed nothing (D-4). This line fails against this
# repository's own corpus, which has orphans and prints them.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index orphans > "${TMPDIR:-/tmp}/ss059-prose.txt"
test ! -s "${TMPDIR:-/tmp}/ss059-prose.txt"
rm -f "${TMPDIR:-/tmp}/ss059-prose.txt"
# 3.2 of spec 052: without the flag it still exits 0 and reports the fact. The
# report is a read verb, and "no source files" is a true and useful thing to say.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage
# 3.2 of spec 052: with the flag it refuses. An assertion over an empty set is
# vacuously true, and a CI step that did not run its check should not be green.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced ; test $? -eq 1
# 3.3 of spec 052: and the message names which of the two empty cases this is.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced 2>&1 | grep -q 'no package was discovered'
rm -rf "${TMPDIR:-/tmp}/ss059"
# 3.4 of spec 052: this repository's own coverage assertion is unaffected, so
# the refusal is unreachable here.
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine compile --check
# --- spec 086's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-4 gives: at the parent commit this
# verb exits 1 and prints nothing, and a pipeline would report that as a JSON
# decode error naming the wrong defect. The file is named for this spec, whose
# mechanism it is, not for 059, whose acceptance the half above is (D-6).
target/release/spec-spine registry show 086 --json > "${TMPDIR:-/tmp}/ss108-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss108-show.json')); assert d['amendsVerification'] == ['052-read-verbs-on-a-code-free-corpus'], d; assert d['amends'] == ['052-read-verbs-on-a-code-free-corpus'], d"
rm -f "${TMPDIR:-/tmp}/ss108-show.json"
# Spec 052's file is not edited (spec 037 3.1): its own block still carries the
# superseded key-set equality. This goes red the moment someone resolves this by
# editing 059 instead.
grep -qF 'set(o) == {"orphaned", "inFlight"}' specs/052-read-verbs-on-a-code-free-corpus/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 059` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss108-spec103.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss108-spec103.txt"
rm -f "${TMPDIR:-/tmp}/ss108-spec103.txt"
```
