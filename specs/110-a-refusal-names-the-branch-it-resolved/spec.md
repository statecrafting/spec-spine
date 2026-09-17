---
id: "110-a-refusal-names-the-branch-it-resolved"
title: "A refusal names the branch it resolved"
status: approved
kind: "core"
created: "2026-09-17"
summary: >
  Spec 071's `## Verification` block greps the push gate's refusal message for
  the literal string `this would update main`, and spec 072 legitimately replaced
  that literal with the branch name the hook resolves, so `spec-spine verify 071`
  has been red since 072 merged. 071 3.2 requires the message to say what was
  actually refused and to name the tag push as allowed; it never required the
  default branch to be spelled `main`, and after 072 a message that said `main`
  on a repository whose default is something else would be saying the one thing
  071 3.2 forbids. The same block checks the message in one of the three copies
  it checks every other assertion in. This spec declares 071's acceptance
  replaced and carries the corrected block, without editing 071's file. It is the
  last of the four follow-ons spec 106 4 named.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "071-a-tag-push-is-not-a-push-to-main"
  - "072-the-default-branch-is-configured-not-assumed"
  - "103-an-amended-acceptance-is-the-one-that-runs"
  # D-7: not behavioural dependencies. This spec cites each of the three
  # preceding repairs and states its own position in the series, so the order
  # those claims assume is declared rather than left to the merge queue.
  - "107-a-version-pin-is-not-a-contract"
  - "108-an-exact-key-set-refuses-what-the-rule-allows"
  - "109-the-answer-is-a-member-not-the-document"
