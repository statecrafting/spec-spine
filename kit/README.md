# spec-spine Claude Code Kit

A ready-to-copy Claude Code skill kit for any repository that adopts
[spec-spine](https://github.com/statecrafting/spec-spine). It layers a complete
governed-development loop on top of the spec-spine substrate: session
initialization, picking the next spec, building it, verifying it, adversarial
review, conventional commits, gated PR creation, and shepherding the PR to a
merge confirmed on disk. The loop is the one an orchestrator drives one spec
per session; the same skills serve a human-driven session.

The full guide (what each piece does, how to adapt it, the governed loop) lives
in the spec-spine docs under **Use with Claude Code**. This directory is the
artifact that guide points at: copy it, customize the bracketed parts, go.

## What it contains

```
kit/
  README.md            # this file
  AGENTS.md            # the cross-agent New Sessions protocol and "Working the backlog"
  settings.json        # Claude Code hooks (read-only) and permissions
  .mcp.json            # empty MCP server template
  scripts/
    verify-spec.sh     # runs a spec's verify:cli blocks; what /verify and a verify stage call
  .claude/
    skills/   15 skills   the loop:  init, setup, next, build, verify, ship, shepherd, spec
                          support:   commit, code-review, validate-and-fix, cleanup,
                                     implement-plan, research, refactor-claude-md
    agents/    4 agents   architect, explorer, implementer, reviewer
    rules/     3 rules    orchestrator-rules, governed-artifact-reads,
                          adversarial-prompt-refusal
```

The hooks read and never write (spec 046): `SessionStart` and `Stop` report
registry and index freshness, `PostToolUse` recompiles the registry after a
`spec.md` edit (the one sanctioned write) and checks index staleness after a
hashed-input edit, and `PreToolUse` refuses a push to `main` and blocks
`gh pr create` on a stale index, uncommitted shards, or a red coupling gate
without an inline `Spec-Drift-Waiver:`. Every hook acts on the repository the
command targets and says so when it skips.

The three rules are the same floor `spec-spine init` already scaffolds, so an
adopter who ran `spec-spine init` can skip them.

## Install

1. Install spec-spine (`cargo install spec-spine-cli`, `npm i -D spec-spine`, or
   `pip install spec-spine`). Verify with `spec-spine --version`.
2. Copy `.claude/` into your repository root. Copy `AGENTS.md`, `settings.json`,
   and `.mcp.json` too if you do not already have them.
3. Copy `scripts/verify-spec.sh` into your repository's `scripts/` (make it
   executable). `/verify` runs it; an orchestrator's verify stage runs the same
   protocol after merge.
4. Customize `AGENTS.md`: replace every `<bracketed>` placeholder (project
   name, source directories, the parallel reads), pin the spec-spine version,
   and write the gate command list under "Working the backlog" (the governance
   floor plus your stack's build, tests, and lints). Adjust the `settings.json`
   permission allow-list to your tools, and tune the hashed-input globs in the
   `PostToolUse` hook to match your `spec-spine.toml [index] extra_hashed_inputs`.
5. Run `/setup` then `/init` in a Claude Code session.

**The skills are not customized.** Every `SKILL.md` is repository-invariant
and ends with a `## Project layer` section naming what it reads from
`AGENTS.md`, `spec-spine.toml`, and the path-scoped rules: the binary
invocation, the version pin, the gate command list, the stack gate, the
never-touch artefacts. Keep the project layer there and the skills stay
byte-identical to the kit, so a kit update is a copy, not a merge. The agents
carry the two placeholders `<your source tree>` and `<your build command>`.

Add on top: a domain-specialist agent (read-only, loads your framework's
reference docs), path-scoped invariant rules (`paths:` frontmatter, loaded on
touch), and a `build-commands.md` rule naming your composite (`make ci`).

## Govern the harness

The harness is what an agent may do, so an adopter claims it in a spec of its
own (commonly the second spec in the corpus) and lets the coupling gate hold
it. The shape that works:

```yaml
establishes:
  - "AGENTS.md"
  - "CLAUDE.md"
  - ".claude/settings.json"
  - { kind: directory, path: ".claude/skills/" }
  - { kind: directory, path: ".claude/agents/" }
  - "scripts/verify-spec.sh"
  # the three floor rules stay with the bootstrap spec that scaffolded them
```

with `AGENTS.md`, `CLAUDE.md`, `.claude/**`, and `scripts/**` listed in
`spec-spine.toml [index] extra_hashed_inputs` so an edit to any of them stales
the index until it is regenerated and committed. The spec's behavior section
tables the hooks and the permission policy; its acceptance criteria say that
editing `settings.json` without editing the spec fails `couple` with `C-001`.

## Intentionally excluded

Project-specific pieces are not shipped; recreate them for your own stack:

- a domain-specialist agent (a framework expert, a ledger guardian): the
  generic "read-only specialist" pattern, named by the invariant rule it
  serves;
- `build-commands.md` and invariant rules: the generic "paths-scoped context
  rule" pattern (`paths:` frontmatter), which the harness spec lists
  individually;
- a `Makefile` composite and a CI workflow: every adopter's differ; the gate
  command list in `AGENTS.md` is the contract the skills read.

## License and origin

Apache-2.0, the same license as spec-spine (see the repository `LICENSE`). The
skills, agents, and rules were extracted from the Open Agentic Platform and
generalized for any spec-spine adopter; they are distributed here under
Apache-2.0.
