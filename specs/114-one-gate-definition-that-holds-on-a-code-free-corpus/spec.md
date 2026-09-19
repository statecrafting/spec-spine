---
id: "114-one-gate-definition-that-holds-on-a-code-free-corpus"
title: "One gate definition, and it holds on a code-free corpus"
status: draft
kind: "tooling"
created: "2026-09-17"
summary: >
  `make gate` exits 1 on the repository `spec-spine init --with-kit` has just
  created. The refusal is spec 059's, which made `index coverage
  --fail-on-untraced` refuse an empty coverage universe rather than pass it
  vacuously, and that is the right answer for a CI step named for the ownership
  assertion and the wrong one for the composite gate a specify-first adopter runs
  from minute one. `kit/AGENTS.md` already states the condition, as a commented
  line; `kit/Makefile` runs the assertion unguarded and `kit/govern.yml`'s
  pull-request leg restates the whole chain a third time, in a file whose own
  header says the gate has one definition rather than two that drift. This spec
  gives the gate one definition, gives it an announced skip rather than a silent
  one, gives it explicit controls for the two steps a caller may legitimately
  not want, and makes the shipped workflow call it from both legs.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "054-effective-config-is-a-governed-read"
  - "059-read-verbs-on-a-code-free-corpus"
  - "064-the-kit-ships-the-composite-gate"
  - "077-compile-warnings-reach-the-gate"
  - "089-a-skip-and-a-failure-are-different-answers"
  - "103-an-amended-acceptance-is-the-one-that-runs"
extends:
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/Makefile", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/govern.yml", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
  # The kit files are embedded verbatim, so a kit edit restamps the generated
  # copy (spec 065 3.6).
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: additive }
  # 3.4's assertion parses the workflow rather than searching its text, and the
  # parser is a dev-dependency of the crate the test lives in. D-11.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/Cargo.toml", nature: additive }
amends: ["077-compile-warnings-reach-the-gate"]
# 3.5: routing the workflow through `make gate` means `kit/govern.yml` stops
# being a written form of the chain, so 077's grep over that file no longer has
# a premise. 077's requirement is untouched and its file is not edited.
amends_verification: ["077-compile-warnings-reach-the-gate"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 114: One gate definition, and it holds on a code-free corpus

## 1. Purpose

### 1.1 The kit's own gate refuses the kit's own corpus

Measured on 2026-09-18 against `58c7fb0` with the 0.20.0 binary, in a scratch
directory with nothing in it but `git init` and one command:

```
$ spec-spine init --with-kit
$ spec-spine compile && spec-spine index
$ make gate SPEC_SPINE=.../spec-spine
spec-spine check --fail-on-unresolved --fail-on-warn
spec-registry: fresh
codebase-index: fresh
spec-spine lint --fail-on-warn
lint: 0 error(s), 0 warning(s), 0 info
spec-spine index coverage --fail-on-untraced
coverage: no source files under any discovered package
--fail-on-untraced asserts that every source file has a specific owning spec,
and there are none to assert about. Nothing was verified.
make: *** [gate] Error 1
```

Two of the three verbs pass. The third refuses, and the adopter's first
experience of the composite gate the kit ships is a failure on a tree the kit
itself just wrote.

### 1.2 The refusal is correct, and its placement is not

Spec 059 §3.2 made this refusal deliberately, against the opposite defect:
`--fail-on-untraced` on a package-free tree used to enumerate zero source files,
compute zero untraced, and exit 0 from inside a CI step named for the whole-tree
ownership assertion. 059's reasoning stands exactly as written:

> A person wiring `--fail-on-untraced` into CI is asserting that the tree is
> fully owned. If the tool cannot see the tree, the honest report is that the
> assertion did not run, and a CI step that did not run its check should not be
> green.

So the verb is right and this spec does not touch it. What is wrong is that
`kit/Makefile` puts that assertion into the default composite target with no
condition, in a file whose header says the language targets are guarded because
"a tree with cargo installed and no `Cargo.toml` is the specify-first case,
which is three of the four governed repositories". The guard was applied to
`test`, `build`, `fmt` and `clippy`, and not to `gate`, which is the target
those same three repositories run first.

059 §"this repository" records why nobody felt it: this corpus has four
discovered packages and ninety-five source files, so the refusal is unreachable
here. Every acceptance that runs `make gate` runs it against this tree.

### 1.3 The kit defines its gate three times, and the three disagree

| Where | The ownership assertion | The freshness verb | The binary | The base |
|---|---|---|---|---|
| `kit/AGENTS.md` fenced list | commented, "if `[coupling] require_ownership` is on" | `spec-spine check` | `spec-spine` | resolved via `symbolic-ref` |
| `kit/Makefile` `gate` | unconditional | `$(SPEC_SPINE) check --fail-on-unresolved --fail-on-warn` | `$(SPEC_SPINE)`, spec 051's resolution order | `$(BASE)`, resolved (spec 072) |
| `kit/govern.yml` pull-request leg | unconditional | `spec-spine check --fail-on-unresolved --fail-on-warn` | bare `spec-spine` from `PATH` | `origin/${{ github.base_ref }}` |

`kit/govern.yml`'s own header comment reads:

> It runs the same `make gate` chain a session runs locally, so the gate has one
> definition rather than two that drift.

Its push leg does exactly that (`run: make gate BASE=...`). Its pull-request leg,
twelve lines below the comment, restates four commands. The restatement is not
gratuitous: `couple` needs `--pr-body`, and `kit/Makefile`'s `gate` target has no
way to pass one, so the leg was open-coded to reach the flag. That is a missing
variable, not a second gate.

The `$(SPEC_SPINE)` difference is the sharper half. Spec 051 established that a
repository which builds its own binary must be governed by the one it builds,
and `kit/Makefile` honours that. The pull-request leg calls bare `spec-spine`, so
on a repository that builds its own, CI's coupling verdict comes from a
different binary than the local gate's.

### 1.4 The two definitions disagree about coupling as well, in opposite directions

Spec 064 §3.2 requires the shipped workflow to run `couple` on `pull_request`
only: a push to the default branch has already merged, so there is nothing left
to refuse, and it carries no PR body, so a waiver line is unrecoverable.

Measured on 2026-09-18 at `58c7fb0`, the push leg does not honour that. It runs
`make gate BASE=origin/${{ github.base_ref || 'main' }}`, and `gate`'s fourth
step is `couple`. On a push `github.base_ref` is empty, so the leg couples
`origin/main` against `HEAD`, which after the push is the same commit: zero paths
checked, a verdict about nothing. It is vacuous rather than wrong, which is why
it was never felt; it is still a `couple` run on a `push` event, which is what
§3.2 says MUST NOT happen.

That is the contradiction this spec has to resolve rather than inherit. Routing
the pull-request leg through the same target makes it sharper, not softer: one
definition containing `couple` cannot serve a leg that must run `couple` and a
leg that must not, unless the caller can say which it is. §3.3 gives it an
explicit control and §3.4 spends it, in the only direction that preserves both
064 §3.2 and the single definition. D-9.

## 2. Territory

This spec establishes no new file. It claims, through `extends`:

| Path | Why |
|---|---|
| `kit/Makefile` | the `gate` target, the ownership guard, and the new `OWNERSHIP`, `COUPLE` and `PR_BODY` variables |
| `kit/govern.yml` | both legs call the target instead of one restating it |
| `crates/spec-spine-core/tests/kit_gate.rs` | where the gate's shape is asserted |
| `crates/spec-spine-core/src/kit_embedded.rs` | generated from `kit/`, restamped by the edit |
| `crates/spec-spine-core/Cargo.toml` | the YAML parser §3.4's assertion needs, as a dev-dependency (D-11) |

`kit/AGENTS.md` is **not** claimed: its fenced list is already the condition this
spec makes the other two honour, so nothing in it changes.

## 3. Behavior

### 3.1 `gate` reaches exit 0 on a corpus with no code

`make gate` MUST exit 0 on the repository `spec-spine init --with-kit` creates,
after `compile` and `index` have been run once, with no source file and no
package manifest anywhere in the tree.

### 3.2 The ownership assertion is conditional, and the skip is announced

The `gate` target MUST decide whether to run `index coverage --fail-on-untraced`
from the adopter's own effective configuration, on the condition
`kit/AGENTS.md` already states: `[coupling] require_ownership`. The effective
value MUST be read through `spec-spine config show` (spec 054), not by reading
`spec-spine.toml`, so a corpus relying on the default gets the default's answer
and not a missing key.

When the assertion does not run, the target MUST print one line saying so and
saying that whole-tree ownership was **not** verified. A silent skip would
recreate, one layer up, the exact dishonesty spec 059 removed from the verb: a
green gate that asserted nothing about ownership. Spec 089 §3.1 is the same rule
for the language targets, and the guard MUST take its explicit `if`/`then`/`else`
shape rather than `&&`/`||`, for the reason 089 gives.

An adopter MUST be able to override the decision from the command line, so a CI
job can demand the assertion whatever the config says. The override is the
`OWNERSHIP` variable: `auto` (the default) reads the effective configuration,
`1` runs the assertion regardless, `0` skips it. A skip under `0` is announced
exactly as a skip under `auto` is, and the line names which of the two decided
it, so a reader of the log is never left guessing whether the configuration or
the caller turned the assertion off.

Any other value MUST be refused, not treated as `auto`. D-15.

**A failed read is not a skip.** If `spec-spine config show` exits non-zero the
target MUST fail with that status, not fall through to the announced skip. The
same holds for a read that succeeds and does not answer: the probe matches on
the line the read prints, so the target MUST require that setting to be present
in one of its two spellings and MUST refuse when neither is. D-16. The
probe cannot be a pipeline into `grep`, because a pipeline reports `grep`'s
status and discards the read's: a binary too old to have the verb, an unparsable
`spec-spine.toml`, or a missing config would all read as "ownership is off" and
produce a green gate. That is the same substitution spec 059 refused inside the
verb and spec 089 refused in the language targets, one layer further out. D-12.

### 3.3 The gate takes a PR body and an explicit coupling control

The `gate` target MUST accept an optional `PR_BODY` naming a file, and pass it
to `couple` as `--pr-body` when it is set. The path MUST reach `couple` as a
single argument, so a body file under a directory with a space in its name is
one path and not two. Unset, the target MUST invoke `couple` exactly as it does
today: an empty `--pr-body` pointing at nothing is not the same command, and a
local session has no PR body to give.

The `gate` target MUST also accept a `COUPLE` variable deciding whether the
coupling step runs at all. It defaults to **on**, so `make gate` in a local
session is the whole governed loop exactly as it is today, and `COUPLE=0` skips
the step. `1` and `0` are the only values; any other MUST be refused rather than
read as "not `0`, so couple". D-15. As with §3.2, the skip MUST be announced on one line that says drift
against a base was **not** checked; a caller that turned the step off still gets
a gate that does not claim to have run it.

`COUPLE` MUST NOT be inferred from whether `PR_BODY` is set. The two answer
different questions: whether to couple is about the event, and whether a waiver
is reachable is about the body. A pull-request job whose PR body happens to be
empty still has a base, still has a head, and still must be refused on drift;
inferring the control from the body would turn an empty description into a
silently skipped coupling gate, which is a bypass no reviewer would see.

### 3.4 The shipped workflow calls the target from both legs

`kit/govern.yml`'s pull-request leg MUST invoke the `gate` target rather than
restating the chain, passing the resolved base and the PR-body file it already
writes to `$RUNNER_TEMP` through `PR_BODY`. Its push leg MUST invoke the same
target with `COUPLE=0`.

The environment variable holding the body's **text** MUST NOT be named
`PR_BODY`. Make imports the environment, and `PR_BODY` names a **path** to the
target; a step exporting the body's text under that name would hand the gate a
path whose value is the body. D-14.

Spec 064 §3.2's two requirements on the workflow are unchanged and MUST still
hold, and the second of them now holds for the first time: the body reaches the
gate through a file, and `couple` runs on `pull_request` only. §1.4 measures the
push leg coupling today; `COUPLE=0` is what stops it, without giving the workflow
a second gate definition to do it with.

After this, the workflow MUST contain no `spec-spine` invocation that duplicates
a step of the target, and a test MUST assert that: a comment claiming one
definition is what this repository already had.

The assertion MUST be made over the workflow's **executable** steps, not over
its text. `kit/govern.yml` names `make gate` twice today, and only one of those
is a step: the other is the header comment on line 3, which says the workflow
"runs the same `make gate` chain a session runs locally" while the pull-request
leg restates the chain instead. A count over the file is therefore green before
this spec and after it, and it is green for a workflow whose comment is the only
truthful thing in it. D-5.

So the test MUST parse `kit/govern.yml` as YAML and assert over the `run:`
values of its steps:

- each leg is identified by its **event condition**, the step's `if:`
  expression, not by its name, its position, or its surrounding prose: a step
  named "Governed loop (pull request)" that runs on a push is a push step;
- the push leg and the pull-request leg each invoke the shared target, and the
  push leg passes `COUPLE=0` while the pull-request leg passes a `PR_BODY` and
  does not disable coupling, which is §3.3's distinction made observable;
- no step's `run:` names a `spec-spine` verb the target already runs.

"Invokes the target" MUST mean a `make` command in the step's `run:` script
naming the `gate` target. It MUST NOT be satisfied by the target's name
appearing in a comment inside the script, in the step's `name:`, or in text the
script echoes. The test MUST carry negative cases for all three, built as
workflow fixtures in the test itself, so the detector is shown refusing each
form rather than assumed to. An assertion that a mention satisfies is the
defect §3.4 exists to close, not a looser version of it. D-10.

Quoted text is one of those mentions, and it is the one that got past the first
implementation. The detector MUST read the script's shell quoting, so a
separator inside a quoted string, or behind a backslash, does not divide one
command into two. `echo 'text; make gate COUPLE=0 ; more text'` runs `echo` and
nothing else; a detector splitting on every `;` reads the middle third of that
string as an invocation of the target carrying `COUPLE=0`, from a step that runs
no `make` at all. The test MUST carry that case in both quotings and for an
escaped separator, alongside positive cases in which a real invocation follows a
real separator. D-17.

The detector MUST NOT read a shell construct it does not model. Where a `run:`
script uses one, the detector MUST refuse the script and the test MUST fail,
rather than reporting the commands it managed to find: an invocation hidden
inside a construct the reader stepped over is reported as a step that invokes
nothing, which is the same false answer pointing the other way. D-17.

The two further assertions built on the same reader hold on the same terms: the
one counting restated `spec-spine` verbs MUST NOT count a verb inside a quoted
string, and the one matching `PR_BODY` against the file the step writes MUST NOT
accept a `>` inside a quoted string as a redirection. D-17.

### 3.5 What spec 077's acceptance now is

This spec's `## Verification` block MUST replace spec 077's in full, through
`amends_verification` (spec 103 §3.1), and 077's file MUST NOT be edited.