amends: ["071-a-tag-push-is-not-a-push-to-main"]
# 3.1: this spec's `## Verification` block IS 071's acceptance from now on.
# 071's own file is not edited (spec 040 3.1), and 072's is not either: 072
# states a rule that is true and complete, and this spec changes none of it.
# What changes is what 071 accepts.
amends_verification: ["071-a-tag-push-is-not-a-push-to-main"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 110: A refusal names the branch it resolved

## 1. Purpose

### 1.1 One red assertion, no defect behind it

Measured on 2026-09-17 against `e4d7d28` with the 0.20.0 binary:

```
$ spec-spine verify 071
verify: 071-a-tag-push-is-not-a-push-to-main: FAILED at command 10 (exit 1)
```

Command 10 is

```
grep -qF 'this would update main' kit/settings.json
```

and the shipped message reads

```
[push-gate] BLOCKED: this would update $def (repo: $root, branch: ${br:-unknown}). ...
```

Spec 072 replaced the literal with the name the hook resolves, in the three
copies of the hook and in the comparisons around it. Every other assertion in
071's block is green. Run one line at a time to the end of the block, this is the
only red one:

| Cmd | Assertion | Status |
|-----|-----------|--------|
| 2, 3 | `kit_hooks` and `scaffold` test suites | green |
| 4 to 6 | the anchored outer match, in all three copies | green |
| 7 to 9 | the unconditional branch refusal is gone, in all three | green |
| 10 | the message says `this would update main` | **red**, it says `$def` |
| 11 | the message names the allowed tag push | green |

`verify` stops at the first failure, so a reader who runs the verb sees command
10 and learns nothing about command 11.

### 1.2 After 072, the literal is the thing 071 3.2 forbids

071 3.2's last paragraph is the requirement command 10 stood for:

> The refusal message MUST say what was actually refused, rather than restating
> the rule, and MUST name the tag push as allowed, so a maintainer who hits it
> can tell a correct refusal from a misfire.

The word `main` appears nowhere in it. 071 was written for a repository whose
default branch is `main`, so the literal was a true rendering of "what was
actually refused" on the day it was written, and 072 then made the protected
branch a resolved value: `$SPEC_SPINE_DEFAULT_BRANCH`, then the remote's own
HEAD, then `main` as a floor.

After 072 the literal is not merely stale. On a repository whose default branch
is `master` or `trunk`, a message reading "this would update main" would name a
branch the gate did not protect and did not refuse a push to, which is precisely
the misfire 071 3.2's sentence exists to let a maintainer detect. The assertion
now points away from the requirement it was written for.

072 declared `amends` on 046, whose 3.4 states the refusal in terms of the
literal branch name, and owed 071 no such edge: 071 3.2 states a property of the
message, and 072 preserved that property while changing the value the message
renders. What 072 invalidated here was an assertion narrower than 071's rule.
This is the same finding spec 108 3.6 records for 093 and 059.

### 1.3 The message is checked in one copy, where everything else is checked in three

Every other grep assertion in 071's block runs against all three copies of the
hook: `kit/settings.json`, `.claude/settings.json`, and the generated
`crates/spec-spine-core/src/kit_embedded.rs`. Three lines for 3.1's anchored
match, three for 3.2's removed refusal, and then **one** for the message.

That asymmetry is not a decision 071 records. It is also not the coverage hole it
looks like, and this spec says so rather than claiming a gap it would then take
credit for closing. The copies are proven equal by the two test suites the block
already runs at commands 2 and 3: `crates/spec-spine-core/tests/kit_hooks.rs`
asserts that the hook bodies in `.claude/settings.json` are the ones
`kit/settings.json` ships (spec 051 3.6), and `tests/scaffold.rs` asserts that
the embedded copy has not drifted from the tree (spec 065). A message present in
one copy and absent from another fails those tests before any grep here runs.

So extending the message assertions to three copies is **consistency with the
surrounding lines, not new coverage**, and 3.3 records it as that. What it buys
is a block a reader can scan without stopping to work out why one assertion is
spelled differently from the six around it, and a grep that does not depend on
another suite's correctness to mean what it says.

A fourth copy exists. `.codex/hooks.json` carries the same bodies and the same
test asserts it, and it arrived with spec 081, long after 071. It stays out of
071's acceptance for the reason D-5 gives about the message's `(repo: ...)`
context: an acceptance block is the record of what **that** spec requires.

### 1.4 The crossing was silent

Spec 072 changed a message another approved spec's acceptance grepped for, and
nothing told 071. `verify` is the one verb that executes what the corpus
declares, so it sits outside the gate chain deliberately (`AGENTS.md`); CI never
runs it. Spec 106 1.3 records the identical mechanism and named this spec's
target among the four remaining; this is the last of them.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 071's `## Verification` block is no longer the one that
runs. Nothing under `crates/`, `kit/` or `.claude/` changes, no schema constant
moves, and no committed shard changes except the two this spec's own frontmatter
produces.

## 3. Behavior

### 3.1 What spec 071's acceptance now is

This spec's `## Verification` block MUST replace spec 071's in full, through
`amends_verification` (spec 103 3.1), and 071's file MUST NOT be edited. The
block is 071's, with command 10 replaced per 3.2, with the message assertions
extended to all three copies per 3.3, and with this spec's own assertions (3.4)
after them under a heading that says whose is whose.

Replacing the block in full rather than the one line follows spec 103 3.5 and
spec 105 D-1: a reader asking what 071 accepts today should find one block that
answers, not a base document plus a patch to apply in their head.

### 3.2 The message assertion reads the resolved name, and the literal's absence

Command 10 MUST be replaced by two assertions per copy:

- that the message reads `BLOCKED: this would update $def`, the name 072
  resolves, rather than any literal branch name;
- that `this would update main` appears **nowhere** in the file.

The negative is not redundant. A hook that gained a resolved message while
leaving the literal one behind on another branch of the same `case` would pass
the positive line and ship two refusals that disagree, which is the shape 071 3.2
was written against. 071's block already uses this idiom: its 3.2 assertions are
three `!  grep` lines asserting that the unconditional refusal is gone.

`BLOCKED:` is included in the matched string so the line reads the refusal the
gate emits rather than any prose that happens to describe it. The hook bodies are
shell embedded in JSON, so a fixed-string grep over the file is what an assertion
about the message can be; `crates/spec-spine-core/tests/kit_hooks.rs` runs the
gate against a command matrix and asserts verdicts, not message text.

### 3.3 The message is asserted in all three copies 071 already checks

Both assertions of 3.2, and 071's surviving assertion that the message names the
allowed tag push, MUST run against `kit/settings.json`, `.claude/settings.json`
and `crates/spec-spine-core/src/kit_embedded.rs`, matching what 071's block
already does for its other six grep assertions.

This is nine lines where 071 had two, and 1.3 is explicit that the eight added
ones are redundancy rather than coverage: `kit_hooks.rs` and `scaffold.rs`,
which the block runs two commands earlier, already refuse a divergence between
the copies. The reason to add them anyway is that a block whose assertions are
spelled the same way is a block a reader can check, and that a grep which does
not lean on another suite's correctness says what it appears to say.

`.codex/hooks.json` MUST NOT be added. It postdates 071 (spec 081), and 1.3 and
D-5 give the reason.

### 3.4 The acceptance

`registry show 110 --json` MUST carry `amendsVerification` and `amends`, both
naming 071 and nothing else: that is the declaration, and it is read through the
CLI rather than off the shard.

Spec 071's file MUST still carry the superseded literal. That is the spec 040 3.1
half, and it is the assertion that goes red if someone ever resolves this by
editing 071 instead, which is the move
`.claude/rules/adversarial-prompt-refusal.md` refuses.

The resolution itself MUST keep passing through spec 103's own cases (`spec103_`
in `crates/spec-spine-core/tests/verify.rs`). No code changes here, so the
mechanism is what is asserted, not a new one. The run MUST be captured and its
summary asserted to name a non-zero pass count: a name filter that matches
nothing exits 0, so the bare line would stay green while asserting nothing (spec
106 D-7). The `registry show` read MUST be redirected rather than piped, for the
reason spec 107 D-4 gives.

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 071` runs, and `cmd_verify::run` refuses a nested call on its
own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so even
reading the plan from inside is a validation failure. The instance-level fact,
that `spec-spine verify 071` and `spec-spine verify 110` both exit 0 on the
merged tree, is what a reviewer runs and what the release sweep runs.

### 3.5 No code changes

No file under `crates/`, `kit/` or `.claude/` is edited, no `*_SCHEMA_VERSION`
constant moves, no embedded schema changes, and no CLI surface is added or
removed. In particular the hook bodies are not touched: they are correct, and
what was wrong was an assertion about them. The mechanism this spec uses was
built and shipped by spec 103 in v0.20.0; this spec is a corpus change that uses
it.

## 4. Out of scope

- **Editing spec 071, or spec 072.** 1.2 and the frontmatter comment. 071 keeps
  the block it was ratified with, which is the record spec 040 3.2 protects, and
  072's text is true as written.
- **Asserting the message by running the gate.** `kit_hooks.rs` runs the gate
  against a `(command, branch)` matrix and asserts verdicts. Extending it to
  capture and assert the refusal text is a change to a file under `crates/`,
  which is code this spec does not touch, and it would need its own spec and its
  own territory. The grep is what an assertion about shell-embedded-in-JSON can
  be from a corpus-only change. D-3.
- **The `(repo: ..., branch: ...)` context in the message.** It reads well and it
  is not 071's: it predates that spec, having arrived with 045/046/047. Asserting
  it inside 071's acceptance would put another spec's contribution under 071's
  name. D-5.
- **The remaining green literal pins, and the piped `registry show` lines.** Spec
  054's block pins `config_version` at `0.1.0`, spec 105's carries a
  `schemaVersion=='0.4.0'` inherited from 098, and specs 105 and 106 both read
  `registry show` through a pipeline. All are green, and a green line in another
  spec's block is not this spec's to change; 106 4 and 107 4 name them.
- **Teaching `amends_verification` in the template.**
  `standards/spec/templates/spec-template.md` does not carry the key spec 103
  added. 106 4 names it as a real gap in territory neither spec owns. With this
  spec the four repairs are done and that gap is the one item of the audit left
  standing.

## 5. Resolved decisions

D-1 (2026-09-17, why the matched string keeps `$def` rather than becoming
branch-agnostic). A looser assertion is available: grep for `this would update `
and stop there, which survives any future change to how the name is produced.
It also passes a hook that reverted to the literal, because `this would update
main` contains `this would update `. The point of the line after 072 is which
name the message renders, so the string that is matched has to contain the
mechanism. `$def` is a shell expansion in a file that is shell embedded in JSON,
which is why the grep is `-F` and the pattern is single-quoted: `-F` and the
quoting together mean the `$` is matched as a character and never expanded, by
grep or by the shell running the line.

D-2 (2026-09-17, why the negative assertion is kept alongside the positive). Two
refusal messages can coexist in one `case` statement, and a hook that grew a
resolved message while keeping a literal one on another arm would pass the
positive line while shipping two refusals that disagree about what is protected.
The negative closes that, and it is the idiom 071's own block already uses for
its 3.2 assertions. Measured: against a copy of `kit/settings.json` with `$def`
substituted back to `main`, the positive line fails and the negative line fails.

D-3 (2026-09-17, why the message is not asserted by running the gate). 071 3.3's
principle is that the gate is asserted by running it, and `kit_hooks.rs` does
exactly that for verdicts. Extending it to assert the refusal **text** would be
better than a grep, and it is a change to a file under `crates/`: new test code,
inside spec 071's territory, from a spec whose 2 says it establishes no code.
That is a different change with a different risk profile, and folding it into an
acceptance repair would make this spec a build. The grep asserts the property
from outside, on all three copies, which is more than the block did before.

D-4 (2026-09-17, why the three-copy extension is called redundancy rather than a
fix). The first draft of 1.3 said "nothing asserts that `.claude/` carries the
message the kit ships", which is false: `kit_hooks.rs` compares the hook bodies
across `kit/settings.json`, `.claude/settings.json` and `.codex/hooks.json` and
asserts they are equal, citing spec 051 3.6, and the block runs it at command 2.
Checking before writing is what caught it. The extension is still made, for
legibility and for independence from another suite, and it is recorded as what it
is. A spec in a series about assertions that claim more than they establish
cannot afford an argument that does the same thing.

D-5 (2026-09-17, why the `(repo: ..., branch: ...)` context is not asserted). It
is part of what makes the refusal actionable and it would be a fair thing for an
acceptance to hold. It is not 071's: `git log -S` puts its arrival in the
045/046/047 commit, before 071 existed. An acceptance block is the record of what
**that** spec requires, and importing a neighbour's contribution into it would
make a future reader attribute the requirement to the wrong spec, which is the
class of confusion this whole series exists to unwind.

D-6 (2026-09-17, what the fail-first evidence is and is not). This spec changes
no code, so the corrected lines pass at the parent commit against the same files
whose contents made 071's original red. That is correct rather than a gap, and it
is spec 106 D-5's finding restated at a fifth and final site.

What an assertion owes instead is **failability against the condition it exists
to catch**, and each line was run against that condition. Measured on 2026-09-17
against copies of `kit/settings.json` with the message rewritten:

| Condition | Line's status |
|---|---|
| the pre-072 message, `this would update main` | the positive line fails |
| the same | the negative line fails |
| the tag-push sentence removed | that line fails |
| the files as they stand | all nine pass |

The one line that is fail-first at the parent in the ordinary sense is
`registry show 110`, a not-found exit 1 there, which is 3.4's half.

D-7 (2026-09-17, why 107, 108 and 109 are declared dependencies). This spec cites
each of them: 1.2 borrows spec 108 3.6's finding about an assertion narrower than
its rule, 3.4 takes spec 107 D-4's redirect rule, and 4 states this as the last of
four. Review of spec 108's pull request established that past-tense claims about
a spec absent from the base are either a false claim or an undeclared ordering,
and that declaring it is the honest resolution. The ordering is real: the four
repairs were built in one sitting and each carries the previous one's findings.
The dependency is on their **records**, not on anything they do; nothing here
executes, reads or relies on their behaviour, and no two of the four blocks share
state.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line. This block is spec 071's acceptance (3.1) as well as this spec's own,
so it is read in two halves and labelled as such.

**Fail-first evidence.** Only `registry show 110` is red at the parent commit, a
not-found exit 1. The corrected lines are not, and measurably so: this spec
changes no code, so they exit 0 against the parent's tree. D-6 records what each
line was measured against instead, and that it failed there.

```verify:cli
# --- spec 071's acceptance, which this block now holds (3.1) ---
cargo build --release --locked
# 071 3.3: the guard that runs the gate against the command matrix.
cargo test -p spec-spine-core --test kit_hooks --locked
# 065's generator keeps the embedded copy in step with the tree.
cargo test -p spec-spine-core --test scaffold --locked
# 071 3.1: the anchored outer match reaches all three copies.
grep -qF "in 'git push'*|*'&& git push'*" kit/settings.json
grep -qF "in 'git push'*|*'&& git push'*" .claude/settings.json
grep -qF "in 'git push'*|*'&& git push'*" crates/spec-spine-core/src/kit_embedded.rs
# 071 3.2: the unconditional branch refusal is gone from all three.
! grep -qF '= main ] && blk=1' kit/settings.json
! grep -qF '= main ] && blk=1' .claude/settings.json
! grep -qF '= main ] && blk=1' crates/spec-spine-core/src/kit_embedded.rs
# 3.2: 071 3.2 requires the message to say what was ACTUALLY refused. Since
# spec 072 that is the branch the hook resolves, so the message renders `$def`
# and not a literal. `-F` with a single-quoted pattern matches the `$` as a
# character: it is expanded neither by grep nor by the shell running the line
# (D-1). Asserted in all three copies (3.3).
grep -qF 'BLOCKED: this would update $def' kit/settings.json
grep -qF 'BLOCKED: this would update $def' .claude/settings.json
grep -qF 'BLOCKED: this would update $def' crates/spec-spine-core/src/kit_embedded.rs
# 3.2: and the literal is gone, not merely joined. Two refusals can coexist in
# one `case`, and a hook that kept a literal arm would pass the lines above
# while shipping two messages that disagree about what is protected (D-2).
! grep -qF 'this would update main' kit/settings.json
! grep -qF 'this would update main' .claude/settings.json
! grep -qF 'this would update main' crates/spec-spine-core/src/kit_embedded.rs
# 071 3.2: the message names the allowed tag push, which is the half that lets
# a maintainer tell a correct refusal from a misfire. Also in all three (3.3).
grep -qF 'A tag push such as' kit/settings.json
grep -qF 'A tag push such as' .claude/settings.json
grep -qF 'A tag push such as' crates/spec-spine-core/src/kit_embedded.rs
# --- spec 110's own mechanism (3.4) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped: at the parent commit this verb exits 1 and prints
# nothing, and a pipeline would report that as a JSON decode error naming the
# wrong defect (spec 107 D-4).
target/release/spec-spine registry show 110 --json > "${TMPDIR:-/tmp}/ss110-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss110-show.json')); assert d['amendsVerification'] == ['071-a-tag-push-is-not-a-push-to-main'], d; assert d['amends'] == ['071-a-tag-push-is-not-a-push-to-main'], d"
rm -f "${TMPDIR:-/tmp}/ss110-show.json"
# Spec 071's file is not edited (spec 040 3.1): its own block still carries the
# superseded literal. This goes red the moment someone resolves this by editing
# 071 instead.
grep -qF "grep -qF 'this would update main' kit/settings.json" specs/071-a-tag-push-is-not-a-push-to-main/spec.md
# The resolution this spec relies on is spec 103's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 071` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.4).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 106 D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss110-spec103.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss110-spec103.txt"
rm -f "${TMPDIR:-/tmp}/ss110-spec103.txt"
```
