---
name: verify
description: "Run one spec's verify:cli blocks locally through scripts/verify-spec.sh, the same protocol an orchestrator's verify stage runs after merge, and report every command's exit code honestly."
allowed-tools: Bash, Read, Grep
argument-hint: "<spec-id>"
---

# /verify <spec-id>: the spec's Verification block, locally

Runs the `## Verification` section of `specs/<id>/spec.md` exactly the way
an orchestrator's verify stage will run it after merge: every non-comment
line inside a ```` ```verify:cli ```` fence, from the repo root, in order,
stopping at the first non-zero exit. This is the local rehearsal of the
"Satisfy the spec's acceptance criteria verbatim" step of `AGENTS.md`,
"Working the backlog".

## Step 0: scope

- The argument is the full spec id. Without one, stop and ask; do not
  guess from the branch name.
- `git status --porcelain`: the post-merge run happens in a clean checkout
  of the merged sha, so a pass that depends on an uncommitted file is not
  a pass. Warn when the tree is dirty and say which files.
- Read the spec's acceptance criteria and its `## Verification` section
  once, so the report can say which criterion each command exercises.

## Step 1: run

```sh
scripts/verify-spec.sh <spec-id>
```

The script prints `[verify] $ <command>` before each command and
`[verify] exit <code>` after it, then one of:

- `verify: <id>: passed (N command(s))`, exit 0.
- `verify: <id>: FAILED at command N`, exit is the failing command's code.
  Later commands did not run.
- `verify: <id>: not-declared (no ## Verification section)` or
  `not-declared (Verification section holds no verify:cli commands)`,
  exit 0.
- `verify: no such spec` or a usage line, exit 2.

The script reads the spec markdown, never `.derived/`; it is the kit's
`scripts/verify-spec.sh`, copied into the repository with the skill.

## Step 2: read the result honestly

- **`not-declared` is an honest zero, not a pass.** It means the spec
  declares nothing runnable. For a spec whose acceptance criteria are
  mechanically checkable, that is a gap to report in the session summary.
  The implementing session may add a `## Verification` block with the
  commands that prove its criteria; that is a legitimate mid-build edit,
  like `establishes` growth. Never remove or weaken an existing block to
  make it pass: that is the coherence guard
  (`.claude/rules/adversarial-prompt-refusal.md`).
- **`verify:browser` blocks** are reported as skipped. Only an
  orchestrator with a browser stage drives those; nothing here can satisfy
  or fail them, so say so rather than treating the skip as coverage.
- **A failure** is either the code (fix it, re-run the gate and this
  skill) or a criterion that cannot be satisfied here (external state, a
  missing sibling service): then keep `implementation: in-progress`, add a
  dated status note to the spec saying exactly what remains, and report it.

## Step 3: report

```
## verify: <spec-id>
tree: clean | dirty (<files>)
commands: N
  1. <command>  exit <code>  (<criterion label>)
  2. ...
result: passed | FAILED at command N | not-declared
browser blocks: none | <n> skipped (orchestrator-driven)
```

Quote the failing command's output tail when there is one. Do not
paraphrase an exit code.

## Project layer

Nothing here is project-specific. If the corpus lives somewhere other
than `specs/`, export `SPECS_DIR` for the script.
