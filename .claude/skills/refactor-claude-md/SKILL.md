---
name: refactor-claude-md
description: "Tighten CLAUDE.md by extracting context-specific guidance into docs and path-scoped rules under .claude/rules, keeping the harness spec coupled and the index fresh."
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
argument-hint: "[path to CLAUDE.md, default ./CLAUDE.md]"
---

# Refactor CLAUDE.md

Reduce the size of `CLAUDE.md` while preserving guidance, by moving
context-specific sections into `docs/` and loading them through
path-scoped rules. In a spec-spine repository `CLAUDE.md`,
`.claude/rules/`, and `AGENTS.md` are usually hashed inputs of the
codebase index and owned by a harness spec: every change here couples to
that spec (a dated decision entry naming the extraction) and stales the
index until `spec-spine index` runs. Find the owner first:

```sh
spec-spine index coverage        # which spec claims the harness files
```

## Process

1. **Read and analyze** the current `CLAUDE.md` in full. Compare it with
   `AGENTS.md`: anything duplicated between them belongs in `AGENTS.md`
   only (it is the cross-agent authority; `CLAUDE.md` carries only what
   Claude Code needs beyond it).

2. **Identify extraction candidates**: sections that are cross-cutting
   patterns rather than core setup, specific to certain directories or
   file types, long and detailed, or better loaded only when relevant.
   Typical candidates: per-package implementation notes, testing
   patterns, a subsystem's conventions, a framework guide.

3. **For each candidate** recommend: the doc name under `docs/`, its
   scope, the `paths:` globs that should trigger it, and the one-line
   reference to keep in `CLAUDE.md`.

4. **Create the files** in this order: extract to `docs/<name>.md`;
   create `.claude/rules/<name>.md` with `paths:` frontmatter and a short
   reminder pointing at the doc; replace the extracted section in
   `CLAUDE.md` with the reference; update any documentation table.

   Path-scoped rule format (the key is `paths`, a YAML list of globs):

   ```markdown
   ---
   paths:
     - "crates/<name>/**"
     - "apps/<name>/**"
   ---

   Two or three key points, and the doc to read: `docs/<name>.md`.
   ```

5. **Couple the change**: add a dated decision entry to the harness spec
   naming the extraction (a new rule file is `establishes` growth for
   that spec if it lists rules individually), then
   `spec-spine compile && spec-spine index` and stage the derived
   directory.

## Key principles

- Extract only context-specific guidance; keep universal rules in
  `CLAUDE.md`.
- Preserve critical information in `CLAUDE.md`: the invariants, the
  commands, the architecture table, the governance mechanics, house
  style.
- Choose meaningful path patterns; a rule that loads everywhere is a `CLAUDE.md` section
  in disguise.
- Keep replacements brief; the reader needs to know where to look.

## Keep in CLAUDE.md

- The invariants and any determinism or hash-stability rule.
- Commands and exit codes.
- The layer-to-package table.
- Governance mechanics (ownership ratchet, committed derived shards,
  read-only hooks).
- House style.

## After extraction

Report the size change (lines before and after), list the files created,
confirm `spec-spine index check` is fresh, and offer `/commit`.

## Project layer

Read from `spec-spine index coverage`: the harness spec. Nothing here is
edited per project.

`$ARGUMENTS`
