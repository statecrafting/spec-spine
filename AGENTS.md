# AGENTS.md: spec-spine

> Governed by `specs/078-the-protocol-has-an-owner/spec.md`.

## New Sessions

Run `/prime` as the mandatory first action of every new session. The command reads this section to derive its execution plan dynamically: any item added here is automatically picked up on the next init. This file is the cross-agent authority (read by Claude Code, Codex CLI, Cursor, Copilot, and any future agent via the AAIF/Linux Foundation AGENTS.md standard).

**Session protocol (executed by `/prime`):**

> AGENTS.md is loaded implicitly as the protocol source: its contents
> are the protocol, so `/prime` does not list AGENTS.md as a parallel
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
   - `spec-spine --version`: the binary's version. **Read this before believing any
     exit code below.** The document already called it a precondition and never
     scheduled it, so the precondition held only for an agent that read the prose
     under the step list (spec 074 3.7).
   - `spec-spine check`: the freshness read for **both** committed trees, the spec
     registry and the codebase index (spec 075; non-fatal, see **Freshness** below)
   - `spec-spine index render`: markdown projection of the committed index
   - `spec-spine index coverage`: which source files no spec specifically claims (spec 032; non-fatal, exit 2 if the index is stale)
   - `spec-spine index diagnostics`: the unresolved-unit diagnostics the committed index records (spec 050; non-fatal, empty output means none)
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts per status
   - `spec-spine registry plan`: the ready set (spec 038): which specs can be worked on now and what blocks the rest; `(nothing ready)` in a finished corpus
   - `spec-spine registry list --ids-only`: spec id list (for latest-spec detection)
   - `ls crates/`: library crate layout
   - `ls specs/`: the spec corpus
   - `ls docs/`: docs surface (design notes, governance)
   - `git log --oneline -10`: recent history
   - `git diff --stat HEAD~1`: last change summary
2. **Emit** the `## primed: spec-spine` summary block. **Consult the
   `--version` read before reporting any freshness verdict**: a version that
   predates the flag a step passed makes that step's exit code meaningless,
   and reporting it as drift sends someone chasing a phantom. Then: a
   layer/crate overview, a `## lifecycle:` sub-section populated from the
   `registry status-report --nonzero-only` output (with the `registry plan`
   ready/blocked line beneath it), the freshness verdicts, the
   unresolved-unit count from `index diagnostics`, recent activity, and a
   "ready to help with" line.

**Read discipline:** the init protocol MUST NOT parse `.derived/**/*.json` directly (no `python`, `jq`, `awk`, `sed` against compiled artifacts). All structural and lifecycle data comes from the `spec-spine` subcommands (`registry`, `index`) and the rendered markdown view. See `.claude/rules/governed-artifact-reads.md`.

**Unresolved units:** `spec-spine index diagnostics` (spec 050) lists the `W-001` / `W-002` diagnostics the committed index records: a unit an owning spec claims that does not resolve yet. Empty output means none, which is the state a finished corpus is in. Report the count, and name the specs when there are any: a spec under way legitimately claims territory it has not written yet (specs 025 and 044), so these are work in flight, not defects. The gate half is `check --fail-on-unresolved`, which this repository's CI runs.

**Freshness:** spec-spine **commits** its compiled artifacts. Since spec 024 both views are committed as per-unit shard trees: `.derived/spec-registry/by-spec/<id>.json` and `.derived/codebase-index/{by-spec,by-package}/*.json` are tracked (only `.derived/**/build-meta.json` is gitignored; no monolithic `registry.json`/`index.json` is committed). The committed shard set is the reference for lifecycle queries, so `/prime` has to know whether it is current.

`spec-spine check` (spec 075) asks about both trees in one call. It compiles in memory and compares against the committed shards **without writing**, and it reports each tree separately: the registry half and the index half each keep the structure their own primitive emits, so the drifted shard names are still there to read back. Its exit code is the more severe of the two, in this order: **`3` then `1` then `2` then `0`**. Read it, and read the two report lines under it, because one code covers two trees:

