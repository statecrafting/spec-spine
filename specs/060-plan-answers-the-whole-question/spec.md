---
id: "060-plan-answers-the-whole-question"
title: "The plan answers the whole question, and names one pick"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "038-registry-plan-ready-set"
  - "044-in-progress-is-in-flight"
  - "045-absent-implementation-defers-to-status"
extends:
  - { spec: "038-registry-plan-ready-set", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
  - { spec: "038-registry-plan-ready-set", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: ".claude/skills/next/SKILL.md" }, role: context }
summary: >
  `registry plan` prints spec ids and a count. Every consumer needs more than
  that, so every consumer adds it back: adopters kept about forty lines of
  Python to join the ready ids against titles and to explain why each blocked
  spec is blocked, and the corpus's own `/next` skill has to make a second call
  to turn an id into something a human can read. The data is all there. `Plan`
  already carries each blocked spec's blockers with the state of each, and the
  registry it was computed from carries every title. This spec makes the prose
  form render what the structure already holds, and adds `--next`, the single
  pick that the one-spec-per-session loop actually asks for. The JSON shape
  gains title fields and is otherwise unchanged.
---

# 060: The plan answers the whole question, and names one pick

## 1. Purpose

Spec 038 built `registry plan` to answer the scheduling question, and it
answers it correctly. `Plan` carries a topologically ordered `ready` list and a
`blocked` list where each entry names its blockers and each blocker carries the
state that makes it a blocker. The ordering of both is contractual. The cycle
case is refused rather than truncated. As a data structure it is complete.

Then the CLI prints this:

```
052-couple-names-the-crossing
ready: 1, blocked: 0
```

Ids and a count. The blocked list, which is the half a person reads when the
ready set is empty, is rendered as a number. The titles, which are in the same
registry the plan was computed from, are absent. So consumers add them back:
the audit found roughly forty lines of Python doing the join in adopting repos,
and this repository's own `/next` skill has to issue a second `registry` call to
turn an id into a line a human can act on.

There is also a shape mismatch. The corpus's working rule is one session, one
spec: `.claude/rules/orchestrator-rules.md` says so, `AGENTS.md` says so, and
every driven session begins by picking exactly one. `plan` returns a list, so
every caller takes the first element and re-derives which "first" means the
right one. That derivation is contractual (topological, ties by ascending id)
and it is written down in `query.rs`, which is not where a caller looks.

These are items 13 and 14 of the adopter audit's ranked backlog for the tool,
and they are one change: the second is the smallest useful projection of the
first.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/query.rs` | 038 | titles on the plan, `--next` selection |
| `crates/spec-spine-core/tests/query.rs` | 038 | acceptance |
| `crates/spec-spine-cli/src/cmd_registry.rs` | 002 | the rendering and the flag |

Spec 038 defines what the ready and blocked sets contain and the order of both.
Nothing here changes either. It says nothing about the prose rendering, which is
the surface this spec adds to.

## 3. Behavior

### 3.1 The prose form renders what the structure holds

`registry plan` MUST render both sets with titles, and MUST render each blocked
spec's blockers with the state of each:

```
ready (1):
  052  The coupling gate names the crossing

blocked (2):
  061  A version pin the CLI can check
       blocked by 054 (draft)
  062  The stale binary is not a stale ledger
       blocked by 054 (draft), 061 (pending)

