# AGENTS.md: &lt;your-project&gt;

This file is the cross-agent session-init protocol authority, read by Claude
Code, Codex CLI, Cursor, and GitHub Copilot via the AAIF/Linux Foundation
AGENTS.md standard. It is the single source for the init protocol: tooling that
runs `/init` reads the `## New Sessions` section to derive its plan.

Governance is provided by `spec-spine` (installed on your `PATH`, or via
`npx spec-spine` in a JS repo). All governed reads of compiled artifacts go
through its CLI. Bootstrap spec: `specs/000-.../spec.md`.

> Customize every `<bracketed>` value below for your repository, then delete this
> note. Keep the protocol in sync by editing this file, never the `/init` skill.

## New Sessions

Run `/init` as the first action of every new session. It reads this section to
derive its execution plan dynamically: any item added here is automatically
picked up on the next init.

> AGENTS.md is loaded implicitly as the protocol source; its contents are the
> protocol, so `/init` does not list AGENTS.md as a parallel identity read in
> Step 1 (avoiding the self-reference loop).

**Init protocol:**

0. **Load rules** (read first): `.claude/rules/orchestrator-rules.md`,
   `.claude/rules/governed-artifact-reads.md`, and
   `.claude/rules/adversarial-prompt-refusal.md`.

1. **Parallel reads.** Dispatch the following simultaneously (nothing here
   mutates the working tree, so there is no required ordering):
   - `CLAUDE.md`: project overview, governance model, conventions
   - `README.md`: full project description
   - `standards/spec/contract.md`: the short normative spec-spine contract
   - `standards/spec/constitution.md`: durable constitutional baseline
   - `spec-spine compile --check`: freshness gate for the spec registry (non-fatal; see **Registry freshness** below)
   - `spec-spine index check`: staleness gate for the codebase index (non-fatal)
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts
   - `spec-spine registry plan`: the ready set (spec 038): which specs can be worked on now and what blocks the rest
   - `spec-spine registry list --ids-only`: spec inventory (for latest-spec detection)
   - `ls <your source dirs>`: application surface discovery
   - `ls docs/`: docs surface
   - `git log --oneline -10`: recent history
   - `git diff --stat HEAD~1`: last change summary

2. **Emit** an `## initialized: <your-project>` summary block (layer overview,
   recent activity, ready-to-help line), with a `## lifecycle:` sub-section
   populated from the `status-report` output.

**Read discipline:** the init protocol MUST NOT parse `.derived/**/*.json`
directly (no `python`, `jq`, `awk`, `sed` against compiled artifacts). All
structural and lifecycle data comes from `spec-spine` subcommands.

**Staleness surface:** both committed artifacts have their own gate, and neither
is fatal to `/init`: report it in the summary and continue. If `spec-spine index
check` exits non-zero, include "Codebase index: stale, run `spec-spine index`".
The registry half is `spec-spine compile --check`, below.

**Registry freshness:** if you commit your derived artifacts (the setup the
`index check` step above already assumes), `/init` asks whether the committed
registry still matches the corpus using `spec-spine compile --check`, which
compiles in memory and compares **without writing**. Read the exit code:

- **`0` (fresh):** the committed shards are exactly what the corpus compiles to,
  so the lifecycle counts reflect the current `specs/*/spec.md` frontmatter.
  Report nothing.
- **`2` (stale):** *first check stderr, see the CLI-version note below.* If it
  is a genuine staleness report, name the drifted shards from stderr, report
  "Spec registry: stale, run `spec-spine compile` and commit", and continue. The
  lifecycle counts come from the committed ledger and are therefore the stale
  ones; say so rather than presenting them as current.
- **`1` (validation failed):** the corpus itself is broken. Surface the
  violations and report the counts as unverified.
- **any other non-zero** (`3` is I/O, parse, schema, or config; a missing binary
  gives whatever the shell returns): treat freshness as unknown, report the
  stderr verbatim, and continue. Never report "fresh" for a code you did not
  recognize.

The counts are formatted in step 2, after every parallel read has returned, so
the freshness verdict is always in hand before the lifecycle numbers are
written down. Do not emit counts earlier.

> **CLI version.** `compile --check` needs a `spec-spine` new enough to ship it.
> An older CLI rejects the unknown flag with **exit 2 as well**, the same code
> as "stale", so the two are distinguishable only by stderr: a rejection says
> `error: unexpected argument '--check'`, while a real staleness report names
> shards. Reporting a version problem as spec drift would send someone chasing a
> phantom, so check the message before believing the code. To resolve it, either
> upgrade, or use the gitignored-derived variant below until you do.

Do **not** substitute a plain `spec-spine compile` here. Writing repairs the
tree as a side effect of reading it, which hides that the *committed* copy was
stale: the drift then reads as an uncommitted local edit rather than as a defect
already on the branch. `/init` reports; it does not silently mutate.