Both edges to 077 are declared, and they record different things. `amends` says
this spec changes something 077 stated without editing 077's file, which is
spec 040's instrument and is the honest declaration here: 077 §3.5 ranges over
"every written form of the chain", and §3.4 removes one of those forms. What is
untouched is 077's **requirement** and its **file**; what changes is the set of
places the requirement ranges over, and an amendment is exactly how that is
recorded. `amends_verification` then says which block runs. Declaring only the
second would leave the change to 077's extension unrecorded; declaring only the
first would leave 077's stale assertion live against changed behavior.

One line moves:

```
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/govern.yml
```

077 §3.5 requires **every written form of the chain** to name the composed flag,
and 077's block checked the five places the chain was written. After §3.4 the
workflow is not one of them: it calls the target, and the target names the flag.
The requirement is untouched, and an assertion that treats a delegation as a
missing flag would refuse the very consolidation the workflow's own header
comment asks for. This is the pattern spec 108 §3.6 and spec 110 §1.2 record:
an assertion narrower than the rule it stands for.

The replacement MUST assert the property instead: that the workflow reaches the
composed flag, by calling the target that names it, and that it does not restate
the chain. The other four written forms keep their line unchanged.

Carrying the block in full rather than a patch follows spec 103 §3.5 and spec
105 D-1.

