---
id: skills
title: Skills Reference
sidebar_position: 5
---

# Skills Reference

The kit ships 15 skills under `kit/.claude/skills/`. Each is a `SKILL.md` in
its own folder. None of them is customized: every skill is
repository-invariant and ends with a `## Project layer` section naming what
it reads from `AGENTS.md` (the binary invocation, the version pin, the gate
command list, the stack gate, the default branch), from `spec-spine.toml`
(closed enums, extra keys, `state_dir`), and from the path-scoped rules
(invariants, never-touch artefacts, specialist agents). Keep the project
layer there and the skills stay byte-identical to the kit, so a kit update
is a copy, not a merge. spec-spine's own repository runs the same fifteen
on itself and pins them byte-identical to the kit with a test (spec 048).

## The loop

In the order `AGENTS.md` "Working the backlog" runs it. These mirror the
stages an orchestrator drives one spec per session through (build, ship,
shepherd, verify), so a human-driven session and a driven one follow the
same protocol.

| Skill | What it does | Wraps |
|---|---|---|
| `init` | Run the `AGENTS.md` New Sessions protocol; reads only, never repairs the tree. | `compile --check`, `index check`, `registry plan` |
| `setup` | One-time setup: install the pinned spec-spine, the stack toolchain, verify the loop once. | the gate as `AGENTS.md` lists it |
| `next` | Name the next work order from the ready set, minus drafts, with in-flight specs and blockers reported. Read-only. | `registry plan --json`, `registry show --json` |
| `build <id>` | One spec, start to finish: preflight, branch, flip `in-progress`, implement inside the territory, gate before every commit, verify, flip `complete`. | steps 2 to 6 of the protocol |
| `verify <id>` | Run the spec's `verify:cli` fences locally, the way a verify stage runs them after merge. `not-declared` is an honest zero, not a pass. | `scripts/verify-spec.sh` |
| `ship` | Gate, review, commit on the feature branch, open the PR. The waiver is a human checkpoint standing authorization never covers. | the gate, `/code-review`, `/commit`, `gh pr create` |
| `shepherd` | Watch the PR's checks by head sha, answer review threads, remediate through the gate (two rounds), merge with squash, confirm the merge on disk. | `gh pr checks`, `gh pr merge`, `git pull --ff-only` |
| `spec` | Author a new spec at the next free ordinal, born `draft`, validated in a temporary copy. Approval stays a human flip. | `registry list --ids-only`, `compile --repo`, `lint --fail-on-warn` |

Two rules `/next` applies on top of `registry plan`: a `draft` whose
dependencies are met is listed as awaiting approval, never offered
(approval is a human act), and a spec at `implementation: in-progress` is
listed as in flight, never offered as new work.

## The support set

| Skill | What it does |
|---|---|
| `commit` | Conventional, impact-focused commit with the spec ordinal as scope; the regenerated shards staged with the change; no AI attribution, no session links, no em dash. |
| `code-review` | Correctness, spec drift, and the legitimacy of every mid-build spec edit, with the gate's read-only forms as evidence and the path-scoped invariant rules applied or delegated. |
| `validate-and-fix` | Run the CI composite `AGENTS.md` names and fix by severity; a `C-002` whose remedy is claiming the file is HIGH, one that needs another spec edited is CRITICAL and goes to a human. |
| `cleanup` | One read-only analyzer agent runs dead-code and duplicate detectors per language and reports with an owning-spec column. |
| `implement-plan` | Execute a cross-cutting plan file with checkpoints; for one whole spec, prefer `/build`. |
| `research` | Parallel sub-agents; corpus questions go through `spec-spine registry` and `index`, external questions through the web. |
| `refactor-claude-md` | Extract context-specific guidance from `CLAUDE.md` into `paths:`-scoped rules, keeping the harness spec coupled and the index fresh. |

## Read skills read

`init`, `next`, `verify`, and `code-review` never run a writing verb: a bare
`compile` inside a review would repair a stale committed registry as a side
effect of reading it, hiding the defect. They use `compile --check` and
`index check`, report a stale verdict, and leave the repair to the session
as later, committed work.

## The verify script

`kit/scripts/verify-spec.sh` runs every non-comment line inside a
```` ```verify:cli ```` fence under a spec's `## Verification` heading, from
the repository root, in order, stopping at the first non-zero exit. It
prints `passed`, `FAILED at command N`, or `not-declared`, counts and skips
`verify:browser` blocks, and reads the spec markdown, never `.derived/`.
Copy it to your repository's `scripts/`; `/verify` calls it there.

## Not shipped

Project-specific pieces: a domain-specialist agent, invariant rules with
`paths:` frontmatter, a `build-commands.md` rule naming your composite, a
`Makefile`, a CI workflow. The gate command list in `AGENTS.md` is the
contract the skills read; see [Adopt in your repo](./adopt-in-your-repo.md).
