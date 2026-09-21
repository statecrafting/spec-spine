---
id: "092-the-engine-ships-governance-not-an-environment"
title: "The engine ships governance, not an environment"
status: draft
kind: "architecture"
created: "2026-09-20"
summary: >
  spec-spine has been shipping two products. One is a governance engine: a
  compiler, an indexer, a classifier, a coupling gate, a verifier. The other is
  a development environment: twenty-nine files of skills, agents, rules, hooks,
  a Makefile and a CI workflow, embedded into the binary as a 2,707-line
  generated module and written out by a public `init --with-kit`, plus two
  further generated projections of the same instructions for other agent
  runtimes. The second product is now Statecraft's: the Statecraft CLI is the
  sole distributor and initializer of the managed development environment, and
  it places the compiled governance artifacts at `.statecraft/derived/`. This
  spec removes the public initialization command, the kit, the embedded kit,
  the agent-tree projections and the parity machinery that held them equal;
  narrows the retained library scaffold to governance starter content and
  nothing else; relocates this repository's own committed derived trees; and
  records, per predecessor, what the removal does to the claims and the
  acceptance those specs left behind.
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "036-declared-state-dir"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "089-nothing-reruns-a-merged-acceptance"
establishes:
  # 3.2 and 3.3: the retained producer and what holds it to its contract. Spec
  # 006 established these and spec 095 removed it with the command it was
  # written for; the files survive, narrowed to what this spec requires of them,
  # so the claim lands on the spec that states the requirement.
  - "crates/spec-spine-core/src/scaffold.rs"
  - "crates/spec-spine-core/tests/scaffold.rs"
