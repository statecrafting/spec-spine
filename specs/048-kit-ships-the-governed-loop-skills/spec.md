---
id: "048-kit-ships-the-governed-loop-skills"
title: "The kit ships one skill set for the governed loop, and this repository runs it"
status: draft
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "029-claude-code-skill-kit"
  - "038-registry-plan-ready-set"
  - "046-kit-hooks-read-never-write"
  - "047-harness-rules-name-the-legitimate-edits"
amends:
  # 029 3.1 enumerates ten skills and 029 3.3 excludes a shepherd skill as
  # project-specific. This spec ships fifteen, including a shepherd, and
  # changes the text of every one of the ten. 029's file is unchanged (spec
  # 040).
  - "029-claude-code-skill-kit"
establishes:
  - "kit/scripts/"
  - "scripts/verify-spec.sh"
  - { kind: directory, path: ".claude/skills/" }
  - { kind: directory, path: ".claude/agents/" }
  - "crates/spec-spine-core/tests/kit_skills.rs"
extends:
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/skills/", nature: superseding }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/agents/", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/README.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/settings.json", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "AGENTS.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Four adopters of the kit had each written the same five skills the kit did
  not ship (next, build, verify, spec, shepherd), three of them from one
  shared template, and every adopter's copy of the ten the kit did ship had
  drifted from the kit's, which had itself gone stale: a renamed tool, a
  rule frontmatter format Claude Code does not read, a "read-only" review
  that ran the writing compile, a commit skill missing the two house rules
  that land in public history, and a drift remedy that contradicted the
  refusal rule spec 047 had just fixed. This spec makes the kit ship one
  repository-invariant set of fifteen skills built around the orchestrated
  loop (spec-spine's registry plan and the coupling gate on one side, an
  orchestrator's build, ship, shepherd, and verify stages on the other),
  moves every project-specific fact into AGENTS.md and the path-scoped
  rules so the skills copy byte for byte, ships the verify script the loop
  needs, ports the substrate-level agent improvements the adopters made,
  and makes this repository run the same fifteen on itself, pinned by a
  test.
---

# 048: The kit ships one skill set for the governed loop, and this repository runs it

## 1. Purpose

### 1.1 Five skills, written four times

The adopter audit (`docs/design/03-adopter-audit-2026-09.md`, kit backlog
item 9) found that every adopter rewriting the kit had written the same
five loop skills the kit lacked: `/next` (which spec now), `/build` (one
spec, one session), `/verify` (the spec's `## Verification` block, run the
way the post-merge verify stage runs it), `/spec` (author the next one,
born `draft`), and `/shepherd` (watch, remediate, merge, confirm on disk).
Three adopters carried a byte-identical `next`, `verify`, and `shepherd`
and differed only in project nouns elsewhere. That is a template the kit
should have shipped.

### 1.2 Ten skills, ten drifts

The ten skills the kit did ship had drifted in both directions. Adopters
tightened them (owning-spec lookup, `C-001` versus `C-002` remedies, the
spec 047 legitimate-edit test, `git fetch` before `couple`, the
standing-authorization clause an orchestrator's ship stage relies on) and
the kit never received the tightening. Meanwhile the kit's own copies had
gone stale: `cleanup` and `research` named a `Task` tool that is now
`Agent`; `refactor-claude-md` documented `globs:` and `imports:`
frontmatter that Claude Code does not read (every real rule file in every
checkout uses `paths:`); `code-review` and `implement-plan` ran a bare
`compile` inside a read-only review, which repairs a stale committed tree
as a side effect of reading it, the trap spec 031 exists to close;
`commit` omitted the session-link and em-dash bans that land in public git
history; and `ship`'s drift remedy said to edit the owning spec's edges,
which is the move spec 047's refusal rule forbids when the owner is
another spec.

### 1.3 Why the copies diverge

Every adopter customized the skills in place: `make spine` here, a golden
vector there, one repository's crate list in another repository's
`cleanup`. Two of the three template adopters carried dangling references
to a third's rule files. The per-project layer turned out to be small and
regular (the binary invocation, the version pin, the gate command list,
the stack gate, an invariants rule, a never-touch artefact), and it
already has a home: `AGENTS.md`, which every skill reads first, and the
path-scoped rules, which load on touch. A skill that cites "the gate as
`AGENTS.md` lists it" needs no edit per project, so the kit's copy and
every adopter's copy can be the same bytes, and a kit update is a copy,
not a merge.

### 1.4 The loop the skills serve

claude-observatory's orchestrator drives one spec per Claude Code session
through build, ship, shepherd, and verify. It injects the target's
`## Working the backlog` section verbatim into the build prompt, grants
standing authorization to push and open a PR but never to write a waiver,
judges completion by re-running the target's gate, and re-runs the spec's
`verify:cli` fences after merge in a clean checkout. The five loop skills
are the human-driven mirror of those stages, and the kit's `AGENTS.md`
already carries the seven-step protocol (spec 047). What was missing was
the skills that sequence it, the script that runs the verify fences, and
the wiring between them.

## 2. Territory

- **`kit/.claude/skills/`** (superseding `extends` on spec 029's `kit/`):
  fifteen `SKILL.md` files, the ten rewritten and the five added.
- **`kit/.claude/agents/`** (additive): the four agents gain the
  substrate-level improvements the adopters made.
- **`kit/AGENTS.md`**, **`kit/README.md`**, **`kit/settings.json`**
  (additive): the commands list, the install and "govern the harness"
  guidance, and the permission allow-list the new skills need.
- **`kit/scripts/`** (this spec's): `verify-spec.sh`.
- **`.claude/skills/`**, **`.claude/agents/`**, **`scripts/verify-spec.sh`**
  (this spec's): this repository's own copies, the dogfood.
- **`AGENTS.md`** (additive on 029's edge, as spec 047 did): this
  repository's commands list and backlog steps name the skills.
- **`crates/spec-spine-core/tests/kit_skills.rs`** (this spec's): the pin.

The three floor rules stay with spec 047. `kit/settings.json`'s hooks stay
with spec 046; only its permission lists change here.

## 3. Behavior

### 3.1 The fifteen

The kit MUST ship exactly these skills under `kit/.claude/skills/`, the
loop in the order "Working the backlog" runs it, then the support set:

| Skill | Role | Wraps |
|---|---|---|
| `init` | execute the `AGENTS.md` protocol; reads only | `compile --check`, `index check`, `registry plan` |
| `setup` | install the pinned binary, verify the loop once | the gate as `AGENTS.md` lists it, `registry plan` |
| `next` | the next work order, read-only | `registry plan --json`, `registry show --json` |
| `build <id>` | one spec, start to finish | steps 2 to 6 of the protocol |
| `verify <id>` | the spec's `verify:cli` fences, locally | `scripts/verify-spec.sh` |
| `ship` | gate, review, commit, PR | the gate, `code-review`, `commit`, `gh pr create` |
| `shepherd` | checks by head sha, remediation, merge, on-disk confirmation | `gh pr checks`, `gh pr merge`, `git pull --ff-only` |
| `spec` | author the next spec, born `draft` | `registry list --ids-only`, `compile --repo`, `lint --fail-on-warn` |
| `commit` | conventional commit, spec ordinal as scope | `git commit` |
| `code-review` | correctness, drift, mid-build edit legitimacy | `compile --check`, `index check`, `couple`, `registry show` |
| `validate-and-fix` | the CI composite, fixed by severity | the gate as `AGENTS.md` lists it |
| `cleanup` | dead code and duplicates, ownership-aware | one `explorer` agent, `index coverage` |
| `implement-plan` | a cross-cutting plan file with checkpoints | the gate when a spec-owned path changes |
| `research` | parallel sub-agents, corpus reads typed | `registry show`, `index render`, `index coverage` |
| `refactor-claude-md` | `CLAUDE.md` into `paths:` rules, harness spec coupled | `index coverage`, `compile`, `index` |

`next` MUST apply two rules on top of `registry plan`: a spec whose
`status` is not `approved` is listed as awaiting approval, never offered
(approval is a human act; `plan` schedules by `implementation` and
dependencies alone), and a spec at `implementation: in-progress` is
listed as in flight, never offered as new work. `build` MUST refuse a
draft, an unmet dependency, a dirty tree, and a missing operator
prerequisite in preflight, MUST flip to `in-progress` and commit the flip
with the regenerated shards before any code, MUST record decisions the
spec is silent on as dated entries (and as drop-box records in the
orchestrator's shape when a driven session), and MUST halt on a
contradiction rather than edit the spec. `verify` MUST report
`not-declared` as an honest zero, never as a pass, and MUST report
`verify:browser` blocks as skipped. `shepherd` MUST read checks for the
recorded head sha only, MUST bound remediation to two rounds, MUST stop
on `DIRTY` or `BEHIND` rather than rebase, MUST confirm the merge on the
local default branch, and MUST never treat standing authorization as
covering a `Spec-Drift-Waiver:`.

### 3.2 The frontmatter contract

Every skill MUST declare `name` (equal to its directory), `description`,
and `allowed-tools`; skills that take an argument declare
`argument-hint`. Tool names are the current ones (`Agent`, never `Task`).
Rule files a skill teaches an agent to write use `paths:` frontmatter.

### 3.3 Repository-invariant, with a project layer

Every skill MUST be free of project nouns, composite names, placeholder
build commands, hardcoded temporary directories, and pre-024 artifact
paths, and MUST end with a `## Project layer` section naming what it
reads from `AGENTS.md` (the binary invocation, the version pin, the gate
command list, the stack gate, the default branch), from `spec-spine.toml`
(closed enums, extra keys, `state_dir`), and from the path-scoped rules
(invariants, never-touch artefacts, specialist agents). The kit's copy
and this repository's copy of each skill MUST be byte-identical; the
adopter's copy is meant to be too.

### 3.4 Read skills read

`init`, `next`, `verify`, and `code-review` MUST NOT invoke a writing
`spec-spine` verb (a bare `compile`, a bare `index`). Freshness is read
with `compile --check` and `index check`; a stale verdict is reported, and
the session repairs and commits it as later, visible work.

### 3.5 The verify script

`kit/scripts/verify-spec.sh` runs every non-comment line inside a
```` ```verify:cli ```` fence under a spec's `## Verification` heading,
from the repository root, in order, stopping at the first non-zero exit;
prints `passed`, `FAILED at command N`, or `not-declared`; counts and
skips `verify:browser` blocks; exits 2 on a missing spec; and reads the
spec markdown, never `.derived/`. This repository's `scripts/verify-spec.sh`
is the same file.

### 3.6 The agents

The four kit agents gain, from the adopters' rewrites: the reviewer runs
the gate as evidence and tests every mid-build spec edit against spec 047
3.2's list; the implementer confirms each touched file is in
`establishes` or covered by `extends`, claims new files in the same
change, and reports decisions recorded; the architect names where the
spec is silent (a decision to record) versus wrong (a halt); the explorer
names the owning spec of every file it cites. The kit's performance
section, verification checklist, and edit-versus-write guidance stay.

### 3.7 The wiring

`kit/AGENTS.md` "Working the backlog" names `/next` at step 1, `/build` at
step 2, `/verify` at step 6, and `/shepherd` at step 7, keeps
`registry plan` as the tool truth beneath `/next`, adds `index coverage`
to the init reads, and lists all fifteen commands. `kit/README.md` lists
the fifteen and the script, states that the hooks read and never write,
says the skills are not customized and why, and shows the harness-spec
shape an adopter uses to claim its own harness (directory units for
`.claude/skills/` and `.claude/agents/`, the hashed-input listing, the
`C-001` acceptance criterion). `kit/settings.json` allows the read verbs
the new skills use (`git fetch`, `git switch`, `gh pr view|checks|list`,
`gh run`, `scripts/*`) and denies the destructive forms the audit found
one adopter denying.

### 3.8 The test

`crates/spec-spine-core/tests/kit_skills.rs` reads the shipped kit and
this repository's harness from disk and asserts 3.1 (the set, and the
verbs each loop skill wraps), 3.2 (the frontmatter), 3.3 (the project
layer, the banned strings, byte identity), 3.4 (a verb scanner over the
read skills), 3.5 (the script ships and is the one this repository runs),
and 3.6 (the reviewer and implementer sentences).

## 4. Out of scope

- A `/burndown` skill (the per-unit remainder inside a spec, from `W-001`
  warnings). One adopter has it; it greps `index render` output, which is
  a typed read by spec 047 but not a versioned contract. It waits on a
  tool verb (`index warnings --json`, tool backlog item 2).
- `spec-spine verify <id>` as a tool verb (tool backlog item 1). The
  script is the protocol until the verb exists; when it does, `/verify`
  wraps the verb and the script retires.
- Propagating the set to the adopters. Each adopter takes the fifteen by
  copy in its own session, keeps its project layer in `AGENTS.md` and its
  rules, and drops its local copies of the five; that is adopter-side
  work, recorded per repository.
- The hooks (spec 046) and the three rules (spec 047).
- Generating the kit from `spec-spine init` (029 4).

## 5. Verification

```verify:cli
cargo test -p spec-spine-core --test kit_skills --locked
cargo test -p spec-spine-core --test kit_hooks --locked
sh -n kit/scripts/verify-spec.sh
scripts/verify-spec.sh 046-kit-hooks-read-never-write
test "$(ls kit/.claude/skills | wc -l | tr -d ' ')" = 15
diff -r kit/.claude/skills .claude/skills
```

## 6. Resolved decisions

- **D-1 (2026-09-06).** The project layer lives in `AGENTS.md` and the
  path-scoped rules, not in per-skill placeholders. Alternative rejected:
  bracketed placeholders scattered through each skill body (the 029
  approach), which is what produced fifteen divergent copies and dangling
  cross-adopter references.
- **D-2 (2026-09-06).** `/next` wraps `registry plan` and adds the
  approval and in-flight rules rather than reimplementing readiness in
  Python. Alternative rejected: the adopters' `registry list --json` plus
  script, which computes the same graph a second time and disagrees with
  `plan` on drafts.
- **D-3 (2026-09-06).** Decision drop-box records use the orchestrator's
  id convention, `<specId>-d<n>`, since the orchestrator seals them.
  Alternative rejected: the adopters' `<spec-id>/D-n`, which also
  validates but names a convention the sealer does not.
- **D-4 (2026-09-06).** The verify protocol ships as a script the skill
  calls, not inline in the skill, so an orchestrator and a human run the
  same file. Alternative rejected: inlining the awk loop in `SKILL.md`,
  which no CI job can call.
- **D-5 (2026-09-06).** `spec-new` (one adopter's variant) retires in
  favour of `spec`, taking its ordinal-from-registry rule, its
  enum-from-`spec-spine.toml` rule, and its frontmatter checkpoint.
