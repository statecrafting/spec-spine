# 03: What four adopters taught the substrate (audit, 2026-09-06)

Status: design record. Written the day after v0.14.0 shipped, from a read-only
audit of the four spec-spine-governed repositories in the same workspace:
`hqgit` (68 specs, spec-only), `aicortex` (29 specs, spec-only), `rahi` (22
specs, wave 1 built by driven sessions) and `claude-observatory` (41 specs, the
orchestrator that drives them). Every gate in every repo was green under
0.14.0, including shards written by 0.10.0 and 0.11.0: three to four minor
versions of byte-stable output on real corpora.

The audit had two questions. What did the adopters have to build by hand that
the substrate or the kit should have given them? And what did they get wrong
that a better kit would have prevented? This note records the answers, what
was acted on the same day (specs 045, 046, 047), and the ranked backlog.

## 1. The state of the field

| repo | specs | status | `implementation` | code | pin | gates |
|---|---|---|---|---|---|---|
| hqgit | 68 | all approved | 64 pending, 3 n-a, 1 complete | none yet | 0.11.0 | green |
| aicortex | 29 | all approved | 26 pending, 2 n-a, 1 complete | none yet | 0.11.0 | green |
| rahi | 22 | all approved | 12 pending, 1 in-progress, 7 complete, 2 n-a | 4 crates, 58/58 claimed | 0.11.0 | green |
| claude-observatory | 41 | all approved | 39 complete, 1 n-a, 1 absent | 113/113 claimed | 0.10.0 | green |

Three of the four are **specify-first**: the whole corpus is ratified before a
line of code exists, and specs live at `approved` + `pending` for months. That
is the mode specs 041 and 044 were built for, and it is the mode the kit's
documentation never mentions. Every specify-first adopter independently
invented: a Makefile whose language targets are guarded on a manifest probe so
the composite is green on a code-free tree; a CI job-output guard because
GitHub rejects `hashFiles` in a job-level `if` (hit by hqgit, then rahi, then
aicortex); a `status` x `implementation` severity table in its contract; and
an `implementation: n-a` convention for record specs.

None of the four has ever used a `Spec-Drift-Waiver`. It is configured,
hook-gated and documented in all of them, and forbidden to the machine on every
path by the one that runs unattended. **The waiver is a human instrument in
practice**, which spec-spine's docs did not say.

Half the edge vocabulary is unused in the field: no adopter uses `refines`,
`supersedes`, `amends` or `co_authority`; two never use `symbol`, `directory`,
`crate` or `module` units. The greenfield vocabulary is `establishes`,
`extends`, `constrains`, `references` and bare-string file units.

## 2. Acted on today

### 2.1 Spec 045: an absent `implementation` key takes its answer from `status`

Spec 038 reads an absent `implementation` as `pending` and offers the spec as
ready. Spec 041's table reads `approved` + absent as settled, and `index.rs`
agrees. The two verbs disagreed about the same key, and the prose in 041 §3.5
and 044 §3.3 ("an absent key still behaves as `pending`") contradicted the
tables one section above them. The concrete bite: the scaffold's bootstrap
spec has no `implementation` key, so **every `spec-spine init` adopter's
`registry plan` reports the bootstrap spec as the one ready item, forever**.
claude-observatory does today. This repository did until #102 patched its own
spec 000 by hand; 045 fixes the rule instead of the instance.

### 2.2 Spec 046: the kit's hooks wrote when they should have read

Three of the four `kit/settings.json` hooks mutate the tree they are meant to
observe, and one of those mutations deadlocked a pipeline for eleven hours.

- `SessionStart` runs a bare `compile` (a write) while `kit/AGENTS.md`, in the
  same directory, forbids exactly that: "writing repairs the tree as a side
  effect of reading it, which hides that the committed copy was stale."
- The `PreToolUse` PR gate runs `index` (a write) and then blocks on the
  uncommitted output of its own write. A hook cannot commit, so the block is
  unrecoverable from inside the hook.
