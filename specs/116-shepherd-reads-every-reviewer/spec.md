---
id: "116-shepherd-reads-every-reviewer"
title: "Shepherd reads every reviewer"
status: draft
kind: "harness"
created: "2026-09-17"
summary: >
  `/shepherd` reads review threads from one GitHub endpoint,
  `pulls/<n>/comments`, which returns only comments anchored to a line of the
  diff. This repository's AI review pass posts with `gh pr comment`, which writes
  to `issues/<n>/comments`, and a review's own body goes to
  `pulls/<n>/reviews`; neither is ever fetched. The second blindness is
  structural and larger: Step 1 routes an all-green PR straight to Step 4, so on
  the path where a reviewer's comment is the only thing standing between a PR and
  a merge, Step 3b does not run at all. The skill then reports `Review threads:
  none`, which is a claim it did not check. This spec makes the thread read cover
  every place a reviewer can write, makes it happen before the merge checkpoint
  on every path, and forbids reporting an absence that was never looked for.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "048-kit-ships-the-governed-loop-skills"
  - "081-the-kit-ships-what-the-loop-calls"
  - "082-a-refusal-is-not-a-remediation-round"
  - "100-one-source-generates-the-agent-trees"
extends:
  # The skill body. 048 pins the repository copy to the kit's, and 100
  # generates both trees from the kit source, so the edit is made once.
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: "kit/.claude/skills/", nature: additive }
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: ".claude/skills/", nature: additive }
  - { spec: "100-one-source-generates-the-agent-trees", unit: ".agents/skills/", nature: additive }
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: additive }
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: "crates/spec-spine-core/tests/kit_skills.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 116: Shepherd reads every reviewer

## 1. Purpose

### 1.1 One endpoint, three places a reviewer writes

`/shepherd` Step 3b, in full:

```sh
gh api "repos/{owner}/{repo}/pulls/<number>/comments" --jq '.[] | {id, path, line, body, user: .user.login}'
```

That endpoint returns **review comments**: the ones anchored to a file and a
line of the diff. GitHub keeps two other kinds elsewhere.

| Where a reviewer writes | Endpoint | Read by Step 3b |
|---|---|---|
| A comment on a line of the diff | `pulls/<n>/comments` | yes |
| A comment on the pull request itself | `issues/<n>/comments` | no |
| The body of a submitted review | `pulls/<n>/reviews` | no |

This repository's own automated reviewer is in the second row. Checked on
2026-09-17 against `7a7a8b6`: `.github/workflows/ai-pr-review.yml` posts its
findings, its API-failure notice, its skip notice, its fork-skip notice and its
Dependabot-skip notice with `gh pr comment`, at lines 318, 345, 366, 387 and 408.
`gh pr comment` writes to `issues/<n>/comments`. Five different things that
workflow can say, and `/shepherd` reads none of them.

The third row matters for the same reason: a reviewer who writes "this needs the
spec claiming the new file" in the body of a `CHANGES_REQUESTED` review, with no
line anchor, leaves nothing in either of the other two.

### 1.2 The green path never reaches Step 3b

Step 1 routes the checks:

> - every required check `SUCCESS`: go to Step 4;

Step 4 is the merge checkpoint. So a PR whose checks are all green goes to merge
without Step 3b running, and the one path where a human reviewer's comment is the
only thing between the PR and the default branch is the path on which the skill
does not look for one.

This is not softened by the `reviewDecision` read. Step 0 fetches it and Step 3b
uses it (`CHANGES_REQUESTED` with no actionable thread is a stop), but Step 3b is
where that sentence lives, so on the green path it is not evaluated either.

### 1.3 The report then states an absence it never measured

The report template's thread line is

```
Review threads: <none | n addressed | n need a human | not read: stopped at CRITICAL>
```

The fourth value exists because spec 082 required it: on the CRITICAL path the
threads are not fetched, and the skill is told in plain words that reporting that
as an absence "is a claim this run did not make". The green path has exactly the
same property and no such value, so it reports `none`.

That is the same defect twice, caught once. This spec closes the second instance
by removing the path rather than adding a fifth value: on a green PR there is
nothing stopping the skill from looking.

## 2. Territory

This spec establishes no new file. `/shepherd`'s body exists in four committed
copies and one generated one, and spec 048 and spec 100 already keep them in
step, so the edit is made in `kit/.claude/skills/shepherd/SKILL.md` and
propagated:

| Path | How it gets there |
|---|---|
| `kit/.claude/skills/shepherd/SKILL.md` | edited |
| `.claude/skills/shepherd/SKILL.md` | `scripts/gen-agent-trees.py`, spec 100 |
| `.agents/skills/shepherd/SKILL.md` | same generator |
| `crates/spec-spine-core/src/kit_embedded.rs` | `scripts/gen-kit-embedded.py`, spec 065 |
| `crates/spec-spine-core/tests/kit_skills.rs` | where a skill's inlined floor is asserted |

Nothing under `crates/spec-spine-core/src/` other than the generated kit module
changes, no verb changes, and no schema constant moves.

## 3. Behavior

### 3.1 The thread read covers every place a reviewer can write

Step 3b MUST read all three of `pulls/<n>/comments`, `issues/<n>/comments` and
`pulls/<n>/reviews`, and MUST say which endpoint each thread came from when it
reports one, so a reader can tell a line comment from a review body.

This obligation holds on **every path that reaches the merge checkpoint** except
the one spec 082 stops before Step 3: a CRITICAL finding still stops the run
with the threads unfetched, keeps its reporting value, and keeps the value it has
for a human, which is that the evidence and the proposed remedy arrive without a
session having edited anything first. Every other path toward merge, green or
remediated, reads all three endpoints in full before Step 4. D-7.

Each read MUST be **paginated**. `gh api` returns a single page by default, so
the command Step 3b ships today stops at the API's page size and a PR with more
comments than that silently loses the rest. The reviews endpoint is the one most
likely to exceed it, because every re-review appends. `--paginate` MUST be passed
on all three, and "read" means every page the endpoint has, not the first one.
D-5, D-7.

Each read's **exit status** MUST be checked before its output is believed, and
the status of the command that performs the read, not of a pipeline stage
downstream of it. A failed `gh api` call prints nothing to stdout and exits
non-zero; piped straight into `--jq`, it is indistinguishable from an endpoint
that returned no threads. A paginated read that fails **after** emitting some
pages fails the same way and MUST be caught by the same status check: the pages
already in hand are a partial result, never a complete one. A read that did not
succeed is an unread endpoint, and §3.3 governs what may be reported about it.
D-5.

A failed read permits **one** retry of that endpoint. If the retry also fails,
the run MUST stop before the merge checkpoint. It MUST name the endpoint that
could not be read, preserve the diagnostics it has (the command, its exit status
and its stderr), and report which of the three reads did succeed. Partial results
never establish complete review coverage, and a merge taken on them is a merge
taken on an absence nobody measured. D-8.

A comment authored by an automated reviewer MUST be treated as a reviewer's
comment. It is triaged by spec 082's severity rules like any other finding, and
this repository's memory of that reviewer is that it can be wrong in a specific
way: it proposes vocabulary and shell behavior that does not exist. Reading it is
not agreeing with it, and §3.4 keeps the existing rule that a design
disagreement is a human's to settle.

### 3.2 The threads are read before the merge checkpoint, on every path

Step 1's routing MUST send an all-green PR through the thread read before Step
4, not around it. A merge is outward-facing and the checkpoint exists to be
informed; arriving at it without having looked at what reviewers wrote makes the
checkpoint answer a question nobody asked.

The red path is unchanged: Step 2 classifies, a CRITICAL finding still stops
before Step 3 and the threads are still not fetched there, which is spec 082's
ruling and §3.3's reporting case.

### 3.3 An unread absence is never reported as an absence

The report's thread line MUST distinguish "no threads exist" from "threads were
not read", on every path where the second is possible. `none` MUST mean **every**
one of §3.1's three reads succeeded and returned no feedback; it MUST NOT be
reported on any other footing. If any read fails, the report MUST carry the value
`could not read <endpoint>`, naming the endpoint, and MUST NOT report `none`.
Coverage that did succeed is reported separately and in its own words, so a
reader sees what was covered beside what was not, and never reads one as the
other.

Spec 082's value for the CRITICAL stop, `not read: stopped at CRITICAL`, is
unchanged and keeps its own meaning. The three values are distinct on purpose:
one says nothing was there, one says an endpoint would not answer, one says the
run stopped before it looked. D-10.

This is the substance of the requirement, not a refinement of it. The failure
mode it forbids is the cheap one: three commands whose errors are swallowed
produce an empty list that reads exactly like a clean PR, and the session then
carries "no review feedback" into a merge checkpoint on the strength of an API
error. Pagination fails the same way one level down, reporting a true absence for
the page it saw and an untrue one for the rest.