### 3.6 Filing this spec moves spec 077's acceptance before the fix exists

`amends_verification` takes effect the moment this spec's frontmatter is in the
corpus. `resolve_acceptance_source` skips only a `superseded` or `retired`
holder (spec 103 §3.2, D-5), so a `draft` holder is live: with this file merged
and unimplemented, `spec-spine verify 077 --plan` selects **this**
block, and `make verify SPEC=077` runs it.

Measured on 2026-09-18 at `270157b`: `verify 077 --plan` already resolves
here, on the unmerged filing branch.

That is spec 103 working as designed, and this spec does **not** change it. The
consequence is about **when this file lands**, not about the rule:

- merged before the build, spec 077's acceptance is this block, and this block
  fails, because it asserts the behavior the build has yet to write. An approved
  spec's acceptance would be red on the default branch, and `verify 077`
  would report the absence of a fix rather than the presence of a regression;
- merged **with** the build, the acceptance it replaces is green on arrival and
  the replacement is never worse than what it replaced.

So this spec is filed and built in one change, or filed on a branch that is not
merged until the build is on it. It is not merged as documentation. The
`amends_verification` declaration is required by §3.5 and is not removed
to make a filing-only merge safe: removing it would leave spec 077's stale
assertion live against changed behavior, which is the defect this spec exists to
resolve. D-8.

## 4. Out of scope

- **Changing `index coverage --fail-on-untraced`.** 059's refusal is correct and
  this spec depends on it staying correct. §1.2. The defect is where the
  assertion is invoked, not what it decides.
- **Editing spec 059, spec 064 or spec 077.** 064 §3.2's two requirements on the
  pull-request leg are preserved by §3.4 rather than amended; 059 is untouched;
  077's file keeps the block it was ratified with.
- **`kit/AGENTS.md`.** Its fenced list already carries the condition. This spec
  brings the other two definitions to it, not the other way round. The two new
  variables are call-site controls on a target the list does not spell, so the
  list is the same document before and after, and spec 113's parity assertion
  between it and the generated protocol is untouched.
- **`.github/workflows/ci.yml`.** This repository's own workflow runs the gate
  target with `BASE=HEAD` and carries a comment saying `couple` "is not reached
  by this target". `couple` is reached; it compares `HEAD` to `HEAD` and checks
  zero paths, which is the same vacuity §1.4 measures in the kit's push leg.
  `COUPLE=0` is now available to make that comment true, and spending it is a
  one-line change in territory spec 064 §3.4 owns and this spec does not claim.
  Surfaced rather than fixed, per `.claude/rules/adversarial-prompt-refusal.md`.
  D-13.
