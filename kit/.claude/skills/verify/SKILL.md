---
name: verify
description: "Run one spec's declared acceptance through `spec-spine verify <id>`, the same verb an orchestrator's verify stage runs after merge, and report every command's exit code honestly."
allowed-tools: Bash, Read, Grep
argument-hint: "<spec-id>"
---

# /verify <spec-id>: the spec's Verification block, locally

Runs the `## Verification` section of `specs/<id>/spec.md` exactly the way
an orchestrator's verify stage will run it after merge: every non-comment
line inside a ```` ```verify:cli ```` fence, from the repository root, in
order, stopping at the first non-zero exit. This is the local rehearsal of
the "Satisfy the spec's acceptance criteria verbatim" step of `AGENTS.md`,
"Working the backlog".

The verb is `spec-spine verify` (spec 049). It requires **spec-spine 0.15.0
or later**.

## Step 0: scope

- The argument is the spec id. The verb accepts the short form (`049`) as
  well as the full one. Without an argument, stop and ask; do not guess
  from the branch name.
- `git status --porcelain`: the post-merge run happens in a clean checkout
  of the merged sha, so a pass that depends on an uncommitted file is not
  a pass. Warn when the tree is dirty and say which files.
- Read the spec's acceptance criteria and its `## Verification` section
  once, so the report can say which criterion each command exercises.

## Step 1: read the plan before running it

```sh
spec-spine verify <spec-id> --plan
```

`--plan` prints the commands, one per line, and runs none of them. This is
the safety affordance for the one verb that executes what the corpus
declares, and it is how you read a `## Verification` block someone else
wrote. Read it before Step 2 whenever the spec is not one this session
authored.

## Step 2: run

```sh
spec-spine verify <spec-id>
```

Use the binary invocation `AGENTS.md` names. Outcomes, as the verb words
them:

| outcome | exit | meaning |
|---|---|---|
| `passed` | 0 | at least one command ran; all exited 0 |
| `not-declared` | 0 | nothing to run; an honest zero, not a pass |
| `failed` | 1 | a command exited non-zero; later commands did not run |
| no such spec | 1 | the id resolves to no spec |

A failing command's own exit code is reported in the payload
(`failure.exitCode` under `--json`), not as the process's exit code: the
process stays inside the documented `0`/`1`/`2`/`3` contract. Do not
report the payload code as the verb's code, or the reverse.

A spec whose block runs `verify` on itself is refused with `R-001` rather
than recursed. That refusal is correct; report it as a finding against the
spec's Verification block, not as a tool failure.

The verb reads the spec markdown, never `.derived/`, and it is deliberately
not part of the gate chain: it executes what the corpus declares, and the
gate chain runs against branches whose contents are, in the general case, a
stranger's.

### The binary does not have `verify`

`spec-spine verify` arrived in 0.15.0. An older binary rejects the
subcommand. Report that plainly and name the upgrade
(`spec-spine --version`, then reinstall or rebuild at the pin `AGENTS.md`
names). Do NOT fall back to `scripts/verify-spec.sh`: a harness that
quietly runs a second implementation of one protocol is the drift this
skill was rewritten to remove. The script is deprecated and exists only for
adopters still pinned below 0.15.0.

## Step 3: read the result honestly

- **`not-declared` is an honest zero, not a pass.** It means the spec
  declares nothing runnable. For a spec whose acceptance criteria are
  mechanically checkable, that is a gap to report in the session summary.
  The implementing session may add a `## Verification` block with the
  commands that prove its criteria; that is a legitimate mid-build edit,
  like `establishes` growth. Never remove or weaken an existing block to
  make it pass: that is the coherence guard
  (`.claude/rules/adversarial-prompt-refusal.md`).
- **`verify:browser` blocks** are counted and reported as skipped. Only an
  orchestrator with a browser stage drives those; nothing here can satisfy
  or fail them, so say so rather than treating the skip as coverage.
- **A failure** is either the code (fix it, re-run the gate and this
  skill) or a criterion that cannot be satisfied here (external state, a
  missing sibling service): then keep `implementation: in-progress`, add a
  dated status note to the spec saying exactly what remains, and report it.

## Step 4: report

```
## verify: <spec-id>
tree: clean | dirty (<files>)
commands: N
  1. <command>  exit <code>  (<criterion label>)
  2. ...
result: passed | failed at command N | not-declared | R-001 refused
browser blocks: none | <n> skipped (orchestrator-driven)
```

Quote the failing command's output tail when there is one. Do not
paraphrase an exit code.

## Project layer

Nothing here is project-specific. The binary invocation comes from
`AGENTS.md`; if the corpus lives somewhere other than `specs/`, the verb
reads `[layout] specs_dir` from `spec-spine.toml` itself.