The CRITICAL stop is the one path that does not read at all, and it already has
its value.

A read that fails MUST NOT be retried indefinitely; one retry, then report the
endpoint with §3.1's stop and §3.3's `could not read <endpoint>` value. Spec 082's
round budget is about remediation, not about reads, and neither this nor a retry
consumes a round (§3.4).

### 3.4 What a thread can and cannot cause

Unchanged from today, restated because §3.1 widens the input: a thread asking
for a concrete change inside the spec's territory is addressed as part of a
remediation round; a thread asking for a different behavior than the spec
describes is a human question, quoted in the report and never resolved by
editing the spec. `reviewDecision: CHANGES_REQUESTED` with no actionable thread
is a stop.

Every finding MUST be triaged against the source and against the governing
requirements **before** any file is edited, which is spec 082 3.1's rule applied
to a reviewer's words rather than to a run log. Reading a thread, classifying it,
retrying a failed read, and rejecting a finding as a false positive consume **no**
remediation round: none of them is work on the pull request. Confirmed findings
whose fix is authorized and inside the spec's territory MAY consume a round, and
this is true even when every required check was green: a round is spent on an
edit, not on a red check.

Spec 082's budget of two rounds, and its definition of what a round is, are
unchanged. Reading more endpoints MUST NOT raise it. A comment that arrives from a
newly read endpoint is worked in the round it would have been worked in had it
arrived from the old one. If substantive findings remain unresolved when the
budget is exhausted, the run MUST stop before the merge checkpoint and report
them; an exhausted budget is not a clearance. D-9.

### 3.5 The copies stay equal

Specs 048 and 081 pin this repository's `.claude/skills/` to the kit's, and spec
100 generates `.claude/skills/` and `.agents/skills/` from the kit source. The
edit MUST be made in the kit source and the trees regenerated, so all four
committed copies and the embedded one carry it. `kit_skills.rs`'s assertion that
each skill's inlined gate floor is a subset of `AGENTS.md`'s list MUST still
pass: this spec adds no gate step.

## 4. Out of scope

- **Resolving threads.** The skill answers threads and reports them; marking a
  thread resolved on the platform is not something it does today and this spec
  does not add it.
- **Changing the remediation budget.** §3.4. Spec 082 set it at two and decided
  what a CRITICAL finding costs; nothing here reopens either.
- **Making the AI reviewer post where the skill already looks.** Moving the
  workflow to `pulls/<n>/comments` would fix this repository and leave the skill
  broken for every human reviewer who comments on the PR rather than a line, and
  for every adopter whose CI posts the ordinary way. The defect is the reader's.
  D-1.
- **Teaching the skill which automated reviewers to distrust.** §3.1 says an
  automated comment is a reviewer's comment and spec 082's severity rules apply.
  A list of reviewers with known failure modes is repository policy, and the
  skill is repository-invariant by spec 048's ruling. D-2.
- **A helper that performs the reads.** The three commands stay in the skill's
  text. No executable is added to `kit/`, `scripts/` or the crates to wrap them,
  and no test territory beyond `crates/spec-spine-core/tests/kit_skills.rs` is
  opened. The subject of this spec is what the shipped instructions say, and a
  helper would move the subject without closing the gap. D-11.
- **A follow-up spec for the residue.** Nothing here is deferred to one. What
  this implementation cannot do, it says in §4's limitation rather than filing
  the obligation forward. D-11.
- **`gh pr view --json comments`.** It returns the `issues/<n>/comments` set and
  not the other two, so it would close one of the two endpoint gaps and read as
  if it had closed all of them. D-3.

**The limitation this implementation keeps.** Every assertion this spec can make
is over the text of a shipped instruction: that the three endpoints are named,
that each command is paginated, that the report line carries the values §3.3
requires, that the routing sentence which skipped the read is gone. None of that
proves a session read what the instruction told it to read, or checked an exit
status, or stopped where §3.1 says stop. A skill is prose an agent obeys, and
obedience is not executable from a test. The limitation is stated rather than
engineered around, because the alternative (a helper the test could drive) is the
scope §4 declines above. D-11.

## 5. Resolved decisions