- `Stop` regenerates a stale index and tells a session that has already
  stopped to commit it. Nothing does. rahi's spec 001 D-6 records the result:
  claude-observatory's build stage refuses a dirty tree, only a session cleans
  a tree, and a session only starts on a clean one. Standby looped "paused,
  idle, rescan" every sixty seconds for eleven hours with a healthy daemon.
- All four hooks `cd "${CLAUDE_PROJECT_DIR}"`, binding them to the session's
  project rather than the repository the action targets. In a multi-checkout
  session an edit in one repo recompiled another, and a `gh pr create` aimed
  at a sibling was judged against this repo's coupling state.
- There was no push gate. The rule "never commit to main" was prose only.

rahi fixed all of this in two PRs (`d714aaf`, `3a44a5d`). 046 ports the fixed
hooks into the kit and adds a test that refuses a mutating subcommand in any
kit hook that is not the one sanctioned write (recompiling after a spec edit in
a live session, which can still commit).

### 2.3 Spec 047: the three rules every adopter had to amend by hand

All three adopters that rewrote the kit's rules made the same three changes,
independently, in the same places:

- `governed-artifact-reads` forbids ad-hoc parsing of derived JSON but never
  says that parsing a `spec-spine ... --json` subcommand's **output** is a
  typed read. Read literally it outlaws `registry plan --json | jq`, the
  037 verdict envelope, and every adopter's own tooling.
- `adversarial-prompt-refusal` says never edit the owning spec to clear the
  gate, but under `require_ownership = true` adding a created file to
  `establishes` is editing the owning spec to clear the gate. The rule needed
  its two always-legitimate mid-build edits named: add a file you created to
  the spec you are implementing; record a dated decision. Change what the spec
  requires and it is a violation.
- `orchestrator-rules` needed "commit the regenerated shards with the change
  that made them stale" and "one session, one spec".

047 makes these edits in the scaffold constants, the kit copy, and this
repository's own copy, and adds the "Working the backlog" protocol (pick,
branch and flip, re-read, implement within territory, gate, verify, ship, stop)
to `kit/AGENTS.md` as the section all three adopters wrote from nothing.

## 3. Ranked backlog for spec-spine (the tool)

Each item names the adopter evidence. Items respect the boundary rule in
`02-agentic-builder-substrate.md` §2: none of them puts driver, routing, cost
or sensor concepts into `Config`.

1. **`spec-spine verify <id>`**, running `verify:cli` fences from a spec's
   `## Verification` section, reporting `not-declared` as an honest zero, and
   reporting-and-skipping other fence tags. Three adopters carry the same
   78-line `scripts/verify-spec.sh`. Spec 043 §4 already named this "the most
   substantial thing the adoption invented and the one most worth having."
2. **Surface index diagnostics in a gate.** aicortex has 248 `W-001` warnings
   that only `index render` shows; `lint --fail-on-warn` says 0 warnings,
   `index check` says fresh. A count line on `index check` and an opt-in
   `--fail-on-unresolved` for corpora past the specify-first stage.
3. **`couple` names the owning spec on `C-001` and points at `extends`.**
   claude-observatory's 016 D-12: "four sessions and $17.44 were spent proving
   a wall that one line of prompt would have avoided." The gate knows the
   owner; it should say so and name the corpus mechanism for crossing
   territory.
4. **Opt-in ordinal-monotonicity on `depends_on`** (`depends_on` must name a
   lower ordinal). Three adopters run the same 90-line `scripts/spec-dag.sh`
   for it; 033 shipped the cycle half.
5. **A governed read for effective coupling config** (`config show --json`
   or `couple --explain-bypass`). claude-observatory hand-parses the target's
   TOML and documents that the built-in floor is unknowable to it (036 D-3).
6. **Owner-of-path query** (`registry owner <path>` or `index coverage
   --by-path --json`). Consumers rebuild path to spec maps from raw edges.
