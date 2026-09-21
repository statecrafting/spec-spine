# 06: The harness and its distribution (2026-09-15)

> **Status (2026-09-21): historical.** Superseded by note 07 4, which
> dispositions every section of this note against the Statecraft boundary, and
> by note 08 2.2, which holds what is left of the harness question. Several
> sections here describe distributing a harness this repository no longer
> distributes. Read it for the measurements, not as a plan.

A design note, not a spec. Nothing here is filed, approved, or implemented.

Notes 04 and 05 record what the **authority records** should become. Neither
records what the **harness** around them should become: how the skills, agents,
rules and hooks that drive the governed loop are versioned, distributed,
pinned, activated and measured. That gap is what this note closes.

Its source is a Claude Code session of 2026-09-15 whose subject was the user's
global agent configuration ("Evaluate global Claude setup"). That session
evaluated personal settings, four repositories' instruction files, Statecraft's
driver and profile code, Cursor's configuration and Swamp's installed skill
package. Most of its output is personal configuration and belongs to its owner.
What is recorded here is the subset that names spec-spine as an owner, plus the
boundaries and handoffs for the parts that do not.

## 0. The four states, kept apart

This note uses one vocabulary throughout, and every row in every table below
carries one of these words. They are not interchangeable.

| State | Meaning |
|---|---|
| **proposed** | Someone recommended it. No owner has adopted it. Most of this note. |
| **adopted** | The owner recorded the decision. For spec-spine that means an approved spec or a dated decision entry; for the ecosystem it means an entry in revision 4's adoption record. |
| **implemented** | Code exists on the default branch and its spec says `implementation: complete`. |
| **released** | A tagged build carries it and the published distributions ship that build. |

A recommendation in the source session is **proposed**, and reading it here
does not advance it. Revision 4's own header says the same thing about its
rows: "Approval status: PROPOSED, awaiting Bart's adoption", with four
statecrafting-profile rows the single recorded exception.

## 1. Provenance, and what the measurements do and do not cover

The source session's measurements, preserved as it reported them:

| Observation | Value |
|---|---|
| Claude Code | 2.1.270, native installation, `claude doctor` clean |
| Globally installed `spec-spine` | 0.18.0 |
| spec-spine checkout it read | `0dc95ca` |
| Other checkouts read | statecraft-cli `ccec621`, butler-ai `275b3a4`, aicortex `932ef9a` |
| Swamp upstream read | `d18f2d86575c0931912a7f6194e041acdb27d6e5` |
| Read-only freshness checks | passed in all four repositories, 46 to 111 ms per observed run |

Three limits travel with those numbers, and they are the session's own:

- The checkout it read is not this one. `0dc95ca` predates 092 to 097; this
  note is written against `3bc004b`, where the corpus is 98 specs, all
  approved, and the in-tree binary answers `0.19.0`. Every claim in section 3
  that names current behavior was re-verified at `3bc004b` and says so.
- Settings presence does not prove a feature is active in a given session.
  Account availability, launch flags, project settings and the host
  application all intervene.
- No paid-model benchmark and no complete autonomous cycle was measured. The
  cost arguments below are reasoned priorities, not measured savings. That is
  precisely why section 3.10 proposes a measurement plan rather than a target.

## 2. The boundary, stated before the proposals

Note 04 opened with a boundary for the same reason: a proposal that names the
wrong owner cannot be evaluated, only argued about.

