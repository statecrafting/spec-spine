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
   - `spec-spine index coverage`: which source files no spec specifically claims (spec 032; non-fatal, exit 2 if the index is stale)
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts per status
   - `spec-spine registry plan`: the ready set (spec 038): which specs can be worked on now and what blocks the rest; `(nothing ready)` in a finished corpus
   - `spec-spine registry list --ids-only`: spec id list (for latest-spec detection)
   - `ls crates/`: library crate layout
   - `ls specs/`: the spec corpus
   - `ls docs/`: docs surface (design notes, governance)
   - `git log --oneline -10`: recent history
   - `git diff --stat HEAD~1`: last change summary
2. **Emit** the `## initialized: spec-spine` summary block: a layer/crate
   overview, a `## lifecycle:` sub-section populated from the
   `registry status-report --nonzero-only` output (with the `registry plan`
   ready/blocked line beneath it), recent activity, and a
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

## Working the backlog

This repository files a spec as `draft`, builds it, then ratifies it in a
separate PR, so the loop here differs from an adopter's ratify-then-build
corpus in step 1 and step 6. One spec per PR, then stop.

1. **Pick or file the spec.** `spec-spine registry plan` prints the ready set
   (`/next` applies the approval and in-flight rules on top of it); in a
   finished corpus it prints `(nothing ready)` and the work is to file the
   next `NNN-slug` from the design backlog (`docs/design/`) with `/spec`. A new spec is born
   `status: draft`, `implementation: pending`, and declares every edge it
   needs, including `amends` on any approved spec whose stated behavior it
   changes, without editing that spec (spec 040).
2. **Branch.** A feature branch named after the spec id. Never commit to
   `main`. `/build <id>` sequences steps 2 to 5 for a spec that is already
   filed; here a filed spec stays `draft` while it is built, so `/build`'s
   preflight accepts `draft` in this repository when a human named the id.
3. **Re-read the design before coding.** If the design is imprecise, record
   the choice in the spec. If it is wrong, stop and report; never rewrite an
   approved spec to match code (`.claude/rules/adversarial-prompt-refusal.md`).
4. **Implement within the territory.** Claim every new file in the new spec's
   `establishes` (or a `// Spec:` header when the file already has an
   owner). Touching a unit another spec owns is an `extends` edge on that
   unit. Never edit `.derived/` by hand.
5. **Run the gate before every commit.** `cargo run -p spec-spine-cli --
   compile`, `... index`, `... lint --fail-on-warn`, `... index check`,
   `... couple --base origin/main --head HEAD`, then `cargo test --workspace
   --locked`, `cargo clippy --workspace --all-targets --locked -- -D
   warnings`, `cargo fmt --all --check`. Commit the regenerated shards with
   the code they describe. The skills call this list "the gate as
   `AGENTS.md` lists it"; the binary is `target/release/spec-spine` (or
   `cargo run -p spec-spine-cli --`), never `npx spec-spine`.
6. **Ship, then ratify.** `/verify <id>` runs the spec's `## Verification`
   block through `scripts/verify-spec.sh`. `/ship` opens the PR with
   `implementation: complete` set once that block holds, and `/shepherd`
   drives the PR to a merge confirmed on disk. After merge, a
   second PR flips `status: draft` to `approved` (the ratify PR), and the
   corpus count moves. A `Spec-Drift-Waiver:` line needs explicit human
   approval and is cited in the PR body.

## Available Agents

Agents live in `.claude/agents/`. Four pipeline agents handle the plan/explore/implement/review cycle:

- `architect`: plans and decomposes tasks, validates approaches against specs. Read-only.
- `explorer`: searches the codebase, traces dependencies, gathers context. Read-only.
- `implementer`: executes focused code changes from an existing plan. Produces minimal diffs.
- `reviewer`: post-change review for bugs, correctness, and spec compliance. Read-only.

## Available Commands

Commands live in `.claude/skills/` (one `SKILL.md` per folder). They are the
kit's fifteen, byte-identical to `kit/.claude/skills/` (spec 048 pins this):
the project layer lives in this file, not in the skills.

The governed loop, in the order "Working the backlog" runs it:

- `/init`: initialize a session (this protocol)
- `/setup`: one-time contributor setup: build the `spec-spine` binary and verify the governed loop
- `/next`: name the next work order from `registry plan`, minus drafts, with in-flight specs and blockers. Read-only
- `/build <id>`: implement one spec start to finish: preflight, branch, flip, implement, gate, verify, flip complete
- `/verify <id>`: run the spec's `## Verification` block locally through `scripts/verify-spec.sh`
- `/ship`: gate, review, commit on the feature branch, open the PR
- `/shepherd`: watch the PR's checks by head sha, remediate through the gate, merge, confirm on disk
- `/spec`: author a new spec at the next free ordinal, born `draft`

The supporting skills:

- `/commit`: create a git commit with an impact-focused conventional message, spec ordinal as scope
- `/code-review`: review the working diff for correctness bugs, spec drift, and illegitimate mid-build spec edits
- `/validate-and-fix`: run the local CI composite and fix discovered issues by severity
- `/cleanup`: dead-code and duplicate detection with ownership-aware recommendations
- `/implement-plan`: execute a cross-cutting plan file step by step with checkpoints
- `/research`: deep research with parallel sub-agents; corpus questions go through `spec-spine`
- `/refactor-claude-md`: tighten a `CLAUDE.md` into path-scoped rules, keeping the harness spec coupled

## Conventions

- Items added to the "New Sessions" init protocol are auto-loaded by `/init`.
- Agents must be self-contained within `.claude/agents/`: no cross-project dependencies.
- Orchestrated workflows must read compiled artifacts (`.derived/**`) through the `spec-spine` binary, never via ad-hoc parsers: see `.claude/rules/governed-artifact-reads.md`.
- Self-governance runs through the in-tree binary (`target/release/spec-spine`), not the published npm/py distributions.
