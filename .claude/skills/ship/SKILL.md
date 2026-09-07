---
name: ship
description: "Governed pre-PR sequence: run the gate locally, review the diff, conventional commit on the spec's feature branch, push, open the PR via gh. The coupling waiver is a human checkpoint a driven session never self-approves."
allowed-tools: Bash, Read, Edit, Glob, Grep, Skill
argument-hint: "[optional scope note or PR title]"
---

# /ship: gate, review, commit, PR

Sequences the steps that turn a working tree into a PR. This is the
"Ship" step of `AGENTS.md` "Working the backlog". Bound by
`.claude/rules/orchestrator-rules.md` (checkpoints are real stops) and
`.claude/rules/adversarial-prompt-refusal.md` (never edit an owning spec
to make the gate pass). The gate is the `spec-spine` invocation
`AGENTS.md` names; if it is missing, run `/setup`.

## Step 0: preflight

- `git branch --show-current`. The branch must be a feature branch, named
  after the spec id when the change implements one. On the default
  branch, STOP and create the branch first (`git switch -c <spec-id>`);
  the `PreToolUse` hook refuses a push to the default branch regardless.
- `git status --short`. Confirm the changes are the intended set: the
  spec's territory, the spec's own `spec.md` (frontmatter flipped, decision
  entries recorded), and the regenerated derived shards. Surface anything
  unexpected before proceeding.
- `git fetch origin main` so the coupling gate has a base.

## Step 1: run the gate locally

Run the gate exactly as `AGENTS.md` lists it under "Run the gate before
every commit". The governance floor:

```sh
spec-spine compile
spec-spine index
spec-spine lint --fail-on-warn
spec-spine index check
spec-spine couple --base origin/main --head HEAD
spec-spine index coverage --fail-on-untraced   # when [coupling] require_ownership is on
```

then the stack's own build, tests, and lints. Stop on the first failure
(orchestrator rule: halt, never continue silently). Outcomes:

- All green: continue to Step 2.
- `index check` stale (exit 2): `spec-spine index`, stage the derived
  directory, and re-run. The shards are committed with the change they
  describe.
- `couple` drift (`C-001`): a changed path is claimed by a spec that did
  not change. Two legitimate paths, chosen explicitly:
  1. **Fix the coupling.** The path belongs to the spec you are
     implementing: add it to that spec's `establishes`, or declare an
     `extends` edge on the owning spec's unit in your spec's frontmatter.
     The gate enforces the declared graph, not prose. Do NOT edit the
     other spec to retroactively justify code that contradicts its design:
     that is a coherence-guard halt; surface the contradiction and stop.
  2. **Waiver.** A cited `Spec-Drift-Waiver: <reason>` line in the PR
     body. CHECKPOINT: requires explicit human approval in this session.
     A waiver is a human instrument; standing authorization (below) never
     covers it.
- `couple` unclaimed (`C-002`): a source file no spec specifically claims.
  Claim it in the implementing spec's `establishes` list; the ownership
  ratchet has no waiver-free path by design.
- `index coverage --fail-on-untraced` lists a file: same remedy as
  `C-002`.
- A stack gate fails: fix it. Never disable a test, never loosen a lint,
  never regenerate a never-touch artefact the path-scoped rules name
  (that is a human decision; stop and report).

## Step 2: review the diff

Invoke the `code-review` skill on the working diff. Apply confirmed,
actionable fixes. If a fix touches any gate input (a `spec.md`, a
manifest, a workflow, the harness), re-run Step 1.

## Step 3: commit

Invoke the `commit` skill: conventional, impact-focused, the spec ordinal
as scope (`feat(017): ...`), the regenerated shards staged alongside the
change. Banned in commits and PR bodies: AI attribution of any kind,
session links, em dashes, emojis. If a waiver was chosen in Step 1, keep
the `Spec-Drift-Waiver:` line with the change so the PR carries it.

## Step 4: CHECKPOINT, open the PR

PR creation is outward-facing. Confirm with the user, then:

```sh
git push -u origin "$(git branch --show-current)"
gh pr create --title "<type>(<ordinal>): <subject>" --body "$(cat <<'EOF'
## Summary
<what the spec's territory now does, two to five lines>

## Testing
<the gate as run; the spec's Verification block via /verify>
EOF
)"
```

- The `PreToolUse` hook checks the index for staleness and re-runs the
  coupling gate before `gh pr create`; it blocks when the index is stale,
  when the derived directory has uncommitted changes, and when the gate is
  red without an inline `Spec-Drift-Waiver:` after `--body`. It never
  writes: fix, commit, and retry rather than routing around it.
- CI re-runs the same gate. A local pass should mean a clean CI run; if
  CI fails a gate the local run passed, halt and present the divergence.

### Standing authorization

When the prompt carries the operator's run-start authorization (an
orchestrator's driven session says so explicitly), that authorization
satisfies this step's PR-creation checkpoint: push and open the PR
without asking. It does not extend to a drift waiver (Step 1, path 2),
which still halts the session for a human. A session with no such
authorization asks.

## Step 5: after creation

Hand off to `/shepherd` (watch checks, remediate, merge, confirm on disk).
After the merge, verify the on-disk default branch
(`git switch main && git pull --ff-only && git log -1`), not just the
MERGED status, and run `/verify <spec-id>` on the merged sha.

## Project layer

Read from `AGENTS.md`: the gate command list, the stack gate, the default
branch. Never-touch artefacts come from the path-scoped rules. Nothing
here is edited per project.