| Concern | Owner | Why not spec-spine |
|---|---|---|
| Personal house style, collaboration defaults | The user's global configuration | Applies in a repository with no corpus at all |
| Permission profile, execution posture, additional directories | The user's global configuration, and Statecraft for launched workers | Describes the machine and the runner, not the authority graph |
| Generic skill and agent bodies | **spec-spine** (`kit/`) | Already the maintained source; 048 and 081 pin this repository's copy to it |
| Package identity, revision, digest, compatibility floor | **spec-spine** | It is the thing being pinned, and `[meta] required_version` already establishes the pattern |
| Installing, upgrading and caching the package | **spec-spine's distribution** (the CLI, npm and PyPI shims) | It already ships a binary per platform; a package is the same problem |
| Repository lifecycle policy, gate commands, architecture, domain rules | The adopting repository | Differs between this producer corpus and every adopter |
| Hook activation, and whether a hook refuses or advises | The adopting repository | 090 already made registration a per-clone act |
| Scheduling, retries, model, effort, budget, timeout, completion | **Statecraft** | It launches the worker and holds the lease |
| Cross-session product memory and retrieval | **Aicortex**, when implemented | Provenance-bearing retrieval on demand, not injected history |
| Product-specific invariants (Butler capture/privacy, Aicortex erasure) | Those repositories | Globalizing them makes unrelated projects pay their context cost |

Two handoffs follow from that table and are worth naming, because they are the
seams where a proposal here becomes someone else's work:

1. **spec-spine publishes a package identity; Statecraft records which one
   ran.** spec-spine cannot observe a worker's resolved skill revision, and
   Statecraft cannot decide what revision a repository requires.
2. **spec-spine states what a check proves; the repository decides when to run
   it.** The gate list in `AGENTS.md` is authored per repository; what the
   verbs refuse is not.

## 3. The proposals

Each subsection is one proposal, with its evidence, its owner, its decision
status, and the decisions that are genuinely open. None is filed.

### 3.1 A small global personal policy

**Proposed. Owner: the user's global configuration, not this repository.**

The finding: two hand-maintained copies of the same house style
(`~/.claude/AGENTS.md`, `~/.claude/CLAUDE.md`) had already drifted in product
names and hook paths, and Claude Code does not load `AGENTS.md` on its own; an
explicit `@AGENTS.md` import from `CLAUDE.md` is what makes the shared file
load. Calling a file a cross-agent standard does not change a loader.

Recorded here only for its boundary: the global layer holds preferences that
apply in a repository with no corpus. It must not hold the spec-spine
lifecycle, the Rust architecture, or a list of repositories. Nothing in this
subsection is spec-spine work.

### 3.2 A versioned, namespaced shared skill package

**Proposed. Owner: spec-spine. The largest item in this note.**

The nine generic skills (`build`, `code-review`, `commit`, `next`, `prime`,
`setup`, `ship`, `spec`, `verify`) were observed byte-identical across four
repositories, with `shepherd` in two versions, the older one still in
butler-ai and aicortex. That is distribution by copying, and it drifts in
exactly the way the second row predicts.

The proposed model, in the source session's terms:

- One versioned spec-spine agent package.
- One globally cached installation per version.
- A repository-owned declaration of the required package revision.
- Namespaced invocation.
- A compatibility check before dispatch.
- A clean-machine installation path for CI and remote workers.
- The resolved package digest recorded in Statecraft's run evidence.

Three constraints that any filing must answer, because each one defeats the
naive version:

**The name-precedence trap.** A personal skill resolves before a project skill
of the same name. Installing `~/.claude/skills/build` can therefore replace a
repository's `/build` while its committed copy sits untouched on disk, which is
the worst possible failure: the governed copy is present, readable, and not the
one that ran. Plugin skills are namespaced and coexist; a non-plugin shared
install needs distinct names. This is the reason the proposal says *namespaced*
rather than *global*.

**The evidence question, which is the adopter's and not the producer's.**
Distributing a package globally does not move anything out of this repository.
`kit/**` is the maintained source: it stays in the tree, stays claimed, and
stays in `[index] extra_hashed_inputs`, so a change to a skill stales this
ledger exactly as it does today. Publishing a package is an additional output
of that source, not a relocation of it.

The question arises one level down, **for an adopter who replaces a local
harness copy with an external package**. Today an adopter who hashes
`.claude/skills/**` has repository-local evidence of the harness that governed
their tree. If those bytes leave their tree, two properties have to survive the
move, and a package model has to say how:

1. The adopter's **required revision or digest is itself governed**: declared
   in a file their corpus hashes, so changing which harness they require is a
   change their own ledger sees.
