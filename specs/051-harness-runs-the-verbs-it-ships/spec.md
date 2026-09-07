---
id: "051-harness-runs-the-verbs-it-ships"
title: "The harness runs the verbs it ships"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "046-kit-hooks-read-never-write"
  - "048-kit-ships-the-governed-loop-skills"
  - "049-verify-declared-acceptance"
  - "050-index-diagnostics-reach-a-gate"
amends:
  # 048 3.1's table binds `/verify` to `scripts/verify-spec.sh`, and 048 3.5
  # calls that script the one this repository runs. This spec binds the skill to
  # `spec-spine verify` instead, which is what 048 4 itself said would happen
  # once the verb existed. 048's file is unchanged (spec 040).
  - "048-kit-ships-the-governed-loop-skills"
  # 046 fixes the hooks' invocation as a bare `spec-spine` resolved from PATH.
  # This spec makes them resolve the binary the project declares, because a
  # repository that builds its own is otherwise governed by a different one.
  - "046-kit-hooks-read-never-write"
establishes:
  - ".claude/settings.json"
extends:
  - { spec: "029-claude-code-skill-kit", unit: "AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/README.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/settings.json", nature: superseding }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/skills/", nature: superseding }
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: ".claude/skills/", nature: superseding }
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: "crates/spec-spine-core/tests/kit_skills.rs", nature: additive }
  - { spec: "046-kit-hooks-read-never-write", unit: "crates/spec-spine-core/tests/kit_hooks.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "specs/049-verify-declared-acceptance/spec.md" }, role: context }
summary: >
  Release v0.15.0 shipped two capabilities the harness was written to want and
  then did not use. Spec 049's `spec-spine verify <id>` retired nothing: all
  nine harness sites still call `scripts/verify-spec.sh`, so one protocol now
  has two implementations whose answers are already worded differently, which is
  the drift 048 1.2 exists to end reappearing inside the repository that wrote
  it. Spec 050's `--fail-on-unresolved` and `index diagnostics` reached no
  caller: the spec written because a warning nobody sees is a unit that quietly
  went unresolved is, in its own dogfood repository, still a warning nobody
  sees. Underneath both sits a third gap that made them possible: every skill
  says to run the gate exactly as AGENTS.md lists it, and AGENTS.md's list omits
  the ownership ratchet CI enforces, so the six skills that inline the step are
  more correct than the file they defer to. This spec makes the harness run what
  the tool ships: `/verify` wraps the verb, AGENTS.md names one gate list that a
  test pins against the skills, this repository turns on 050's gate and reads
  its diagnostics at session start, the kit's hooks resolve the binary the
  project declares instead of whatever a bare name finds on PATH, and this
  repository runs the hooks it ships to adopters.
---

# 051: The harness runs the verbs it ships

## 1. Purpose

Spec 048 made the kit ship one skill set and made this repository run it, pinned
by a test. Specs 049 and 050 then added two capabilities that the skill set was
the natural consumer of. Neither reached it. This spec closes the distance, and
fixes the structural reason the distance opened.

### 1.1 The verb shipped and the harness kept the script

Spec 048 4 named the trigger in advance:

> `spec-spine verify <id>` as a tool verb (tool backlog item 1). The script is
> the protocol until the verb exists; when it does, `/verify` wraps the verb and
> the script retires.

The verb shipped in v0.15.0. The harness did not notice. There are zero
occurrences of `spec-spine verify` anywhere under `.claude/`, `kit/`, or either
`AGENTS.md`, and nine sites still route through `scripts/verify-spec.sh`. The
two implementations have already diverged in the only place a caller reads:

```
script: not-declared (Verification section holds no verify:cli commands)
verb:   not-declared (no verify:cli commands under ## Verification)
```

That is one protocol answering the same question two ways, which is exactly the
condition 048 1.2 was written to end.

### 1.2 The gate list the skills defer to is not the gate CI runs

Every skill that runs the gate says to run it "exactly as `AGENTS.md` lists it".
`AGENTS.md` step 5 lists `compile`, `index`, `lint --fail-on-warn`,
`index check`, `couple`, then the stack's build, tests and lints. CI's
`self_governance` job additionally runs `index coverage --fail-on-untraced` as
an enforced step, and `[coupling] require_ownership` is on in this repository.

Four skills (`build`, `ship`, `setup`, `validate-and-fix`) inline the coverage
step anyway. So the skills are currently more correct than the authority they
name, and an agent that obeys the instruction literally skips the ownership
ratchet and learns about it from CI. A deferral target that is wrong is worse
than no deferral target, because it is followed.

This is the structural fault beneath 1.1 and 1.3: a single source of truth that
nothing checks is a single source of drift.

### 1.3 The hooks the kit ships resolve a binary this repository does not use

`kit/settings.json` ships four hooks that gate pushes and PR creation and report
freshness. They invoke a bare `spec-spine`, resolved from `PATH`. `AGENTS.md`
says the opposite for this repository: "Self-governance runs through the in-tree
binary (`target/release/spec-spine`), not the published npm/py distributions",
and the init protocol says so again.

On the maintainer's machine at the time of writing, `PATH` resolves
`spec-spine` to 0.14.0 while the checkout is 0.15.0. A hook wired that way
gates a 0.15.0 corpus with a 0.14.0 judgement: it cannot see `verify`, it cannot
see `index diagnostics`, and a `compile --check` from a binary that predates the
checkout is the phantom-staleness trap the init protocol already documents.

This repository also does not install the hooks at all, which is how the defect
stayed invisible: `kit_hooks.rs` asserts properties of `kit/settings.json` as a
JSON document, and nothing ever runs it. Four skills describe hook behavior to
their reader (a push gate, a PR-time coupling gate, a recompile after a spec
edit) that does not happen here. The one repository positioned to catch a hook
defect is the one repository not running them.

## 2. Territory

This spec establishes `.claude/settings.json`: this repository's own Claude Code
configuration, which until now did not exist.

It extends, without changing their purpose, the units the kit and harness
already own: `AGENTS.md` and `kit/AGENTS.md` (the gate list and the init
protocol), `kit/README.md` (the install steps), `kit/settings.json` (the hooks'
binary resolution), `kit/.claude/skills/` and `.claude/skills/` (the `verify`
and `validate-and-fix` skills), and the two kit tests.

It amends 048 and 046 as the frontmatter records: 048 because 048 3.1 binds
`/verify` to the script and 048 3.5 calls that script the one this repository
runs, and 046 because 046 fixes the hooks' invocation as a bare name. Neither
`spec.md` is edited (spec 040).

It does not touch the engine. No file under `crates/*/src/` changes.

## 3. Behavior

### 3.1 `/verify` wraps the verb

The `/verify` skill MUST invoke `spec-spine verify <id>`, through the binary
invocation `AGENTS.md` names, and MUST NOT invoke `scripts/verify-spec.sh`. The
skill MUST document the verb's outcomes as the verb words them, and MUST
describe `--plan` as the way to read a `## Verification` block before running
it.

The skill MUST also state the version floor: the verb requires spec-spine
0.15.0 or later. When the configured binary does not carry `verify`, the skill
MUST report that plainly and name the upgrade, rather than silently falling back
to a different implementation. A harness that quietly runs a second
implementation is the condition this spec exists to remove.

`kit/AGENTS.md`, `AGENTS.md`, and `kit/README.md` MUST name the verb where they
currently name the script.

### 3.2 The scripts stay one release, deprecated

`scripts/verify-spec.sh` and `kit/scripts/verify-spec.sh` MUST NOT be deleted by
this spec. Spec 049 4 gave the reason and it is still true: removing the script
from `kit/` strands every adopter whose pinned spec-spine predates the release
carrying the verb, and all four adopters are behind v0.15.0, which shipped the
same day this spec was filed.

`kit/README.md` MUST mark the script deprecated, name the release that replaces
it, and state that an adopter on 0.15.0 or later can delete their copy.

This also keeps spec 048's `## 5. Verification` block executable. Two of its six
commands run the script; deleting the script would leave an approved spec's
declared acceptance unable to pass, and repairing that by editing 048 is the
move the coherence guard exists to stop. The retirement is 4's declared
follow-on, and its trigger is adopters upgrading, not this spec merging.

### 3.3 One gate list, and a test that pins it

`AGENTS.md` "Run the gate before every commit" MUST list every command CI
enforces, including `index coverage --fail-on-untraced`. The order is the local
one, not CI's: `compile` and `index` write, and the local gate runs them before
the checks, while CI runs `compile --check` because it must never repair the
tree it is judging.
`kit/AGENTS.md`'s template list MUST carry the same step, marked as conditional
on `[coupling] require_ownership`, since an adopter may not have it on.

`crates/spec-spine-core/tests/kit_skills.rs` MUST assert that the governance
floor each skill inlines is a subset of the list `AGENTS.md` names. The test
reads both files from disk, as it already does for the byte-identity assertion.
A skill that tells its reader to run a command the authority does not list, or
an authority that drops a command the skills run, MUST fail the build.

The assertion is a subset relation, not equality: a skill may legitimately name
fewer steps than the full gate (`/code-review` uses the read-only forms), but it
may never name a governance step the authority omits.

### 3.4 This repository runs spec 050's gate

CI's `self_governance` job MUST run `index check --fail-on-unresolved`. The
corpus records zero diagnostics today, so the flag costs nothing now and refuses
the accumulation 050 1.3 measured at 248 in another adopter.

The init protocol in `AGENTS.md` MUST read `spec-spine index diagnostics` among
its parallel reads, and `/init` MUST report the count. A read verb that no
protocol reads is 050's own gap repeated one level up.

`kit/AGENTS.md` MUST offer both as commented, opt-in lines rather than defaults:
050 3.2 made the flag opt-in precisely so a corpus that ratifies before it builds
can carry warnings while work is under way, and the kit's template must not
decide that for an adopter.

### 3.5 The hooks resolve the binary the project declares

`kit/settings.json`'s hooks MUST resolve the spec-spine binary in this order,
using the first that exists:

1. `$SPEC_SPINE_BIN`, when set;
2. `./target/release/spec-spine`, relative to the repository the hook acts on;
3. `spec-spine` from `PATH`.

The repository the hook acts on is the one the existing hooks already compute
(the `git rev-parse --show-toplevel` of the command's target), not the session's
project, so a multi-repository session keeps resolving per repository.

The order is the point: a repository that builds its own binary is governed by
the binary it builds. Falling through to `PATH` keeps the kit working for an
adopter who installs the published CLI and never builds one.

`crates/spec-spine-core/tests/kit_hooks.rs` MUST assert the resolution order is
present in every hook that invokes the binary, and MUST keep asserting 046's
read-never-write property unchanged: this spec changes which binary the hooks
call, never what they are permitted to do with it.

### 3.6 This repository runs the hooks it ships

`.claude/settings.json` MUST exist in this repository and MUST carry the four
hooks `kit/settings.json` ships, so the hooks are exercised where a defect is
caught rather than only asserted as JSON. It MUST also carry the kit's `deny`
list, which refuses `cargo publish`, `npm publish`, `gh release create`,
`git push --force`, and the destructive `rm -rf` forms.

It MAY differ from `kit/settings.json` in its permission `allow` list, which is
machine-local convenience rather than governance. It MUST NOT differ in the
hooks or in `deny`.

The file is committed. `.claude/settings.local.json` remains untracked and
personal.

### 3.7 `/validate-and-fix` stops asserting facts this repository lacks

The `/validate-and-fix` skill opens by naming a composite ("commonly `make ci`")
as the single source of truth and telling its reader not to rediscover
validation commands. This repository has no Makefile and `AGENTS.md` names no
composite, so the skill's `## Project layer` points at nothing.

The skill MUST treat the composite as optional: use it when `AGENTS.md` names
one, and otherwise run the gate list `AGENTS.md` gives directly, which is what
the skill's own step 1 already does.

Its substrate note MUST stop asserting which paths a project hashes. It
currently tells the reader that `[index] extra_hashed_inputs` "lists the
harness, design docs, workflows, and standards"; in this repository it lists
`standards/**` and `.github/workflows/**` only, so editing a skill does not
stale the index here. The note MUST tell the reader to read the config instead
of telling them what it says.

## 4. Out of scope

**Deleting the two verify scripts.** 3.2 states the reason and names the
trigger. The follow-on is filed when the four adopters are on 0.15.0 or later,
and it will amend 048 for the units it removes.

**Propagating any of this to the adopters.** Each adopter takes the kit by copy
in its own session, as 048 4 already established. The hook binary-resolution fix
matters most to an adopter who builds from source, and none currently do.

**A summary line for `index orphans` on an empty result.** This was filed as a
finding and does not survive spec 011. 011 3.3 states "empty list implies empty
output" and documents `test -z "$(spec-spine index orphans)"` as the supported
gating idiom; a summary line would invert that gate for every adopter using it.
The silence is specified and load-bearing, not a papercut. `index diagnostics`
matching its sibling is therefore coherent, and neither verb changes here.

**Making `.claude/settings.json` a hashed input.** Adding it to
`[index] extra_hashed_inputs` would make every hook edit stale every index
shard, which is 3.7's complaint in reverse. If the harness should be hashed, that
is a decision about `spec-spine.toml` and belongs in its own spec with the cost
stated.

**Any engine change.** No file under `crates/*/src/` changes. The two test files
this spec touches are acceptance, not behavior.

**Tightening the PR-gate hook's waiver match.** The `PreToolUse` hook accepts a
coupling failure when the command string contains `--body` followed by
`Spec-Drift-Waiver`, so a branch name or commit message carrying that text and
interpolated into `gh pr create` would satisfy it without a waiver in the body.
The behavior predates this spec, and CI's coupling gate is the authoritative
one: the hook is an early warning that cannot approve a merge. Tightening the
match is a change to what spec 046 requires and belongs in a spec that says so.

**A `/burndown` skill.** Unchanged from 048 4: it waits on a tool verb, and 050
shipped `index diagnostics --json`, which is a candidate substrate for it. That
is a separate spec.

## 5. Verification

```verify:cli
cargo build --release --locked
cargo test -p spec-spine-core --test kit_skills --locked
cargo test -p spec-spine-core --test kit_hooks --locked
test "$(ls kit/.claude/skills | wc -l | tr -d ' ')" = 15
diff -r kit/.claude/skills .claude/skills
grep -q "spec-spine verify" .claude/skills/verify/SKILL.md
grep -q "index coverage --fail-on-untraced" AGENTS.md
grep -q "fail-on-unresolved" .github/workflows/ci.yml
grep -q "index diagnostics" AGENTS.md
test -f .claude/settings.json
target/release/spec-spine verify 044 --plan
target/release/spec-spine index check --fail-on-unresolved
target/release/spec-spine index coverage --fail-on-untraced
```

## 6. Resolved decisions

**2026-09-07: the scripts are deprecated, not deleted.** Spec 049 4 deferred
retirement until the verb shipped. It shipped in v0.15.0 on the day this spec was
filed, and all four adopters are still behind it. Deleting the kit copy now
would strand them, and deleting this repository's copy would break spec 048's
declared acceptance, which runs it twice. Rejected: deleting both and editing
048's Verification block to match, which is amending an approved spec to suit
code written after it.

**2026-09-07: the hooks resolve the in-tree binary first.** Rejected: leaving the
bare name and putting the right binary on `PATH`. That makes correctness depend
on an untracked machine setting, and the machine that motivated this spec had
`spec-spine` on `PATH` at a version one release behind the checkout, which is
precisely the failure the order prevents.

**2026-09-07: the gate list keeps the local order, not CI's.** The draft of 3.3
required CI's order. That is wrong: `compile` and `index` write, and a local
session runs them before the checks, while CI runs `compile --check` precisely
because it must not repair the tree it judges. The requirement is the complete
set, not the sequence. Recorded rather than silently narrowed.

**2026-09-07: the skill/AGENTS.md gate assertion is a subset, not equality.**
Equality would force every skill to restate the full list including the stack's
build and tests, which is not what a read-only review skill should run. Subset
catches the drift that actually occurred (an authority omitting a step its
skills run) without dictating each skill's scope.

**2026-09-07: the gate chain has one definition in the test.** Second-round
review noted that the CI scanner and the parse guard each carried the same
hardcoded verb list, so a verb added to the chain in one place and not the other
would make the subset assertion vacuous for that verb without failing. Both now
read one `GOVERNANCE_VERBS` const. The list stays closed on purpose: the
invariant is about the gate chain `standards/spec/contract.md` defines, not
about every verb CI happens to run.

**2026-09-07: `[ -n "$sc" ]` stays after a successful resolve.** The same review
observed that `spec_spine_bin` returns 0 only when it echoes a non-empty path,
so the extra test in `SessionStart` is vacuously true. Kept deliberately: it
costs nothing and holds if the resolver's contract ever changes, which is the
same reason the other three hooks test it.

**2026-09-07: `.github/workflows/ci.yml` needs no ownership edge.** Review of
this change read the CI edit as an uncovered path and asked for an `extends`
edge. `.github/` is on the coupling gate's built-in bypass floor
(`couple.rs::DEFAULT_BYPASS_PREFIXES`), so a workflow edit raises no `C-001`
unless a spec specifically claims that path, which would then override the floor
(spec 030). Claiming it would create the refusal, not prevent one. `couple`
reports 14 paths checked and no drift on this change, locally and in CI.

**2026-09-07: the gate-list parser asserts what it parsed.** The same review
noted that `find` takes the first match, so a fenced example between the anchor
and the gate list would be read as the gate list and every assertion below would
go vacuous without failing. Correct: the test now requires every extracted line
to be a governance verb, and a decoy fence makes it fail with the line it
misread.

**2026-09-07: `.claude/settings.json` is committed and matches the kit on hooks
and deny, but not on allow.** The permission allow list is machine-local
convenience and varies by contributor; the hooks and the destructive-command
refusals are governance and must not.
