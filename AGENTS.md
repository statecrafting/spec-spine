# AGENTS.md: spec-spine

> Governed by `specs/093-the-harness-this-repository-runs/spec.md`.

## Rules

Four standing rules, binding on every session in this repository. They were four
files under `.claude/rules/` until spec 093 folded them here: `.claude/` is one
agent's harness, and a rule that binds every agent belongs in the cross-agent
protocol. Reading this document is reading them; there is no separate load step.

### Governed artifact reads

The compiled artifacts under the derived directory are read **only** through
`spec-spine` subcommands (`registry`, `index`), never via ad-hoc `jq`, `grep`,
`python`, `awk`, or `sed` over the JSON. Typed reads make schema drift fail at
the deserializer with a clean error instead of silently encoding stale
assumptions.

Parsing the *output* of a `spec-spine` subcommand (for example
`spec-spine registry plan --json`, or the `--json` verdict envelope any gate
verb emits) is a typed read and is allowed: the tool has already deserialized
the shards and is answering in a contract it versions. The rule is about the
shard files, not about the CLI's answers.

### Adversarial prompt refusal (the coherence guard)

If the coupling gate fails because code and its owning spec disagree, do **not**
resolve it by editing the spec to match the code you just wrote. Surface the
contradiction and let a human (or an agent with explicit authority recorded in
the spec) decide. Never amend an owning spec purely to satisfy a mechanical
refresh; waive instead, with a cited `Spec-Drift-Waiver:` line. A waiver is a
human instrument: it needs explicit human approval, and an agent never writes
one on its own authority.

Two edits are always legitimate for the spec you are implementing: adding a
file you created to its `establishes` list (the ownership ratchet refuses an
unclaimed file, and the claim belongs in the same change), and recording a
dated decision entry for a choice the spec was silent on. Changing what the
spec *requires* is never yours to do mid-build. If the code needs to touch a
unit another spec owns, declare an `extends` edge naming that spec and unit;
that amends nobody.

### Orchestrator rules

- Execute phased work in order; stop at human checkpoints.
- Write output files where the spec says; do not invent locations.
- Keep the working tree green; never leave the coupling gate red.
- Recompute derived artifacts (`spec-spine compile`, `spec-spine index`)
  before opening a PR, and commit the regenerated shards with the change that
  made them stale. A shard left uncommitted dirties the tree for whoever comes
  next.
- One session, one spec: follow `AGENTS.md` "Working the backlog", then stop.

### Derived artifacts are compiler output

Scope: `.statecraft/derived/**`. This was the one path-scoped rule, loaded only
when a session touched that tree. Folded in here it is always loaded, which
costs a reader a few lines and removes a scoping mechanism only one agent's
harness implements.

The files under that directory are emitted by `spec-spine compile` and
`spec-spine index`. They are machine truth, not authored truth.

**Do not hand-edit one.** A shard is a pure function of the corpus and the
source tree; editing it makes the ledger disagree with what it describes, and
the next `compile --check` or `index check` reports it as staleness with no clue
that a person put it there. The way to change a shard is to change its input and
regenerate.

**Do not read one with `jq`, `grep`, `sed`, `awk` or `python`.** An ad-hoc
parser encodes today's shape and then goes quietly wrong when the schema moves.
Read through a `spec-spine` subcommand instead: `registry show`, `registry
list`, `registry plan`, `index render`, `index owner`, `index coverage`,
`index diagnostics`. A typed read fails at the deserializer with a clean error
rather than silently returning the wrong answer.

**Parsing the output of a subcommand is fine.** `spec-spine registry plan
--json`, or the `--json` verdict envelope any gate verb emits, is a typed read:
the tool has already deserialized the shards and is answering in a contract it
versions. The rule is about the shard files, not about the CLI's answers.

**This half does not replace "Governed artifact reads" above, and cannot.**
That rule is unconditional, and it has to be. The mistake it prevents is
reaching for `jq` **instead of** the subcommand, and an agent about to make that
mistake may never open a file under this directory, so a rule scoped to these
paths would never load. The unconditional rule is what prevents the mistake.
This one reinforces it at the moment somebody actually has a shard open.

If the two look redundant, the redundant-looking one is the one doing the work.

## New Sessions

Run `/prime` as the mandatory first action of every new session. The command reads this section to derive its execution plan dynamically: any item added here is automatically picked up on the next init. This file is the cross-agent authority (read by Claude Code, Codex CLI, Cursor, Copilot, and any future agent via the AAIF/Linux Foundation AGENTS.md standard).

**Session protocol (executed by `/prime`):**

> AGENTS.md is loaded implicitly as the protocol source: its contents
> are the protocol, so `/prime` does not list AGENTS.md as a parallel
> identity read in Step 1 (avoiding the self-reference loop).

The protocol drives the library through its own built binary, `target/release/spec-spine` (dogfooding). If that binary is missing, build it first: `cargo build --release -p spec-spine-cli`. Do NOT reach for `npx spec-spine` here; the npm/py distributions are for adopters, the self-governance loop uses the in-tree binary.