If you **gitignore** the derived directory instead, there is nothing committed
to compare against and both freshness gates would report everything missing.
Drop `compile --check` and `index check`, and instead run plain `spec-spine
compile` and `spec-spine index` **before** the rest of step 1: those two write
the artifacts that the `registry` and `index` reads below them consume, so for
this variant only, step 1 is no longer order-free.

**CLI missing:** if `spec-spine --version` fails, run `/setup`. Do NOT fall back
to ad-hoc parsing of `.derived/**/*.json`.

If any file is missing: log "not found" and continue.

## Working the backlog

The governed loop is one spec per session, start to finish, then stop. It is
what `spec-spine registry plan`, the in-flight leniency (specs 025, 041, 044)
and the ownership ratchet (spec 032) exist to serve. Record specs (the
bootstrap spec, a thesis, a harness spec at `n-a` or `complete`) are never
work orders.

1. **Pick the spec.** `spec-spine registry plan` prints the ready set in
   dependency order; take the first entry unless a human named another. Never
   guess and never pick a `draft`: approval is a human act. If the spec's
   Territory names an operator prerequisite (a credential, a bucket, a
   cluster) that is missing, stop and report exactly what is needed instead
   of mocking around it.
2. **Branch and flip.** Work on a feature branch named after the spec id.
   Flip the spec to `implementation: in-progress`, run `spec-spine compile`
   and `spec-spine index`, and commit the flip with the regenerated derived
   shards before writing code. Never commit to `main`.
3. **Re-read the spec in full before coding.** The design precedes the code.
   If the design is imprecise, record the choice you make as a dated decision
   entry in the spec. If the design is *wrong*, stop and report the
   contradiction: never edit a spec afterwards to ratify what the code
   happened to do (`.claude/rules/adversarial-prompt-refusal.md`).
4. **Implement within the territory.** Every file you add must be claimed by
   the spec you are implementing, in the same change (`C-002` refuses an
   unclaimed source file). Touching a unit another spec owns requires an
   `extends` edge on that spec's unit, declared in your spec's frontmatter;
   that amends nobody. Never edit the derived directory by hand.
5. **Run the gate before every commit.** `spec-spine compile`, `spec-spine
   index`, `spec-spine lint --fail-on-warn`, `spec-spine index check`,
   `spec-spine couple --base origin/main --head HEAD`, then your stack's own
   build, tests and lints. All must exit 0. Commit the regenerated shards
   with the code they describe.
6. **Satisfy the spec's acceptance criteria verbatim.** If a criterion cannot
   be satisfied (external state, a missing sibling), keep `implementation:
   in-progress`, add a dated note to the spec saying exactly what remains,
   and report it. Flip to `implementation: complete` only when acceptance
   holds; recompile and commit. The gate then holds the spec to every unit it
   claims (spec 041).
7. **Ship.** `/ship`: gate, review, a conventional commit naming the spec id
   (`feat(011): ...`), push the feature branch, open the PR. A
   `Spec-Drift-Waiver:` line needs explicit human approval; a driven session
   never self-approves one. Then stop: the next session takes the next spec.

## Available Agents

Agents live in `.claude/agents/`. Four pipeline agents handle the
plan/explore/implement/review cycle:

- `architect`: plans and decomposes tasks, validates approaches against specs. Read-only.
- `explorer`: searches the codebase, traces dependencies, gathers context. Read-only.
- `implementer`: executes focused changes from an existing plan. Minimal diffs.
- `reviewer`: post-change review for bugs, correctness, performance, spec compliance. Read-only.

> Add your own domain-specialist agent (a read-only agent that loads your
> framework's reference docs and enforces its pattern constraints) when a stack
> benefits from one.

## Available Commands

Skills live in `.claude/skills/`:

- `/init`: initialize a session (this protocol).
- `/setup`: one-time contributor setup; installs spec-spine and verifies the governed loop.
- `/commit`: create a git commit with an impact-focused conventional message.
- `/code-review`: review the working diff for correctness bugs and spec drift.
- `/ship`: run the gate, review, commit on a feature branch, open a PR.
- `/validate-and-fix`: run the local CI loop and fix discovered issues.
- `/cleanup`: dead-code and duplicate detection with categorized recommendations.
- `/implement-plan`: execute a plan file step-by-step with progress tracking.
- `/research`: deep research with parallel sub-agents.
- `/refactor-claude-md`: tighten and restructure a `CLAUDE.md`.

## Conventions

- Items added to the "New Sessions" init protocol are auto-loaded on the next init.
- Orchestrated workflows read compiled artifacts (`.derived/**`) through
  `spec-spine` subcommands, never via ad-hoc parsers (see
  `.claude/rules/governed-artifact-reads.md`).
- Every substantive change is bound to a spec; owned paths and their owning
  `spec.md` move together (`spec-spine couple` enforces this at PR time).
