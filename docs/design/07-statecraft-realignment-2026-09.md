# 07: The Statecraft realignment (2026-09-20)

A design note. What it records is filed as spec
`092-the-engine-ships-governance-not-an-environment`; this note holds the
boundary and the disposition of the earlier notes, not a second copy of the
spec.

## 1. The boundary

| Owner | Surface |
|---|---|
| **Statecraft CLI** | project onboarding and initialization, the managed development environment, the agent harness, reusable workflows and their per-agent delivery adapters, approvals, eligibility, policy, evidence, recovery |
| **spec-spine** | governance semantics: compile, index, classify, couple, lint, verify, attest; the deterministic engine; governance-compatible starter templates, produced as data |

The engine does not initialize a project and does not distribute a harness.
Statecraft does not reimplement governance; it calls the engine's library and
its CLI.

One invariant binds both sides and neither may weaken it: **a solo developer
can use every local Statecraft CLI and spec-spine capability with no Statecraft
platform account, login or hosted connection.** Local initialization,
execution, approvals, policy, eligibility, evidence, recovery and governance
verification are all local. Authentication to a model provider is a separate
question from platform authentication. Team enrollment is explicit and
project-scoped; for an enrolled project the platform coordinates shared
approvals, eligibility and policy, and offline operation does not silently
bypass them. No part of spec-spine's deterministic engine depends on platform
authentication or on a user's global Statecraft installation.

## 2. The producer contract

Statecraft consumes exactly one function:

```
spec_spine_core::scaffold_init_json(config_json: &str) -> Result<String, Error>
```

It returns the serialized `Scaffold` (`files: [ScaffoldFile]`) and it is pure:
no filesystem writes, no environment reads, no Statecraft discovery, no process
launches, no network, no timestamps, no registration, no agent activation. It
produces governance starter content only: `spec-spine.toml`, the constitution,
the contract, the authoring templates, the bootstrap spec and a `.gitignore`
fragment. It emits no `AGENTS.md`, no `CLAUDE.md`, no `.claude/`, `.codex/` or
`.agents/`, no skills, agents, hooks or MCP configuration, no CI workflow and
no `Makefile`.

The `.gitignore` output is content for the consumer to reconcile, not
permission to replace an existing file: it is returned with `append: true` and
an `appendMarker`.

Layout values Statecraft passes, all relative to the repository root and never
to the configuration file's directory:

```
specs_dir     = "specs"
standards_dir = "standards/spec"
derived_dir   = ".statecraft/derived"
state_dir     = ".statecraft/state"
```

This is a producer contract, not a public initialization command under a new
name. Spec 092 3.1 removes `spec-spine init` and forbids a replacement verb.

## 3. The managed layout

```
AGENTS.md                 the user's, with one Statecraft-inserted first line:
                          @.statecraft/AGENTS.md
.statecraft/AGENTS.md     Statecraft-generated project instructions
.statecraft/derived/      compiled governance artifacts, COMMITTED
.statecraft/state/        tooling's own working files, ignored
spec-spine.toml           root, unchanged
specs/ standards/spec/    root, unchanged
```

`.statecraft/` as a whole is **not** runtime state. Only `.statecraft/state/`
and the derived tree's `build-meta.json` are excluded. Anything else under
`.statecraft/` is an ordinary governed file and every check sees it.

## 4. Disposition of note 06

Note 06 (`06-harness-and-distribution-2026-09.md`) proposed a harness
distribution owned by spec-spine. Its rows are dispositioned here. Nothing in
note 06 was ever filed, so this is a change of owner for proposals, not the
withdrawal of any approved requirement.

