# 08: What the realignment and the collapse left (2026-09-20)

A design note, not a spec. It is the handoff from the session that built specs
092 to 096: what is done, what is left, what each remaining item is blocked on,
and the order that makes sense.

Read `docs/corpus-map.md` first if you arrived holding a spec number from
before 2026-09-20. The corpus was renumbered; a bare old ordinal now resolves,
to a different document.

**Updated 2026-09-21.** Both handoff branches are merged and ratified; the
items this note scheduled are struck through where they are done, with what
closed them. What remains open is §2.2, §2.4, §2.6, §2.7 and §2.8.

## 1. What is done

The two handoff branches (`120-...` and `121-...`, the pre-renumber names) are
merged. The corpus is 100 specs, `000` to `099`, all implemented; `registry
plan` reports nothing ready and nothing blocked. Specs 098 and 099 are `draft`
with their builds merged, which is this repository's normal state between a
build PR and its ratification PR.

- `spec-spine init`, `--with-kit`, `kit/`, `kit_embedded.rs`, both kit
  generators, `.agents/`, `.codex/` and `website/` are **gone**.
- The library keeps one producer, `scaffold_init_json`, seven governance files,
  pure. That contract is frozen for the Statecraft CLI.
- The derived trees are at `.statecraft/derived/`; `state_dir` is
  `.statecraft/state`. The engine treats the **configured** derived root as
  derived in the bypass floor and in both walks.
- The gate is the root `Makefile`; CI calls it on both event legs.
- 27 specs removed, 21 of them merged into two; 93 survivors renumbered; the
  corpus is 97 specs, contiguous `000` to `096`.

## 2. What is left, in order

### 2.1 ~~Decide the merge, then ratify~~ done

Specs 092 through 097 are merged, built and `approved`. Ratification of 098 and
099 is still owed, as a PR separate from the build, and is a human's flip.

The two acceptance blocks recorded below as red by construction are green now
that the work is on the default branch; the paragraph is kept because it names
the pattern, which recurs on every wave branch.

Two acceptance blocks are red **by construction** until this lands on the
default branch, and both are recorded rather than repaired:

- spec 081 runs the coupling gate against the default branch, which on a branch
  is not empty;
- spec 089 runs the sweep against `origin/main` with the built-in ledger, and
  the ledger this change writes describes a corpus that exists only here.

Spec 096's block is red because 096 is filed and unbuilt, which is the normal
transient state of a draft here.

### 2.2 `.claude/`: the last tree, and what has to exist before it goes

Spec 092 §3.5 and spec 093 keep `.claude/` as **this repository's own**
development instruction, not as anything it distributes, and name the condition
for removing it: Statecraft's global delivery being concretely available.

**The answer, as of 2026-09-21, is that complete removal is possible and
nothing here forecloses it.** The mechanism exists on the other side and the
ordering is agreed by both corpora. What is in the way is work, in a fixed
order, most of it not this repository's.

**The counterparty has recorded its half.** Statecraft's spec 002 (approved,
`implementation: in-progress`) carries it: §3.14 is the delivery mechanism, one
content-addressed harness under the product home, adapters that point at it,
and **no copy in any repository**; §3.22 records this repository's state at the
handover and fixes the order; §3.23 fixes what delivered content must satisfy,
including the ten skills, the four agents, the three skill assertions and the
seven hook contracts. Read those three sections before planning anything here:
they are the requirement, and this note is only the local view of it.

Three of §3.14's four adapter rules answer the objections a global harness
raises. Every delivered name is Statecraft-namespaced, so it cannot collide
with a generic user skill. Every delivered behavior is **gated to Statecraft
projects**, applying only inside a repository holding
`.statecraft/environment.json` and inert everywhere else, which is what stops a
global `/ship` firing in an unrelated tree. And writing into a native location
such as `~/.claude/` is an explicit operator action under `home apply`, never a
side effect.

What is left in the way, so the decision is made against facts rather than
tidiness:

