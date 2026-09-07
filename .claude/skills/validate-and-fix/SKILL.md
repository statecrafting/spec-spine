---
name: validate-and-fix
description: "Run the local CI composite (the exact gate CI runs) and fix what it surfaces by severity, with coupling failures that need a spec decision and never-touch artefacts escalated to a human."
allowed-tools: Bash, Read, Edit, Glob, Grep, Agent
---

# /validate-and-fix

Run the local CI loop and fix what it surfaces. The composite `AGENTS.md`
names (commonly `make ci`) is the single source of truth for what CI
validates: if it passes locally, CI passes too. Do not rediscover
validation commands by grepping manifests; read the composite's
definition.

## 1. Run the composite

Run the gate exactly as `AGENTS.md` "Working the backlog" lists it under
"Run the gate before every commit". The governance floor is
`spec-spine compile`, `spec-spine index`, `spec-spine lint --fail-on-warn`,
`spec-spine index check`, `spec-spine couple --base origin/main --head HEAD`,
and `spec-spine index coverage --fail-on-untraced` where ownership is
required; the stack's build, tests, and lints follow. Run
`git fetch origin main` first if the coupling gate cannot find its base.

Capture full output (file paths, line numbers, messages) and categorize:

- **CRITICAL** (human decision, do not fix silently): a coupling failure
  (`C-001` drift, or a `C-002` whose only remedy is editing a spec you
  are not implementing); any change to a never-touch artefact the
  path-scoped rules name (a golden vector, a fixture chain, an evaluation
  baseline); a dependency cycle; an ambient input (clock, environment,
  unordered map) reaching a hashed path.
- **HIGH**: test failures, build breaks, index or registry staleness, a
  `C-002` whose remedy is claiming the file in the spec being implemented
  (the legitimate edit `.claude/rules/adversarial-prompt-refusal.md`
  names).
- **MEDIUM**: `spec-spine lint` warnings (the gate runs `--fail-on-warn`),
  linter findings, type errors, dependency advisories with a fix.
- **LOW**: formatting, wording in prose, minor cleanups.

If a check is missing from the composite, add it to the composite and to
the CI workflow in the same change; never introduce a validation as a
one-off script.

## 2. Fix by phase

- **Phase 1, safe quick wins**: LOW and MEDIUM findings that cannot break
  anything. Verify each by re-running the narrowest target the stack
  offers (a formatter, one lint, one test file).
- **Phase 2, functionality**: HIGH findings one at a time; re-run the
  affected target after each. Never disable or skip a failing test; fix
  the cause. Stale shards: run `compile` and `index`, stage the derived
  directory with the change that made them stale.
- **Phase 3, critical**: present each CRITICAL finding with the evidence
  and a proposed remedy, then wait. Refusing the destructive step is
  sometimes the right answer. A never-touch artefact that would change
  means the encoding or the baseline changed: that is a schema or design
  decision, a spec amendment, and a human decision, in that order.
- **Phase 4, verification**: re-run the composite end to end.

## 3. Error handling

- **Rollback**: `git stash push -m "pre-validate-and-fix"` before any
  change; offer instant rollback if a fix regresses.
- **Partial success**: continue past a fix that fails; separate successes
  from failures; give manual instructions for what you could not fix.
- **Governed reads**: read the derived directory only through `spec-spine`
  subcommands (`.claude/rules/governed-artifact-reads.md`).

## 4. Parallel execution

Launch several agents concurrently only for independent fixes that touch
non-overlapping files; keep ordered or cross-cutting changes sequential.
Each agent verifies its own fix with the narrowest target before
reporting.

## 5. Final verification

Re-run the composite, confirm no new findings, and summarize:
`Fixed X/Y issues, Z require human decision. CI: {PASS|FAIL}`.

## Substrate notes

- `spec-spine lint` runs with `--fail-on-warn`: a warning is a failure.
- The coupling gate compares `HEAD` against `origin/main`; fetch first.
- The codebase index hashes more than `spec.md`: `spec-spine.toml
  [index] extra_hashed_inputs` lists the harness, design docs, workflows,
  and standards. Editing any of them without regenerating the index fails
  the staleness check. The hooks only report staleness; they never
  regenerate. The session runs `spec-spine index` and commits the result.
- `.claude/settings.json` and `.mcp.json` are hashed byte for byte when
  listed as hashed inputs: editor reformatting trips the gate even when
  the JSON is unchanged.
- With `[coupling] require_ownership` on, every source file must be
  claimed by a spec (`spec-spine index coverage` shows the debt, always
  zero on a green tree).

## Project layer

Read from `AGENTS.md`: the composite and the stack gate. Read from
`.claude/rules/`: the never-touch artefacts and any post-feature checklist
the project keeps. Nothing here is edited per project.
