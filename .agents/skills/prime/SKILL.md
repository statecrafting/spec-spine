---
name: prime
description: "Initialize a session by executing the cross-agent New Sessions protocol declared in AGENTS.md. Reads only; never repairs the tree."
allowed-tools: Bash, Read, Glob, Grep
---

# /prime: session bootstrap

Thin dispatcher. The canonical protocol lives in `AGENTS.md` under
`## New Sessions`, the cross-agent AAIF/Linux Foundation standard read by
Claude Code, Codex CLI, Cursor, Copilot, and an orchestrator's driven
sessions alike.

## What to do

1. Read `AGENTS.md`: the section from `## New Sessions` inclusive to the
   next `## ` heading exclusive. That section is the step list.
2. Load the standing rules it names first (`.claude/rules/`), then execute
   the protocol, using parallel tool calls wherever it says "dispatch
   simultaneously".
3. Emit the structured summary the protocol prescribes: the
   `## primed: <project>` block (the layer model, a `## lifecycle:`
   sub-section from `registry status-report`, the `registry plan`
   ready/blocked line, the freshness verdicts, recent activity, and a
   ready-to-help line).

This dispatcher deliberately does not duplicate the step list: `AGENTS.md`
is the single source of truth. Evolve the protocol by editing `AGENTS.md`,
never this file, so every agent stays in sync.

## Rules

- The protocol's governed reads go through the `spec-spine` invocation
  `AGENTS.md` names. If `spec-spine --version` fails, run `/setup` first
  (an in-tree build if `AGENTS.md` says the binary is built from source);
  never fall back to parsing `.derived/` by hand.
- `/prime` reports, it does not mutate: `spec-spine check` is the freshness
  read, never a bare `compile` or `index`. It answers for both committed
  trees and writes nothing. A stale verdict is reported with the shards it
  names, per tree, and the session continues; repairing the tree is the
  session's later, committed work, not a side effect of reading it.
- Establish the binary's version before believing any exit code, which is the
  read `AGENTS.md` step 1 schedules. A binary older than the checkout can
  reject a flag the protocol passes, and reporting that as drift sends someone
  chasing a phantom. Exit-code semantics live in `AGENTS.md`, which is where
  the project layer lives.
- A file the protocol names but cannot find is logged as "not found" and
  the protocol continues.

## Project layer

Nothing in this file is project-specific. The project name, the binary
invocation, the read list, and the summary shape all come from
`AGENTS.md`.
