# AGENTS.md: spec-spine

## New Sessions

Run `/init` as the mandatory first action of every new session. The command reads this section to derive its execution plan dynamically: any item added here is automatically picked up on the next init. This file is the cross-agent authority (read by Claude Code, Codex CLI, Cursor, Copilot, and any future agent via the AAIF/Linux Foundation AGENTS.md standard).

**Init protocol (executed by `/init`):**

> AGENTS.md is loaded implicitly as the protocol source: its contents
> are the protocol, so `/init` does not list AGENTS.md as a parallel
> identity read in Step 1 (avoiding the self-reference loop).

The protocol drives the library through its own built binary, `target/release/spec-spine` (dogfooding). If that binary is missing, build it first: `cargo build --release -p spec-spine-cli`. Do NOT reach for `npx spec-spine` here; the npm/py distributions are for adopters, the self-governance loop uses the in-tree binary.

0. **Load rules.** Read `.claude/rules/orchestrator-rules.md`,
   `.claude/rules/governed-artifact-reads.md`, AND
   `.claude/rules/adversarial-prompt-refusal.md` (the three the library
   scaffolds for every adopter via `spec-spine init`, and which it
   carries for itself).
1. **Parallel reads.** Dispatch the following simultaneously (nothing here
   mutates the working tree, so there is no required ordering):
   - `CLAUDE.md`: project overview and conventions
   - `README.md`: full project description
   - `standards/spec/contract.md`: normative spec-system summary
   - `standards/spec/constitution.md`: durable principles (tier 2)
   - `spec-spine compile --check`: freshness gate for the spec registry (non-fatal; see **Registry freshness** below)
   - `spec-spine index check`: staleness gate for the codebase index (non-fatal)
   - `spec-spine index render`: markdown projection of the committed index
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts per status
   - `spec-spine registry list --ids-only`: spec id list (for latest-spec detection)
   - `ls crates/`: library crate layout
   - `ls specs/`: the spec corpus
   - `ls docs/`: docs surface (design notes, governance)
   - `git log --oneline -10`: recent history
   - `git diff --stat HEAD~1`: last change summary
2. **Emit** the `## initialized: spec-spine` summary block: a layer/crate
   overview, a `## lifecycle:` sub-section populated from the
   `registry status-report --nonzero-only` output, recent activity, and a
   "ready to help with" line.

**Read discipline:** the init protocol MUST NOT parse `.derived/**/*.json` directly (no `python`, `jq`, `awk`, `sed` against compiled artifacts). All structural and lifecycle data comes from the `spec-spine` subcommands (`registry`, `index`) and the rendered markdown view. See `.claude/rules/governed-artifact-reads.md`.

**Staleness surface:** both committed trees have their own gate, and each is non-fatal to `/init`: report it in the summary and continue. If `spec-spine index check` exits non-zero, include "Codebase index: stale, run `spec-spine index`". If the index is not built and `render` fails, report "Codebase index: not built" and continue without structural counts. The registry half is `spec-spine compile --check`, whose verdicts are spelled out under **Registry freshness** below.

**Registry freshness:** spec-spine **commits** its compiled artifacts. Since spec 024 both views are committed as per-unit shard trees: `.derived/spec-registry/by-spec/<id>.json` and `.derived/codebase-index/{by-spec,by-package}/*.json` are tracked (only `.derived/**/build-meta.json` is gitignored; no monolithic `registry.json`/`index.json` is committed). The committed shard set is the reference for lifecycle queries, so `/init` has to know whether it is current.

`/init` asks with `spec-spine compile --check` (spec 031), which compiles in memory and compares against the committed shards **without writing**. Read the exit code:

- **`0` (fresh):** the committed shards are exactly what the corpus compiles to, so the lifecycle counts below reflect the current `specs/*/spec.md` frontmatter. Report nothing.
- **`2` (stale):** *read stderr before believing it* (see **Stale binary** below). For a genuine staleness report, report "Spec registry: stale, run `spec-spine compile` and commit" **and name the drifted shards from its stderr**, then continue. The lifecycle counts come from the committed ledger and are therefore the stale ones; say so rather than presenting them as current.
- **`1` (validation failed):** the corpus itself is broken. Surface the violations, and report the lifecycle counts as **unverified**: they still come from the committed ledger, but with the corpus failing validation there is no way to say whether that ledger corresponds to it. Fixing the violations is the first task of the session, not an aside.
- **any other non-zero** (`3` is I/O / parse / schema / config): treat freshness as unknown, report stderr verbatim, and continue. Never report "fresh" for an exit code you did not recognize.

The counts are formatted in step 2, after every parallel read has returned, so the verdict is always in hand before the numbers are written down.

**Stale binary:** `target/release/spec-spine` is whatever was last built, which is not necessarily this checkout. A binary predating `compile --check` rejects the unknown flag with **exit 2**, the same code as "stale", so the two separate only by stderr: a rejection says `error: unexpected argument '--check'`, while a real report names shards. Rebuild (`cargo build --release -p spec-spine-cli`) and re-run rather than reporting phantom drift. Rebuilding is cheap and is the right reflex whenever the binary predates recent commits.

Do **not** substitute a plain `spec-spine compile` here. Writing would repair the tree as a side effect of reading it, which hides the fact that the *committed* copy was stale: the drift then looks like an uncommitted local edit instead of a defect on the branch (this is exactly how the spec 017/021 drift reached `main` unnoticed). `/init` reports; it does not silently mutate.

**Binary missing or stale:** if the `spec-spine` binary is not built, or predates the commits in this checkout, run `cargo build --release -p spec-spine-cli` and continue (see **Stale binary** above for why a stale one misreports freshness). Do NOT fall back to ad-hoc parsing of `.derived/**`.

If any file is missing: log "not found" and continue.

## Available Agents

Agents live in `.claude/agents/`. Four pipeline agents handle the plan/explore/implement/review cycle:

- `architect`: plans and decomposes tasks, validates approaches against specs. Read-only.
- `explorer`: searches the codebase, traces dependencies, gathers context. Read-only.
- `implementer`: executes focused code changes from an existing plan. Produces minimal diffs.
- `reviewer`: post-change review for bugs, correctness, and spec compliance. Read-only.

## Available Commands

Commands live in `.claude/skills/` (one `SKILL.md` per folder):

- `/init`: initialize a session (load context, lifecycle, recent activity)
- `/setup`: one-time contributor setup, build the `spec-spine` binary and verify the compile then index then lint then couple loop works
- `/commit`: create a git commit with an impact-focused conventional message
- `/code-review`: adversarial review of the current diff for bugs and spec drift
- `/ship`: gate (coupling), review, commit, and PR creation in one governed sequence

## Conventions

- Items added to the "New Sessions" init protocol are auto-loaded by `/init`.
- Agents must be self-contained within `.claude/agents/`: no cross-project dependencies.
- Orchestrated workflows must read compiled artifacts (`.derived/**`) through the `spec-spine` binary, never via ad-hoc parsers: see `.claude/rules/governed-artifact-reads.md`.
- Self-governance runs through the in-tree binary (`target/release/spec-spine`), not the published npm/py distributions.