53 specs: 1 ready, 2 blocked, 50 not schedulable
```

Three facts the current output does not carry, each already in hand:

- **Titles.** Joined from the same registry `plan` was computed from. No second
  load, no second call.
- **Why each blocked spec is blocked.** `BlockedSpec.blocked_by` is a list of
  `Blocker { id, state }` and the state is the reason. Printing the count and
  discarding the reasons is discarding the part a reader needs.
- **The remainder.** `plan` excludes specs the corpus has moved past
  (`superseded`, `retired`) or that someone has already answered "no" for
  (`complete`, `n-a`, `deferred`). Today those specs vanish silently, so the two
  numbers do not add up to the corpus and a reader cannot tell whether the
  missing ones were excluded or lost. One "not schedulable" figure closes it.

The ordering contract is untouched: `ready` stays topological with ties by
ascending id, `blocked` stays ascending by id, and each entry's blockers stay in
that spec's authored `depends_on` order. Spec 038 §3.2 requires the report to be
a pure function of the corpus, and adding titles from that same corpus keeps it
one.

`(nothing ready)` stays exactly as it is when `ready` is empty. It is the line a
finished corpus prints and it already carries its own count.

### 3.2 `--next`

`registry plan --next` MUST print exactly one spec: the first element of
`ready`.

```
052  The coupling gate names the crossing
```

With `--json`, the single spec object rather than an array, so a consumer does
not index into a one-element list to reach the thing it asked for.

An empty ready set exits `0` and prints `(nothing ready)`. It is a true answer
to "what should I work on", not a failure, and a driven session that treats
"nothing to do" as an error will stop for the wrong reason. `Error::NotFound`
would be wrong here: nothing was asked for by name.

`--next` MUST be a projection of `plan` and MUST NOT reimplement selection. The
first element of the topological order is already the defined pick; this flag
names it so that callers stop re-deriving what "first" means. If the ordering
contract ever changes, it changes in one place and `--next` follows.

`--next` and the full listing are mutually exclusive by nature, and passing
`--next` simply prints the one line. There is no interaction with a status
filter, because `plan` takes none.

### 3.3 The JSON shape gains titles and nothing else

`plan --json` MUST gain a `title` on each ready entry and on each blocked entry.

Ready entries are bare id strings today, so this is a **breaking** shape change
for that array: strings become objects. Blocked entries are already objects and
gain a field additively.

Making `ready` an array of objects is the right call rather than adding a
parallel `readyTitles` array. Two arrays that must be zipped by position is the
shape that generates the exact join code this spec exists to delete, and the
asymmetry with `blocked` (objects on one side, strings on the other) is already
a thing consumers work around.

`plan` emits no verdict envelope and writes no artifact, so
`VERDICT_SCHEMA_VERSION` and `REGISTRY_SCHEMA_VERSION` are both untouched. The
change is to one read verb's `--json` output and MUST be called out in the
release notes as breaking for that verb.

### 3.4 It still answers what is claimed, not what is done

`query.rs` records the reason `plan` is the right input to scheduling and the
wrong input to acceptance: `implementation` is self-declared, no gate verifies
that a spec marked `complete` has any code, and a scheduler that reads a claim
must stay a different mechanism from an adjudicator that verifies one.

Nothing here changes that, and `--next` MUST NOT be read as an instruction. It
is the corpus's answer to "what is schedulable first", which is a claim about
claims. `spec-spine verify <id>` (spec 049) is the verb that checks whether the
work was done, and it deliberately runs after a merge rather than inside this
one.

## 4. Out of scope

**Choosing differently.** `--next` returns the first element of the existing
order. Priority fields, effort estimates, or a weighting over `depends_on` fan-out
are a scheduler, and a scheduler is not the ledger's job.

**Filtering the plan.** No `--status`, no `--domain`. `plan` partitions the whole
corpus by schedulability, and a filter on top of that partition would produce a
ready set that is not the ready set.

**Reporting the excluded specs individually.** The "not schedulable" figure is a
count. Which specs those are, and why each was excluded, is `registry list` plus
`status-report`, and duplicating them inside `plan` would make the verb's output
the corpus.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test query --locked
# The prose form carries titles and the not-schedulable remainder.
target/release/spec-spine registry plan | grep -q 'not schedulable'
# --next prints one spec. Until this ships, it is an unknown argument.
target/release/spec-spine registry plan --next
# The ready array carries titles, so no consumer needs a second call.
target/release/spec-spine registry plan --json | grep -q '"title"'
# The ledger is untouched by a read verb.
target/release/spec-spine compile --check
```
