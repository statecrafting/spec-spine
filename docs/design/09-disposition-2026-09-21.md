# 09: The documentation backlog, dispositioned (2026-09-21)

A design note, not a spec. It reconciles every proposal this repository's
documentation carries against the corpus as it stands on 2026-09-21, and it is
**the current backlog record**. Notes 05 and 06 are historical, note 07 §4 is
their disposition against the Statecraft boundary, and note 08 is the handoff
from the realignment. This note supersedes none of them as a record; it
supersedes all of them as the place to look for what is open.

Measured against `4ab1b31e` with the in-tree binary, `spec-spine 0.21.0`. Corpus
state at that commit: 100 specs, `000` to `099`, every one `approved` and
`complete`; `registry plan` reports nothing ready and nothing blocked; `check`
is fresh on both trees; `index coverage` is 97/97 specifically claimed;
`index diagnostics` is empty.

**An empty ready set is not an empty backlog.** `registry plan` schedules filed
specs. Every item below is a proposal that was never filed, a decision nobody
has taken, or work whose counterparty is another repository, and none of those
is visible to the planner. That is the gap this note exists to close.

## 0. How to read an id in this note

Spec 095 renumbered the corpus. Notes 04, 05 and 06 were written before that and
were repaired in place by spec 098, which could not decide every form: note 05
still carries five bare ordinals that no longer resolve (its own §4 and spec
098 D-5 record why, and leaving them was the right call, not an oversight).

**Every id in this note is a current id.** Where a source note says something
else, the translation is given once, here:

| Note 05 line | Says | Is now |
|---|---|---|
| 389 | `093, amended by 103` | 074, amended by 082 |
| 402 | `102, amending 094 §4` | 081, amending 073 §4 |
| 421 | `080, 099, 104` | all three collapsed into 093 |
| 424 | `101, amending 069 §3.1 and 079 §3.1` | 080 (the two section refs were already repaired) |
| 506 | `103 adds amends_verification` | 082 |

`docs/corpus-map.md` resolves any other pre-collapse citation, both directions.

## 1. The disposition matrix

Seven dispositions, used strictly:

- **implemented**: code on the default branch, an approved spec says so.
- **superseded**: the proposal's subject no longer exists, or another owner's
  decision replaced it.
- **transferred**: a different repository owns it now.
- **decision**: a human must choose before anything is filed.
- **draftable**: could be filed today; nothing blocks it.
- **filed**: filed by this session, `status: draft`.
- **deferred**: deliberately not scheduled, with the reason.

A proposal is not closed because the wave around it shipped, and it is not
reopened because an old note still calls it proposed.

### 1.1 Correctness and engine behavior

| # | Item | Source | Owner | Disposition | Evidence | Next action |
|---|---|---|---|---|---|---|
| E1 | Deleted paths attributed through the head index | note 08 §2.4; 092 §3.11.1, D-11 | spec-spine | **filed** as spec 100 | Reproduced 2026-09-21 in a fixture corpus: a removal that deletes the file and withdraws the claim in one change is refused `C-001` against the package floor. The refusal's own remedy 2 raises `I-004` and `check --fail-on-unresolved` exits 1 | Build 100. It is the next work order |
| E2 | Mode-only and binary changes reach the gate | note 05 §9.2 | spec-spine | **implemented** | spec 073 | none |
| E3 | Staged coupling, `--include-uncommitted` | note 05 §9.2, R-3 | spec-spine | **implemented** | spec 081, amending 073 §4 | none |
| E4 | `I-004` refusal exits through the staleness code (N6) | note 05 §9.3 | spec-spine | **implemented** | message half with spec 098; exit-code half spec 080, amending 069 §3.1 and 079 §3.1 | none |
| E5 | Exit-code-aware PR gate | note 05 §9.2 | spec-spine | **implemented** | spec 093 (three pre-collapse specs collapsed into it) | none |
| E6 | Versioned, sorted read documents (F8, D7) | note 04 §9 D7 | spec-spine | **implemented** | spec 074, acceptance amended by 082 | none |
| E7 | Stray shard orphaned at the verbs | note 05 §9.2 | spec-spine | **implemented** | spec 076 | none |
| E8 | One hash, one construction, one name (F9) | note 04 | spec-spine | **implemented** | spec 077 | none |
| E9 | Claim window declared, near misses reported | note 05 §9.2 | spec-spine | **implemented** | spec 075 | none |
| E10 | Markdown blind spot in `C-002` | note 05 §9.2 | spec-spine | **deferred** | Answered by spec 064's opt-in; the seven inert `// Spec:` headers are inert by design | none unless a spec argues otherwise |
| E11 | Coherence guard is a prompt, not a gate (G7) | note 02 §G7; note 04 D-4 | spec-spine | **deferred** | Design-first by standing decision: the naive rule (refuse a diff that both narrows a claim and edits the claimed code) refuses legitimate refactors, which are the same shape | Do not file a mechanism before it is designed |