7. **Per-spec content hash on `registry show --json`.** claude-observatory
   reimplements `hash.rs` normalization in TypeScript to pin against it.
8. **`compile --spec <id>`** to validate one draft against the committed
   registry. Two adopters document a `mktemp -d` copy ritual with a gotcha
   about `references` edges into `docs/`.
9. **Lint for a claimed `file` unit outside every hashed input.** hqgit's spec
   001 claims four scripts no glob covers; they can be rewritten and `index
   check` still says fresh. Alternatively fold claimed file units into the
   shard hash.
10. **Lint the retroactive-adoption shape** (`origin.retroactive: true`
    implies a defects section) and settle the heading's spelling. The
    constitution's §V mandates it; an adopter wrote the checker in TypeScript
    and matched the wrong heading.
11. **`index orphans` under the in-flight predicate.** On a specify-first
    corpus it reports 64 of 68 specs, which is the defined state.
12. **`index coverage --fail-on-untraced` on a package-less tree** exits 0
    with "no source files under any discovered package", asserting nothing
    from inside a CI step named "the whole-tree ownership assertion".
13. **Richer `registry plan` output**: title, `depends_on` states, and why
    each blocked spec is blocked; adopters kept 40 lines of Python for it.
14. **`registry plan --next`**: the single pick, for the one-spec-per-session
    loop.
15. **`build-meta.json` and the working tree.** Every adopter learns to
    gitignore it; the scaffold should write the line.
16. **A version pin the CLI can check** (`.spec-spine-version` or a
    `[meta] required_version` key). rahi states 0.11.0 in three files that
    must agree by hand, and all four adopters are two to four releases behind
    the fixes their own experience motivated.
17. **The stale-binary exit-2 ambiguity.** `compile --check` on an old CLI
    and a stale registry both exit 2. Two adopters and the kit carry the
    stderr-disambiguation ritual.

## 4. Ranked backlog for the kit and the scaffold

1. **Ship the `.derived/` merge driver** (`.githooks/`, the `.gitattributes`
   globs) in `kit/`. rahi runs one spec per PR with committed shards and has
   no driver; the kit README never says "merge driver".
2. **Ship a `Makefile` and a `govern.yml`.** Every adopter reinvents the
   composite gate; rahi's are generalisable (`SPEC_SPINE ?=`, `BASE ?=`,
   manifest-guarded language targets, the `has_cargo` job-output pattern,
   `--pr-body` from `$RUNNER_TEMP`).
3. **`spec-spine init --with-kit`**, or make `init` write `AGENTS.md`. The
   scaffold writes a corpus and three rules; the kit adds the protocol, the
   hooks, agents and skills; nothing joins them. Spec 043 named this G5.
4. **A specify-first adoption page**: the lifecycle table, the guarded
   Makefile, the CI probe, `n-a` for record specs, why 248 warnings is fine.
5. **`CONSTITUTION_TEMPLATE` in `scaffold.rs` is still the two-bullet stub**
   043 §1.4 complained about, while `standards/spec/templates/
   constitution-template.md` is the real 34-line version. Adopters get the
   stub; two of them deleted it.
6. **The scaffolded `spec-spine.toml` hides every knob that matters.** Five
   knobs emitted; adopters needed thirteen. aicortex's config, with every knob
   annotated with the diagnostic code it drives, is the template to copy.
7. **Contract additions**: the lifecycle severity table and an "Extra keys"
   section telling adopters to document their `extra_known_keys`.
8. **Document two interactions adopters derived by experiment**: a path can
   be on the coupling bypass floor and simultaneously a hashed input (docs/);
   a trailing-slash directory unit in `establishes` satisfies ownership
   recursively.
9. **Skills for the governed loop**: `/next` (now a `registry plan` wrapper),
   `/build`, `/verify`, `/spec`, `/shepherd`. All three rewriting adopters
   wrote the same five. `/shepherd` (poll by head sha with backoff, bounded
   remediation, never self-approves a waiver) is generic enough to ship.