- **`0` (both fresh):** the committed shards are exactly what the corpus compiles to, so the lifecycle counts below reflect the current `specs/*/spec.md` frontmatter. Report nothing.
- **`2` (stale):** *check the `--version` read from step 1 before believing it* (see **Stale binary** below): a binary predating the verb cannot be reporting drift. For a genuine staleness report, say **which tree** the output named, report "Spec registry: stale, run `spec-spine compile` and commit" or "Codebase index: stale, run `spec-spine index`" accordingly, **and name the drifted shards from its stderr**, then continue. The lifecycle counts come from the committed ledger and are therefore the stale ones; say so rather than presenting them as current.
- **`1` (validation failed, or unresolved units refused):** with `--fail-on-unresolved` this code also covers a refused unresolved-unit diagnostic, so read the report lines to tell the two apart. If the corpus itself fails validation, surface the violations and report the lifecycle counts as **unverified**: they still come from the committed ledger, but with the corpus failing validation there is no way to say whether that ledger corresponds to it. Fixing the violations is the first task of the session, not an aside. This outranks `2` because staleness is not meaningful against a corpus that does not validate.
- **`3` (I/O / parse / schema / config):** a read that could not be performed has not answered. Treat freshness as unknown for **both** trees, report stderr verbatim, and continue. Never report "fresh" for an exit code you did not recognize.

If the index is not built and `render` fails, report "Codebase index: not built" and continue without structural counts.

The counts are formatted in step 2, after every parallel read has returned, so the verdict is always in hand before the numbers are written down.

**Stale binary:** `target/release/spec-spine` is whatever was last built, which is not necessarily this checkout.

**Ask `spec-spine --version` before believing any exit code.** Every binary ever released answers it, and it exits 0. If the version predates the flag you are about to pass, rebuild (`cargo build --release -p spec-spine-cli`) or reinstall; do not interpret the exit code of a flag the binary does not have. Rebuilding is cheap and is the right reflex whenever the binary predates recent commits.

That precondition replaces an older ritual of matching clap's English on stderr, which was pinned to a dependency's message format and could not survive a clap release. Since spec 063 a new binary maps every usage error to **exit 3**, so exit 2 from any verb means staleness and nothing else; but the binary that reports the wrong code is by definition the old one, so a procedure that may be talking to an old binary cannot rely on the new behavior. Where a repository sets `[meta] required_version` (spec 062), the check happens on every run and this manual step is unnecessary.

Do **not** substitute a plain `spec-spine compile` or `spec-spine index` here. Writing would repair the tree as a side effect of reading it, which hides the fact that the *committed* copy was stale: the drift then looks like an uncommitted local edit instead of a defect on the branch (this is exactly how the spec 017/021 drift reached the default branch unnoticed). `/prime` reports; it does not silently mutate. `spec-spine check` carries the same never-writes contract, which is why the protocol can call it.

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
5. **Run the gate before every commit.** The governance floor, in this
   order (`compile` and `index` write; the checks follow):

   ```sh
   spec-spine compile
   spec-spine index
   spec-spine check --fail-on-unresolved --fail-on-warn
   spec-spine lint --fail-on-warn
   spec-spine index coverage --fail-on-untraced
   spec-spine couple --base "$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || echo origin/main)" --head HEAD
   ```

   The base ref is resolved from the repository rather than assumed to be
   `origin/main` (spec 072). Set `$SPEC_SPINE_DEFAULT_BRANCH` to override
   the branch the push gate protects and `kit/Makefile` compares against.

   then the stack's own gate: `cargo test --workspace --locked`,
   `cargo clippy --workspace --all-targets --locked -- -D warnings`,
   `cargo fmt --all --check`. Commit the regenerated shards with the code
   they describe. The skills call this list "the gate as `AGENTS.md` lists
   it", and `kit_skills.rs` asserts each skill's inlined floor is a subset
   of it (spec 051), so a step added here reaches every skill. The binary
   is `target/release/spec-spine` (or `cargo run -p spec-spine-cli --`),
   never `npx spec-spine`.

   Every step above is enforced by CI's `self_governance` job. CI runs
   `check` in place of `compile` and `index`, because a gate must never
   repair the tree it is judging.
6. **Ship, then ratify.** `/verify <id>` runs the spec's `## Verification`
   block through `spec-spine verify <id>` (spec 049). `/ship` opens the PR with
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

- `/prime`: prime a session (this protocol)
- `/setup`: one-time contributor setup: build the `spec-spine` binary and verify the governed loop
- `/next`: name the next work order from `registry plan`, minus drafts, with in-flight specs and blockers. Read-only
- `/build <id>`: implement one spec start to finish: preflight, branch, flip, implement, gate, verify, flip complete
- `/verify <id>`: run the spec's `## Verification` block locally through `spec-spine verify <id>`
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

- Items added to the "New Sessions" session protocol are auto-loaded by `/prime`.
- Agents must be self-contained within `.claude/agents/`: no cross-project dependencies.
- Orchestrated workflows must read compiled artifacts (`.derived/**`) through the `spec-spine` binary, never via ad-hoc parsers: see `.claude/rules/governed-artifact-reads.md`.
- Self-governance runs through the in-tree binary (`target/release/spec-spine`), not the published npm/py distributions.
