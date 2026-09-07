---
name: code-review
description: "Review the current diff for correctness bugs, spec drift, and illegitimate mid-build spec edits, with the governed gate as evidence, and emit an evidence-oriented findings list."
allowed-tools: Read, Grep, Glob, Agent, Bash(git status:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git rev-parse:*), Bash(git fetch:*), Bash(spec-spine:*), Bash(make:*), Bash(grep:*)
argument-hint: "[scope] - e.g. \"branch\", \"working tree\", \"<a directory>\""
---

# /code-review: correctness, spec drift, invariants

Reviews the current diff against three questions: does the change have
correctness or edge-case bugs, does it still match its owning spec's
contract, and does it hold the invariants the project's path-scoped rules
name. Output is an evidence-oriented findings list, each line citing
`file:line`. Nothing authored is modified. The gate's read-only forms
(`compile --check`, `index check`) are used so the review never dirties
the tree; a stale verdict is itself a finding.

## Step 0: scope the diff

```sh
git fetch origin main
git status --short && git diff --stat && git log --oneline -10
git diff origin/main...HEAD --stat    # committed delta
git diff HEAD --stat                  # uncommitted delta
git diff origin/main...HEAD --name-only; git diff HEAD --name-only
```

Note which classes changed: source, specs (`specs/**/spec.md`), standards
(`standards/**`), the harness (`.claude/**`, `AGENTS.md`, `CLAUDE.md`,
workflows), scripts, docs, derived shards.

## Step 1: the gate stays green

```sh
spec-spine compile --check                      # exit 2: committed registry shards are stale
spec-spine index check                          # exit 2: committed index shards are stale
spec-spine lint --fail-on-warn
spec-spine couple --base origin/main --head HEAD
spec-spine index coverage                       # ownership: unclaimed and floor-only files
```

then the stack's own build, tests, and lints as `AGENTS.md` lists them.

- A `couple` failure is the headline finding: cite the file the gate
  named and the owning spec whose declared edges fail to cover it.
- An unclaimed file from `coverage` is a finding against the implementing
  spec's `establishes`.
- A `lint` or freshness failure is a corpus finding: cite the diagnostic
  verbatim. Stale shards mean the change forgot to run `compile` and
  `index` and commit the result; the fix is to do that, not to hide it.

## Step 2: spec-contract match

For each changed source file, confirm the change is consistent with the
contract of its owning spec rather than only with the gate's mechanical
pass. Governed reads, through the CLI:

```sh
spec-spine registry show <spec-id> --json          # declared surface and edges
spec-spine registry relationships <spec-id>        # its typed neighborhood
```

- Does the code do what the behavior section says and nothing it
  forbids? Cite the requirement label.
- Are the acceptance criteria satisfied verbatim and is `## Verification`
  runnable?
- If a spec was edited: only `establishes` growth, a dated decision entry,
  a dated status note, a new `extends` edge, and the `implementation` flip
  are legitimate mid-build edits. Anything that changes what the spec
  requires is a coherence-guard finding
  (`.claude/rules/adversarial-prompt-refusal.md`), severity CRITICAL.
- Flag drift where code does something the spec's narrative does not
  describe even when `couple` passes (an over-broad edge).

## Step 3: correctness pass

Read the changed source and look for each of the following, with a
`file:line` and a one-sentence evidence claim:

- Logic and edge-case bugs (off-by-one, unhandled empty or error cases,
  boundary values, overflow on untrusted lengths).
- Error-path correctness: the right error variant and the right exit code
  where the project maps them (`spec-spine` uses `0` ok, `1` validation
  or drift, `2` stale, `3` I/O, parse, schema, or config).
- Language hygiene the project's lints enforce (read the stack rule under
  `.claude/rules/`): unsafe operations, panics in library code, public
  boundaries, dependency direction, the manifest metadata that names a
  crate's or package's spec.
- Determinism hazards anywhere: unordered-map iteration reaching output,
  locale- or platform-dependent behavior, unstable ordering in emitted
  JSON, a clock or environment read on a hashed path.
- Hygiene: stray debug prints, commented-out code, dead branches, secrets
  in logs.
- House style in authored text: no em dash (`grep -rn $'\xe2\x80\x94'`
  over the changed files), no session links, no AI attribution.

## Step 4: invariants pass

Decide from the changed paths in Step 0 which path-scoped rules under
`.claude/rules/` apply. For each, either check rule by rule and report
held / violated / not applicable, or delegate to the specialist agent the
rule names (spawn it with the `Agent` tool, giving it the branch, the
base, and the owning spec id) and fold its verdict in. A changed
never-touch artefact (a golden vector, a fixture chain, an evaluation
baseline) is CRITICAL regardless of what the diff says about it: that is
a human decision.

No rule applies: say "not applicable" under `### Invariants`; do not skip
the section silently.

## Step 5: findings report

```
## Review: <scope>
Base: origin/main | Head: <branch> | Files: <n> | +<a>/-<d>
Gate: compile --check <fresh|stale> | index check <fresh|stale> | lint <ok|N> | couple <ok|C-001|C-002> | coverage <n unclaimed> | stack <ok|FAIL>
Owning spec: <id> | Mid-build spec edits: <none|legitimate|coherence-guard finding>

### Findings (severity-ordered)
- [CRITICAL|CORRECTNESS|SPEC-DRIFT|GATE|HYGIENE] <claim> at `file:line`
  Evidence: <one sentence, cited>
  Fix: <specific recommendation>

### Invariants
- <rule or specialist>: <held | violated (folded above by number) | not applicable>

### Clean
- <dimensions checked with nothing found>
```

If nothing is found, say so plainly and report the gate result and the
invariant verdicts as the evidence. To proceed with fixes, the user (or
`/ship`) names the findings to apply.

## Project layer

Read from `AGENTS.md`: the stack gate. Read from `.claude/rules/`: the
stack hygiene rule, the invariant rules, and the specialist agents they
name. Nothing here is edited per project.
