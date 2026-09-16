---
name: next
description: "Name the next spec to build: the ready set from spec-spine registry plan, minus anything a human has not approved, with in-flight specs and honest blockers reported separately. Read-only."
allowed-tools: Bash, Read
---

# /next: the next work order

The backlog is the spec corpus. Step 1 of `AGENTS.md` "Working the
backlog" picks the spec, and this skill computes that pick from typed
reads. Two rules sit on top of what the tool reports:

- **Approval is a human act.** `spec-spine registry plan` (spec 038)
  schedules by `implementation` and `depends_on`; it will offer a
  `status: draft` spec whose dependencies are met. A draft is never a
  work order: drop it from the ready set and list it as "awaiting
  approval".
- **One session, one spec.** A spec at `implementation: in-progress` is
  in flight (spec 044): it belongs to another session or to an
  interrupted one. Report it separately; never offer it as new work
  unless a human names it.

## Step 1: the ready set, typed

```sh
spec-spine registry plan --json
```

Exit 0 prints `{"ready": [...ids in dependency order...], "blocked": [{"id", "blockedBy": [{"id", "state"}]}]}`.
A non-zero exit is a halt: `V-014` names a dependency cycle (schedule
nothing from a broken graph; print the diagnostic verbatim), and any
other failure means the registry is unreadable (`/setup`, or a stale
binary).

Parsing this output is a typed read (`.claude/rules/governed-artifact-reads.md`).
Reading `.derived/` is not.

## Step 2: apply the two rules

For each ready id, read its record:

```sh
spec-spine registry show <id> --json
```

- `status` is not `approved`: move it to "awaiting approval".
- `implementation` is `in-progress`: move it to "in flight".
- Otherwise it is a work order. Keep the tool's order.

Record specs (the bootstrap, a thesis, a harness spec at `n-a` or
`complete`) never appear in `ready`; if one does, its `implementation`
is wrong and that is a finding, not a pick.

## Step 3: the pick, cross-checked

Take the first work order unless a human named another. Read its
`## Territory` (or equivalent) for an operator prerequisite (a service, a
credential, a sibling repository) that is missing: that is a stop at
step 1 of the protocol, reported exactly, not mocked around.

## Step 4: report

```
## next: <id>
title: <title>
status: approved; implementation: pending
depends_on:
  - <dep>: <implementation state>
also ready: <ids in order, or none>
in flight: <ids at in-progress, or none>
awaiting approval: <draft ids the tool would have offered, or none>
blocked: <id>: <blockedBy id and state>; ...
prerequisites: <none named / what the Territory requires>
```

When nothing is a work order, print `## next: none ready` and still list
in flight, awaiting approval, and blocked with their reasons. Never hide
an unapproved spec and never offer one.

## Rules

- Never guess an id from `ls specs/`; the registry is the source.
- `/next` is read-only. It does not flip, branch, or commit; `/build <id>`
  does.

## Project layer

Nothing here is project-specific. Which specs are records is visible from
their `implementation` value, not from a list in this file.