D-1 (2026-09-17, why the reader is fixed rather than the writer). `gh pr
comment` is the ordinary way to comment on a pull request and the ordinary place
a human reviewer's non-line comment lands. A skill that reads one of the three
endpoints is wrong for every corpus, not only this one, and repointing this
repository's workflow would hide that while leaving adopters with the same
blindness and no measurement to find it by.

D-2 (2026-09-17, why no reviewer is named as unreliable). This repository has a
recorded finding that its AI review pass invents vocabulary and shell behavior,
and the correct response to that is the triage spec 082 already requires: read
the finding, check the claim against the source, and grade the finding and the
proposed fix separately. Encoding a specific reviewer's reputation in a
repository-invariant skill would be a project fact in a shared file, which is the
arrangement spec 048 exists to prevent.

D-3 (2026-09-17, why three explicit endpoints rather than one convenience read).
`gh pr view --json comments` is shorter and returns exactly the set that Step 3b
misses today, which makes it the most tempting wrong answer here: it would make
this repository's AI reviewer visible and leave review bodies unread, with no
line in the skill admitting it. Naming the three endpoints keeps the coverage
legible to the next reader, which is the same reason §3.1 requires the report to
say where a thread came from.

D-5 (2026-09-18, why pagination and read status are requirements and not
implementation detail). This spec's title is that shepherd reads every reviewer,
and the drafted §3.1 would have delivered three endpoints read one page deep,
with errors indistinguishable from silence. Both gaps produce the same wrong
output as the defect the spec was filed against: a report saying no reviewer
asked for anything, on a PR where one did.

They are stated as requirements because §3.3 already forbids the conclusion and
nothing in the drafted §3.1 prevented it. `gh api` without `--paginate` returns
one page, and a failed call prints nothing to stdout while `--jq` turns that into
an empty result; a session following the drafted step would report `none` in both
cases in good faith. An acceptance block over a skill's text can witness
`--paginate` in the shipped command. It cannot witness that a session checked an
exit status, which is why §3.3 carries the obligation in the words the session
reads rather than only in a grep.

D-6 (2026-09-18, why a failed read is not a remediation round). Spec 082 budgets
two rounds for *fixing* things. An endpoint that would not answer is a fact about
the read, and retrying it is not work on the PR. One retry bounds it; after that
the report says the endpoint is unread, which is the honest value §3.3 exists to
preserve and is strictly better than a `none` nobody can audit.

D-7 (2026-09-19, the obligation is on every path toward merge, and only the
CRITICAL stop is exempt). §1.2's defect is a routing defect: the read was
correct where it ran and the green path did not run it. Stating the requirement
per-path rather than per-step closes it without leaving a second path to be
discovered later, because the exemption is now a named one rather than a gap.
Spec 082's CRITICAL stop keeps both halves of what it was given: the run stops
with nothing edited, and the report says so in the value 082 required. "All
pages" is part of the same sentence for the same reason: an endpoint read one
page deep is an endpoint partly read, and the spec's title says every reviewer.

D-8 (2026-09-19, why a twice-failed read stops the run rather than annotating the
report). D-6 settled that a failed read costs no round. It did not say what
happens next, and the honest value in a report is not a substitute for not
merging: a merge is the outward-facing, hard-to-reverse action the whole
checkpoint exists for, and taking it while one of three review endpoints is
unread is taking it on an absence nobody measured. So the retry is bounded at one,
and the second failure stops before Step 4 with the endpoint named, the
diagnostics preserved, and the successful reads reported for what they are. A
partial read is a partial read in the report and in the routing alike.

D-9 (2026-09-19, what a round costs when the checks were green). Spec 082 3.3
spends a round on an edit, and it was written where the trigger was a red check.
Widening the input to review threads separates the two: what costs a round is
editing the branch, not the colour of CI. Reading, classifying, retrying a failed
read and rejecting a false positive are all work on the *question*, not on the
pull request, and charging them would make a careful triage more expensive than a
careless one, which is backwards for a repository whose recorded experience of its
own AI reviewer is that it proposes vocabulary that does not exist. A confirmed
finding whose fix is authorized and inside the territory is an edit, so it costs a
round even on a green PR. The budget stays at two, 082's definition of a round
stays as written, and an exhausted budget with substantive findings open is a
stop, not a clearance: the alternative would let two spent rounds buy a merge that
neither round earned.