1. **The delivery does not exist yet.** Statecraft spec 002 is `in-progress`
   and no `.statecraft/environment.json` exists in this repository, so this
   repository is not an enrolled Statecraft project and would receive nothing
   from a gated global harness today. Enrolling it is a prerequisite step, not
   a consequence.
2. **The assertions do not survive the move on their own.** This is the blocker
   with a price tag, and both corpora state it: `harness_hooks.rs` (1,124
   lines, which extracts each hook body and runs it as a program over a matrix
   of command spellings, branch names and derived-tree states) and
   `harness_skills.rs` (about 800) read `.claude/` out of the repository with
   `include_str!`. A hermetic test cannot read `$HOME`, and a governed artifact
   here is a pure function of `(Config, file contents)` of **this** tree, so
   neither suite can follow the files to a global home. They are reimplemented
   where the files land, or the seven hook contracts and the three skill
   assertions become unenforced. Statecraft §3.23 accepts the cost and calls it
   not optional; until it is paid, removal trades live enforcement for tidiness.
3. **A session working here loads `.claude/skills/` for the governed loop**
   (`/prime`, `/next`, `/build`, `/verify`, `/ship`, `/shepherd`, `/spec`,
   `/commit`, `/code-review`, `/setup`). Removing the tree before the global
   harness reaches a session here leaves the loop unavailable. `AGENTS.md`
   still carries the protocol, the four rules and the gate list, so a session
   would be instructed rather than instruction-less, but it would have no
   commands, and `/setup` is where "install Statecraft" would have to be said.
4. **The push gate is already global and already proven.**
   `~/.claude/hooks/push-gate.sh`, registered as a `PreToolUse(Bash)` hook in
   `~/.claude/settings.json`, protects every repository on this machine rather
   than only this one. It was **copied, not moved**, for reason 2: moving it
   would have deleted its only assertions while `.claude/` stayed. When the
   tree goes, the project copy goes with it. It is repository-agnostic git
   policy, needing `git` and `jq` and knowing nothing about spec-spine, which
   is why it could go first; the PR gate and the two session hooks are
   spec-spine-specific and cannot.
5. ~~`.claude/rules/`~~ **done, 2026-09-20.** The four rules are `AGENTS.md`'s
   `## Rules` section (spec 093 D-4) and the directory is gone. This was the
   one class that could never have relocated to a global home: they are this
   repository's own governance and under `~/.claude/rules/` they would bind
   every project the user opens. What it gave up is Claude Code's
   auto-discovery of `.claude/rules/*.md`; a session meets them at `/prime`
   instead of at startup.
6. **The corpus surgery this repository owes**, all of it precedented and none
   of it hard, but it belongs in the one change that removes the tree:
   - five specs carry `.claude/**` units (061, 062, 064, 092, 093) and one
     carries a `references` to a skill file (053). There is no edge that
     withdraws a claim, so this is a direct frontmatter edit under a named
     authority, which is exactly the instrument spec 092 D-3 used for 29
     predecessors;
   - three `[index] extra_hashed_inputs` globs (`.claude/settings.json`,
     `.claude/skills/*/SKILL.md`, `.claude/agents/*.md`) go in the same change.
     A glob matching nothing is an `L-010` warning and `lint --fail-on-warn` is
     in the gate, so leaving them is not an option; removing them restales
     every shard, which is a regeneration and not a problem;
   - `harness_hooks.rs` and `harness_skills.rs` are deleted here, and spec 093's
     §3 and §5 lose their subject. 093 is a sixteen-spec consolidation, so what
     goes is its harness half, not the document;
   - `AGENTS.md` names `.claude/agents/` and `.claude/skills/` as locations in
     four places, and those sentences become wrong the moment the tree moves.
7. **`AGENTS.md` is not part of this question.** It is the cross-agent protocol
   and the project layer, established by spec 093 §4.12, and it stays whatever
   happens to `.claude/`, unless Statecraft takes it through
   `.statecraft/AGENTS.md` and the bridge (§2.6).

