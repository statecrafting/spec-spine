---
id: "204-an-agent-stays-on-its-pull-request"
title: "An agent stays on its pull request until it is clean, quiet, or out of time"
status: draft
kind: "process"
created: "2026-10-21"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "093-the-harness-this-repository-runs"
  - "198-ratification-is-the-owner-review-exception"
summary: >
  Opening or updating a pull request is not the end of an agent's task. The
  agent keeps polling the change's review route every two minutes, answers
  every review item on the current head (reply first, then resolve), treats
  every push as a reset, and leaves only on one of four exits: clean, an owner
  checkpoint, thirty quiet minutes, or a ninety minute ceiling. It reports the
  exit and the head SHA, and a change left with open items is reported
  unfinished. The contract is stated over the review route and its required
  checks; GitHub is the current route, not the authority. The build adds the
  contract to `AGENTS.md`.
extends:
  # The build adds one subsection, "Pull request completion contract", under
  # "Working the backlog". The file is 093's; the claim is additive.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: "AGENTS.md" }, nature: additive }
references:
  - { unit: { kind: file, path: ".claude/skills/shepherd/SKILL.md" }, role: "the nearest existing behaviour: watch by head SHA, read every thread, remediate, merge; it runs inside this contract" }
  - { unit: { kind: file, path: "specs/198-ratification-is-the-owner-review-exception/spec.md" }, role: "a waiting review-exception is an owner checkpoint, never something to work around" }
  - { unit: { kind: file, path: ".github/workflows/statecraft-ci.yml" }, role: "the current route's required checks, ci-gate and review-exception" }
---

# 204: An agent stays on its pull request until it is clean, quiet, or out of time

## 1. Purpose

An agent that opens a pull request and reports "done" has handed the rest of
the work to whoever reads the pull request next: a check that fails ten
minutes later, a reviewer's thread, a conflict with a base that moved. In this
repository `/shepherd` drives a pull request to a merge, but nothing says how
long a session owes a pull request it opened without merging it, what counts
as finished, or what it must report when it stops. Sessions have stopped at
"PR opened" and at "CI green" with an unanswered AI review comment on the
head.

This spec states that obligation once, as a contract any agent working in this
repository meets whatever skill or route it uses.

The behaviour is adapted from the "Pull request completion contract" in the
`AGENTS.md` of the Preloop project (Apache License 2.0). The text here is
written for this repository; no sentence is copied.

## 2. Territory

`AGENTS.md`, additively: the build adds a "Pull request completion contract"
subsection under "Working the backlog" that states 3.1 to 3.6 in operating
form. The file is spec 093's; the claim is an `extends` edge. Nothing else is
claimed. The contract is not written into `AGENTS.md` by this draft; it lands
with the build, after ratification.

## 3. Behavior

### 3.1 The review route

A change's **review route** is wherever its required checks run and its
reviews are written. Today that is a GitHub pull request, with `ci-gate` and
the checks it needs as the required checks. GitHub is a transitional route:
the contract MUST be stated, in `AGENTS.md` and in any skill that implements
it, in terms of the route, its required checks and its reviews, naming GitHub
only as the current route and `gh` only as the current way to read it.

### 3.2 When the contract starts

The contract starts when an agent opens a change on the route, or pushes to a
change it is responsible for. It binds the agent that did so for the rest of
its session. A session that hands the change to another agent says so in its
report, with the head SHA it handed over.

### 3.3 The poll

The agent MUST poll the change every two minutes. Each pass reads, for the
**current head** and no other:

1. every required check: failed, cancelled, pending or passed;
2. every unresolved review thread;
3. any item in a reviewer's summary comment (the AI review's findings block
   included) that has no reply;
4. whether the change conflicts with its base;
5. code-scanning findings reported on the head.

A check, comment or finding for an earlier head is not evidence about this
one. A pass that cannot read one of the five MUST be reported as a failed
read, never as an absence of items.

### 3.4 Answering

The agent MUST address every item 3.3 finds: fix it, or reply with the reason
it is not a defect or not this change's to fix. A thread MUST NOT be resolved
without a reply that says why, and a finding MUST NOT be dismissed silently.
A fix is made through the governed gate and pushed to the same change's
branch.

