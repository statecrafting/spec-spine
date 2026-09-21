---
id: "087-the-answer-is-a-member-not-the-document"
title: "The answer is a member, not the document"
status: approved
kind: "core"
created: "2026-09-17"
summary: >
  Spec 053's `## Verification` block compares the whole of
  `registry plan --next --json` against one spec object, and spec 094 moved that
  object under a `next` member so the empty ready set could be a value rather
  than a missing document, so `spec-spine verify 060` has been red since 093
  merged. 093 declared an `amends` edge to 060 for exactly this change and
  carried the replacement text for 060's rule; what it could not reach was 060's
  acceptance, because spec 037 forbids editing the amended file and the block
  lives inside it. The same block leaves 060 3.2's strongest requirement
  unasserted: that `--next` is a projection of `plan` and does not reimplement
  selection, which a one-element fixture cannot show. This spec declares 060's
  acceptance replaced and carries the corrected block, without editing 060's
  file. It is the third of the four follow-ons spec 084 4 named.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "053-plan-answers-the-whole-question"
  - "074-a-governed-read-names-its-version"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  # D-7: not behavioural dependencies. This spec cites 107's and 108's decision
  # records and states its own position in the series, so the order those claims
  # assume is declared rather than left to the merge queue.
  - "085-a-version-pin-is-not-a-contract"
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
amends: ["053-plan-answers-the-whole-question"]
# 3.1: this spec's `## Verification` block IS 060's acceptance from now on.
# 060's own file is not edited (spec 037 3.1), and 093's is not either: 093
# already amended 060's rule and carried the replacement text, and this spec
# changes none of it. What changes is what 060 accepts.
amends_verification: ["053-plan-answers-the-whole-question"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 109: The answer is a member, not the document

## 1. Purpose

### 1.1 One red assertion, and this one has an amendment behind it

Measured on 2026-09-17 against `3173bb0` with the 0.20.0 binary:

```
$ spec-spine verify 060
verify: 053-plan-answers-the-whole-question: FAILED at command 8 (exit 1)
```

Command 8 is, in full, because an abbreviation of it misleads (D-9):

```
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss060" registry plan --next --json | python3 -c 'import json,sys; assert json.load(sys.stdin)=={"id":"001-alpha","title":"First thing"}'
```

and the verb answers

```json
{ "next": { "id": "001-alpha", "title": "First thing" }, "schemaVersion": "0.1.0" }
```

Run one line at a time to the end of the block, command 8 is the only red one:

| Cmd | Assertion | Status |
|-----|-----------|--------|
| 3 | the scratch corpus builds | green |
| 4 | `ready[0]` carries `id` and `title` | green |
| 5 | each `blocked` entry carries title and blocker states | green |
| 6, 7 | the prose form renders `not schedulable` and `blocked by 001-alpha` | green |
| 8 | the whole document equals the pick | **red**, the pick is under `next` |
| 9 | `plan --next` on an empty ready set exits 0 | green |
| 11 | `compile --check` | green |

`verify` stops at the first failure, so a reader who runs the verb sees command
8 and learns nothing about the other six.

### 1.2 This is not an over-strict assertion; it is an amended rule

The three red blocks beside this one are all assertions stricter than the rule
they stood for. This one is different, and the difference is worth stating,
because the repair looks identical and the reasoning is not.

060 3.2 states the shape as a contract: under `--json`, "the single spec object
rather than an array, so a consumer does not index into a one-element list to
reach the thing it asked for." The block asserted exactly that, and it was
right to. Spec 094 then **changed the contract**, deliberately and with its
reasons recorded: `plan --next`'s empty case emitted the bare literal `null`,
which cannot carry a version member, so the pick moved under a nullable `next`
and the document became an object in both cases. 093 3.5 carries the
replacement text and 093's frontmatter carries the `amends` edge naming 060.

So 060's rule was amended by the book, and the failure here is narrower than in
specs 049, 059 and 071: 093 amended the rule and could not reach the acceptance,
because spec 037 forbids editing the amended file and the block lives inside it.
That is spec 083 1.2's finding, which was about spec 080 and spec 079, restated
at a second site. Spec 082 built `amends_verification` after 093 shipped, so the
route did not exist when 093 needed it.

The consequence for this spec is that the corrected line asserts the **amended**
contract, with 093's document shape and 060's pick, rather than a loosened
version of 060's original.

### 1.3 The block never asserted 060's strongest requirement

060 3.2 asks for more than a shape:

> `--next` MUST be a projection of `plan` and MUST NOT reimplement selection.
> The first element of the topological order is already the defined pick; this
> flag names it so that callers stop re-deriving what "first" means.

The fixture 060's block builds has **one** ready spec. Against a one-element
ready set, "the first element" is the only element: an implementation that
reimplemented selection, or that returned the last element, or the
alphabetically greatest, passes command 8 and command 4 unchanged. The rule that
`--next` is a projection is the one 060 3.2 argues hardest for and the one its
acceptance cannot see.

One more spec in the fixture fixes it. With two ready specs the block can assert
that `--next`'s pick **is** `ready[0]` of the same corpus, and the inherited
`ready[0]` assertion stops being a one-element tautology and starts being a
claim about order.

### 1.4 The crossing was silent

Spec 094 changed a document shape another approved spec's acceptance pinned,
declared the `amends` edge, and nothing told 060's block. `verify` is the one
verb that executes what the corpus declares, so it sits outside the gate chain
deliberately (`AGENTS.md`); CI never runs it. Spec 084 1.3 records the identical
mechanism and names this spec's target among the four remaining.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 060's `## Verification` block is no longer the one
that runs. Nothing under `crates/` changes, no schema constant moves, and no
committed shard changes except the two this spec's own frontmatter produces.

## 3. Behavior

### 3.1 What spec 053's acceptance now is

This spec's `## Verification` block MUST replace spec 053's in full, through
`amends_verification` (spec 082 3.1), and 060's file MUST NOT be edited. The
block is 060's, with command 8 replaced per 3.3, with the fixture extended and
the projection asserted per 3.4, and with this spec's own assertions (3.5) after
them under a heading that says whose is whose.

Replacing the block in full rather than the one line follows spec 082 3.5 and
spec 083 D-1: a reader asking what 060 accepts today should find one block that
answers, not a base document plus a patch to apply in their head.

060's own dated decision of 2026-09-08, that the shape assertions run against a
scratch corpus because "the live corpus is an input, not a contract", MUST be
carried across unchanged. It is the reason the fixture exists, it is correct,
and 3.4's extension is a change to that fixture's contents, never a move back to
the live corpus.

### 3.2 The verb's exit status is the line's status

Commands 4, 5 and 8 pipe a verb into `python3`, so each line reports `python3`'s
status and the verb's is discarded. The replacement MUST redirect each document
to a file under `${TMPDIR:-/tmp}/ss060` and assert against the files, so a verb
that fails takes the line down at the redirect.

The rule reaches every line whose verb must **succeed**, which in this block is
every line: 3.5's `registry show`, where at the parent commit the verb exits 1
and prints nothing and a pipeline would report a JSON decode error for what is a
not-found, and the two prose assertions, which 060 wrote as
`registry plan | grep -q ...` and which are captured to a file here for the same
reason. Spec 085 D-4 records the redirect rule and spec 086 D-8 records its
boundary: a line whose verb is required to exit non-zero keeps its pipeline,
because there the filter's status is the one that should decide the line. This
block has no such line.

The fixture root stays at `${TMPDIR:-/tmp}/ss060`, which is named for spec 053,
whose acceptance this block is, not for this spec, which merely holds it.
Documents captured for this spec's own assertions are named for **109** and sit
outside that root, which keeps the two halves legible and leaves the fixture
line free to begin with `rm -rf`.

### 3.3 The pick is read from the member 093 put it in

The whole-document equality MUST be replaced by three claims:

- the document's keys are **sorted**, which is spec 094 3.2's rule for every
  governed read;
- it carries a non-empty `schemaVersion`, which is 093's addition and the half
  that fails against pre-093 output;
- `next` equals the pick, `{"id": "001-alpha", "title": "First thing"}`, which is
  060 3.2's requirement that the pick is the single spec object carrying both
  members, read through 093's envelope.

The pick is compared by value rather than by `id` alone, because 060 3.3's
reason for putting titles in the structure is that "no consumer needs a second
call", and an assertion that reads only the id would pass a `--next` that
dropped the title.

These are fixture values, not corpus state: the same block creates the corpus
three lines above, so naming `001-alpha` here is not the calendar-shaped
assertion 060's own 2026-09-08 decision rejected.

### 3.4 The fixture gains a second ready spec, so "first" can fail

The fixture MUST carry a third spec, `003-gamma`, approved and pending with no
dependencies, so that `ready` holds two entries.

The replacement MUST then assert that `--next`'s `next` equals `ready[0]` of
`plan --json` **on the same corpus**, which is 060 3.2's projection requirement.
Measured: with two ready specs, a `--next` answering `003-gamma` fails both that
line and the pick assertion of 3.3, and with one ready spec it fails neither.

The inherited `ready[0]` assertion gains from the same change without being
edited: against two entries it is a claim about ordering, where against one it
was a tautology.

Nothing else in the fixture moves. `002-beta` stays blocked by `001-alpha`, the
`blocked` assertions are untouched, and the prose assertions still match;
measured, the prose form reports `3 specs: 2 ready, 1 blocked, 0 not
schedulable`.

### 3.5 The acceptance

`registry show 109 --json` MUST carry `amendsVerification` and `amends`, both
naming 060 and nothing else: that is the declaration, and it is read through the
CLI rather than off the shard.

Spec 053's file MUST still carry the superseded whole-document equality. That is
the spec 037 3.1 half, and it is the assertion that goes red if someone ever
resolves this by editing 060 instead, which is the move
`.claude/rules/adversarial-prompt-refusal.md` refuses.

The resolution itself MUST keep passing through spec 082's own cases (`spec103_`
in `crates/spec-spine-core/tests/verify.rs`). No code changes here, so the
mechanism is what is asserted, not a new one. The run MUST be captured and its
summary asserted to name a non-zero pass count: a name filter that matches
nothing exits 0, so the bare line would stay green while asserting nothing (spec
084 D-7).

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 060` runs, and `cmd_verify::run` refuses a nested call on its
own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so even
reading the plan from inside is a validation failure. The instance-level fact,
that `spec-spine verify 060` and `spec-spine verify 109` both exit 0 on the
merged tree, is what a reviewer runs and what the release sweep runs.

### 3.6 The empty ready set stays where spec 094 put it

The block MUST keep 060's `registry plan --next` line against this repository,
which asserts that an empty ready set is exit 0 rather than a failure, and MUST
NOT grow an assertion on that document's contents.

093 3.8 requires `registry plan --next --json` on a corpus with an empty ready
set to emit `next: null` at exit 0, asserted in
`crates/spec-spine-cli/tests/cli.rs`, and records that constructing that corpus
is the test's work because "without it the `null` path is unexercised". That case
is covered, in a test that owns a fixture for it. Duplicating it here would need
a second scratch corpus for a path another spec already guards, and asserting it
against **this** repository instead would pin corpus state, which is the defect
060's own 2026-09-08 decision was written to avoid.

### 3.7 No code changes

No file under `crates/` is edited, no `*_SCHEMA_VERSION` constant moves, no
embedded schema changes, and no CLI surface is added or removed. The mechanism
this spec uses was built and shipped by spec 082 in v0.20.0; this spec is a
corpus change that uses it.

## 4. Out of scope

- **Editing spec 053, or spec 094.** 1.2 and the frontmatter comment. 060 keeps
  the block it was ratified with, which is the record spec 037 3.2 protects, and
  093 already amended 060's rule by the book and carried the replacement text.
- **The empty-ready-set document.** 3.6. Guarded by 093 3.8 in
  `crates/spec-spine-cli/tests/cli.rs`, which owns a fixture for it.
- **The last red block.** Spec 093 is red for the same family of reason and needs
  its own amendment, because `amends_verification` replaces a target's whole
  block with the amender's single one. Spec 084 4 names all four; 107 took 056,
  108 took 059, and this is the third.
- **The remaining green literal pins, and the piped `registry show` lines.** Spec
  047's block pins `config_version` at `0.1.0`, spec 083's carries a
  `schemaVersion=='0.4.0'` inherited from 098, and specs 083 and 106 both read
  `registry show` through a pipeline. All are green, and a green line in another
  spec's block is not this spec's to change; 106 4 and 107 4 name them.
- **Teaching `amends_verification` in the template.**
  `standards/spec/templates/spec-template.md` does not carry the key spec 082
  added. 106 4 names it as a real gap in territory neither spec owns.

## 5. Resolved decisions

D-1 (2026-09-17, why the pick is compared by value and not by id). The narrow
repair is `d["next"]["id"] == "001-alpha"`, which fixes the redness. It also
drops the half of 060 3.3 that the fixture exists to show: titles are in the
structure so that "no consumer needs a second call", and an id-only assertion
passes a `--next` that returns the id alone. Comparing the object by value costs
the same line and asserts both members. Spec 082's block, which is 093's
acceptance, reads only `d["next"]["id"]`, and that is right for 093: 093's rule
is about the envelope, not about what the pick carries.

D-2 (2026-09-17, why this spec does not read as a loosening). Specs 085 and 108
correct assertions that were stricter than their rules, and a reader meeting
three of these in a row could take the family to mean that acceptance blocks
should assert less. This one does not fit that shape, and 1.2 says so explicitly:
060's assertion matched 060's rule exactly, and 093 changed the rule with an
`amends` edge and replacement text. The corrected line is not looser than the
original, it is the same strength against the amended contract, and 3.4 makes
the block strictly stronger than it was. Recorded because the distinction is the
whole reason this spec is separate from 107 and 108 rather than a fourth bullet
in either.

D-3 (2026-09-17, why the fixture gains a spec). Editing an inherited fixture is
scope this spec had to justify, and the justification is that the fixture as
built cannot show the rule 060 3.2 argues hardest for. With one ready spec,
"`--next` is the first element of `ready`" has no failing case: first, last,
greatest and any reimplementation all return the same object. Measured, adding
`003-gamma` makes a `--next` that answers `003-gamma` fail two lines, and the
inherited `ready[0]` assertion becomes a claim about order. The alternative,
leaving the projection rule unasserted and noting the gap in 4, is what spec 086
D-3 chose for 059's id-ordering rule; the difference is that there no fixture
change could avoid a vacuous assertion on a corpus with at most two entries,
while here one `printf` chained into a line that already has three does it.

D-4 (2026-09-17, why the empty ready set is not asserted here). See 3.6. It is
covered by 093 3.8 in `crates/spec-spine-cli/tests/cli.rs` with a fixture built
for it. The tempting cheap version, asserting the text of `registry plan --next`
against **this** repository because its ready set happens to be empty today, is
the exact defect 060's own 2026-09-08 decision entry was written about, one line
below where it would go. The inherited line stays as 060 wrote it: a bare
invocation that asserts exit 0 and nothing about the contents.

D-5 (2026-09-17, why the pipelines become redirects). `cmd --json | python3 -c
...` reports `python3`'s status, so a verb that failed and printed nothing feeds
`json.load` an empty document and the line fails for the wrong reason with a
message naming the wrong defect. Three of 060's lines have the form and 3.4's
new assertion needs two documents at once, which is where spec 084 D-4 records
the hazard turning silent: two failures can compare equal. The `registry show`
line is included for the reason spec 085 D-4 gives, which review established on
that spec's pull request rather than in drafting.

D-6 (2026-09-17, what the fail-first evidence is and is not). This spec changes
no code, so the corrected lines pass at the parent commit `3173bb0` against the
same binary whose output made 060's original red. That is correct rather than a
gap, and it is spec 084 D-5's finding restated at a fourth site.

What an assertion owes instead is **failability against the condition it exists
to catch**, and each line was run against that condition. Measured on 2026-09-17
by substituting documents against the extended fixture:

| Condition | Line's status |
|---|---|
| pre-093 output, the bare pick object | fails (`KeyError: 'next'`) |
| `next` holding a one-element array | fails |
| the pick answering `003-gamma` | fails, and so does the projection line |
| `ready[0]` and `next` disagreeing | fails |
| keys emitted in struct order rather than sorted | fails |
| no `schemaVersion` member | fails |
| the tree as it stands, with the extended fixture | passes |

The one line that is fail-first at the parent in the ordinary sense is
`registry show 109`, a not-found exit 1 there, which is 3.5's half.

D-7 (2026-09-17, why 107 and 108 are declared dependencies). This spec cites
both: D-2 contrasts its own shape with the corrections those two make, D-3
compares its fixture decision with spec 086 D-3's, and 3.2 takes spec 085 D-4's
redirect rule together with spec 086 D-8's boundary on it. 4 states this as the
third of four. Review of spec 086's pull request established that a past-tense
claim about a spec absent from the base is either a false claim or an undeclared
ordering, and that declaring it is the honest resolution. The ordering is real:
the four repairs were built in one sitting and each carries the previous one's
findings. The dependency is on their **records**, not on anything they do;
nothing here executes, reads or relies on their behaviour, and no two of the four
blocks share state.

D-8 (2026-09-17, what the fixture assertions rest on, and why the contingency is
loud). Review of the pull request named three assumptions this block carries, and
all three are real. They are recorded here rather than only in a review thread,
because a contingency a reader cannot find is the thing this series exists to
unwind.

`ready[0]` is `001-alpha` only because the two ready specs are independent and
the order is settled by the ordinal tie-break spec 046 governs. If that tie-break
ever moved, the assertion would **fail**, loudly and by value: `p['ready'][0]` is
compared against the whole object, so a reordered plan reports
`{'id': '003-gamma', ...}` and the line goes red. It does not go vacuous. The
projection line beside it would keep passing, correctly, because what it asserts
is that `--next` follows `ready` wherever `ready` points, which is 060 3.2's
actual rule.

The two prose greps depend on `registry plan` rendering the `not schedulable`
label at a count of zero. 060's block established that it does, and this spec
measured it again on the extended fixture. If the CLI ever suppressed the section
at zero those lines fail, again visibly, and a reader is sent here.

`grep -qE 'test result: ok\. [1-9][0-9]* passed'` depends on the stable Rust test
harness's output format, which is outside this project's control. It is the form
spec 084 D-7 introduced, and the alternative, a bare filtered run, is the vacuous
pass that entry exists to close. A dependency on an external format that fails
closed is better than an assertion that cannot fail.

The common property is the one spec 085 D-3 turns on: each of these fails in the
direction that sends a reader to a decision entry, rather than passing while
asserting less than it appears to.

D-9 (2026-09-17, why command 8 is quoted in full). The first draft of 1.1 quoted
the line with its `import json,sys;` prefix and its `--repo` argument elided, for
width. Review read the elision as the line itself and concluded that command 8
had been failing on `NameError: name 'json' is not defined` rather than on the
shape, and that 1.1's whole account of the redness was therefore wrong.

It is not: the imports are in 060's file at line 238, and the measured failure at
`e4d7d28` is an `AssertionError` on the comparison, which is what 1.1 says. But
the objection is the right shape even though its conclusion is not, and the
defect it lands on is this spec's: a quotation that changes what a command does
is not a quotation. In a spec whose subject is assertions that say less than they
appear to, an elision that makes an assertion appear to say **more** than it does
is the same error pointing the other way. The line is now reproduced verbatim,
and the next reader can run it.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture under `${TMPDIR:-/tmp}/ss060` and the documents
captured beside it carry the state instead. This block is spec 053's acceptance
(3.1) as well as this spec's own, so it is read in two halves and labelled as
such.

**Fail-first evidence.** Only `registry show 109` is red at the parent commit, a
not-found exit 1. The corrected lines are not, and measurably so: this spec
changes no code, so they exit 0 against the parent's binary. D-6 records what
each line was measured against instead, and that it failed there.

**Spec 053's decision of 2026-09-08 is carried across.** The shape assertions run
against a scratch corpus rather than this repository's plan, because this
repository's plan is corpus state and corpus state moves. 3.4 changes what that
corpus contains and never where the assertions point.

```verify:cli
# --- spec 053's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test query --locked
# A scratch corpus with two ready specs and one blocked by the first, so the
# shape assertions below hold whatever this repository's own backlog is doing,
# and so "the first element of ready" has a failing case (3.4, D-3).
rm -rf "${TMPDIR:-/tmp}/ss060" && mkdir -p "${TMPDIR:-/tmp}/ss060/specs/001-alpha" "${TMPDIR:-/tmp}/ss060/specs/002-beta" "${TMPDIR:-/tmp}/ss060/specs/003-gamma" && : > "${TMPDIR:-/tmp}/ss060/spec-spine.toml" && printf -- '---\nid: "001-alpha"\ntitle: "First thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/001-alpha/spec.md"\n---\n\n# 001-alpha\n## body\n' > "${TMPDIR:-/tmp}/ss060/specs/001-alpha/spec.md" && printf -- '---\nid: "002-beta"\ntitle: "Second thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\ndepends_on:\n  - "001-alpha"\nestablishes:\n  - "specs/002-beta/spec.md"\n---\n\n# 002-beta\n## body\n' > "${TMPDIR:-/tmp}/ss060/specs/002-beta/spec.md" && printf -- '---\nid: "003-gamma"\ntitle: "Third thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/003-gamma/spec.md"\n---\n\n# 003-gamma\n## body\n' > "${TMPDIR:-/tmp}/ss060/specs/003-gamma/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss060" compile >/dev/null
# The two documents are captured to files, so each verb's own exit status is its
# line's status, which a pipeline into `python3` would not be (3.2, D-5).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss060" registry plan --json > "${TMPDIR:-/tmp}/ss060/plan.json"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss060" registry plan --next --json > "${TMPDIR:-/tmp}/ss060/next.json"
# 060 3.3: the ready array carries titles, so no consumer needs a second call.
# With two ready specs this is also a claim about order (3.4).
python3 -c "import json; p=json.load(open('${TMPDIR:-/tmp}/ss060/plan.json')); assert p['ready'][0]=={'id':'001-alpha','title':'First thing'}, p"
# 060 3.1: and each blocked entry carries its title and the state of every
# blocker, rather than a count of them.
python3 -c "import json; b=json.load(open('${TMPDIR:-/tmp}/ss060/plan.json'))['blocked'][0]; assert b['title']=='Second thing', b; assert b['blockedBy'][0]['id']=='001-alpha', b; assert b['blockedBy'][0]['state'], b"
# 060 3.1: the prose form renders what the structure holds, remainder included.
# Captured once rather than piped twice, so the verb's own exit status is a
# line's status and the two assertions read the same rendering (3.2).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss060" registry plan > "${TMPDIR:-/tmp}/ss060/plan.txt"
grep -q 'not schedulable' "${TMPDIR:-/tmp}/ss060/plan.txt"
grep -q 'blocked by 001-alpha' "${TMPDIR:-/tmp}/ss060/plan.txt"
# 3.3: 060 3.2's pick, read from the member spec 094 moved it into. Sorted keys
# and the version member are 093 3.2's rule for every governed read, and the
# pick is compared by value so a `--next` that dropped the title fails (D-1).
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss060/next.json')); k=list(d); assert k==sorted(k), k; assert d['schemaVersion'], d; assert d['next']=={'id':'001-alpha','title':'First thing'}, d"
# 3.4: 060 3.2's projection requirement, which a one-element ready set cannot
# show. `--next` names the first element of `ready` rather than reimplementing
# selection; with two ready specs, answering `003-gamma` fails this line.
python3 -c "import json; n=json.load(open('${TMPDIR:-/tmp}/ss060/next.json'))['next']; p=json.load(open('${TMPDIR:-/tmp}/ss060/plan.json')); assert p['ready'][0]==n, (p['ready'], n)"
# 060 3.2: and an empty ready set is a true answer at exit 0, not a failure.
# This repository is that case now. Nothing is asserted about the contents:
# that document's `next: null` path is guarded by spec 094 3.8 in
# crates/spec-spine-cli/tests/cli.rs, and asserting it here would pin corpus
# state, which 060's own decision of 2026-09-08 rejects (3.6, D-4).
target/release/spec-spine registry plan --next
rm -rf "${TMPDIR:-/tmp}/ss060"
# 060 3.3: the ledger is untouched by a read verb.
target/release/spec-spine compile --check
# --- spec 087's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-5 gives: at the parent commit this
# verb exits 1 and prints nothing. The file is named for this spec, whose
# mechanism it is, not for 060, whose acceptance the half above is (3.2).
target/release/spec-spine registry show 109 --json > "${TMPDIR:-/tmp}/ss109-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss109-show.json')); assert d['amendsVerification'] == ['053-plan-answers-the-whole-question'], d; assert d['amends'] == ['053-plan-answers-the-whole-question'], d"
rm -f "${TMPDIR:-/tmp}/ss109-show.json"
# Spec 053's file is not edited (spec 037 3.1): its own block still carries the
# superseded whole-document equality. This goes red the moment someone resolves
# this by editing 060 instead.
grep -qF 'assert json.load(sys.stdin)=={"id":"001-alpha","title":"First thing"}' specs/053-plan-answers-the-whole-question/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 060` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss109-spec103.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss109-spec103.txt"
rm -f "${TMPDIR:-/tmp}/ss109-spec103.txt"
```