### 1.2 Consumer contract and evidence

| # | Item | Source | Owner | Disposition | Evidence | Next action |
|---|---|---|---|---|---|---|
| C1 | Readiness read as approval (N2) | note 05 §9.3 | spec-spine | **filed** as spec 101 | Confirmed 2026-09-21: `docs/api.md` line 270 describes `plan`'s shape and never says membership is scheduling. `ReadySpec` is `{id, title}` | Build 101. Documentation only, no emitted byte moves |
| C2 | `status` on `ReadySpec` | note 05 §9.3 | spec-spine | **filed** as spec 102 | Same read. Explicitly "not required to make the contract truthful" | Build only for a named consumer (spec 102 D-1). Independent of 101 |
| C3 | Portable strict-verifier fixtures | note 04 §7, D-6; AE §8 | spec-spine emits; consumer owns the bundle | **filed** as spec 103 | Note 04 offered three consumers "the cases as runnable commands in 068's block". Those commands run against this tree and this key; no external verifier can execute them | Build 103. D-6 (the neutral verifier's home) stays a family decision and is not in it |
| C4 | AuthoritySnapshot, `frame/1` | note 04 §4.2 | spec-spine | **implemented** | spec 070 | none |
| C5 | AuthorityDelta | note 04 §4.3 | spec-spine | **implemented** | spec 071 | none |
| C6 | Verifier checks the bytes it was given | note 04 P0 | spec-spine | **implemented** | spec 068; AE §8's four bold rows closed | none |
| C7 | Committed index compared, not trusted | note 04 P0 | spec-spine | **implemented** | spec 069, amending 022 §5 | none |
| C8 | Build identity (D-1) | note 04 §9 | spec-spine | **implemented as documented limit** | Decided 2026-09-13 for `v0.19.0`: document the limit, stated in AE §2. Neither a commit stamp nor a pre-release `main` version was adopted | none |
| C9 | Requests to statecraft-cli (R1 to R7), the control plane (S1 to S4), hqgit (H1) | note 04 §7 | those repositories | **transferred** | Each names the consumer that would act; none changes a contract from here | Nothing here. C3 is the one piece this repository owes |

### 1.3 The deferred design backlog

Every row is **specified enough to draft and deliberately not scheduled**. §5
states the disposition this note proposes for all of them at once.

| # | Item | Source | Register | Disposition | Why not now |
|---|---|---|---|---|---|
| P1 | Obligation records | note 04 §4.4 | A01, A02, A08 | **deferred** | Prerequisite met (`frame/1` has an owning spec since 070), so this is buildable. No consumer names it. SP-03 proposes deferral and is not adopted; R-5 is the open decision |
| P2 | ContextClosure | note 04 §4.5 | B04, B05 | **deferred** | As P1. Also needs D-5's obligation vocabulary to be settled first if closures are to reference obligations |
| P3 | WorkScope | note 04 §4.6 | B01 | **deferred, and narrower than the note says** | Its overlap half is **implemented**: spec 072 reports that two ready specs can collide. What is left is the mutable/shared/ownSpec scope document |
| P4 | Declared impact and conflict sets | note 04 §5 P2 | A04, A05, B03 | **deferred** | Needs P1's obligation ids to name what an impact is against |
| P5 | Interface references, digest-pinned imports | note 04 §5 P2 | B22, A06 | **deferred** | Needs a second corpus in this family to pin against. Discovery, fetching and trust in the exporter stay outside |
| P6 | Reviewed move mapping | note 04 §5 P2 | A07 | **deferred** | Spec 100 §3.6 keeps the two-facts semantics a mapping would build on, so filing 100 first is the right order |
| P7 | Typed overlays: effect contracts, budgets, dependency rationale, contract adapters | note 04 §5 P2 | B15, B18, B24, A09 | **deferred, via the overlay seam** | `docs/overlay-contract.md` is the existing mechanism. Core `Unit` identity is untouched by all four, which is what makes them overlay work rather than engine work |
| P8 | Waiver lifecycle over explicit inputs | note 04 §5 P2 | B17 | **deferred** | Pure evaluation of caller-supplied time, ancestry and usage. No counter in an authored file, and no atomic consumption |
| P9 | A10 authoring-tool adapters, B23 intent manifest | feature register | A10, B23 | **deferred** | SP-03 proposes deferral. A10 is adjacent to harness delivery, which is now Statecraft's, so it should be re-scoped before it is re-proposed |
| P10 | Language bindings (napi / pyo3 / cgo) | `docs/bindings-plan.md` | | **deferred by mandate** | The document's own header: design only, no binding code in this repository. The `&str -> Result<String, Error>` facade is the seam and is maintained; spec 100 §3.8 keeps it additive |

### 1.4 Harness, layout and the Statecraft boundary

| # | Item | Source | Owner | Disposition | Evidence | Next action |
|---|---|---|---|---|---|---|
| H1 | Global personal policy | note 06 §3.1 | the user's global configuration | **transferred** | note 07 §4: never spec-spine's | none |
| H2 | Versioned namespaced harness package | note 06 §3.2 | Statecraft | **superseded** | note 07 §4; statecraft-cli spec 002 §3.14 | none |
| H3 | Repository-local layer named explicitly | note 06 §3.3 | this repository | **implemented** | `AGENTS.md` carries the project layer and the `## Rules` section (spec 093 §4.12, D-4) | none |
| H4 | Supported-agent adapters, one maintained source | note 06 §3.4 | Statecraft | **superseded** | note 07 §4 | none |
| H5 | Task-specific startup reads | note 06 §3.5 | Statecraft | **superseded as a deliverable** | note 07 §4. This repository's own `AGENTS.md` protocol stays its own | none |
| H6 | Build eligibility from repository policy | note 06 §3.6 | Statecraft | **superseded** | note 07 §4. The governance half (`registry plan`) was always spec-spine's and stays | none |
| H7 | One scheduler | note 06 §3.7 | Statecraft | **superseded** | note 07 §4 | none |
| H8 | Validation boundaries and evidence reuse | note 06 §3.8 | mixed | **decision** | Governance-side reuse stays a spec-spine question; orchestration-side is Statecraft's. Nobody has separated them | Low priority; no measured defect |
| H9 | Hook verdicts, `Stop` on exit 1 and 3 | note 06 §3.9 | Statecraft | **transferred** | Partly delivered here by specs 093 and 104's predecessors; H-6 is now Statecraft's question | none here |
| H10 | Measurement plan | note 06 §3.10 | Statecraft | **superseded** | note 07 §4: the thing it measured is not distributed here | none |
| H11 | `.claude/` retirement | note 08 §2.2; note 07 §6 | both | **blocked, order fixed** | §4 of this note is the acceptance checklist | Nothing until Statecraft delivers |
| H12 | `@.statecraft/AGENTS.md` bridge | note 08 §2.6; 092 §3.10 | Statecraft initializer | **blocked** | `.statecraft/environment.json` does not exist here; an import of a missing file is a broken instruction | Enrol this repository first (§4 step 1) |
| H13 | Kit-era items: B1 generated protocol drift, B2 `govern.yml`, B3 install ratchet, B5 one agent-tree source | note 05 §3, §9.1 | spec-spine | **superseded** | The kit is gone (092) and `scaffold_init_json` emits no `AGENTS.md`, no `CLAUDE.md` and no `.claude/`. B1's subject, the scaffolded protocol string, no longer exists | none. Do not refile |
| H14 | B4 `/shepherd` misses `issues/<n>/comments` | note 05 §3 | spec-spine | **implemented** | spec 093 (absorbed the pre-collapse shepherd spec) | none |

### 1.5 Documentation, adoption and release

| # | Item | Source | Owner | Disposition | Evidence | Next action |
|---|---|---|---|---|---|---|
| A1 | Adopter migration note before the next release | note 08 §2.8 | spec-spine | **draftable, and it gates the tag** | Six repositories pin releases; measured in §3.1 | Write it before the next release, not after |
| A2 | Where the deleted `website/` documentation goes | note 08 §2.7 | **decision** | Deleted, not moved; content is in git at that branch's parent. Two facts were rescued into `docs/adoption-guide.md` and `docs/api.md` before deletion; the rest is unexamined | §6 D-3 |
| A3 | Enable `[coverage] governed_scope` here | note 05 §9.4 R-7 | **decision** | Measured in §3.2: exactly three paths would become `C-002` | §6 D-2 |
| A4 | Note 05's five stale ordinals | 098 §4, D-5 | spec-spine | **deferred, deliberately** | 098 D-5: a rule decided every ordinal it touched, and these five are indistinguishable from counts without reading the sentence | §0 translates them. Do not sweep |
| A5 | Prose citing removed documents | note 08 §3 | spec-spine | **not a defect** | An approved spec's prose records what was true at ratification and is not edited (spec 037) | none |
| A6 | `.statecraft/derived/attestation/` accumulation | note 08 §3 | spec-spine | **not a defect** | On-demand output, gitignored | none |
| A7 | Adopter-side follow-ups | note 03 §5 | each adopter | **transferred** | "None of these is spec-spine's to fix" | none here |

## 2. What this note corrects in note 08

Note 08 is the right entry point and three of its statements have gone stale.
Correcting them here rather than editing it keeps it a dated handoff.

1. **"Specs 098 and 099 are `draft`"** (§1). Both were ratified in `8c974217`
   on 2026-09-21. The corpus is 100 approved specs.
2. **"the corpus is 97 specs, contiguous `000` to `096`"** (§1, last bullet).
   That was true of the collapse itself. Four specs landed after it.
3. **§2.5's "done" and §2.1's "done"** are accurate; §2.2, §2.4, §2.6, §2.7 and
   §2.8 are accurate as open. §2.4 is now filed as spec 100, which is the one
   line of §2 that changes.

Note 08 §2.6's inventory of what a `.claude/` retirement must touch was checked
path by path against the tree and **is accurate**: five specs carry `.claude/**`
units (061, 062, 064, 092, 093), one carries a `references` to a skill file
(053), and there are exactly three `.claude` globs in `[index]
extra_hashed_inputs`. §4.3 restates it with the typed reads that confirmed it.

## 3. The two measurements this note adds

### 3.1 Who actually pins spec-spine, measured 2026-09-21

Read-only inspection. Note 03 and note 08 both say "four repositories"; there
are six, and two of them are the counterparty.

| Repository | `[meta] required_version` | Workflow / Makefile pin | `derived_dir` | Local harness copy |
|---|---|---|---|---|
| `hqgit` | `>=0.18.0` | `v0.18.0` | `.derived` | `.claude/` with `rules/`, `skills/`, `agents/`, `settings.json` |
| `aicortex` | `>=0.20.0` | `v0.20.0` | `.derived` | as above, plus `agent-memory/` |
| `rahi` | absent | `0.20.0` in `Makefile` | `.derived` | as above, plus `agent-memory/` |
| `claude-observatory` | absent | `0.15.0` | `.derived` | archived |
| `statecraft` | `=0.20.0` | | `.derived` | |
| `statecraft-cli` | `=0.20.0` | | `.statecraft/derived` | |

Three facts a migration note must carry and could not have guessed:

- **`rahi` has no version pin at all**, only a `Makefile` default. Spec 055's
  floor cannot fire there, so a binary/corpus mismatch is silent.
- **Only `statecraft-cli` has moved to the managed layout.** Every other adopter
  is on `.derived`, which is the product default and is **not** deprecated by
  spec 092: the relocation is this repository's configuration and Statecraft's
  managed layout, not a change to what an unconfigured corpus gets.
- **`claude-observatory` is archived** and three releases behind. It should be
  named as out of support rather than quietly counted.

### 3.2 What enabling `governed_scope` here would cost, measured 2026-09-21

The candidate scope is the governance files already in `[index]
extra_hashed_inputs`, which is the set with a claim to being governed. Ownership
read with `index owner`, one path at a time.

| Set | Files | Unclaimed |
|---|---|---|
| `.githooks/*` | 4 | 0 |
| `scripts/*` | 5 | 1: `scripts/bump_version.py` |
| `.github/workflows/*.yml` | 5 | 2 |
| `standards/spec/**.md` | 4 | 1: `standards/spec/templates/constitution-template.md` |
| `.claude/skills/*/SKILL.md` | 10 | 0 |
| `.claude/agents/*.md` | 4 | 0 |
| hashed `docs/*.md` | 5 | 0 |
| `crates/spec-spine-types/schemas/*.json` | 6 | 0 |
| root files | `AGENTS.md`, `CLAUDE.md`, `Makefile`, `install.sh` | 1: `CLAUDE.md` |

The bypass floor is evaluated **before** the ratchet, and `config show` confirms
`.github/` and `docs/` are on it. So the two unclaimed workflows never reach
`C-002` and are not a gap. The real cost of enabling the scope is exactly three
paths:

1. `CLAUDE.md`
2. `scripts/bump_version.py`
3. `standards/spec/templates/constitution-template.md`

Each is closable before the switch is thrown. `bump_version.py` takes a
`# Spec:` comment header, which claims the file with no frontmatter edit
anywhere (spec 075's window, `.py` claims with `#`). The other two are not in
`coverage.rs::SOURCE_EXTS` and need a frontmatter claim in some spec.

## 4. The Statecraft retirement acceptance checklist

The condition spec 092 §3.5 and spec 093 name for removing `.claude/` is that
Statecraft's global delivery is **concretely available**. This section is what
"concretely" has to mean, so the decision is made against evidence rather than
against an announcement.

Read from `statecraft-cli` spec `002-environment-lifecycle` §§3.13, 3.14, 3.22,
3.23, 3.24 and its dated decisions of 2026-09-20 and 2026-09-21, without
modification.

### 4.1 The counterparty's state, as its own corpus records it

- Spec 002 is `approved`, `implementation: **in-progress**`. It moved back from
  `complete` on 2026-09-21 because §3.24, the consented settings modification,
  is **specified and not implemented**: `delivery::NEVER_TOUCHED` still lists
  `settings.json`, `native_destination` links only `skills/` and `agents/`, and
  no consent flow exists.
- §3.14's mechanism exists: one content-addressed harness under the home,
  adapters that point at it, no copy in any repository, delivery **evaluated**
  (`reached` / `not-reached` / `unverified`) rather than assumed.
- §3.23 accepts the assertion-reimplementation cost and calls it not optional.
  For its **own** hook it is discharged: `crates/statecraft-home/tests/harness_hooks.rs`
  extracts the shipped body and runs it against each contract.
- §3.22's ordering is agreed on both sides, in the same words.
- Persistent agent memory is **out of it**: the owner's disposition of
  2026-09-21 is that `.claude/agent-memory/` belongs to `aicortex` and moves
  under this repository's own decision, not with this delivery.

So the blocker with a price tag is narrower than note 08 §2.2 states: the
pattern for reimplementing a harness assertion outside this tree is proven. What
has not happened is its application to the four hook events and ten skills that
live here.

### 4.2 Acceptance, item by item

None of these is satisfied today. Each is a check with an observable answer.

| # | Requirement | How it is evidenced |
|---|---|---|
| 1 | **Build and harness identity.** A named Statecraft build, and the content-addressed harness revision digest it delivers | `statecraft --version` plus the revision digest, both recorded in the retirement PR body |
| 2 | **Project gating and namespacing.** Delivered behavior applies only inside a repository holding `.statecraft/environment.json`, and every delivered name is Statecraft-namespaced | §3.14 rules 2 and 3, demonstrated by an unrelated repository in which nothing fires |
| 3 | **The managed instruction file exists before its bridge.** `.statecraft/AGENTS.md` is written here, and only then is `@.statecraft/AGENTS.md` inserted as the first line of the root `AGENTS.md` | The file on disk, then §3.13's idempotent insertion, with every other line preserved |
| 4 | **Actual delivery in a Claude Code session here.** Not a file matching a digest: §3.14's load rule evaluated to `reached`, with the chain named | A session in this repository whose `/prime`, `/next`, `/build`, `/verify`, `/ship`, `/shepherd`, `/spec`, `/commit`, `/code-review` and `/setup` all resolve |
| 5 | **The three skill assertions**, enforced by an executing test wherever the files land | No skill names a gate flag its project's `AGENTS.md` omits; a read-only skill never invokes a writing verb; each skill wraps the tool verbs rather than restating them |
| 6 | **The seven hook contracts**, enforced the same way | §3.23's list: read-never-repair; binary resolution order; target from the command; read the verdict; establish the verb first; no non-zero is green; resolve the protected branch |
| 7 | **All four hook events covered.** `SessionStart`, `PostToolUse`, `PreToolUse` (which carries the push gate and the PR gate together) and `Stop` | The counterparty's own 2026-09-21 correction: four, not three |
| 8 | **Push and PR gates, including the derived-tree states.** Staged and untracked shards both seen | spec 093 §3.13's matrix, reproduced against the delivered hook |
| 9 | **Project governance and user instruction preserved.** `AGENTS.md`'s `## Rules`, the project layer, and `.claude/settings.local.json` untouched | Diff of both files across the delivery |
| 10 | **Duplicate global hooks resolved.** `~/.claude/hooks/push-gate.sh` is already installed and registered; a delivered push gate must not double-fire | One gate runs per push, demonstrated |
| 11 | **Rollback.** The delivery is removable and this repository returns to a working loop | §3.13.4's removal rule, exercised |
| 12 | **Replacement demonstrated before deletion.** `harness_hooks.rs` and `harness_skills.rs` are not deleted until an executing replacement exists and is green | The replacement suite's own run, named in the retirement PR |

### 4.3 The corpus surgery a later authorized retirement performs

Prepared now so that change is mechanical. Confirmed 2026-09-21 with
`index owner`; nothing here is performed by this session.

**Unit claims to withdraw** (direct frontmatter edits under a named authority;
no edge withdraws a claim):

| Spec | Unit | Edge |
|---|---|---|
| `061-shipped-is-not-the-same-as-working` | `.claude/skills/` | extends |
| `062-one-name-one-freshness-verb` | `.claude/skills/`, `.claude/agents/`, `.claude/settings.json` | extends |
| `064-compile-warnings-reach-the-gate` | `.claude/skills/` | extends |
| `092-the-engine-ships-governance-not-an-environment` | `.claude/skills/`, `.claude/agents/`, `.claude/settings.json` | extends |
| `093-the-harness-this-repository-runs` | `.claude/skills/`, `.claude/agents/`, `.claude/settings.json` | establishes |

**Reference to withdraw**: `053-plan-answers-the-whole-question` names
`.claude/skills/next/SKILL.md` as a `references` unit. Non-owning, and still a
dangling claim once the file goes.

**Hashed inputs to remove** from `[index] extra_hashed_inputs`, all three in the
same change, because a glob matching nothing is `L-010` and `lint
--fail-on-warn` is in the gate:

```
".claude/settings.json"
".claude/skills/*/SKILL.md"
".claude/agents/*.md"
```

Removing them restales every shard. That is a regeneration, not a problem.

**Governing text that loses its subject**: spec 093's §3 and §5. It is a
sixteen-spec consolidation, so what goes is its harness half, under a named
amendment in the retiring spec; the document stays.

**Prose to correct**: `AGENTS.md` names `.claude/agents/` and `.claude/skills/`
as locations at lines 8, 234, 243 and 268.

**Tests**: `harness_hooks.rs` (1,124 lines) and `harness_skills.rs` (about 800)
are deleted **only** after item 12 above.

### 4.4 What is retired, and what is not

"Retire `.claude/`" and "delete `.claude/`" are different acts, and the note
that conflates them will delete something nobody decided to.

| Path | Disposition |
|---|---|
| `.claude/skills/`, `.claude/agents/`, `.claude/settings.json` | harness content: retired when §4.2 is satisfied |
| `.claude/settings.local.json` | the user's own file, claimed by no spec. Not harness content, not retired here |
| `.claude/agent-memory/` | `aicortex`'s subject by the owner's 2026-09-21 disposition. Claimed by no spec. Stays until that store can hold it, on a sibling's schedule |

So the directory survives the retirement of its harness content. Only a separate
decision empties it.

### 4.5 What spec-spine owes Statecraft, and what it is owed

| Direction | Item |
|---|---|
| **owed to Statecraft** | one line telling it that the bridge condition is understood and that this repository will insert `@.statecraft/AGENTS.md` once `.statecraft/AGENTS.md` exists (§3.22's "what remains is telling the counterparty") |
| **owed to Statecraft** | the ten skills and four agents as adoptable content, which are already repository-invariant and need no change to be adopted |
| **owed by Statecraft** | items 1 to 11 of §4.2 |
| **owed by Statecraft** | §3.24 implemented, since without it the four hook events have no route to a user's `settings.json` |
| **neither's** | `.claude/agent-memory/`, which is aicortex's |

## 5. A proposed clarification of SP-03

grand-refactor revision 4's **SP-03** recommends deferring obligations,
WorkScope, ContextClosure, A10 and B23 beyond the local slice, to be reopened
for a named consumer need. It is **not adopted**: revision 4's adoption record
holds only four statecrafting-profile rows, so §1.3's items are neither
scheduled nor withdrawn.

This note proposes a clarification, and records it **as proposed**:

> **Specify now, implement only for a named consumer need.** A deferred
> proposal is written down as a contract when the documentation already
> supports one, and built when a consumer names the need.

What it changes: a deferred item stops being an undocumented intention. What it
does not change: nothing is scheduled by writing it down, and a draft spec is
not a work order (spec 093's `/next` rule).

Spec 102 is this note's worked example. It is filed, its contract is complete,
and its own D-1 states that it is built when a consumer names the need. Spec
103 is the counter-example that keeps the rule honest: three consumers are
already named in note 04 §7, so it is buildable now.

**This is a proposal. It is not recorded as adopted here and it must not be
recorded as adopted in grand-refactor on the strength of this note.** Adoption
is the owner's act, in revision 4's adoption record.

## 6. Decisions a human still owes

| # | Question | Options | Recommendation |
|---|---|---|---|
| **D-1** | Is §5's clarification of SP-03 adopted? | (a) adopt as written; (b) adopt full SP-03 deferral, leaving §1.3 undocumented; (c) leave open | **(a)**. It costs one sentence and converts ten undocumented intentions into reviewable contracts without scheduling any of them. (b) keeps re-deriving the same proposals; (c) is where we already are |
| **D-2** | Enable `[coverage] governed_scope` here? | (a) close the three gaps in §3.2, then enable; (b) enable and accept three `C-002` refusals; (c) leave off | **(a)**, as its own spec, after spec 100. The cost is three paths, two of them one-line claims. (b) puts the gate in a red state on purpose; (c) leaves the governance files spec 078 was written for outside the ratchet in the repository that wrote it |
| **D-3** | What of the deleted `website/` does Statecraft recreate, and where? | (a) audit the deleted tree and route each page; (b) declare the two rescued facts sufficient and let the rest stay in git; (c) defer to Statecraft entirely | **(b) plus a named audit**: `docs/adoption-guide.md` and `docs/api.md` already carry what only existed on the site, and nothing has been reported missing since. Recreating a site is Statecraft's product decision, not a documentation debt here |
| **D-4** | Does the adopter migration note ship before the next release? | (a) yes, it gates the tag; (b) ship the release and follow with the note | **(a)**. Six repositories pin releases and three carry a kit copy. A release whose notes do not say `init` is gone and ids moved creates the support load the note prevents |
| **D-5** | Obligation vocabulary (note 04 D-5), if P1 is ever scheduled | kinds, id syntax, frontmatter versus a fenced block, whether `constraint` is a fourth kind | No recommendation. It should be decided by the consumer that needs obligations, not in advance |

## 7. The next single work order

**Build spec 100.** It is the only filed, unblocked correctness defect; it is
reproduced rather than predicted; its fix is bounded to two files and their
tests; and spec 100 §3.6 settles the semantics P6 would later build on, so
building it first is also the right order for the deferred backlog.

After it, in this order and each independently deliverable: spec 101 (one
paragraph of `docs/api.md`, plus one hashed-input glob), the adopter migration
note (D-4), spec 103, and D-2's governed-scope spec. Spec 102 waits for a
consumer.