D-10 (2026-09-19, three values, spelled out). §3.3's requirement was stated as a
distinction and not as a vocabulary, which is the shape that gets satisfied by
widening an existing value until it covers two cases and names neither. The
report therefore carries `could not read <endpoint>` for an endpoint that would
not answer, unchanged `not read: stopped at CRITICAL` for the path that never
looked, and `none` only when all three reads succeeded and returned no feedback.
Successful coverage is reported in its own right beside a failure, so a reader is
never handed a partial read wearing a complete one's words.

D-11 (2026-09-19, the implementation stays in the skill text and the existing
test). The reads could be wrapped in a helper the tests drive, which would let an
assertion witness an exit-status check rather than the sentence requiring one.
That trades the spec's subject for its testability: `/shepherd` is instructions,
adopters copy the file, and an adopter who copies a skill that shells out to a
binary they did not install is worse off than one who copies three commands.
Nothing is deferred to a follow-up spec either; the residue is written down as
§4's limitation, where the next reader meets it, instead of being filed where it
can age.

D-12 (2026-09-19, why the reads also pass `--slurp`). §3.1 requires every page
and says nothing about what the pages look like once they land, which left a gap
the first implementation fell into: `gh api --paginate` writes **each page as its
own top-level JSON value**, so a two-page read redirects `[...][...]` into the
file and no JSON parser will read it. The failure fires exactly when pagination
does, which is the condition this spec exists to handle, and it would have been
found by the first PR with more than thirty comments rather than by a test. The
reads therefore pass `--slurp`, which wraps the pages in one outer array, and the
skill says the file is an array of pages and how to flatten it. This is the
output format of a required read, not a new requirement: §3.1's obligation,
§3.3's values and §3.4's budget are unchanged, and the alternative that would have
satisfied a parser (piping into `--jq`) is the one §3.1 forbids because it
destroys the read's exit status.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line. The subject is a skill body, so the assertions are over its text and
over the parity the generators guarantee; what a session does with it is not
executable from here, and §4 does not pretend otherwise.

**Fail-first evidence**, measured at the parent commit:

| Line | At the parent (measured 2026-09-19) |
|---|---|
| `grep 'issues/<number>/comments'`, `grep 'pulls/<number>/reviews'` | red in all four copies, both absent (exit 1) |
| per-endpoint `--paginate` (§3.1) | red: `--paginate` appears zero times in either copy, so the one shipped read is unpaginated |
| `! ... gh api ... --jq` (§3.1) | red, the shipped read pipes into `--jq` and loses its own status |
| `could not read <endpoint>` on the `Review threads:` line (§3.3) | red, the line offers `none`, `n addressed`, `n need a human`, `not read: stopped at CRITICAL`, and nothing for an endpoint that would not answer |
| `Thread reads:` line (§3.3) | red, absent |
| `! grep 'every required check \`SUCCESS\`: go to Step 4'` | red, that is exactly what Step 1 says (one occurrence) |
| `grep '\`SUCCESS\`' | grep 'Step 3b'` (§3.2) | red, the all-green route names Step 4 and not Step 3b |
| `one retry`, `before Step 4`, `even when every required check was` | red, all three absent |
| the five `shepherd_*` tests added to `kit_skills.rs` | red: all five fail against the parent's skill text, each naming the line it read |
| `registry show 116` | red, not found, exit 1 |

Five lines are **green at the parent and stay green**, and are preservation
rather than evidence: `pulls/<number>/comments`, which this spec keeps rather
than replaces; `not read`, spec 082's value for the CRITICAL path; the fuller
`not read: stopped at CRITICAL`, which is a guard against the cheap way to add
§3.3's second missing-read reason, namely widening the existing value until it
covers both and names neither; `at most two rounds`, the budget §3.4 does not
touch; and `gh pr comment` in the workflow, which is what keeps §1.1's
measurement true of the live arrangement rather than of a workflow that has
since moved.