extends:
  # 3.5 and 3.6: the three `kit_*` suites are RENAMED, not replaced. Their
  # subject survives the kit (the files this repository still has), so the
  # claim follows the file rather than being withdrawn and re-established:
  # 046, 048 and 064 keep owning the suites they wrote, at their new paths.
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_hooks.rs", nature: reductive }
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_skills.rs", nature: reductive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: reductive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "Makefile", nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: "AGENTS.md", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/skills/", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/settings.json", nature: additive }
  # 3.1: the verb and its dispatch leave the CLI.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: reductive }
  # 3.2 and 3.3: the retained producer and its exported facade.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: reductive }
  # 3.8: the configured derived directory is derived everywhere, not only where
  # the default was spelled out.
  - { spec: "036-declared-state-dir", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # 3.12: the real-corpus parse pin read spec 093's own block, which this spec
  # replaces, so the pin moves to a spec that still holds one and the
  # substitution itself becomes the assertion.
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-core/tests/verify.rs", nature: corrective }
  # 3.6: the dev-dependency comment names the suite that reads the workflow,
  # which is `tests/gate.rs` now.
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/Cargo.toml", nature: corrective }
  # 3.7: the layout this repository is governed under, and everything that
  # spells the old path.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "spec-spine.toml", nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".githooks/", nature: additive }
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
  # 3.6 and 3.10: the protocol this repository runs, and the documents that
  # told an adopter to install a kit.
  - { spec: "093-the-harness-this-repository-runs", unit: "AGENTS.md", nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
  # 3.8: the governed-scope walk skips the configured derived root too, for
  # the reason 3.8 gives about the other two walks.
  - { spec: "078-governed-scope-is-declared-not-inferred", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  # 3.4: the specify-first note told a reader to install the kit's gate.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/specify-first.md", nature: reductive }
  # 3.1: the npm shim's README told a reader to run `npx spec-spine init`.
  # `npm/` is 007's subtree and an explicit claim outranks the `**/README.md`
  # bypass (spec 008), so the one line removed is declared here. Named as the
  # SUBTREE 007 establishes rather than as the file: a file unit carries no
  # span, so claiming `npm/README.md` directly would add an `L-008`
  # unwitnessed claim to a file nothing hashes, which is a worse record than
  # the one line being fixed.
  - { spec: "006-distribution", unit: "npm/", nature: reductive }
  # 3.5: this repository's own harness is repository-owned development
  # instruction from here, not a distribution source.
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/skills/", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/agents/", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/rules/orchestrator-rules.md", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/settings.json", nature: additive }
# 3.11: every spec whose stated behavior this removal changes. Declared here,
# in the amending spec, so no predecessor's prose is edited to mention its
# successor (spec 037 3.1). The frontmatter withdrawals 3.11 requires are a
# different act, recorded there and taken under this spec's authority.
amends:
  - "022-index-sharding"
  - "032-stdout-closed-reader"
  - "054-the-scaffold-ships-what-adopters-wrote"
  - "055-a-version-pin-the-cli-can-check"
  - "056-the-contract-records-the-lifecycle-table"
  - "058-the-shipped-default-hashes-what-it-names"
  - "061-shipped-is-not-the-same-as-working"
  - "062-one-name-one-freshness-verb"
  - "064-compile-warnings-reach-the-gate"
  - "067-a-short-id-names-the-same-spec-at-every-verb"
  - "068-a-verifier-checks-the-bytes-it-was-given"
  - "070-an-authority-snapshot-says-what-it-read"
  - "083-an-amendment-carries-the-acceptance-it-replaces"
  - "089-nothing-reruns-a-merged-acceptance"
# 3.12: the acceptance this spec replaces, derived from the sweep of the
# implemented tree (`scripts/verify-sweep.sh`), not from a prediction. Each of
# these blocks reads a path this change removed, so `## Verification` below is
# their acceptance from here.
#
# One is deliberately absent because another spec already holds it and 103 3.2
# resolves the chain to whoever holds it now: 098's is 105's, and 105 is listed,
# so the chain lands here. Listing the held spec as well is `V-019`, which is
# the corpus refusing to guess which of two amenders is the authority. The three
# other holders this list once relied on (113 for 065, 110 for 071, 114 for 077)
# were collapsed by spec 095, so 077, whose document survives, is named here
# directly, and 065 and 071 went with their holders.
amends_verification:
  - "054-the-scaffold-ships-what-adopters-wrote"
  - "055-a-version-pin-the-cli-can-check"
  - "056-the-contract-records-the-lifecycle-table"
  - "058-the-shipped-default-hashes-what-it-names"
  - "062-one-name-one-freshness-verb"
  - "064-compile-warnings-reach-the-gate"
  - "083-an-amendment-carries-the-acceptance-it-replaces"
references:
  - { unit: { kind: file, path: "docs/design/06-harness-and-distribution-2026-09.md" }, role: context }
---
# 120: The engine ships governance, not an environment

## 1. Purpose

### 1.1 Two products, one repository

Measured on 2026-09-20 at `24bc52f` with the 0.21.0 binary:

| Surface | Size |
|---|---|
| `kit/`, tracked | 29 files, 161,150 bytes |
| `crates/spec-spine-core/src/kit_embedded.rs`, generated from it | 2,707 lines |
| `.agents/skills/`, generated projection | 10 files |
| `.codex/agents/` + `.codex/hooks.json`, generated projection | 5 files |
| test files whose subject is one of the above | 4 (`kit_gate`, `kit_hooks`, `kit_skills`, `agent_trees`), 4,077 lines |
| specs holding a frontmatter unit on one of those paths | 29 |

None of it compiles a spec, indexes a package, classifies a change or verifies
an acceptance block. It is a development environment: which skills a session
loads, which hooks fire, what a `Makefile` target runs, what an adopter's
`AGENTS.md` says. The engine is the other half of the repository, and the two
halves have been versioned, released, gated and reasoned about as one thing.

### 1.2 The owner has moved

The Statecraft CLI is now the sole distributor and initializer of the managed
development environment. It owns project onboarding, the agent harness, the
reusable workflows and their per-agent delivery. spec-spine owns governance
semantics: compilation, indexing, classification, verification, and the
governance-compatible starter content a new corpus needs.

That is a boundary, not a layering. Two consequences follow immediately and
this spec is both of them: the public `spec-spine init` command has no owner
here any more, and the managed project's layout is Statecraft's to choose.

### 1.3 What the managed layout is

Statecraft initializes a project with:

| Path | Who writes it | Committed |
|---|---|---|
| `spec-spine.toml` | spec-spine's library, through Statecraft | yes |
| `specs/`, `standards/spec/` | spec-spine's library, through Statecraft | yes |
| `.statecraft/derived/` | `spec-spine compile` / `index` | yes, less `build-meta.json` |
| `.statecraft/state/` | tooling's own working files | no |
| `.statecraft/AGENTS.md` | Statecraft | yes |
| `AGENTS.md` (root) | the user, with a one-line bridge from Statecraft | yes |

The bridge is a single first line, `@.statecraft/AGENTS.md`, inserted into a
root `AGENTS.md` whose remaining content is the user's and is preserved.

`.statecraft/` is therefore **not** a state root. Only `.statecraft/state/` is,
plus the transient build metadata inside the derived tree. A rule that ignored
or excluded the whole directory would put the committed governance ledger
outside version control and outside every check that reads it.

### 1.4 Why there is no compatibility path

`init --with-kit` is an onboarding command. Its whole output is the thing being
transferred to another owner, and an onboarding command that writes a
half-transferred environment is worse than one that is absent: the adopter gets
two initializers with different opinions about the same paths and no way to
tell which one is current. A deprecation window would keep the embedded kit,
its generator, its parity suites and its 29 spec claims alive for the length of
the window, which is the entire cost this spec exists to remove.

The engine's own distribution is unaffected. `spec-spine` remains installable
from crates.io, npm, PyPI and `install.sh`, and every governance verb it has
keeps working on a repository nobody initialized with Statecraft. The mandatory
ecosystem invariant is that a solo developer can run all of it locally with no
platform account, login or hosted connection; nothing in this spec adds a
dependency on any of those, and 3.2 states the producer's purity as a contract
rather than as an implementation detail.

## 2. Territory

This spec establishes the repository-owned gate definition. It extends the
engine modules the relocation and the narrowing touch, this repository's
configuration, its CI workflow and its protocol, and the three renamed test
suites, whose owners keep them: a suite whose subject survives the kit follows
its file rather than being withdrawn and re-established under a new owner
(3.11).

It also removes paths that 29 approved specs claim. 3.11 states how that is
resolved, and it is the only part of this document that edits another spec's
file.

| Path | What happens |
|---|---|
| `Makefile` | new: the gate, repository-owned |
| `crates/spec-spine-core/tests/gate.rs` | renamed from `kit_gate.rs`, narrowed to this repository's own gate |
| `crates/spec-spine-core/tests/harness_hooks.rs` | renamed from `kit_hooks.rs`, narrowed to `.claude/settings.json` |
| `crates/spec-spine-core/tests/harness_skills.rs` | renamed from `kit_skills.rs`, narrowed to `.claude/skills/` |
| `kit/` | removed, 29 files |
| `crates/spec-spine-core/src/kit_embedded.rs` | removed |
| `scripts/gen-kit-embedded.py` | removed |
| `scripts/gen-agent-trees.py` | removed |
| `.agents/`, `.codex/` | removed |
| `crates/spec-spine-cli/src/cmd_init.rs` | removed |
| `crates/spec-spine-cli/tests/init.rs` | removed |
| `crates/spec-spine-core/tests/agent_trees.rs` | removed with the trees it asserted |
| `.derived/` | moved to `.statecraft/derived/` |

## 3. Behavior

### 3.1 The public initialization command is removed

`spec-spine init` MUST NOT exist. The subcommand, its clap definition, its
`--force` and `--with-kit` flags, its dispatch arm and `cmd_init.rs` MUST all
be removed, and no verb MUST be added that recreates it under another name.

`spec-spine --help` MUST NOT name `init` and MUST NOT name `with-kit`.
`spec-spine init` MUST fail as an unknown subcommand, which spec 093 maps to
exit 3.

### 3.2 One retained producer, and it stays pure

`spec_spine_core::scaffold_init_json(config_json: &str) -> Result<String, Error>`
MUST be retained, with its present signature and its present serialized
`Scaffold` / `ScaffoldFile` response shape. It is the boundary Statecraft
consumes and it is not renamed, restructured or replaced by this spec.

It MUST remain a pure function of its argument. It MUST NOT write to the
filesystem, read the environment, discover or launch Statecraft, spawn a
process, open a network connection, read a clock, register anything or activate
an agent. The library MUST NOT require the Statecraft CLI to be installed, and
MUST behave identically with an empty or absent home directory.

`scaffold_init_with` MUST be removed along with the `with_kit` parameter: with
the kit gone it has no second behavior to select, and no retained consumer
needs it. `scaffold_init` and the `Scaffold` / `ScaffoldFile` types stay
exported, because the JSON facade is a projection of them and a consumer
linking the library directly is a supported shape.

### 3.3 What the scaffold produces, and what it must not

The returned file set MUST be exactly:

| File | Source |
|---|---|
| `spec-spine.toml` | the documented starter config, config-aware (spec 054) |
| `<standards_dir>/constitution.md` | the tier-2 constitution |
| `<standards_dir>/contract.md` | the normative summary |
| `<standards_dir>/templates/spec-template.md` | the authoring template |
| `<standards_dir>/templates/constitution-template.md` | the constitution template |
| `<specs_dir>/000-bootstrap/spec.md` | the bootstrap spec |
| `.gitignore` | the exclusion fragment for transient metadata and runtime state |

It MUST NOT emit `AGENTS.md`, `CLAUDE.md`, `.claude/`, `.codex/`, `.agents/`,
skills, agents, hooks, MCP configuration, CI workflows or a `Makefile`. The
three `.claude/rules/` files and the `AGENTS.md` the scaffold wrote under specs
093 and 065 are environment, and the environment has an owner.

Every path MUST honor `config.layout` and MUST be relative to the repository
root, never to the configuration file's directory. The layout values Statecraft
passes are `specs_dir = "specs"`, `standards_dir = "standards/spec"`,
`derived_dir = ".statecraft/derived"`, `state_dir = ".statecraft/state"`, and a
non-default value for any of them MUST produce a coherent scaffold rather than
a default one.

The `.gitignore` entry MUST remain content for the consumer to reconcile, not
an instruction to replace an existing file: it carries `append: true` and an
`appendMarker`, and the writer is Statecraft's.

### 3.4 The kit and its embedding are removed

`kit/`, `crates/spec-spine-core/src/kit_embedded.rs` and
`scripts/gen-kit-embedded.py` MUST be removed. No embedded copy of an agent
harness MUST remain in the binary, and `tests/scaffold.rs` MUST NOT assert a
`kit/` tree it can no longer read.

### 3.5 The projections go; this repository's own harness stays

`.agents/`, `.codex/` and `scripts/gen-agent-trees.py` MUST be removed. They
exist to deliver one instruction set to several agent runtimes, which is the
delivery-adapter problem Statecraft owns globally under `~/.statecraft/`. Two
trees generated from a third, held equal by a test, is parity machinery for a
distribution this repository no longer performs.

`.claude/` MUST be preserved: `settings.json`, `skills/`, `agents/`, `rules/`
and `agent-memory/`. It is this repository's own development instruction and
its own enforcement, it is what a session working here actually loads, and its
replacement, Statecraft's global delivery, is not concretely available yet.
What changes is its status: it is repository-specific development instruction
from here, not a source anything is generated from and not a product anything
distributes. It is retired when Statecraft delivers its replacement, and not
before.

### 3.6 The gate has one definition and this repository owns it

A root `Makefile` MUST carry the `gate`, `refresh`, `verify`, `build`, `test`,
`fmt`, `clippy` and `help` targets, with the semantics `kit/Makefile` carried:
spec 094's composite chain, spec 094's explicit `if`/`then`/`else` guards,
spec 094's `OWNERSHIP` and `COUPLE` controls with their refusal of an
unrecognized value and their announced skips, and spec 093's base-ref
resolution. Nothing about the gate's enforcement strength changes; only who
owns the file does.

One control is added: `HEAD`, the ref the coupling gate compares **to**,
defaulting to `HEAD`. A pull-request CI leg must diff `base.sha...head.sha`,
both frozen event SHAs, because the checked-out `refs/pull/N/merge` HEAD
re-resolves against the current base on every run and a gate diffing to it folds
in changes merged after the pull request opened. One definition can serve both
callers only if the caller can say which ref it means, which is spec 094 §3.3's
own argument for `COUPLE` applied to the other endpoint. D-10.

`.github/workflows/ci.yml` MUST invoke `make gate` against that file and MUST
NOT reference `kit/Makefile`. It MUST call the target on both event legs rather
than restating the chain beside it: the push and merge-queue leg with
`COUPLE=0`, the pull-request leg with `COUPLE=1` and the frozen SHAs. The named
steps that re-ran `check`, `index coverage` and `lint` after `make gate` are
removed with the restatement they were: they ran the same verbs a second time,
and a chain with two spellings is the defect spec 094 exists to prevent. `AGENTS.md` MUST NOT tell a session to run a gate
from a directory that no longer exists. The required check set MUST be
unchanged: `test`, `self_governance`, `determinism`, `ai-review`, aggregated by
`ci-gate`.

### 3.7 This repository's derived trees move to `.statecraft/derived/`

`[layout] derived_dir` MUST be `.statecraft/derived` and `[layout] state_dir`
MUST be `.statecraft/state`. The committed shard trees move with it, in one
change with everything that spells the old path: the configuration, the CI and
determinism workflows, `.gitignore`, `.gitattributes`, the merge driver and the
commit hook under `.githooks/`, the maintainer sweep, the documentation, and
every `## Verification` command that names `.derived/` as a path in **this**
repository rather than inside a fixture it builds itself.

`.derived/` MUST NOT exist afterwards. The default `derived_dir` for a
repository that configures nothing stays `.derived`: this is a change to this
repository's configuration, not to the product's default.

### 3.8 The configured derived directory is derived everywhere

Two places in the engine spelled the default rather than reading the
configuration, and both are load-bearing the moment the configuration differs:

- The coupling bypass floor (`couple.rs::DEFAULT_BYPASS_PREFIXES`) lists
  `.derived/`. Compiler output that no longer matches that literal is judged as
  source, so every regenerated shard becomes a `C-001` drift refusal and, under
  `require_ownership`, an unclaimed-file `C-002`.
- The source and territory walks prune `index.resolver_exclusions`, which
  matches path **components**, and the declared `state_dir`. `.derived` was
  reachable as a component; `.statecraft/derived` is not, and no
  `resolver_exclusions` entry can express it without excluding every directory
  named `derived` anywhere in the tree.

The configured `derived_dir` MUST therefore be treated as derived in both
places: it joins the built-in bypass floor, and it is pruned from the walks the
way `state_dir` is. Neither is expressible in configuration and neither may be
cancelled by it, for the reason spec 036 3.5 gives about the state root.

The literal `.derived/` MUST remain on the floor. A repository that configures
nothing keeps the behavior it has, and a repository mid-migration has both
paths answered.

### 3.9 `.statecraft/` is not runtime state

Only `.statecraft/state/` and the derived tree's `build-meta.json` MUST be
excluded from version control. A file under `.statecraft/` that is neither MUST
remain visible to the ownership walk, to `index coverage` and to the coupling
gate exactly as a file anywhere else in the repository is.

The three answers MUST be distinguishable by measurement, not by inspection: a
source file under `.statecraft/` is reported by `index coverage`, one under
`.statecraft/state/` is not, and one under `.statecraft/derived/` is not.

### 3.10 The instruction bridge is Statecraft's to insert

This repository's root `AGENTS.md` MUST keep its content. The
`@.statecraft/AGENTS.md` bridge MUST NOT be added here by hand: the line is an
import, an import of a file that does not exist is a broken instruction, and
`.statecraft/AGENTS.md` is written by the Statecraft initializer. Applying the
bridge to this repository is a consumer exercise, and until the initializer
exists it is recorded as outstanding rather than performed.

### 3.11 Removed paths, and the claims they leave behind

29 approved specs hold a frontmatter unit on a path 3.4, 3.5 or 3.1 removes.
The implementation resolves them as follows, and the resolution is a
measurement of what the engine does, not a preference:

A file unit whose path does not exist is `I-004`; a directory unit is `I-007`.
`classify_unresolved` downgrades neither for a spec that is not in flight, and
every one of the 29 is `approved` with `implementation: complete`. Nothing in
the grammar withdraws a claim: `supersedes` is additive and explicitly leaves
the predecessor holding the unit, `amends` names a spec and not a unit, and
`retired` is a status whose effect on an unresolved unit is to take its answer
from `status`, which for a settled spec is the error. There is no instrument,
and inventing one for this change would be a grammar extension smuggled in as a
cleanup.

So the unit claims MUST be withdrawn from the 29 predecessors' frontmatter
directly, and this spec's `amends` list MUST name every one of them. That is
the precedent spec 061 3.6 set and spec 093 D-7 recorded, for exactly this
situation and in the same words: a directory claim on a directory git no longer
carries is a hard error with no withdrawal instrument in the grammar, so the
edit is made directly, under a named authority, and the predecessor's **prose
is left as written**.

Three rules MUST hold for every such edit:

1. **Only the claim.** The removed lines are the `establishes` / `extends`
   entries naming a removed path, and the acceptance commands 3.12 covers.
   No requirement, no decision entry, no section of prose is edited.
2. **The record is in this spec.** A predecessor is not edited to mention its
   successor (spec 037 3.1). Its frontmatter loses a claim on a path that is
   gone; why is here.
3. **Nothing is retitled or restatused.** A spec that shipped a kit still says
   it shipped a kit, and its status stays `approved`. Retirement is a separate
   judgement about a document's standing, it is not automatic, and this spec
   does not make it. `29-claude-code-skill-kit` remains an accurate record of
   what was true between 2026-08 and this change.

A spec left with no ownership edge at all after the withdrawal MUST NOT be
deleted or hidden; it keeps its `## Verification` disposition under 3.12 and
its document stands. One such spec exists: 029, whose whole territory was
`kit/`. It is marked `superseded` with `superseded_by` naming this spec, and
this spec carries the `supersedes` edge. That is the corpus's own vocabulary
for an authority that moved, and it is not a way of dodging a diagnostic: the
only diagnostic it changes is `L-001`, which asks whether an **ordinary** spec
forgot to declare its territory. A withdrawn document claiming nothing is
making a correct statement, so `L-001` MUST skip a `superseded` or `retired`
spec. It MUST NOT skip anything else: a withdrawn spec that still names a unit
is still held to every diagnostic about that unit.

### 3.11.1 What the coupling gate says about the removal, measured

The gate refuses seven of the removed paths, and the refusal is a property of
the gate rather than of this change. Measured on 2026-09-20 against this
branch:

```
C-001 'crates/spec-spine-cli/src/cmd_init.rs' changed without an authoring
      edit to any owning spec (001-compile-registry)
```

`couple` resolves a path's owners from the **head** index. At head the file
does not exist, so no specific claim covers it and the answer falls through to
its package's manifest floor, spec 001. The specs that actually owned it, 006
and 065, **are** edited in this same change: that is what 3.11 required of
them. The gate cannot see that, because by the time it looks the claim it
would have matched is the one the change withdrew.

This spec does not fix it. Reading a deleted path's ownership at the base needs
the gate to hold two indexes, which `couple_with` does not take and which is a
seam through the CLI layer of its own, and a realignment is not the place to
open it. 4 names it as the follow-up, and D-11 records the two dispositions
available at PR time: the base-side read as its own spec, or a human
`Spec-Drift-Waiver:` line citing this section. An agent writes neither on its
own authority.

### 3.12 Acceptance replacement

A `## Verification` block whose commands read a removed path can no longer
pass, and `scripts/verify-sweep.sh` (spec 089) runs the whole corpus's declared
acceptance, so the failure is observable rather than theoretical.

Each affected block MUST be resolved by exactly one of:

- **Replacement.** This spec declares the predecessor in `amends_verification`
  and its own `## Verification` block becomes that spec's acceptance
  (spec 082 3.2). Used where the block's subject is the removed surface.
- **Correction in place.** Where the block's subject survives and only a path
  spelling changed (`.derived/` to `.statecraft/derived/`), the command is
  corrected under this spec's authority, as a path edit and nothing else.

The `amends_verification` list MUST be derived from a measured sweep of the
implemented tree, not from a prediction: a spec whose block still passes MUST
NOT have its acceptance replaced, and a spec whose block fails MUST NOT be left
failing. Every entry MUST also appear in `amends` (`V-018`).

Correction in place MUST NOT be used to make a failing assertion pass by
weakening it. If a block asserts something the realignment made false, the
answer is replacement, which says so, not an edited assertion that hides it.

**The measurement.** `scripts/verify-sweep.sh --rev <implementation commit>`,
run on 2026-09-20 against the implemented tree before any acceptance was
touched: **38 passed, 34 failed, 48 exempt, 0 not-declared, 0 not-run**. The 34
are dispositioned as follows, and nothing else in the corpus is touched:

| Disposition | Count | Which |
|---|---|---|
| Replaced (`amends_verification`) | 25 | the blocks whose subject is the removed surface |
| Resolved by an existing chain | 4 | 065 through 113, 071 through 110, 077 through 114, 098 through 105: each holder is replaced, so the chain lands here. Listing a held spec as well is `V-019` |
| Corrected in place | 3 | 084, 085, 087, each naming this repository's own `.derived/attestation/` path. Re-run green afterwards |
| Left failing, deliberately | 1 | 102, whose block runs the coupling gate against the default branch. It fails for 3.11.1's reason and for no other, and it is a pre-merge artifact: on the default branch the diff it takes is empty |
| This spec's own | 1 | 120 |

25 + 4 + 3 + 1 + 1 = 34: every failure is accounted for exactly once.

One correction in place is made to a block the sweep reported as **passed**:
spec 089's `! grep -rqF 'verify-sweep' … kit/` passed only because `grep -r`
over a directory that no longer exists errors and the `!` inverts the error into
a success. Removing the dead path is the difference between a line that passes
and a line that asserts, and the rule against rewriting a passing block is about
not weakening assertions, which this strengthens.

**One test read a replaced block.** `core/tests/verify.rs` pinned this
repository's own spec 093, command for command, as the real-world fixture for
the parser: "the parse must agree with the corpus it governs". Replacing 048's
acceptance made that plan 120's block, so the pin moved to spec 072, which still
holds its own, and the case it left behind became its own assertion: a plan
built for 048 keeps 048's id and names 120 as `acceptanceFrom`, which is spec
082 3.4's rule that the substitution is stated and never silent, exercised
against the real corpus instead of a fixture.

**A third state exists and is recorded rather than repaired.** Some blocks now
pass *vacuously*: spec 061's, for instance, holds `test ! -e
kit/scripts/verify-spec.sh` and two `! grep` lines over files under `kit/`, and
every one of those is trivially true once the tree is gone. The sweep reports
them as `passed`, which is honest about the exit code and says nothing about the
assertion. They are **not** rewritten here. Replacing the acceptance of a spec
whose block passes is precisely what the rule above forbids, and repairing
vacuity across the blocks this change emptied is an audit of its own, of the
shape specs 084 to 110 each were. What this spec owes is to say so, which is
this paragraph, so the next reader of a green sweep row knows which kind of
green it is.

### 3.13 The unmerged spec 117 proposal is withdrawn

Spec 117, "The derived-tree question, asked honestly", was drafted on
2026-09-17 and never merged; its only copy is on the branch
`113-the-harness-delivers-what-it-documents` at `487bbd9`. Its subject is the
PR-gate hook's `git diff --quiet -- .derived/` test, in four copies:
`kit/settings.json`, `.claude/settings.json`, `.codex/hooks.json` and
`kit_embedded.rs`. Three of those four are removed by this spec.

The proposal is withdrawn as filed. It MUST NOT be implemented here: its
territory is largely gone, its acceptance greps files that will not exist, and
implementing a hook body in copies scheduled for deletion is work with no
subject. The ordinal 117 stays reserved and unused, the branch MUST NOT be
deleted, and the defect it measured stays true of the one copy that remains.

That defect is therefore carried forward as an open item rather than closed:
the retained `.claude/settings.json` PR gate still asks a question blind to a
staged shard and to an untracked one. It is not fixed here because this spec's
subject is the boundary, and a hook whose owner is about to change is not the
place to land a behavior change. 4 records it.

## 4. Out of scope

- **Fixing the PR gate's derived-tree test.** 3.13. The defect is real, the
  branch is preserved, and the fix belongs to whoever owns the hook after
  Statecraft's delivery lands. D-7.
- **Retiring `.claude/`.** 3.5 keeps it and says what would retire it.
  Removing this repository's working harness before its replacement exists
  would leave no development instruction at all.
- **Statecraft's side of the boundary.** The initializer, the bridge insertion,
  `~/.statecraft/`, the delivery adapters and the enrollment model are the
  Statecraft CLI's, specified there. This spec states only what the producer
  guarantees and what the layout is.
- **Applying the bridge to this repository.** 3.10. It needs the initializer.
- **Changing the default `derived_dir`.** 3.7. The default stays `.derived`;
  this repository's configured value is what moves.
- **A new grammar instrument for withdrawing a claim.** 3.11 measures why one
  would be needed and declines to add it in a realignment. If the corpus wants
  a `withdraws` edge it is its own spec, with its own diagnostics and its own
  answer for what an amended acceptance does. D-3.
- **Retiring or deleting any historical spec.** 3.11 rule 3.
- **Repairing the blocks this change left passing vacuously.** 3.12's third
  state. An audit of the same shape as specs 084 to 110, and the sweep spec 089
  built is what schedules it; what it cannot do is tell a vacuous green from a
  real one, which is why the paragraph exists.
- **Teaching `couple` to read a deleted path's ownership at the base.**
  3.11.1 measures the gap and D-11 records why it is not closed here. It is a
  behavior change to the gate, needing a second index at the library boundary
  and its own acceptance; this spec would be the first caller, not the right
  author.
- **Releasing.** No version is bumped and nothing is published. The npm, PyPI
  and `install.sh` distributions of the engine are untouched.

## 5. Resolved decisions

D-1 (2026-09-20, no deprecation window, no shim, no dual installer). The owner
directed a deliberate break and 1.4 gives the reason that makes it the cheaper
choice: the deprecated surface is an initializer, and two initializers with
different opinions about the same paths is a worse failure than an absent one.
A window would also keep `kit_embedded.rs`, its generator, three parity suites
and 29 spec claims alive for its whole length, which is the entire cost being
removed.

D-2 (2026-09-20, the producer's shape is not redesigned). `Scaffold` and
`ScaffoldFile` keep every field, including `overwrite`, `executable`, `append`
and `append_marker`, even though the retained file set uses only `append` and
`append_marker`. The fields are a contract with a consumer implementing against
them concurrently, the removal of the kit is not a reason to renegotiate a
response shape, and a field that is always `false` costs a consumer nothing.
Removing internal code behind the boundary is encouraged and done; redesigning
the boundary is not.

D-3 (2026-09-20, the withdrawal is a direct edit under a named authority). See
3.11 for the measurement. The three alternatives were: a new grammar instrument
(a spec of its own, with diagnostics and an acceptance story, and not a thing
to invent inside a realignment); restatusing the 29 predecessors to `retired`
and making a retired spec's unresolved units non-fatal (which is exactly the
"retirement labels automatically remove obligations" shape that would let any
future removal launder itself through a status flip); or leaving the claims
standing and allowlisting the errors (which is the gate weakening this corpus
refuses). The direct edit is the one the corpus already has precedent for,
it is reviewable line by line in `git log -p`, and it leaves every predecessor's
prose intact.

D-4 (2026-09-20, `.agents/` and `.codex/` go, `.claude/` stays). The test is
whether the tree is *read by this repository's own sessions* or *generated to
be delivered elsewhere*. `.claude/` is read here, every session. `.agents/` and
`.codex/` are projections generated by `gen-agent-trees.py` so that other
runtimes get the same instructions, which is delivery, which is Statecraft's.
Keeping them would mean keeping the generator and the equality suite for a
distribution this repository no longer performs.

D-5 (2026-09-20, the gate definition moves to a root `Makefile`, not into CI).
Spec 094's argument is that the chain has one definition and that CI runs the
artifact rather than a restatement of it; spec 094 D-1 records what happened
when `govern.yml` restated the commands instead of calling the target. Moving
the definition into `ci.yml` would put it back where a local session cannot run
it and where the two copies drift again. The file's owner changes; its role
does not.

D-6 (2026-09-20, the derived directory is answered in the engine, not in
configuration). `[coupling] bypass_prefixes` could carry `.statecraft/derived/`
in this repository's own config, and that would make this repository work. It
would leave every other repository that configures a non-default `derived_dir`
with a coupling gate that judges its own compiler output as drift, which is a
trap that fires at PR time, in someone else's repository, with a refusal that
names their shards. The configuration key spells an exception; the engine
should not need one to know what its own output is.

D-7 (2026-09-20, spec 117 is withdrawn rather than implemented or deleted).
3.13. Withdrawn, not abandoned: the measurement is reproduced in its own
document, the branch that holds it is preserved, and 4 names the remaining
copy. Implementing it would mean writing a hook body into three files this same
change deletes.

D-8 (2026-09-20, `.statecraft/derived/` is committed and `.statecraft/state/`
is not). The committed shard tree is what makes `check` a freshness gate on a
pull request, which is one of the system's two central mechanisms; moving it to
a new parent is not a reason to stop committing it. `state_dir` is spec 036's
ungoverned root and keeps its meaning. The two live under one parent and are
classified separately, which is why 3.9 requires the distinction to be provable
by measurement rather than asserted.

D-11 (2026-09-20, the seven C-001 refusals are reported, not engineered away).
3.11.1 measures them. Three ways to make them disappear were available and each
is worse than the report. Claiming the removed paths in this spec's frontmatter
turns every one into a `W-001` unresolved claim, which CI refuses with
`--fail-on-unresolved`, so the gate's refusal would simply move. Claiming the
surviving container (`crates/spec-spine-core/tests/`, or the crate) makes this
spec the specific owner of every file under it and rewrites `index coverage`'s
attribution for about forty files, to clear seven deletions. Editing spec 001's
`spec.md` so the floor owner appears in the diff is a cosmetic edit to an
approved spec made solely to satisfy a mechanical refresh, which
`.claude/rules/adversarial-prompt-refusal.md` forbids by name. What is left is
to say what the gate says and let a human decide, which is what a waiver is for.

D-10 (2026-09-20, `HEAD` is a gate variable, not a workflow-side `couple`
call). The alternative was to leave `make gate` with `--head HEAD` and keep a
separate `couple` step in CI for the pull-request leg, which is what the
workflow did before. That is two spellings of the chain again, and the second
one is the one that carries the waiver read and the frozen SHAs, so the half
most likely to be wrong is the half nothing tests. A variable with a default
that is right for every local caller costs one line and keeps the count at one.

D-9 (2026-09-20, acceptance replacement is measured, not predicted). 3.12. A
prediction of which blocks break would be a list written by the person who
broke them, checked by nobody. The sweep spec 089 established runs the corpus's
declared acceptance and reports five outcomes; the `amends_verification` list
is its `failed` set, and a block that still passes keeps its own acceptance.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line.

**Fail-first evidence**, measured on 2026-09-20 at `24bc52f`, the parent of
this spec's branch:

| Line | At the parent |
|---|---|
| `! target/release/spec-spine init --help` | red: `init` exists and exits 0 |
| `! test -e kit` | red: 29 tracked files |
| `! test -e crates/spec-spine-core/src/kit_embedded.rs` | red: 2,707 lines |
| `! test -e .agents` / `! test -e .codex` | red: 10 and 5 tracked files |
| `! test -e crates/spec-spine-cli/src/cmd_init.rs` | red: 192 lines |
| `test -f Makefile` | red: absent, the gate lives in `kit/Makefile` |
| `! test -e .derived` | red: 242 committed shards |
| `test -d .statecraft/derived/spec-registry/by-spec` | red: absent |
| the scaffold file-set assertion | red: the set includes `AGENTS.md` and three `.claude/rules/` files |
| the `.statecraft/` coverage triple | red: `couple`'s floor and the walks answer the old path only |
| `registry show 120` | red: not found, exit 1 |

**Green at the parent, and a guard rather than evidence.** `! grep -rqF
'scaffold_init_with' crates/` is red at the parent and green after, but the two
`! grep` lines about `.derived/` in `Makefile` and `ci.yml` are green at the
parent for the trivial reason that neither file exists there. They are in the
block because they guard the shape of the *fix*: a `Makefile` copied from
`kit/Makefile` with the path left alone satisfies `test -f Makefile` and every
other structural line here.

```verify:cli
cargo build --release --locked
# 3.1: the public command is gone, asserted in both directions. The negative is
# the load-bearing one: a binary that still dispatches `init` answers 0 here.
! target/release/spec-spine init --help
target/release/spec-spine --help > "${TMPDIR:-/tmp}/ss120-help.txt" 2>&1
! grep -qF 'with-kit' "${TMPDIR:-/tmp}/ss120-help.txt"
! grep -qE '^[[:space:]]+init([[:space:]]|$)' "${TMPDIR:-/tmp}/ss120-help.txt"
rm -f "${TMPDIR:-/tmp}/ss120-help.txt"
# 3.4 and 3.5: the removed product surface, path by path.
! test -e kit
! test -e crates/spec-spine-core/src/kit_embedded.rs
! test -e scripts/gen-kit-embedded.py
! test -e scripts/gen-agent-trees.py
! test -e crates/spec-spine-cli/src/cmd_init.rs
! test -e crates/spec-spine-cli/tests/init.rs
! test -e crates/spec-spine-core/tests/kit_gate.rs
! test -e crates/spec-spine-core/tests/kit_hooks.rs
! test -e crates/spec-spine-core/tests/kit_skills.rs
! test -e crates/spec-spine-core/tests/agent_trees.rs
! test -e .agents
! test -e .codex
# 3.2: and the kit-only API with it. A `pub fn` left behind with no kit to
# select would compile and mean nothing.
! grep -rqF 'scaffold_init_with' crates/
! grep -rqF 'kit_embedded' crates/
# 3.5: this repository's own harness is still here, and still loaded.
test -f .claude/settings.json
test -f .claude/skills/prime/SKILL.md
test -f .claude/rules/adversarial-prompt-refusal.md
test -f AGENTS.md
# 3.2 and 3.3: the retained producer, through the exported facade, with the
# layout Statecraft passes and with a non-default layout. Asserted in Rust
# because the facade is a library boundary and no CLI verb reaches it.
cargo test -p spec-spine-core --test scaffold --locked > "${TMPDIR:-/tmp}/ss120-scaffold.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss120-scaffold.txt"
rm -f "${TMPDIR:-/tmp}/ss120-scaffold.txt"
# 3.6: the gate is repository-owned, names no kit, and runs. COUPLE=0 because
# this line has no base to diff against; the coupling half runs in CI and in
# the gate the session runs before every commit.
test -f Makefile
! grep -rqF 'kit/Makefile' .github/workflows/
! grep -qF 'kit/Makefile' AGENTS.md
! grep -qF 'kit/Makefile' CLAUDE.md
# 3.7: and nothing still points at the old path. Asserted through git rather
# than by grepping prose: a tracked file under `.derived/` is the failure, and
# a sentence mentioning the old path in a design note is not.
test "$(git ls-files .derived | wc -l | tr -d ' ')" = 0
! grep -qF ' .derived/' .gitattributes
! grep -qE '^\.derived/' .gitignore
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
# 3.6: an unrecognised control word is still refused rather than read as a
# default, which is spec 094's rule and the reason the file moved intact.
! make gate SPEC_SPINE=target/release/spec-spine OWNERSHIP=yes COUPLE=0
! make gate SPEC_SPINE=target/release/spec-spine COUPLE=maybe
# 3.7: the committed trees are at the configured path and the old one is gone.
! test -e .derived
test -d .statecraft/derived/spec-registry/by-spec
test -d .statecraft/derived/codebase-index/by-spec
target/release/spec-spine config show > "${TMPDIR:-/tmp}/ss120-cfg.txt"
grep -qF 'derived_dir = ".statecraft/derived"' "${TMPDIR:-/tmp}/ss120-cfg.txt"
grep -qF 'state_dir = ".statecraft/state"' "${TMPDIR:-/tmp}/ss120-cfg.txt"
rm -f "${TMPDIR:-/tmp}/ss120-cfg.txt"
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine lint --fail-on-warn
# 3.9: .statecraft is not wholesale ignored. The derived ledger is tracked and
# the state root is not, asserted through git rather than by reading the file.
! git check-ignore -q .statecraft/derived/spec-registry/by-spec/092-the-engine-ships-governance-not-an-environment.json
git check-ignore -q .statecraft/state/anything
git check-ignore -q .statecraft/derived/spec-registry/build-meta.json
git ls-files --error-unmatch .statecraft/derived/spec-registry/by-spec/000-spec-spine-bootstrap.json > /dev/null
# 3.8 and 3.9: the mechanism, on a fixture whose derived tree is at the new
# path. A source file under `.statecraft/` is governed; one under the state
# root and one under the derived tree are not. Built from nothing each run, so
# an empty or default corpus cannot produce the green.
cargo test -p spec-spine-core --test index --locked statecraft > "${TMPDIR:-/tmp}/ss120-idx.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss120-idx.txt"
cargo test -p spec-spine-core --test couple --locked derived > "${TMPDIR:-/tmp}/ss120-cpl.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss120-cpl.txt"
rm -f "${TMPDIR:-/tmp}/ss120-idx.txt" "${TMPDIR:-/tmp}/ss120-cpl.txt"
# 3.7: a relocated tree is still refused when it is stale, missing or has a
# stray shard. Run end to end against a scratch corpus at the new path.
cargo test -p spec-spine-cli --test cli --locked statecraft_derived > "${TMPDIR:-/tmp}/ss120-cli.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss120-cli.txt"
rm -f "${TMPDIR:-/tmp}/ss120-cli.txt"
# 3.6: the suites that replaced the removed ones still assert what they
# asserted about the files this repository still has.
cargo test -p spec-spine-core --test gate --locked
cargo test -p spec-spine-core --test harness_hooks --locked
cargo test -p spec-spine-core --test harness_skills --locked
# 3.11: nothing claims a removed path. Asserted through the governed read
# rather than by grepping frontmatter: a claim on a path that does not exist is
# an unresolved claim, and `--fail-on-unresolved` above is exactly the refusal
# for it. A grep would also match its own line in this block, which is how spec
# 071's first attempt at a pattern assertion passed against itself.
target/release/spec-spine index diagnostics > "${TMPDIR:-/tmp}/ss120-diag.txt" 2>&1
test ! -s "${TMPDIR:-/tmp}/ss120-diag.txt"
rm -f "${TMPDIR:-/tmp}/ss120-diag.txt"
# 3.11 rule 3: the one spec left owning nothing is superseded by name, not
# retitled, deleted or quietly left claiming a directory that is gone.
target/release/spec-spine registry show 029 --json > "${TMPDIR:-/tmp}/ss120-029.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss120-029.json')); assert d['status'] == 'superseded', d['status']; assert d['supersededBy'] == '092-the-engine-ships-governance-not-an-environment', d"
rm -f "${TMPDIR:-/tmp}/ss120-029.json"
# 3.13: the ordinal stays reserved and the branch that holds the proposal is
# still here.
! test -e specs/117-the-derived-tree-question-asked-honestly
git rev-parse --verify 113-the-harness-delivers-what-it-documents > /dev/null
git cat-file -e 487bbd9:specs/117-the-derived-tree-question-asked-honestly/spec.md
# Declared and read through the CLI, redirected rather than piped (spec 085 D-4).
target/release/spec-spine registry show 120 --json > "${TMPDIR:-/tmp}/ss120-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss120-show.json')); assert d['id'] == '092-the-engine-ships-governance-not-an-environment', d"
rm -f "${TMPDIR:-/tmp}/ss120-show.json"
# The stack's own gate, last, because a green governance loop over code that
# does not compile asserts nothing.
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```