| Note 06 row | Proposed owner | Disposition |
|---|---|---|
| 3.1 a small global personal policy | the user's global configuration | unchanged; never spec-spine's |
| 3.2 a versioned, namespaced shared skill package | spec-spine | **superseded**: skill packaging and global installation are Statecraft's |
| 3.3 the repository-local layer | the adopting repository | stands; a repository keeps its own project layer |
| 3.4 supported-agent adapters, one maintained source | spec-spine | **superseded**: delivery adapters are Statecraft's, globally under `~/.statecraft/` |
| 3.5 task-specific startup reads | spec-spine (`AGENTS.md`, `/prime`) | **superseded** as a spec-spine deliverable: which reads a session performs at startup is a harness decision. This repository's own `AGENTS.md` protocol stays its own |
| 3.6 build eligibility belongs to repository policy | spec-spine (`build` skill) + policy | **superseded**: eligibility orchestration is Statecraft's. The governance half (what `registry plan` answers) was already spec-spine's and stays |
| 3.7 one scheduler | spec-spine | **superseded**: scheduling is Statecraft's |
| 3.8 validation boundaries and evidence reuse | mixed | governance-side reuse stays a spec-spine question; orchestration-side reuse is Statecraft's |
| 3.9 hook verdicts and enforcement boundaries | spec-spine | partly delivered by specs 093 and 104 against this repository's own hooks. Distribution of hooks is Statecraft's; H-6 stays open and is now Statecraft's question |
| 3.10 a measurement plan | spec-spine | **superseded**: the thing it measures is not distributed here any more |

Note 05's wave B ("the kit and the harness deliver what they document") is
likewise superseded in the parts that concern kit distribution. What remains of
it is what this repository does to its own `.claude/` tree.

## 5. Spec 117

`117-the-derived-tree-question-asked-honestly` was drafted 2026-09-17 and never
merged. It is withdrawn as filed: three of the four hook copies it targets are
removed by spec 092, and spec 095's renumber leaves no ordinal 117 to reserve.

The defect it measured, a PR-gate test blind to a staged shard and to an
untracked one, was true of the single retained copy in `.claude/settings.json`.
Spec 092 4 recorded it as open, pointing at an unmerged branch and a commit on
it for the measurement. That reference was only readable in the clone that held
the branch, and spec 092's acceptance asserted both existed, so those two lines
passed for their author and failed in CI and in every other checkout. The
carry-forward is therefore a restatement, not a pointer: the measurement, the
three states, the cancellation case and the matrix are spec 093 3.13, and the
hook was fixed against it on 2026-09-21.

## 6. What is outstanding

| Item | Owner | State |
|---|---|---|
| `@.statecraft/AGENTS.md` bridge in this repository's root `AGENTS.md` | Statecraft initializer | not applied: the initializer must write `.statecraft/AGENTS.md` first, and an import of a missing file is a broken instruction |
| Retirement of this repository's `.claude/` tree | Statecraft global delivery | held: `.claude/` stays until its replacement is concretely available |
| The PR-gate derived-tree test | spec 093, which specifies the surviving hook | closed 2026-09-21: spec 093 3.13, with the matrix in `harness_hooks.rs`. See 5 |
| Ratification of spec 092 | a human | done: `approved`. The realignment was built under a specific owner authorization, and the flip was a human's |

## 7. The collapse that followed (2026-09-20)

Removing the kit correctly left the corpus describing a product that no longer
exists: 102 withdrawn unit claims across 29 specs, 29 specs whose acceptance was
one block standing for all of them, and 41 documents reading as live instruction
about deleted files. The owner authorized a one-time exception to the amendment
rules to settle it, and spec 095 is the record.

- six specs deleted outright (their whole subject was removed);
- twenty-one merged into two: spec 093 is the whole of what `.claude/` must
  contain, spec 094 the one definition of the governed loop and the three
  boundaries that enforce it;
- the 93 survivors renumbered to `000` through `092`, chronological order kept,
  with the three new specs at `093` through `095`.

`docs/corpus-map.md` maps every removed id to the spec that answers for it and
every old ordinal to its new one. Every citation in this note, in the other
design notes, and in the specs was rewritten with it; citations in git history,
merged pull requests and adopter repositories were not, which is what the map is
for.