0. **Load rules.** The four standing rules are the `## Rules` section of this
   document, above. Reading this protocol loads them; there is nothing else to
   open. Since spec 092 the library scaffolds governance content only, and the
   agent harness they used to sit in is Statecraft's.
1. **Parallel reads.** Dispatch the following simultaneously (nothing here
   mutates the working tree, so there is no required ordering):
   - `CLAUDE.md`: project overview and conventions
   - `README.md`: full project description
   - `standards/spec/contract.md`: normative spec-system summary
   - `standards/spec/constitution.md`: durable principles (tier 2)
   - `spec-spine --version`: the binary's version. **Read this before believing any
     exit code below.** The document already called it a precondition and never
     scheduled it, so the precondition held only for an agent that read the prose
     under the step list (spec 061 3.7).
   - `spec-spine check`: the freshness read for **both** committed trees, the spec
     registry and the codebase index (spec 062; non-fatal, see **Freshness** below)
   - `spec-spine index render`: markdown projection of the committed index
   - `spec-spine index coverage`: which source files no spec specifically claims (spec 029; non-fatal, exit 2 if the index is stale)
   - `spec-spine index diagnostics`: the unresolved-unit diagnostics the committed index records (spec 044; non-fatal, empty output means none)
   - `spec-spine registry status-report --json --nonzero-only`: lifecycle counts per status
   - `spec-spine registry plan`: the ready set (spec 035): which specs can be worked on now and what blocks the rest; `(nothing ready)` in a finished corpus
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

**Read discipline:** the init protocol MUST NOT parse `.statecraft/derived/**/*.json` directly (no `python`, `jq`, `awk`, `sed` against compiled artifacts). All structural and lifecycle data comes from the `spec-spine` subcommands (`registry`, `index`) and the rendered markdown view. See **Governed artifact reads** above.

**Unresolved units:** `spec-spine index diagnostics` (spec 044) lists the `W-001` / `W-002` diagnostics the committed index records: a unit an owning spec claims that does not resolve yet. Empty output means none, which is the state a finished corpus is in. Report the count, and name the specs when there are any: a spec under way legitimately claims territory it has not written yet (specs 023 and 044), so these are work in flight, not defects. The gate half is `check --fail-on-unresolved`, which this repository's CI runs.

**Freshness:** spec-spine **commits** its compiled artifacts. Since spec 022 both views are committed as per-unit shard trees: `.statecraft/derived/spec-registry/by-spec/<id>.json` and `.statecraft/derived/codebase-index/{by-spec,by-package}/*.json` are tracked (only `.statecraft/derived/**/build-meta.json` is gitignored; no monolithic `registry.json`/`index.json` is committed). The committed shard set is the reference for lifecycle queries, so `/prime` has to know whether it is current.

`spec-spine check` (spec 062) asks about both trees in one call. It compiles in memory and compares against the committed shards **without writing**, and it reports each tree separately: the registry half and the index half each keep the structure their own primitive emits, so the drifted shard names are still there to read back. Its exit code is the more severe of the two, in this order: **`3` then `1` then `2` then `0`**. Read it, and read the two report lines under it, because one code covers two trees:

- **`0` (both fresh):** the committed shards are exactly what the corpus compiles to, so the lifecycle counts below reflect the current `specs/*/spec.md` frontmatter. Report nothing.
- **`2` (stale):** *check the `--version` read from step 1 before believing it* (see **Stale binary** below): a binary predating the verb cannot be reporting drift. For a genuine staleness report, say **which tree** the output named, report "Spec registry: stale, run `spec-spine compile` and commit" or "Codebase index: stale, run `spec-spine index`" accordingly, **and name the drifted shards from its stderr**, then continue. The lifecycle counts come from the committed ledger and are therefore the stale ones; say so rather than presenting them as current.
- **`1` (validation failed, or unresolved units refused):** with `--fail-on-unresolved` this code also covers a refused unresolved-unit diagnostic, so read the report lines to tell the two apart. If the corpus itself fails validation, surface the violations and report the lifecycle counts as **unverified**: they still come from the committed ledger, but with the corpus failing validation there is no way to say whether that ledger corresponds to it. Fixing the violations is the first task of the session, not an aside. This outranks `2` because staleness is not meaningful against a corpus that does not validate.
- **`3` (I/O / parse / schema / config):** a read that could not be performed has not answered. Treat freshness as unknown for **both** trees, report stderr verbatim, and continue. Never report "fresh" for an exit code you did not recognize.

If the index is not built and `render` fails, report "Codebase index: not built" and continue without structural counts.

The counts are formatted in step 2, after every parallel read has returned, so the verdict is always in hand before the numbers are written down.

**Stale binary:** `target/release/spec-spine` is whatever was last built, which is not necessarily this checkout.