2. **Run evidence identifies the package actually resolved**, not merely the
   one requested. A declared floor and a resolved revision are different facts,
   and only the runner can observe the second (§2, handoff 1).

Those two clauses are what makes the adopter's substitution honest. Neither
requires this repository to stop hashing its own source.

**`init --with-kit` is not this.** It scaffolds local files and refuses to
clobber (095 §3.3). It has no notion of a version, an upgrade, a cache, or a
compatibility floor. A global distribution mode is new work, not a flag.

Open decisions: whether the package is a Claude Code plugin (which gives
packaging and cache behavior but does not by itself prove a repository ran the
required revision), a `spec-spine`-managed directory, or both; whether the
required revision lives in `spec-spine.toml` beside `[meta] required_version`;
and what a compatibility failure does (refuse, warn, or report).

### 3.3 The repository-local layer, named explicitly

**Proposed. Owner: the adopting repository. Mostly already true here.**

What stays local, in the proposal: lifecycle policy, architecture, acceptance
criteria, gate commands, domain rules, and hook activation. This repository
already puts the project layer in `AGENTS.md` rather than in the skills, which
is 093's ruling and 093's shape.

The item that is **not** already true is lifecycle policy as a *queryable*
thing rather than prose. See 3.6.

### 3.4 Supported-agent adapters, and one maintained source

**Proposed. Owner: spec-spine. Wave B5 is a proper part of it, not a
substitute.**

Note 05 §3 B5 records that `.agents/skills/` is fifteen tracked files carrying
the pre-081 skill set, with `.Codex/rules/` paths that exist on no filesystem,
and that the ruling is to generate the supported trees from one source with a
parity test, in the shape `kit_embedded.rs` already uses.

That ruling stands and is the right first step. It is worth being exact about
what it does not do: generating two trees from one source gives **parity**. It
does not give global installation, upgrade, pinning, a compatibility floor, or
a recorded resolved identity. B5 and 3.2 are the same story at two scales, and
filing B5 alone must not be reported as delivering this note.

Adapter shape, from the observed reference implementations: Swamp records which
agents a repository supports and materializes each agent's instruction and
settings format, installing skills globally on `repo init` and refreshing them
on `repo upgrade`, with `~/.claude/skills` for Claude and `~/.agents/skills`
for Cursor and others. Cursor documents compatibility with both directories, so
three copies for three tools is a choice, not a requirement. Cursor's CLI
permissions use `Shell(...)` where Claude uses `Bash(...)`: a shared semantic
policy needs per-tool rendering, and copying the JSON is not an adapter.

One deliberate divergence from that reference: Swamp's global copy is mutable.
For governed development the proposal is to preserve a content-addressed or
otherwise immutable revision per run, so the question "which harness produced
this evidence" has an answer after the fact.

### 3.5 Task-specific startup reads

**Proposed. Owner: spec-spine (`AGENTS.md` protocol and `/prime`).**

`/prime` currently executes the full New Sessions protocol: README,
constitution, contract, rendered index, coverage, diagnostics, lifecycle
counts, ready set, spec list, three directory listings, recent history and the
last commit's diff. That is the right report for an explicit orientation
request. It is mandated as the first action of **every** session, including a
worker that was handed one spec id.

The proposed split, unchanged from the source:

| Startup path | Reads |
|---|---|
| Ordinary development | Short project instructions, worktree status, the relevant task or spec |
| Statecraft worker | Work order, execution profile, owning spec, required dependencies |
| Explicit `/prime` | Full repository orientation, as today |
| Governance diagnostic | Full freshness, coverage and diagnostics |
| After compaction | Short task checkpoint and changed facts |

The full `/prime` stays. What is proposed is to stop making its entire report a
universal prerequisite. Note that `AGENTS.md` is itself a hashed input here, so
this is a change that stales every shard: expensive and deliberate, exactly as
`CLAUDE.md` says.