```verify:cli
cargo build --release --locked
# 3.1: all three endpoints are named, in the kit source and in the copy this
# repository runs.
grep -qF 'pulls/<number>/comments' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'issues/<number>/comments' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'pulls/<number>/reviews' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'issues/<number>/comments' .claude/skills/shepherd/SKILL.md
grep -qF 'pulls/<number>/reviews' .claude/skills/shepherd/SKILL.md
# 3.1: every read is paginated, asserted on the command line that performs it.
# `gh api` returns one page by default, so three endpoints read one page deep is
# the same wrong answer one level down (D-5). A count of `--paginate` is not the
# assertion: three flags on one endpoint would satisfy it (D-7).
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'pulls/<number>/comments' | grep -qF -- '--paginate'
# 3.1 / D-12: and `--slurp`, without which the paginated file is a run of
# top-level JSON values rather than one document.
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'pulls/<number>/comments' | grep -qF -- '--slurp'
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'issues/<number>/comments' | grep -qF -- '--slurp'
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'pulls/<number>/reviews' | grep -qF -- '--slurp'
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'issues/<number>/comments' | grep -qF -- '--paginate'
grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -F 'pulls/<number>/reviews' | grep -qF -- '--paginate'
# 3.1: and the read's own exit status is checkable, which a pipe into `--jq`
# destroys.
! grep -F 'gh api' kit/.claude/skills/shepherd/SKILL.md | grep -q -- '--jq'
# 3.1 / D-8: one retry, then a stop before the merge checkpoint.
grep -qF 'one retry' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'before Step 4' kit/.claude/skills/shepherd/SKILL.md
# 3.3 / D-10: the report line carries the value for an endpoint that would not
# answer, spelled as the spec spells it, beside the CRITICAL stop's own value.
# Asserted on the report TEMPLATE line, not over the file: `failed` already
# appears at the parent in `--log-failed` and in the fmt/clippy prose, so a
# file-wide grep for it is green before this spec and witnesses nothing
# (measured 2026-09-18).
grep -E '^Review threads:' kit/.claude/skills/shepherd/SKILL.md | grep -qF 'could not read <endpoint>'
grep -E '^Review threads:' .claude/skills/shepherd/SKILL.md | grep -qF 'could not read <endpoint>'
# 3.3: and `not read` keeps its CRITICAL-stop meaning beside the new value,
# rather than being widened to cover both and telling a reader neither.
grep -qF 'not read: stopped at CRITICAL' kit/.claude/skills/shepherd/SKILL.md
# 3.3: successful coverage is reported on its own line, never folded into the
# value above it.
grep -qE '^Thread reads:' kit/.claude/skills/shepherd/SKILL.md
# 3.5: and in the two generated trees, which is the generators' job, not a
# second edit.
grep -qF 'issues/<number>/comments' .agents/skills/shepherd/SKILL.md
grep -qF 'issues/<number>/comments' crates/spec-spine-core/src/kit_embedded.rs
# 3.2: the green path no longer routes past the thread read. The literal that
# sent an all-green PR straight to the merge checkpoint is gone, and the route
# that replaced it names the step it now passes through (a deletion alone would
# satisfy the negative).
! grep -qF 'every required check `SUCCESS`: go to Step 4' kit/.claude/skills/shepherd/SKILL.md
! grep -qF 'every required check `SUCCESS`: go to Step 4' .claude/skills/shepherd/SKILL.md
grep -F '`SUCCESS`' kit/.claude/skills/shepherd/SKILL.md | grep -qF 'Step 3b'
grep -F '`SUCCESS`' .claude/skills/shepherd/SKILL.md | grep -qF 'Step 3b'
# 3.3: the report still distinguishes an unread absence, and says so for the
# path that still cannot read (the CRITICAL stop).
grep -qF 'not read' kit/.claude/skills/shepherd/SKILL.md
# 3.4 / D-9: the two-round budget is untouched, and what draws on it is an edit
# rather than a red check.
grep -qF 'at most two rounds' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'After two remediation rounds' kit/.claude/skills/shepherd/SKILL.md
grep -qF 'even when every required check was' kit/.claude/skills/shepherd/SKILL.md
! grep -qF 'three rounds' kit/.claude/skills/shepherd/SKILL.md
# 3.5: the trees agree, byte for byte, and the generators are the reason. The
# five `shepherd_*` tests added for this spec are in the first of these.
cargo test -p spec-spine-core --test kit_skills --locked
cargo test -p spec-spine-core --test agent_trees --locked
cargo test -p spec-spine-core --test scaffold --locked
python3 scripts/gen-agent-trees.py --check
# 1.1: the workflow this repository runs still posts where 3.1 now reads, so the
# measurement that motivated the spec is still the live arrangement.
grep -qF 'gh pr comment' .github/workflows/ai-pr-review.yml
# Declared and read through the CLI, redirected rather than piped (spec 107 D-4).
target/release/spec-spine registry show 116 --json > "${TMPDIR:-/tmp}/ss116-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss116-show.json')); assert d['id'] == '116-shepherd-reads-every-reviewer', d"
rm -f "${TMPDIR:-/tmp}/ss116-show.json"
```
