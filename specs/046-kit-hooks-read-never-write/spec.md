---
id: "046-kit-hooks-read-never-write"
title: "The kit's hooks observe the tree; they do not repair it"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "029-claude-code-skill-kit"
  - "031-registry-freshness-check"
amends:
  # 029 3 describes `kit/settings.json` as "the Claude Code hooks template"
  # and the website's configuration page describes each hook as recompiling
  # or regenerating. This spec changes what three of the four hooks do. 029's
  # text is unchanged (spec 040).
  - "029-claude-code-skill-kit"
establishes:
  - "crates/spec-spine-core/tests/kit_hooks.rs"
extends:
  - { spec: "029-claude-code-skill-kit", unit: "kit/settings.json", nature: superseding }
references:
  - { unit: { kind: file, path: "kit/AGENTS.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Three of the four Claude Code hooks the kit ships write into the tree they
  are meant to observe. `SessionStart` runs a bare `compile`, which repairs a
  stale committed registry as a side effect of checking it; the `PreToolUse`
  PR gate runs `index` and then blocks on the uncommitted output of its own
  write, which a hook cannot commit; `Stop` regenerates a stale index and asks
  a session that has already ended to commit it. All four also bind to the
  session's project directory rather than the repository the action targets,
  so a multi-checkout session judges one repo by another's state. An adopter
  running these hooks under an orchestrator stalled for eleven hours on a tree
  the `Stop` hook had dirtied, because the orchestrator refuses to start a
  session on a dirty tree and only a session cleans one. That adopter fixed
  every one of these in its own copy; this spec ports the fixes into the kit,
  adds the push gate the kit's prose rule had no enforcement for, and pins the
  read-only property with a test that scans every hook body for a writing
  subcommand.
---

# 046: The kit's hooks observe the tree; they do not repair it

## 1. Purpose

### 1.1 The kit contradicted itself

`kit/AGENTS.md`, under **Registry freshness**, is emphatic:

> Do **not** substitute a plain `spec-spine compile` here. Writing repairs the
> tree as a side effect of reading it, which hides the fact that the *committed*
> copy was stale: the drift then looks like an uncommitted local edit instead of
> a defect on the branch. `/init` reports; it does not silently mutate.

`kit/settings.json`, in the same directory, ran `spec-spine compile` on every
`SessionStart`. The protocol and the hook that fires before it disagreed about
the one thing the protocol argues hardest about, and every adopter that copied
the kit copied the disagreement. Of the four adopters audited in
`docs/design/03-adopter-audit-2026-09.md`, three rewrote the hook to
`compile --check`; the fourth still runs the bare `compile` twice, once in the
hook and once as the first step of its own init protocol.

### 1.2 A hook cannot commit, so a hook must not write

The `PreToolUse` gate on `gh pr create` ran `spec-spine index` and then refused
the PR if `.derived/` was dirty. Its own write is what made `.derived/` dirty,
and nothing inside a `PreToolUse` hook can commit. The block was unrecoverable
from where it was raised, and it mutated a checkout that another session might
have been mid-build in.

The `Stop` hook was the same defect at a worse moment. It regenerated a stale
index and printed "review `git diff .derived/` and include it in your commit"
to a session that had already stopped. Nothing committed the result, so every
session that ended with a stale index left the tree dirty. rahi's spec 001 D-6
records what that did under claude-observatory:

> claude-observatory's build stage refuses to start on an unclean tree, a
> refusal pauses the run, and a paused run never resumes without a human.
> After spec 015 merged and verified, standby looped "driving rahi, paused,
> flight slot released, idle, rescan" every sixty seconds for eleven hours
> with the daemon healthy.

Only a session or a human cleans a tree, and a session only starts on a clean
tree. A hook that dirties the tree at session end therefore stalls the pipeline
on dirt it produced itself, and the stall has no mechanical exit.

### 1.3 The hooks judged the wrong repository

All four hooks began with `cd "${CLAUDE_PROJECT_DIR:-.}"`. That binds them to
the session's project, not to the repository the action names. In a session
with sibling checkouts open, an edit under another repo recompiled and
staleness-checked this one, and a `git push` or `gh pr create` aimed at a
sibling was judged against this repo's branch and coupling state. rahi's spec
001 D-5 records the fix, which this spec adopts.

### 1.4 The rule with no enforcement

The kit's protocol says to work on a feature branch and never commit to main.
Nothing enforced it. Every adopter that rewrote the hooks added a push gate.

## 2. Territory

`kit/settings.json` (spec 029's, superseded in content here) and a new test
file, `crates/spec-spine-core/tests/kit_hooks.rs`, which reads the shipped kit
file at compile time and asserts the properties below. No library code
changes. The website's Claude Code configuration page describes the hooks and
is updated to match; it is on the documentation bypass floor.

## 3. Behavior

### 3.1 Hooks read

No hook in `kit/settings.json` MAY run a `spec-spine` subcommand that writes
into the repository, with the single exception in 3.2. The read verbs the hooks
need are `compile --check` (spec 031), `index check`, and `couple`. `compile`
and `index` without their check forms write committed shards and are refused.

`SessionStart` runs `compile --check` and decodes its exit code (0 fresh, 2
stale, 1 invalid, anything else unknown) into the freshness line, alongside
`index check`. `Stop` runs `index check` and, when stale, prints the command to
run and the reason it was not run here. The `PreToolUse` PR gate runs
`index check` first and blocks with the fix instruction if stale, then refuses
uncommitted `.derived/`, then runs `couple`.

### 3.2 The one sanctioned write

`PostToolUse` on `Edit|Write` of a `specs/*/spec.md` file MAY run `compile`.
The actor is a live session that has just edited a spec and can commit the
recompiled shards with that edit; this is the case that keeps `compile --check`
green in CI without a separate step, and it is the only context in which the
hook and the committer are the same party. The hook body says so in a comment,
because the exception is the kind that gets copied without its reason.

### 3.3 Action hooks act on the repository the action targets

`PostToolUse` derives the repository root from the edited file
(`git -C "$(dirname "$fp")" rev-parse --show-toplevel`). `PreToolUse` reads
the hook payload once, honours an explicit `cd <dir>` prefix in the command,
falls back to the hook's own `cwd`, and asks git for the toplevel. Both pass the
result to `spec-spine --repo "$root"`, and both exit quietly (the PR gate with
a one-line explanation) when `$root/specs` does not exist, because the target
is then not a spec-spine corpus. Neither MAY reference `CLAUDE_PROJECT_DIR`.

`SessionStart` and `Stop` are session events with no target other than the
project, and keep the project directory; they gain the `specs` guard.

### 3.4 The push gate

`PreToolUse` refuses any `git push` whose refspec names `main` (`origin main`,
`HEAD:main`, `:main`, `origin +main`) or whose current branch is `main`, with
exit 2 and a message naming the repository and branch. It runs before the PR
gate and independently of it. `git push --force*` and `git push -f *` join the
permission deny list.

### 3.5 A hook that skips says so

Every hook that cannot do its job (no `spec-spine` on `PATH`, no `jq`, target
not a corpus) prints one line saying what it skipped and why, instead of
exiting quietly. A silent hook is indistinguishable from a passing one.

### 3.6 The waiver is a human instrument

The PR gate's refusal message says so: a `Spec-Drift-Waiver` may be included
only with explicit human approval, never on the agent's own authority. All four
audited adopters forbid the machine from writing one; none has ever used one.
The kit's message is where an agent reads the rule at the moment it matters.

### 3.7 The test

`tests/kit_hooks.rs` includes `kit/settings.json` at compile time and asserts:
all four events are present; every hook body contains at least one recognised
`spec-spine` call (so the scanner is not passing vacuously); no invocation
other than 3.2's is a write; the action hooks derive their root from the
action and never name `CLAUDE_PROJECT_DIR`; the push gate matches every listed
refspec form and the current branch; every hook names its skip condition; and
the scanner itself recognises `index`, `compile` and `--repo`-prefixed forms as
writes. Against the kit as it stood before this spec, four of the seven tests
fail.

## 4. Out of scope

**Dogfooding the hooks in this repository.** spec-spine has no
`.claude/settings.json`, which is how three hooks shipped writing when they
should read. Installing them here changes every contributor's session and is a
decision for its own change; 3.7 is the minimum that keeps the defect from
returning silently.

**A hook for agents without hooks.** Codex CLI has no `PreToolUse`. The
coherence guard as a real gate is spec 043's wave 4 and stays there.

**The `PostToolUse` glob list.** It is extended with `AGENTS.md`, `CLAUDE.md`,
`Makefile` and `docs/design/*` to match what adopters hash, and it remains an
adopter-tuned list as `kit/README.md` says.

## 5. Verification

- `tests/kit_hooks.rs` passes against the shipped kit and fails against the
  kit at the parent commit.
- Each hook body passes `sh -n`.
- Run in this checkout with a stale committed registry and index, every hook
  reports staleness and `git status` shows no change under `.derived/`.
- The push gate exits 2 on `git push origin main` and 0 on a feature branch
  push; the PR gate skips with a message when the command's `cd` names a
  directory with no `specs/`.