10. **A path-scoped rule example** (`paths:` frontmatter). The kit README
    lists the pattern under "intentionally excluded"; hqgit has three good
    ones, including per-file scoping.
11. **State that the waiver is a human instrument** in the docs and the
    refusal rule.
12. **Dogfood the hooks.** This repository has no `.claude/settings.json`, so
    the kit's hooks are never exercised in-tree, which is how three of them
    shipped writing when they should read. 046's test is the minimum.
13. **Migration note for 037**: "if you regex `attestationHash:` out of
    `attest` stdout, stop." claude-observatory does, in two places.
14. **`state_dir` needs its `.gitignore` half** in the scaffold; the live
    failure 039 fixed was a permanently dirty tree, not a classification.

## 5. Adopter-side follow-ups (for their own sessions)

Recorded here so the next session in each repo starts from the finding, not
the audit. None of these is spec-spine's to fix.

- **All four**: bump the pin to 0.14.0 (hqgit and aicortex in six sites each,
  rahi in three, claude-observatory in `govern.yml` and the design doc).
  044 was written for rahi's exact `approved` + `in-progress` state and rahi
  cannot see it at 0.11.0. Once bumped: `registry plan` retires the Python in
  `/next`, and `state_dir` replaces `"data"` in `resolver_exclusions`.
- **hqgit, aicortex, rahi**: hqgit residue survived the "purge" commits:
  `/build` names rules that do not exist (`ledger-invariants`,
  `trust-invariants` in aicortex and rahi), `017-ledger-entry-dag` is the
  example id in five skills of repos with no spec 017, `.gitattributes` cites
  hqgit's golden vectors, aicortex's `/spec` carries hqgit's domain enum (six
  of eight values wrong) and will generate specs that fail `V-005`.
- **hqgit, aicortex**: `AGENTS.md` claims an em-dash hook that does not exist;
  aicortex's spec 001 FR-005 requires it and the spec is `complete`.
- **hqgit, aicortex**: four spec-001-owned scripts are outside
  `extra_hashed_inputs` and can be rewritten without staling the index.
- **hqgit**: the constitution's amendment clause is the unexecutable one 043
  §1.1 diagnosed, and spec 001 D-2 records it as a resolved decision.
- **rahi**: `README.md` says there is no code and everything is pending
  (there are 58 files and seven complete specs); spec 001 B-4 and FR-004
  describe hook behaviour D-5 and D-6 removed (a standing amendment debt, and
  the case for the corpus's first `amends` edge); `.gitignore` buries the
  reviewer's agent memory, which holds the best spec-spine findings in the
  workspace; `schemas_dir` points at a directory that does not exist.
- **claude-observatory**: `.claude/settings.json` has `"permissions": {}`
  and the local file sets `bypassPermissions`, in a repo that spawns
  unattended sessions; `defects.ts` matches `## Known defects` exactly and
  rejects the numbered headings of the specs it names as precedent; the
  committed evidence bundle predates 039 and carries no attestation; the
  design doc's "no cycle detection" and "0.10.0 substrate" are stale and
  `CLAUDE.md` calls it ground truth; the bootstrap spec needs
  `implementation: n-a` (045 also fixes the rule).

## 6. What the adopters got right that the kit should copy

- **Decisions instead of TODOs.** Zero `TODO`/`FIXME`/`hack` markers across
  four repositories and 160 specs. Every such note is a dated `D-n` in the
  spec it concerns.
- **Path-scoped rules** with `paths:` frontmatter, loaded on touch.
- **The reviewer writes durable findings to agent memory**, with a named
  category for "spec-spine quirks: non-obvious toolchain behaviours".
- **`extends` as the answer to `couple`.** claude-observatory carries 86
  `extends` entries (80 additive, 6 superseding), each with a comment saying
  why: the best worked corpus of cross-territory edits in the family.
- **Skill frontmatter with `allowed-tools`** scoped per command
  (`Bash(spec-spine:*)`, `Bash(git diff:*)`), which the kit's skills lack.