**Ask `spec-spine --version` before believing any exit code.** Every binary ever released answers it, and it exits 0. If the version predates the flag you are about to pass, rebuild (`cargo build --release -p spec-spine-cli`) or reinstall; do not interpret the exit code of a flag the binary does not have. Rebuilding is cheap and is the right reflex whenever the binary predates recent commits.

That precondition replaces an older ritual of matching clap's English on stderr, which was pinned to a dependency's message format and could not survive a clap release. Since spec 093 a new binary maps every usage error to **exit 3**, so exit 2 from any verb means staleness and nothing else; but the binary that reports the wrong code is by definition the old one, so a procedure that may be talking to an old binary cannot rely on the new behavior. Where a repository sets `[meta] required_version` (spec 055), the check happens on every run and this manual step is unnecessary.

Do **not** substitute a plain `spec-spine compile` or `spec-spine index` here. Writing would repair the tree as a side effect of reading it, which hides the fact that the *committed* copy was stale: the drift then looks like an uncommitted local edit instead of a defect on the branch (this is exactly how the spec 016/021 drift reached the default branch unnoticed). `/prime` reports; it does not silently mutate. `spec-spine check` carries the same never-writes contract, which is why the protocol can call it.

**Binary missing or stale:** if the `spec-spine` binary is not built, or predates the commits in this checkout, run `cargo build --release -p spec-spine-cli` and continue (see **Stale binary** above for why a stale one misreports freshness). Do NOT fall back to ad-hoc parsing of `.statecraft/derived/**`.

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
   changes, without editing that spec (spec 037).
2. **Branch.** A feature branch named after the spec id. Never commit to
   `main`. `/build <id>` sequences steps 2 to 5 for a spec that is already
   filed; here a filed spec stays `draft` while it is built, so `/build`'s
   preflight accepts `draft` in this repository when a human named the id.
3. **Re-read the design before coding.** If the design is imprecise, record
   the choice in the spec. If it is wrong, stop and report; never rewrite an
   approved spec to match code (**Adversarial prompt refusal** above).
4. **Implement within the territory.** Claim every new file in the new spec's
   `establishes` (or a `// Spec:` header when the file already has an
   owner). Touching a unit another spec owns is an `extends` edge on that
   unit. Never edit `.statecraft/derived/` by hand.
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
   `origin/main` (spec 093). Set `$SPEC_SPINE_DEFAULT_BRANCH` to override
   the branch the push gate protects and the root `Makefile` compares against.
   `make gate` runs exactly this list, read-only, and CI calls that target
   rather than restating the chain (spec 092 3.6).

   then the stack's own gate: `cargo test --workspace --locked`,
   `cargo clippy --workspace --all-targets --locked -- -D warnings`,
   `cargo fmt --all --check`. Commit the regenerated shards with the code
   they describe. The skills call this list "the gate as `AGENTS.md` lists
   it", and `harness_skills.rs` asserts each skill's inlined floor is a subset
   of it (spec 093), so a step added here reaches every skill. The binary
   is `target/release/spec-spine` (or `cargo run -p spec-spine-cli --`),
   never `npx spec-spine`.

   Every step above is enforced by CI's `self_governance` job. CI runs
   `check` in place of `compile` and `index`, because a gate must never
   repair the tree it is judging.
6. **Ship, then ratify.** `/verify <id>` runs the spec's `## Verification`
   block through `spec-spine verify <id>` (spec 043). `/ship` opens the PR with
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

Commands live in `.claude/skills/` (one `SKILL.md` per folder): the ten specs
093 and 081 settled on. Since spec 092 they are **this repository's own**
development instruction rather than a set it distributes, and they stay until
Statecraft's global delivery concretely replaces them. The project layer lives
in this file, not in the skills.

The governed loop, in the order "Working the backlog" runs it:

- `/prime`: prime a session (this protocol)
- `/setup`: one-time contributor setup: build the `spec-spine` binary and verify the governed loop
- `/next`: name the next work order from `registry plan`, minus drafts, with in-flight specs and blockers. Read-only
- `/build <id>`: implement one spec start to finish: preflight, branch, flip, implement, gate, verify, flip complete
- `/verify <id>`: run the spec's `## Verification` block locally through `spec-spine verify <id>`
- `/ship`: gate, review, commit on the feature branch, open the PR
- `/shepherd`: watch the PR's checks by head sha, remediate through the gate, merge, confirm on disk
- `/spec`: author a new spec at the next free ordinal, born `draft`

The skills the loop calls:

- `/commit`: create a git commit with an impact-focused conventional message, spec ordinal as scope
- `/code-review`: review the working diff for correctness bugs, spec drift, and illegitimate mid-build spec edits

## Conventions

- Items added to the "New Sessions" session protocol are auto-loaded by `/prime`.
- Agents must be self-contained within `.claude/agents/`: no cross-project dependencies.
- Orchestrated workflows must read compiled artifacts (`.statecraft/derived/**`) through the `spec-spine` binary, never via ad-hoc parsers: see **Governed artifact reads** above.
- Self-governance runs through the in-tree binary (`target/release/spec-spine`), not the published npm/py distributions.