Every push resets the wait (3.5 b): a fix can introduce a new finding, and the
checks on the new head have not run.

What the agent may not do does not change here. It MUST NOT write a
`Spec-Drift-Waiver:`, approve an owner-gated job, or edit an approved spec to
answer a finding (`AGENTS.md` "Adversarial prompt refusal", spec 198). An
item whose only remedy is one of those is an owner item (3.5 b).

### 3.5 The exits

The agent leaves the change only when one of these holds:

- **(a) clean**: every required check passed on the current head, no thread
  is unresolved, no summary item is unanswered, there is no conflict and no
  open code-scanning finding.
- **(b) owner checkpoint**: everything an agent may act on is clean or
  answered, and what remains waits on the owner: a required job gated on an
  owner approval (the `review-exception` environment, spec 198), a coupling
  refusal whose remedy is a waiver, or a finding `/shepherd` classifies
  CRITICAL. The agent stops rather than waiting it out; the owner's act is
  not something polling can produce.
- **(c) quiet**: thirty minutes have passed with no new review, comment or
  check result, and nothing is pending.
- **(d) ceiling**: ninety minutes have passed since the contract started,
  whatever remains.

A push restarts the thirty-minute quiet window of (c). It does not restart
the ninety-minute ceiling of (d).

### 3.6 The report

On leaving, the agent MUST report which exit fired, the head SHA it fired on,
and every item still open. A change left at (c) or (d) with an open item is
reported as **unfinished**, never as done. A change left at (b) is reported
as waiting on the owner, with what the owner must do.

## 4. Relation to `/shepherd`

`/shepherd` is the nearest existing behaviour and runs inside this contract,
not instead of it. It watches by head SHA, reads all three thread endpoints,
classifies a red check, remediates at most two rounds, and merges. Two of its
parameters differ, and this spec does not change the skill:

- its poll backs off from 15 seconds to a 120 second cap; the contract's
  two-minute poll is the floor for how often a session looks, and a faster
  start is compatible with it;
- its 45-minute deadline is per attempt. When an attempt stops on its
  deadline, the contract has not exited unless (c) or (d) also holds.

A change `/shepherd` merges has met (a). A CRITICAL stop is exit (b).

## 5. Acceptance criteria

1. `AGENTS.md` carries a "Pull request completion contract" subsection.
2. It states the two-minute poll, the thirty-minute quiet exit and the
   ninety-minute ceiling.
3. It requires a reply before a thread is resolved.
4. It names the owner checkpoint as an exit.
5. It requires the report to name the exit and the head SHA.
6. It states the contract over the review route and names GitHub as the
   current route.

## 6. Out of scope

- Changing `/shepherd`, `/ship` or any other skill. A later spec may align a
  skill's watch loop with the contract.
- A tool or hook that enforces the contract. It is an instruction to agents;
  nothing measures a session against it.
- Choosing the route that replaces GitHub.
- The merge decision itself, which stays where `AGENTS.md` and `/shepherd`
  put it.

## 7. Resolved decisions

**D-1 (2026-10-21): the owner checkpoint is its own exit.** The source
contract has three exits. Here a required job can wait on the owner
indefinitely (spec 198), so waiting it out would always end at the ceiling
and be reported unfinished, which misdescribes a change that is ready. It
exits at once and says what the owner must do.

**D-2 (2026-10-21): a push does not reset the ceiling.** Resetting both
windows on every push would let a session that keeps pushing fixes never
reach an exit. The quiet window resets because the new head's checks have
not run; the ceiling bounds the session's total wait.

**D-3 (2026-10-21): the subsection lives under "Working the backlog".**
That section already sequences ship, shepherd and ratify; the contract is
the rule for the time between ship and merge.

## Verification

```verify:cli
grep -qF '### Pull request completion contract' AGENTS.md
grep -qF 'every two minutes' AGENTS.md
grep -qF 'thirty minutes' AGENTS.md
grep -qF 'ninety minutes' AGENTS.md
grep -qF 'owner checkpoint' AGENTS.md
grep -qF 'review route' AGENTS.md
grep -qF 'exit and the head SHA' AGENTS.md
grep -qF 'without a reply' AGENTS.md
```