Open decision: whether the paths are separate skills, one skill with a mode
argument, or a protocol section the caller selects. Which reads each path
carries is performance-sensitive and wants the 3.10 numbers; that a worker
handed one spec id should not be required to run the full orientation is a
coherence argument and does not.

### 3.6 Build eligibility belongs to repository policy

**Proposed. Owner: spec-spine (the kit's `build` skill) plus each repository's
policy. A live contradiction, not a hypothetical.**

The kit's shared `build` skill says a draft spec is never built, a dirty tree
or non-default branch is a halt, and an argument is required. This
repository's `AGENTS.md` "Working the backlog" explicitly files a spec as
`draft`, builds it, and ratifies it in a separate PR, and step 2 already
carries the exception in prose: "`/build`'s preflight accepts `draft` in this
repository when a human named the id".

That prose exception is the evidence for the proposal. The lifecycle rule is
compiled into a shared skill and patched in project prose, which is the
arrangement 048 exists to prevent. The proposal is to make the skill **ask the
repository's policy whether a work order is eligible** rather than embed
`status == approved`.

Four preflight shapes are proposed instead of one "must start clean on the
default branch":

1. Starting a new work order.
2. Resuming a partially implemented work order.
3. Repairing a PR.
4. Continuing inside an already isolated worktree.

**What to specify first, and what to leave open.** The requirement is that the
shared skill **defers to repository policy** on eligibility, and that it
supports the three states a real work order is in: new work, resumed work, and
repair. That much is specifiable today against the contradiction above, and it
is the part worth filing.

How the policy is expressed is a separate and later question. A `[policy]`
table in `spec-spine.toml` would make eligibility a governed, queryable fact
and is one candidate; so is a documented convention the skill reads from
`AGENTS.md`, or an explicit human-named id as today's prose exception already
allows. A machine-readable schema is **one possible design, not an established
requirement**, and nothing here should be read as having chosen it.

### 3.7 One scheduler

**Proposed. Owner: Statecraft, with a constraint on spec-spine's skills.**

Statecraft's observed implementation already derives permission arguments from
a recorded execution profile, supports `bypass` and `guarded`, passes explicit
models to the worker, controls max turns and timeout, and maps build and ship
to the strong tier with shepherd and verify to the fast tier (observed default
pair: `claude-opus-5` strong, `claude-sonnet-5` fast). A consequence worth
recording: changing a global `model` or `defaultMode` does not necessarily
change what a Statecraft worker runs.

Recommended ownership, as proposed:

| Responsibility | Owner |
|---|---|
| Select the next eligible work order | Statecraft |
| Lease the work order and worktree | Statecraft |
| Choose model, effort, budget, timeout | Statecraft execution profile |
| Load work-order context | A thin skill or worker prompt |
| Implement and investigate | The Claude worker |
| Validate repository invariants | Repository commands and CI |
| Retry with evidence | Statecraft |
| Resume after interruption | Statecraft |
| Record actual completion | Statecraft |
| Cross-session product memory | Aicortex, when implemented |

The constraint on spec-spine: "one worker, one spec" is compatible with full
autonomy when the scheduler launches the next worker. It becomes friction when
every skill independently decides a human must restart the next stage. Do not
run a Statecraft stage loop, a Claude dynamic workflow and a skill-level
polling loop as competing schedulers over the same work.

One observed mismatch to hand back: the guarded baseline profile allows git,
the GitHub CLI, Bun and spec-spine, but not the full Rust and Make toolchain
this family's gate actually needs. Ambient global allowances can hide that. A
profile that claims to be self-contained should declare the commands it
requires. That is Statecraft's to fix; it is recorded here because spec-spine's
gate list is what the profile must cover.

### 3.8 Validation boundaries and evidence reuse

**Proposed. Owner: spec-spine's skills, with the rule stated once.**

The loop can run overlapping checks during preflight, implementation commits,
verification, shipping, PR preparation, remediation and post-merge
verification. Some repetitions are justified because the tree changed; others
re-check identical inputs.

The proposed contract:

- Targeted tests during implementation.
- The full project gate at a defined delivery boundary.
- Re-run affected checks after remediation.
- Verify the actual merge result.
- **Reuse evidence only when its commit or tree, toolchain, command and
  relevant inputs all still match.**

Independent CI verification is not the target and does not move. What the
proposal removes is ambiguous ownership of the same local check.

### 3.9 Hook verdicts and enforcement boundaries

**Proposed. Owner: spec-spine (`kit/settings.json` and the repository copy).
Partly answered by 090 already; one item in this subsection is an unfiled
defect with two cross-references that read as if it were filed.**

Re-verified at `3bc004b` against the 0.19.0 binary:

| Finding | Status at `3bc004b` |
|---|---|
| The `Stop` hook calls every nonzero `check` result "STALE" | **Fixed by 099.** The hook runs `"$sc" check >/dev/null 2>&1 && exit 0` and otherwise prints `[freshness] STALE`. Exit 1 (validation, or a refused unresolved unit) and exit 3 (I/O, parse, schema, config) are reported as staleness. |
| The PR hook mislabels the same codes | **Fixed by 080, completed by 104.** The gate switches on the code; 104 added spec 093's `check --help` probe on the exit-2 arm, so a binary predating the verb is no longer called stale there. |
| The PR hook's derived test is `git diff --quiet -- .derived/` | **Current.** That asks about unstaged tracked changes. A staged shard and an untracked new shard both pass it. `git diff HEAD -- .derived/` plus an untracked check is the honest form of the question; a tested CLI verb answering "is the derived tree committed" is better still. |
| Push interception matches command text | **Current and bounded by design.** 071 gave it a (command, branch) matrix; `git -C`, wrappers and alternate quoting remain outside what shell-text matching can decide. |
| Missing `jq` yields "skipped" and success | **Current.** Advisory and enforcing behavior share one exit path. CI is the enforcement boundary, which is the right place; the hooks should say which they are. |
| `Stop` runs per response, not per session | **Current.** The name suggests session end; in Claude Code it fires when a response finishes. |

**The unfiled one, now filed.** `couple --head HEAD` builds a `base...head`
diff, so the documented pre-commit coupling check cannot see the change being
committed. Spec 094 §4 placed it out of scope and said it "is filed
separately"; spec 073 §4 placed it out of scope and pointed back at 090.
Checked at `3bc004b` and again at `a6ef6e3`: no spec in the corpus filed it, and
the phrase in 094 §4 was untrue from the day it was written.

**Spec 081 files it** (2026-09-16, H-7 decided: fix it in `couple`).
`--include-uncommitted` unions the committed range with `git diff HEAD`, off by
default so a dirty runner cannot move a CI verdict. 094 §4's phrase is amended
there; 073 §4's sentence says only that 090 "named" it, which was accurate, and
is left alone.

### 3.10 A measurement plan

**Proposed. Owner: spec-spine for the repository-side numbers, Statecraft for
the run-side ones.**

Parts of sections 3.2, 3.5 and 3.8 are argued from cost, and no cost was
measured.

**What measurement gates, and what it does not.** A measurement is required
before **claiming a performance improvement** and before **choosing a
performance-sensitive default** (which reads a startup path performs, how much
of `/prime` a worker inherits, where a reuse threshold sits). It is not a
precondition for filing a spec. Three kinds of work here proceed without it:
fixing something that is wrong (3.9's remedy line, the hook verdicts),
authoring a design (3.2's package contract), and removing an instruction that
contradicts another instruction (3.6's eligibility rule). Those are correctness
and coherence, and a benchmark does not make them more or less true. Blocking
them on a benchmark would be the same error as claiming a speedup without one.

| Dimension | Where it comes from |
|---|---|
| Startup cost per session path | Wall time and tool-call count for `/prime` versus a worker path |
| Context at the first assistant turn | Tokenizer counts for the loaded instructions plus tool output, per path. Instruction bytes and word counts (the §6 table in the source session) are a proxy for comparing documents, not a token measurement, and must not be reported as one |
| Gate repetition | How many times an identical check runs against an unchanged tree in one work order |
| Retries | Statecraft's retry count, with the reason |
| Human interventions | Stops that required a person, and whether policy or a skill caused each |
| Accepted outcomes | Work orders accepted, **including those that needed remediation**. Remediation is the loop working, not a failure; count it separately rather than excluding it from the numerator, or the metric rewards a harness that never catches anything |

The freshness check itself is cheap (46 to 111 ms observed). The cost is
repeated tool activity, repeated output, and remediation caused by a
misleading result. That last term is why 3.9 is not cosmetic, and why it does
not wait for this table.

## 4. What this note deliberately does not import

- **The personal settings inventory.** The source session evaluated roughly
  forty global settings, 169 allow entries, 22 additional directories (7 of
  which do not exist), and proposed replacement files. That is the user's
  configuration. It does not belong in a governed corpus, and copying it here
  would create a second, stale copy of a file the user edits directly.
- **Statecraft execution profiles.** Model, effort, budget, timeout and
  permission posture describe the runner. Recorded as a boundary in section 2
  and as an ownership table in 3.7; not specified here.
- **Aicortex retrieval.** A natural future layer for reusable learning, which
  should return provenance-bearing context on demand rather than inject a
  history collection into every worker. Its contracts are aicortex's.
- **Butler and Aicortex domain rules.** Product-specific, and globalizing them
  would make unrelated projects pay their context cost and risk applying the
  wrong constraint.

## 5. Open decisions, and what each one bears on

None of these is settled, and none should be settled by a build.

**They are open questions, not gates.** Each one bears on the shape of a
proposal in section 3. None of them blocks an independent correctness fix, and
a spec that does not depend on one should not wait for it: the `Stop` hook's
remedy line, `/shepherd`'s missing reviewer, `govern.yml`'s duplicate gate and
the generated-protocol drift are all filable while every row below stays open.

| # | Decision | Bears on |
|---|---|---|
| H-1 | Is the shared harness distributed as a Claude Code plugin, a spec-spine-managed package, or both? | 3.2, and therefore the scope of 3.4 |
| H-2 | Where does a repository declare the required harness revision, and what does a compatibility failure do? | 3.2 |
| H-3 | For an **adopter** who replaces a local harness copy with an external package: where is their required revision declared so their own corpus hashes it, and how does run evidence name the package actually resolved? `kit/**` stays hashed here either way | 3.2 |
| H-4 | Do the startup paths become separate skills, one skill with a mode, or a caller-selected protocol section, and which reads does each carry? | 3.5. The second half wants 3.10's numbers; the first half does not |
| H-5 | How is repository eligibility policy expressed: a governed schema key, a documented `AGENTS.md` convention, or an explicit human-named id? | 3.6. Open **after** the skill respects policy and handles new, resumed and repair work, which does not wait on it |
| H-6 | Does the `Stop` hook refuse, or advise? Its verdict is wrong either way; what it should do when `check` exits 1 or 3 is a policy choice | 3.9 |
| H-7 | Is the staged-coupling defect fixed in `couple` (a new mode), in the harness (commit then check before push), or declared permanently out of scope with the two cross-references corrected? | 3.9, and the accuracy of 094 §4 |

## 6. Relationship to the other notes

- **Note 04** owns the authority records. This note owns the harness that
  carries them. They meet at 3.2's "resolved package identity in run evidence",
  which is an evidence-record question note 04's vocabulary could answer.
- **Note 05** owns the filed and unfiled backlog. Its wave B is the delivery
  debt in the current harness; this note is the redesign wave B does not
  reach. Note 05 §9 accounts for both against one table.
- **grand-refactor** owns ecosystem sequencing. Nothing in this note is
  dispatched by revision 4, and revision 4's spec-spine rows (SP-01 to SP-03)
  do not mention the harness. That is a gap in the ecosystem record, not a
  decision against it, and section 2's ownership table is the handback.