So the order is fixed, and both corpora say the same thing about it: Statecraft
delivers the global harness with a Claude Code adapter and reimplements the
assertions; this repository is enrolled and confirms a session here still has
its loop and its hooks; **then** `.claude/` goes, with spec 093's harness half
superseded in the same change. Doing it in the other order is the failure spec
092 §3.5 was written to avoid, and no schedule pressure converts one order into
the other.

### 2.3 ~~Build spec 096~~ done

`spec-spine compact` is built and 096 is `approved` / `complete`.

### 2.4 File the `couple` base-side ownership read

Spec 092 §3.11.1 measures it and D-11 records why it was not fixed there: the
coupling gate resolves a **deleted** path's owners from the head index, where
the claim the same change withdrew no longer exists, so a correct removal is
reported as `C-001` against the package floor. Fixing it needs the gate to hold
two indexes, which `couple_with` does not take. Unfiled.

### 2.5 ~~The vacuous-acceptance audit~~ done, 2026-09-21

Spec 092 §3.12's third state: a block that passes only because the file it
asserts about is gone. Read rather than swept, because spec 089's sweep reports
the exit code and cannot tell a vacuous green from a real one.

The audit is small and its result is recorded in 092 §3.12: exactly **one live
block** carried vacuous lines, spec 061's three. Thirteen specs' own blocks are
dead text whose acceptance has moved to an amender (`verify <id> --plan --json`
names the holder in `acceptanceFrom`), and a line in a block that never runs
asserts nothing either way. A vacuous line whose subject survives under another
name was corrected to read the surviving file, with an existence line so the
negative cannot go vacuous again; one whose subject is gone was removed with
the reason kept.

### 2.6 The instruction bridge

`@.statecraft/AGENTS.md` as the first line of the root `AGENTS.md`. Blocked on
the Statecraft initializer writing `.statecraft/AGENTS.md`: an import of a file
that does not exist is a broken instruction, which is why spec 092 §3.10
refuses to add it by hand.

Statecraft spec 002 §3.22 reports the condition as satisfiable on its side
today: §3.13 is the rule and §3.17's initialization flow writes the file, so
what is left is running it here, which is also step 1 of §2.2.

### 2.7 Where the documentation goes

`website/` was deleted, not moved. Its content is in git history at that
branch's parent. Someone has to decide what of it Statecraft recreates, and
where. Two facts that existed **only** on the site were rescued into
`docs/adoption-guide.md` and `docs/api.md` before the deletion; the rest is
unexamined.

### 2.8 Adopters

Four repositories pin releases and copied the kit. Nothing is released yet, so
this is pre-release work: the migration note has to say that `init` and the kit
are gone, that the harness comes from Statecraft, and that spec ids moved, with
`docs/corpus-map.md` as the reference.

## 3. Known residue that is not a defect

- **Prose citing removed documents.** Spec 074's prose names two `website/`
  files. An approved spec's prose is a record of what was true when it was
  ratified and is not edited (spec 037). The map resolves the ids; the paths are
  history.
- **`scripts/verify-spec.sh`** is the script `spec-spine verify` absorbed. It is
  still here and still claimed; no skill calls it, and `harness_skills.rs`
  asserts that.
- **`.statecraft/derived/attestation/`** accumulates on-demand output. 103 files
  for spec ids the corpus no longer has were removed in this session; it will
  accumulate again and is gitignored.

## 4. What must not be undone

- The producer contract (`scaffold_init_json`, its name, its response shape, its
  purity). The Statecraft CLI is implementing against it.
- The exception in spec 095 was **one change**. Spec 037 governs again: an
  amendment is declared once, in the amending spec, and the amended file is not
  edited. A future removal takes the ordinary path or argues for its own
  exception.
- `docs/corpus-map.md`. It is the only thing in the tree that can resolve a
  pre-collapse spec id, and every citation outside this repository still uses
  them.