- **Whether this repository should run the gate with the assertion off.** It
  should not, and it will not: `require_ownership` is `true` here, so §3.2's
  probe runs the assertion exactly as today. D-2.
- **The `.githooks/` files the same `init --with-kit` writes, which are
  unclaimed sources in an adopter with a package at the root.** They are the
  next reason a fresh adoption's gate goes red, and they are spec 115's. This
  spec's §3.1 acceptance is measured on a tree with no package, where they are
  invisible; 115 depends on this spec for the case where they are not.
- **A `spec-spine` verb answering "is the ownership assertion meaningful here".**
  The probe in §3.2 is a grep over a governed read, which is enough for a
  Makefile and no more. D-3.

## 5. Resolved decisions

D-1 (2026-09-17, why the condition is `require_ownership` and not "a package was
discovered"). Both would clear §3.1. The tree-shaped probe is closer to what the
verb actually refuses, and it answers a different question than the adopter asked:
a repository with `require_ownership` off has said it does not want unclaimed
files refused, and running the assertion anyway because a package happens to
exist would surprise it on the day it adds one. `require_ownership` is also the
condition `kit/AGENTS.md` already publishes, so this spec brings the Makefile to
a documented rule rather than inventing a second one.

D-2 (2026-09-17, what this changes in this repository). Nothing observable.
`spec-spine config show | grep -q 'require_ownership = true'` matches here, so
the assertion runs, in the same position in the chain, with the same flags. The
measurement is recorded because a spec whose fix is invisible in its own corpus
has to say where it was tested instead: §3.1's acceptance builds a scratch
adopter, which is the only place the defect exists.

D-3 (2026-09-17, why the probe greps a governed read rather than parsing it).
`.claude/rules/governed-artifact-reads.md` allows parsing a subcommand's output,
and `config show`'s text form is a stable governed read (spec 054). A `--json`
read would need `jq` or `python3` in the adopter's CI image, which the kit's
Makefile deliberately does not require; the hooks already treat a missing `jq` as
a reason to skip, and a gate that skips for want of a JSON parser is worse than a
grep over a line the tool prints on purpose.

D-5 (2026-09-18, why §3.4's assertion is over parsed steps and not a count).
Measured on 2026-09-18 at `270157b`: `grep -c 'make gate' kit/govern.yml`
returns 2 already, from the header comment on line 3 and the push leg's `run:`
on line 57. The drafted acceptance line, `test "$(grep -c 'make gate'
kit/govern.yml)" -ge 2`, was therefore green at the parent commit and could not
witness the change it was written for. It would also stay green if a later edit
deleted a leg and added a second mention to the comment.

A count is the wrong instrument for the property §3.4 states, which is about
which commands run. Parsing the workflow and asserting over `run:` values
measures that directly, and it is the only form that can distinguish a leg from
a sentence about a leg. The corrected acceptance asserts the push leg and the
pull-request leg each invoke the target, and that no step restates a verb the
target runs.

D-6 (2026-09-18, why the scratch fixture needs a commit before `BASE=HEAD`).
`git init` leaves no `HEAD` to resolve: `git rev-parse HEAD` in the drafted
fixture fails with "unknown revision". The drafted `make gate ... BASE=HEAD`
still failed at the parent, but at `index coverage --fail-on-untraced`, which is
§3.1's defect and runs before `couple` ever gets its base. So the fixture proved
the right thing for the wrong reason, and the moment §3.1 is fixed the same line
would fail again on a missing base, naming a defect this spec did not introduce
and does not own. The corrected fixture commits the scaffolded tree before the
gate runs, which is also what an adopter's first commit looks like.

D-7 (2026-09-18, why the skip-announcement check keeps its exit status). The
drafted line piped `make` into `grep -qi 'ownership'`, which discards `make`'s
exit status: the pipeline reports `grep`'s. §3.1 requires the gate to exit 0 on
that tree and §3.2 requires it to announce the skip, and a pipeline can only
answer the second. Measured at the parent, that line exits 1 while `make` exits
2, so it was red for the wrong reason as well. The corrected form redirects to a
file, asserts the exit status, then reads the file, which is the same separation
spec 107 D-4 records for `--json` reads.

D-8 (2026-09-18, why this spec is not merged as a filing on its own).
Recorded because the obvious way to clear a backlog is to merge the drafts and
build them later, and for a spec carrying `amends_verification` that is not a
neutral act. §3.6 measures what it costs: spec 077's acceptance becomes this
block as soon as the frontmatter is in the corpus, because spec 103 §3.2's
resolver excludes only `superseded` and `retired` holders and a `draft` holder
resolves like any other.

The alternative considered and rejected was excluding `draft` holders from
resolution. It would make filing free, and it would also mean an amendment's
acceptance does not take effect until ratification, which in this repository's
producer lifecycle happens **after** the build has merged (`AGENTS.md`, "Working
the backlog"). The replacement would then be inert over exactly the window it is
written for, and the stale assertion it replaces would run against the new
behavior. That is a change to spec 103's governed behavior with its own
trade-offs, and it is not this spec's to take: it belongs in a spec against 103,
deliberately, or nowhere.

The cheap fix is scheduling: file and build in one change. This spec does that.

D-9 (2026-09-18, why coupling gets an explicit control rather than an inference).
§1.4 states the contradiction: 064 §3.2 says the shipped workflow runs `couple`
on `pull_request` only, §3.4 says both legs call one target, and that target's
last step is `couple`. Three resolutions were available.

Taking `couple` out of the target was rejected first: it is the PR-time gate the
whole loop exists for, and a local `make gate` that silently stops short of it
would be a worse dishonesty than the one §3.2 is removing.

Inferring the decision from `PR_BODY` was rejected second, and is forbidden by
§3.3 rather than merely unused. It reads plausibly, because the pull-request leg
is the leg with a body, and it fails in the direction nobody audits: a PR opened
with an empty description, which GitHub permits, would set `PR_BODY` to a file
holding the empty string. A guard keyed on "is a body present" then has to choose
between treating an empty file as present, which makes the control a no-op the
day someone passes `--pr-body` locally, and treating it as absent, which turns
every description-less PR into a coupling gate that did not run. Neither is a
property a reviewer can see from the workflow.

The explicit `COUPLE` variable was taken. It defaults to on, so nothing about a
local `make gate` changes and no adopter has to learn it; the one caller that
needs it off is the push leg, which says so in the file a reader is already
looking at. The two variables stay orthogonal: `COUPLE` is about the event,
`PR_BODY` is about whether a waiver is reachable, and a pull-request leg sets
both.

D-10 (2026-09-18, why the detector carries negative fixtures). §3.4's assertion
is itself the kind of assertion this repository keeps getting wrong: a check that
passes for the right reason today and for the wrong reason after the next edit.
"The step invokes `make gate`" is a substring away from "the step mentions
`make gate`", and the three ways a mention arrives are all present in this very
workflow: a `#` comment inside a `run:` block, a step `name:`, and echoed text.
A detector that accepted any of them would report one gate definition for a
workflow that had gone back to two, which is the failure spec 110 §1.2 and spec
105's "a block cannot be its own witness" both describe. The fixtures are written
into the test rather than into `kit/govern.yml`, so the refusals are exercised on
every run without shipping a broken workflow to prove it.

D-11 (2026-09-18, why a YAML parser is a dev-dependency and how the claim is
made). §3.4 requires the assertion to be over parsed steps. `serde_yaml` is
already a workspace dependency and already a direct dependency of
`spec-spine-core`, but an integration test under `tests/` cannot see a crate's
ordinary dependencies, so it is added to that crate's `[dev-dependencies]`. No
new dependency enters `Cargo.lock` and no shipped artifact gains one.
`crates/spec-spine-core/Cargo.toml` has no specific owner, only spec 001's
package-manifest floor, and the coupling gate refuses a floor-owned path whose
owning spec did not change. The remedy is the one `couple` itself prints: an
`extends` edge naming 001 and the path, which amends nobody. A hand-rolled
line scanner was rejected: it would be a fourth text search dressed as a parser,
and §3.4 asks for the opposite.

D-12 (2026-09-18, why the ownership probe cannot be a pipeline). Drafted as
`config show | grep -q ...`, the guard would have inherited `grep`'s exit status
and discarded the read's. Every way the read can fail, a binary predating spec
054's verb, an unparsable `spec-spine.toml`, a config schema the binary rejects,
produces no matching line, so the guard would have announced an honest-sounding
skip and exited 0. That converts a configuration failure into a green gate,
which is exactly the substitution spec 059 removed from the verb. The target
therefore captures the read, checks its status, fails with it, and only then
reads the captured text. §3.2.

D-13 (2026-09-18, why `.github/workflows/ci.yml` is not touched). Its
self-governance job runs `make -f kit/Makefile gate ... BASE=HEAD` under a
comment asserting `couple` "is not reached by this target". It is reached, and
checks zero paths. `COUPLE=0` makes the comment true in one word, and the file
is in spec 064 §3.4's territory, which this spec does not claim. The adjacent
fix is the classic way a build grows past its spec, and the coherence rule says
surface it. Recorded in §4 so the next spec against 064 has the measurement.

D-14 (2026-09-18, why the workflow's body variable is renamed). The
pull-request leg already exported the body as `PR_BODY`, and §3.3 gives the
target a `PR_BODY` meaning a file path. Make imports the environment, so the two
would occupy one name with two meanings, and the only thing keeping them apart
would be that a command-line assignment outranks an environment one in GNU make.
That is true and it is not a property anyone should have to know to read the
file. The step now exports `PR_BODY_TEXT` and writes it to the file whose path
it passes as `PR_BODY`, so the name that means a path only ever holds a path.

D-15 (2026-09-18, why an unrecognised control value is refused rather than
defaulted). Raised in review of the implementation PR. Written as a two-way
`if`/`else`, `OWNERSHIP` accepted `1` and `0` and sent everything else to the
configuration branch, and `COUPLE` treated every value but `0` as "couple". Both
read plausibly and both fail in the direction the caller cannot see: a person
typing `OWNERSHIP=yes` is asking for the assertion and would have got a
config-governed run under the word they chose to override it with, and a person
typing `COUPLE=false` to turn coupling off would have got coupling.

That is the same substitution D-12 refuses one step earlier in the same recipe:
an input the target could not honour, answered as though it had been. A control
whose typo silently means the opposite of what was typed is not a control, and
the whole point of §3.2's override and §3.3's `COUPLE` is that a caller can
state a decision the file then observably carries out. Both now exit 3 on an
unrecognised value, naming the value and the accepted set. The cost is that an
adopter who guessed a spelling gets a refusal instead of a run; that is the
cheap direction to be wrong in, and it is the one spec 059 chose for the verb.

D-16 (2026-09-18, why an unanswerable configuration read is also refused).
Raised in review alongside D-15, and a different failure from D-12's. D-12
covers a read that FAILS; this covers a read that succeeds and says nothing the
probe can use. The probe greps for `require_ownership = true`, so a `config
show` whose format drifted, to `require_ownership=true` without the spaces, say,
would match no line, be indistinguishable from the setting being off, and
produce an announced skip that was really a failed probe. D-3 accepted the
coupling to that text and gave its reasons; what D-3 did not settle is what
happens when the text moves.

The target therefore requires the setting to be THERE, in one of the two
spellings the read can print, and exits 3 naming the problem when neither
appears. Measured in both directions against a stand-in whose `config show`
prints the key without spaces: refused at exit 3, where before it announced the
skip and exited 0. The alternative, parsing `--json`, is still rejected for the
reason D-3 gives.


D-17 (2026-09-18, why §3.4's detector reads quoting, and what it refuses
instead). Shipped at `bd0fa40`, the detector split each line of a `run:` script
on `|`, `;` and `&` wherever those characters appeared, and only then asked
whether a command's head was `make`. Independent review reproduced the
consequence through the YAML fixture path §3.4 requires:
`run: echo 'text; make gate COUPLE=0 ; more text'` split into three commands, the
middle one `make gate COUPLE=0`, and `gate_invocation` returned
`Some({"COUPLE": "0"})` for a step that runs one `echo`. That is a mention
satisfying the invocation assertion, which is the thing §3.4 already forbade in
its other three forms. The requirement did not change; the implementation did
not meet it. The fixture at case 3 passed only because its echoed text carried
no separator.

The same reader is behind the other two assertions, and both were measured
wrong in the same way: `spec_spine_verbs("echo 'a; spec-spine check
--fail-on-unresolved --fail-on-warn'")` answered `["check"]`, which would refuse
a workflow that delegates correctly, and the `PR_BODY` check ran a string search
over the script with the whitespace around `>` removed, so
`echo "wrote > $RUNNER_TEMP/pr-body.txt"` satisfied "the step writes this file"
while writing nothing. One root cause, three assertions, corrected together.

The correction is a reader for the subset of `sh` these scripts use: quoting,
backslash escapes, comments, the separators, and redirections, with the
redirect targets read off the parsed commands instead of matched in the text.
It is deliberately **not** a shell parser and is not described as one. Every
construct it can recognise but not model, a command substitution, a
here-document, a subshell, a shell group, a process substitution, a `case` arm
terminator, an unterminated quote, is refused, and `script_commands` turns the
refusal into a panic. A test helper that cannot read a script must fail the test rather than
answer from the part it understood: an invocation hidden inside a skipped
construct would otherwise read as a step that invokes nothing. The shipped
`kit/govern.yml` uses none of the refused forms, and a test asserts that, so the
refusal costs the kit nothing today and is what a future script using one will
hit.

An existing crate was considered and not used. `shell-words` splits a command
line into words and treats `;` and `|` as ordinary characters, so it answers the
smaller half of the question and not the half this defect is in, which is where
one command ends and the next begins; a full shell grammar is a dependency and a
surface out of proportion to a test helper reading five scripts. The bounded
reader with an explicit refusal list is the smaller claim, and it is the one
that can be checked.

Review of the first correction raised two forms, and they resolved in opposite
directions once each was run rather than reasoned about. `;;` outside a `case`
was accepted: `/bin/sh` and `dash` both call `echo a;; echo b` a syntax error,
so the reader was accepting a construct the shell rejects, and a `case` is
already refused by the `)` its arms carry. It is now refused in its own right,
which is what "refuses what it does not model" has to mean. `> > file` was
reported as a valid POSIX redirection the reader spuriously refuses; it is a
syntax error in both `/bin/sh` ("syntax error near unexpected token `>`") and
`dash` ("redirection unexpected"), so the refusal is correct and nothing
changed. Measured, not assumed, because the two readings are indistinguishable
from the prose alone.

A second pass named the input twin of `>&`. `>&2` is consumed, because it names
no file and changes no command; `<&` reached the generic path and refused with
"a redirection with no target", which is a true refusal giving a false reason.
It is now refused by name and exercised, so no refusal path in the reader is
left unverified. The other half of that pass, that the descriptor-strip branch
leaves the quoted flag set, is correct and is not a defect: the branch guard
requires that flag to be false. The reset is written beside the one next to it
regardless, so the word's state is cleared in one place rather than left correct
by a condition the reader has to re-derive.

`invocations()`, the line-based scanner over `kit/Makefile` target bodies, was
inspected and is **not** changed here. It has the same shape of gap, a quoted
`spec-spine` mention inside an `echo` would be counted, and the gap is inert:
the `gate` target's announcement lines name no verb, and the direction of the
error is a chain set that is too large, which makes
`no_workflow_step_restates_a_verb_the_one_gate_definition_runs` stricter and
makes `every_kit_gate_invocation_names_a_real_verb` fail loudly on the invented
verb. It is spec 064's helper and its territory; recorded as measured, not as
fixed.


## Verification

Each line is one command, run independently: no shell variable survives to the
next line. This block is spec 077's acceptance (§3.5) as well as this spec's own,
so it is read in two halves and labelled as such.

**Fail-first evidence**, measured on 2026-09-18 at `58c7fb0`:

| Line | At the parent |
|---|---|
| `make gate` on a scratch adopter (§3.1) | red, exit 1: §1.1's measurement, reproduced |
| the `ownership` message (§3.2) | red, nothing is printed because nothing is skipped |
| the "not verified" half of that message (§3.2) | red, absent for the same reason |
| the four `! grep` lines over `kit/govern.yml` | red, the pull-request leg restates all four commands |
| `OWNERSHIP`, `COUPLE`, `PR_BODY`, `pr-body`, `config show`, `require_ownership` in `kit/Makefile` | red, all six absent |
| `COUPLE=0` in `kit/govern.yml` | red, absent: §1.4's measurement |
| the `one_gate_definition` tests (§3.4) | red, they do not exist; written against the shipped workflow they fail on the pull-request leg |
| `registry show 114` | red, not found, exit 1 |

**Fail-first evidence for the §3.4 correction**, measured on 2026-09-18 at
`bd0fa40`, the merged build (D-17):

| Line | At `bd0fa40` |
|---|---|
| `the_one_gate_definition_detector_reads_shell_quoting_not_raw_separators` | red on its first negative: `gate_invocation` answered `Some({"COUPLE": "0"})` for `run: echo 'text; make gate COUPLE=0 ; more text'`, where `None` is required |
| `the_one_gate_definition_detector_refuses_a_script_it_cannot_read` | red: `echo $(make gate)` parsed without complaint instead of being refused |
| `! grep` for the unquoted split | red: `split(['\|', ';', '&'])` is the merged detector's own line |
| the restatement half | red as measured directly: `spec_spine_verbs("echo 'a; spec-spine check --fail-on-unresolved --fail-on-warn'")` answered `["check"]` |
| the redirection half | red as measured directly: with `normalize_redirects`, `echo "wrote > $RUNNER_TEMP/pr-body.txt"` satisfied "the step writes this file" |

The two named tests were run against the merged algorithm restored in place, and
against the correction: 3 red of 19 there, 19 green here. The third red,
`the_one_gate_definition_serves_both_legs_through_explicit_controls`, was an
artifact of the restored stand-in rather than a defect at `bd0fa40`: the merged
code answered that assertion through `normalize_redirects`, which the stand-in
did not carry. The redirection defect is recorded above from its own direct
measurement, which is the honest evidence for it.

The §3.4 tests were measured in both directions during the build: red against the
shipped workflow, naming the pull-request leg as not invoking the target and
naming the four verbs it restates, and green against the corrected one. The count
they replace, `test "$(grep -c 'make gate' kit/govern.yml)" -ge 2`, was **green
at the parent** and could witness nothing (D-5). Their negative fixtures are red
in the other direction by construction: each is a workflow that mentions the
target without invoking it, and the assertion is that the detector says so
(D-10).

The scratch fixture's own prerequisite is likewise measured rather than assumed:
`git rev-parse HEAD` on the drafted fixture fails with "unknown revision", so
`BASE=HEAD` had no base to resolve, and the gate's exit 1 at the parent came from
§3.1's defect running first (D-6). The corrected fixture commits before it gates.

Two lines are **green at the parent and stay green**, and are here as
preservation rather than as evidence: `RUNNER_TEMP` in the workflow, which is
spec 064 §3.2's requirement that the body travel as a file, and the `|| echo`
negative over the Makefile, which guards against the wrong shape of fix rather
than against today's code (spec 089 §3.1). The 077 half is green at the parent by
construction, since this spec weakens none of it; what moved is one line whose
premise §3.5 removes.

```verify:cli
cargo build --release --locked
# --- spec 077's acceptance, which this block now holds (3.5) ---
# 077 3.2 the flag exists on the primitive, in its writing and checking forms.
./target/release/spec-spine compile --check --fail-on-warn
# 077 3.3 and it is reachable from the composed verb, which is the form CI runs.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
# 077 3.2, 3.4, 3.6 the gating, the tally, and the byte-identical-shards assertion.
cargo test -p spec-spine-core --test compile --locked
cargo test -p spec-spine-cli --test cli --locked
# 077 3.5 spec 051's subset assertion still holds after the gate list changed.
cargo test -p spec-spine-core --test kit_skills --locked
# 077 3.5 spec 064's in-order walk of kit/Makefile against the AGENTS.md list.
cargo test -p spec-spine-core --test kit_gate --locked
# 077 3.5 065's generator keeps the embedded copy in step with kit/.
cargo test -p spec-spine-core --test scaffold --locked
# 077 3.5 every written form of the chain names the composed flag. The workflow
# is no longer one of them: since 114 3.4 both its legs call the target that
# names the flag, and the lines below assert that delegation rather than a
# restatement (114 3.5).
grep -qF 'check --fail-on-unresolved --fail-on-warn' AGENTS.md
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/AGENTS.md
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/Makefile
grep -qF 'check --fail-on-unresolved --fail-on-warn' .github/workflows/ci.yml
# --- spec 114's own requirements ---
# 3.4: both legs invoke the target, each leg is identified by its event
# condition, the push leg disables coupling and the pull-request leg does not,
# and no step restates a verb the target runs. Asserted over the workflow parsed
# as YAML, not over the file's text: `grep -c 'make gate'` returns 2 at the
# parent already, from the header comment on line 3 and the push leg, so a count
# is green before this change and after it (D-5). The same tests carry the
# negative fixtures in which a comment, a step name, or echoed text mentions the
# target while the invocation is absent (D-10).
cargo test -p spec-spine-core --test kit_gate --locked one_gate_definition > "${TMPDIR:-/tmp}/ss114-defs.txt" 2>&1
# With a non-zero pass count, so a filter that matched nothing cannot pass for a
# run (spec 106 D-7).
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss114-defs.txt"
# 3.4 + D-17: the detector reads the script's shell quoting, and refuses a
# construct it does not model. Named line by line, because the filtered run above
# stays green if either test is deleted, and the correction is exactly what those
# two tests carry.
grep -qF 'the_one_gate_definition_detector_reads_shell_quoting_not_raw_separators ... ok' "${TMPDIR:-/tmp}/ss114-defs.txt"
grep -qF 'the_one_gate_definition_detector_refuses_a_script_it_cannot_read ... ok' "${TMPDIR:-/tmp}/ss114-defs.txt"
# And the split that manufactured the invocation is gone. Red at `bd0fa40`,
# where that expression is the detector's own line (D-17).
! grep -qF "split(['|', ';', '&'])" crates/spec-spine-core/tests/kit_gate.rs
# 3.4: and the workflow no longer writes a second copy of the chain.
! grep -qF 'spec-spine check --fail-on-unresolved --fail-on-warn' kit/govern.yml
! grep -qF 'spec-spine lint --fail-on-warn' kit/govern.yml
! grep -qF 'spec-spine index coverage --fail-on-untraced' kit/govern.yml
! grep -qF 'spec-spine couple --base' kit/govern.yml
# 3.3: the target takes a PR body, and the coupling step has an explicit
# control that the push leg is what spends.
grep -qF 'PR_BODY' kit/Makefile
grep -qF 'pr-body' kit/Makefile
grep -qF 'COUPLE' kit/Makefile
grep -qF 'COUPLE=0' kit/govern.yml
# 064 3.2's file route is unchanged. Green at the parent: preservation, not
# evidence.
grep -qF 'RUNNER_TEMP' kit/govern.yml
# 3.2: the guard reads the effective config, takes 089's explicit if/else, and
# offers the command-line override.
grep -qF 'config show' kit/Makefile
grep -qF 'require_ownership' kit/Makefile
grep -qF 'OWNERSHIP' kit/Makefile
# And not the shape 089 refuses. Green at the parent, where the line is
# unguarded: this guards the wrong fix, not today's code.
! grep -qE 'index coverage[^|]*\|\| echo' kit/Makefile
# 3.2: and the probe is not a pipeline, whose status would be grep's and not the
# read's (D-12). Comment lines are dropped first: the file explains at length why
# the pipeline is wrong, and a search that cannot tell a command from a sentence
# about one is the instrument 3.4 spends a whole paragraph rejecting.
! grep -v '^ *#' kit/Makefile | grep -qE 'config show[^>]*\| *grep'
# 3.1: a corpus with no code at all passes the gate the kit ships it.
rm -rf "${TMPDIR:-/tmp}/ss114" && mkdir -p "${TMPDIR:-/tmp}/ss114" && git -C "${TMPDIR:-/tmp}/ss114" init -q .
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss114" init --with-kit >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss114" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss114" index >/dev/null
# `BASE=HEAD` needs a HEAD, and `git init` alone leaves none. At the parent this
# line failed at `index coverage` (3.1's defect) before `couple` ever asked for a
# base, so the missing base was invisible; fixing 3.1 would expose it as an
# unrelated failure (D-6). Committing the scaffolded tree is also what an
# adopter's own first commit is.
git -C "${TMPDIR:-/tmp}/ss114" add -A && git -C "${TMPDIR:-/tmp}/ss114" -c user.email=t@e -c user.name=t commit -qm scaffold
git -C "${TMPDIR:-/tmp}/ss114" rev-parse --verify HEAD >/dev/null
make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD
# 3.2: and it said so rather than skipping in silence. Redirected, not piped: a
# pipeline reports `grep`'s status and discards `make`'s, so it cannot witness
# 3.1's exit 0 and 3.2's announcement together (D-7).
make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD > "${TMPDIR:-/tmp}/ss114-gate.txt" 2>&1
grep -qi 'ownership' "${TMPDIR:-/tmp}/ss114-gate.txt"
# 3.2: and the line names the consequence, not just the step. "skipping coverage"
# is the silent skip one sentence longer; what 3.2 requires is that the reader is
# told whole-tree ownership was not verified.
grep -qiE 'not verified|did not run|not asserted|nothing was verified' "${TMPDIR:-/tmp}/ss114-gate.txt"
# 3.3: the coupling control announces its skip in the same way, and the gate
# still exits 0 when it is spent.
make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD COUPLE=0 > "${TMPDIR:-/tmp}/ss114-nocouple.txt" 2>&1
grep -qiE 'not checked|did not run|not run' "${TMPDIR:-/tmp}/ss114-nocouple.txt"
# 3.2: the command-line override demands the assertion whatever the config says,
# so on that same code-free tree the gate refuses again. This is the line that
# separates a real control from a variable nothing reads.
! make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD OWNERSHIP=1 > "${TMPDIR:-/tmp}/ss114-own1.txt" 2>&1
grep -qF 'fail-on-untraced' "${TMPDIR:-/tmp}/ss114-own1.txt"
# 3.2: and a configuration read that FAILS is a failure, not an announced skip.
# The stand-in answers every verb from the real binary except `config`, which it
# refuses; if the guard were a pipeline into `grep` this gate would print the
# skip line and exit 0 (D-12).
printf '#!/bin/sh\nif [ "$1" = config ]; then echo "simulated config failure" >&2; exit 3; fi\nexec %s "$@"\n' "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss114-wrapper" && chmod +x "${TMPDIR:-/tmp}/ss114-wrapper"
! make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="${TMPDIR:-/tmp}/ss114-wrapper" BASE=HEAD > "${TMPDIR:-/tmp}/ss114-cfgfail.txt" 2>&1
! grep -qi 'was NOT verified' "${TMPDIR:-/tmp}/ss114-cfgfail.txt"
# 3.2 + 3.3: an unrecognised control value is refused, not silently defaulted
# to the branch the caller was trying to override (D-15). Both controls, both
# directions of the mistake.
! make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD OWNERSHIP=yes > "${TMPDIR:-/tmp}/ss114-badvar.txt" 2>&1
grep -qF 'is not one of' "${TMPDIR:-/tmp}/ss114-badvar.txt"
! make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="$PWD/target/release/spec-spine" BASE=HEAD COUPLE=false > "${TMPDIR:-/tmp}/ss114-badvar2.txt" 2>&1
grep -qF 'is not one of' "${TMPDIR:-/tmp}/ss114-badvar2.txt"
# And the refusal did not masquerade as a skip: no announcement line was printed.
! grep -qi 'was NOT verified' "${TMPDIR:-/tmp}/ss114-badvar.txt"
# 3.2: a configuration read that SUCCEEDS but answers nothing the probe can use
# is refused too, not read as "ownership is off" (D-16). The stand-in prints the
# setting in a spelling the probe does not match.
printf '#!/bin/sh\nif [ "$1" = config ]; then echo "[coupling]"; echo "  require_ownership=true"; exit 0; fi\nexec %s "$@"\n' "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss114-fmtwrap" && chmod +x "${TMPDIR:-/tmp}/ss114-fmtwrap"
! make -C "${TMPDIR:-/tmp}/ss114" gate SPEC_SPINE="${TMPDIR:-/tmp}/ss114-fmtwrap" BASE=HEAD > "${TMPDIR:-/tmp}/ss114-fmtfail.txt" 2>&1
! grep -qi 'was NOT verified' "${TMPDIR:-/tmp}/ss114-fmtfail.txt"
grep -qF 'could not be read' "${TMPDIR:-/tmp}/ss114-fmtfail.txt"
# 3.2: this repository's own verdict is unchanged, because the probe matches
# here and the assertion runs (D-2).
target/release/spec-spine config show | grep -qF 'require_ownership = true'
make -f kit/Makefile gate SPEC_SPINE=target/release/spec-spine BASE=HEAD
# 064 3.4: the gate is still read-only, and the tree it judged is unchanged.
target/release/spec-spine compile --check
target/release/spec-spine index check
# The replacement is declared, read through the CLI rather than off the shard,
# and redirected rather than piped (spec 107 D-4).
target/release/spec-spine registry show 114 --json > "${TMPDIR:-/tmp}/ss114-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss114-show.json')); assert d['amendsVerification'] == ['077-compile-warnings-reach-the-gate'], d; assert d['amends'] == ['077-compile-warnings-reach-the-gate'], d"
# Spec 077's file is not edited (spec 040 3.1): its own block still greps the
# workflow for the chain. This goes red if someone resolves this by editing 077.
grep -qF "grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/govern.yml" specs/077-compile-warnings-reach-the-gate/spec.md
rm -rf "${TMPDIR:-/tmp}/ss114"
rm -f "${TMPDIR:-/tmp}/ss114-defs.txt" "${TMPDIR:-/tmp}/ss114-show.json" "${TMPDIR:-/tmp}/ss114-gate.txt" "${TMPDIR:-/tmp}/ss114-nocouple.txt" "${TMPDIR:-/tmp}/ss114-own1.txt" "${TMPDIR:-/tmp}/ss114-wrapper" "${TMPDIR:-/tmp}/ss114-cfgfail.txt" "${TMPDIR:-/tmp}/ss114-badvar.txt" "${TMPDIR:-/tmp}/ss114-badvar2.txt" "${TMPDIR:-/tmp}/ss114-fmtwrap" "${TMPDIR:-/tmp}/ss114-fmtfail.txt"
```
