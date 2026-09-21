# 08: What the realignment and the collapse left (2026-09-20)

A design note, not a spec. It is the handoff from the session that built specs
092 to 096: what is done, what is left, what each remaining item is blocked on,
and the order that makes sense.

Read `docs/corpus-map.md` first if you arrived holding a spec number from
before 2026-09-20. The corpus was renumbered; a bare old ordinal now resolves,
to a different document.

## 1. What is done

Two branches, neither merged.

| Branch | Head at handoff | Subject |
|---|---|---|
| `120-the-engine-ships-governance-not-an-environment` | `e4e7050` | the Statecraft realignment (spec 092, filed there as 120) |
| `121-the-corpus-describes-what-exists` | see `git log` | the collapse, the renumber, the website removal, spec 096 |

Branch 2 contains branch 1. Everything below assumes branch 2.

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

### 2.1 Decide the merge, then ratify (human, blocks everything else)

Specs 092, 093, 094, 095 are `draft` and 096 is `draft` and unbuilt. This
repository ratifies in a PR separate from the build.

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

What is actually in the way, so the decision is made against facts rather than
tidiness:

1. A session working here loads `.claude/skills/` for the governed loop:
   `/prime`, `/next`, `/build`, `/verify`, `/ship`, `/shepherd`, `/spec`,
   `/commit`, `/code-review`, `/setup`. Removing the tree with nothing in its
   place leaves no development instruction at all.
2. `.claude/settings.json` carries four hooks that are live enforcement: the
   push gate, the PR gate, and the two session hooks that read `check`'s
   verdict. They are asserted by `tests/harness_hooks.rs` against the file on
   disk. A global replacement has to carry them or the enforcement goes with the
   tree.
   **Partly discharged, 2026-09-20.** The push gate is repository-agnostic git
   policy: it needs `git` and `jq` and knows nothing about spec-spine. It is now
   installed globally at `~/.claude/hooks/push-gate.sh`, registered as a
   `PreToolUse(Bash)` hook in `~/.claude/settings.json`, so every repository on
   this machine is protected rather than only this one. It was **copied, not
   moved**: `harness_hooks.rs` exercises that body as a program over a matrix of
   command spellings and branch names, and a test cannot read `$HOME` and stay
   hermetic. Moving it today would delete the only assertions this gate has
   while `.claude/` itself stays, and buy nothing. The two run in sequence and
   refuse identically; when the tree finally goes, the project copy goes with it
   and the global one is already in place and already proven. The PR gate and
   the two session hooks are spec-spine-specific and stay until Statecraft
   delivers.

3. ~~`.claude/rules/`~~ **done, 2026-09-20.** The four rules are now
   `AGENTS.md`'s `## Rules` section (spec 093 D-4) and the directory is gone.
   This was the one class that could never have relocated to a global home: the
   rules are spec-spine governance, and under `~/.claude/rules/` they would bind
   every project the user opens. Folding them into the cross-agent protocol
   removed them from `.claude/` without moving them anywhere. What it gave up is
   Claude Code's auto-discovery of `.claude/rules/*.md`: a session now meets
   them at `/prime` instead of at startup.
4. `.claude/settings.json`, `.claude/agents/*.md` and `.claude/skills/*/SKILL.md` are hashed inputs
   (`spec-spine.toml [index] extra_hashed_inputs`). Removing them restales every
   shard, which is a regeneration, not a problem, but it belongs in the same
   change.
5. `AGENTS.md` is **not** part of this question. It is the cross-agent protocol
   and the project layer, established by spec 093 §4.12, and it stays whatever
   happens to `.claude/`, unless Statecraft takes it through
   `.statecraft/AGENTS.md` and the bridge (§2.6).

So the order is: Statecraft delivers the harness globally with a Claude Code
adapter, this repository verifies a session still has its loop and its hooks,
and only then does `.claude/` go, with spec 093 superseded in the same change.
Doing it in the other order is the failure spec 092 §3.5 was written to avoid.

### 2.3 Build spec 096

`spec-spine compact`. The spec is filed with the design worked out from five
measured defects; its acceptance names each of them. This is ordinary
`/build` work and is the only item here that is ready now.

### 2.4 File the `couple` base-side ownership read

Spec 092 §3.11.1 measures it and D-11 records why it was not fixed there: the
coupling gate resolves a **deleted** path's owners from the head index, where
the claim the same change withdrew no longer exists, so a correct removal is
reported as `C-001` against the package floor. Fixing it needs the gate to hold
two indexes, which `couple_with` does not take. Unfiled.

### 2.5 The vacuous-acceptance audit

Spec 092 §3.12's third state: a block that passes only because the file it
asserts about is gone. The two known instances went with the collapse, but the
audit was never done. Spec 089's sweep reports the exit code and cannot tell a
vacuous green from a real one; this needs reading.

### 2.6 The instruction bridge

`@.statecraft/AGENTS.md` as the first line of the root `AGENTS.md`. Blocked on
the Statecraft initializer writing `.statecraft/AGENTS.md`: an import of a file
that does not exist is a broken instruction, which is why spec 092 §3.10
refuses to add it by hand.

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
