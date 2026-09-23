# 09: The documentation backlog, dispositioned (2026-09-21)

A design note, not a spec. It reconciles every proposal this repository's
documentation carries against the corpus as it stands on 2026-09-21, and it is
**the current backlog record**. **Its implementation condition for deferred
work (section 5, D-1, section 7's last paragraph) is superseded by D-7, section
10, dated 2026-09-22.** Notes 05 and 06 are historical, note 07 §4 is
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

## 5. SP-03, clarified and adopted

> **[superseded 2026-09-22 by D-7, section 10]** The implementation half of the
> rule below ("implement only for a named consumer need") no longer governs.
> The specification half stands. The section is preserved as the record of
> what was decided on 2026-09-21.

grand-refactor revision 4's **SP-03** recommends deferring obligations,
WorkScope, ContextClosure, A10 and B23 beyond the local slice, to be reopened
for a named consumer need. This note proposed one narrowing of it. The owner
adopted that narrowing on 2026-09-21, in these words:

> Adopt note 09 D-1 as "specify now; implement only for a named consumer need."
> This authorizes concrete deferred draft contracts, not implementation of
> every proposal.

The adopted rule, stated once:

> **Specify now, implement only for a named consumer need.** A deferred
> proposal is written down as a contract when the documentation already
> supports one, and built when a consumer names the need.

What it changes: a deferred item stops being an undocumented intention. What it
does not change: nothing is scheduled by writing it down, a draft spec is not a
work order (spec 093's `/next` rule), and SP-03's ownership assignments from
revision 3's 06 are untouched (spec-spine for source contracts, statecraft-cli
for adapter and submission consumers, Statecraft for runtime binding).

Spec 102 is the worked example. It is filed, its contract is complete, and its
own D-1 states that it is built when a consumer names the need. Spec 103 is the
counter-example that keeps the rule honest: three consumers are already named
in note 04 section 7, so it is buildable now.

**Where the adoption is recorded.** In grand-refactor's revision-4 adoption
record, dated 2026-09-21, scoped to SP-03 alone. No other row of that package
is adopted by it, no historical proposal is rewritten, and its own deferral of
*implementation* stands.

## 6. Owner decisions, recorded 2026-09-21

Section 1's tables were written while these were open. All five are now
answered, and a sixth rule was set in the same instruction.

### D-1: SP-03's clarification is adopted

> **[superseded 2026-09-22 by D-7]** Drafting stays authorized. "Schedulable
> never, until a consumer names it" is replaced by D-7's evaluation.

**Ruling: (a), adopt as written.** Section 5 carries the rule and the record.

Consequence for section 1.3: every deferred row there becomes draftable now and
schedulable never, until a consumer names it. Drafting them is authorized;
implementing them is not.

### D-2: `[coverage] governed_scope` is enabled here, after the gaps are closed

**Ruling: (a).** Close the three measured gaps of section 3.2 through
legitimate ownership, then enable the reviewed scope in its own spec, filed
after spec 100.

Two constraints attach to the ruling:

- **Bypass semantics are not weakened as part of it.** The floor, the
  configured additions, their match rules and their precedence over the scope
  all stay exactly as spec 005 section 3.5, spec 078 section 3.3 and spec 092
  section 3.8 have them. A path the floor exempts stays exempt after the scope
  is on. If closing a gap appears to require loosening a bypass, that is the
  signal to stop and re-scope, not to loosen it.
- **The three-path list is re-measured before the switch is thrown**, not
  assumed still exhaustive. It was measured against `4ab1b31e`; the tree has
  moved since, and this session adds files to `docs/`.

**What stays exempt, and is therefore not covered by the ratchet even with the
scope on.** Stating it plainly, because "governed scope enabled" reads as
"everything is covered" and it is not:

| Still exempt | Why |
|---|---|
| everything under the built-in bypass floor, including `.github/`, `docs/`, `README.md` and the configured `derived_dir` | the floor is evaluated **before** the ratchet, so a floor path never reaches `C-002` however the scope is written |
| everything under `[coupling] bypass_prefixes` | same evaluation order; additive to the floor |
| everything under `[layout] state_dir` | declared ungoverned (spec 036); excluded from classification by construction |
| anything matched by `[index] resolver_exclusions` | pruned before the universe is built |
| anything listed in `[coverage] governed_scope_exclusions` | carved back out after the scope is applied |
| a **deleted** path | `C-002` exempts deletions (spec 029), scope or no scope |
| every path outside a discovered package **and** outside the declared scope | the scope is an allowlist, not a wildcard: a file in neither set is invisible to the ratchet exactly as before |

In particular, the two unclaimed `.github/workflows/*.yml` files measured in
section 3.2 remain exempt after the scope is on. They are not a gap being
closed; they are a floor path that the scope cannot reach, and the spec must
say so rather than imply the workflows became governed.

### D-3: one bounded audit of the deleted site, with retention here

**Ruling: (a) as performed, not (b).** The audit is in section 8. "No omission
has been reported" is explicitly **not** its acceptance criterion: the deleted
tree was extracted from git and compared claim by claim against the current
documentation surface, and the criterion is that every distinct claim is
accounted for by a named disposition.

Routing, as ruled:

- **Engine semantics, API and standalone governance guidance stay here.**
  Section 8 records what that turned out to be and where it landed.
- **Statecraft onboarding and harness instruction become a handoff for
  Statecraft.** Not recreated here, not deleted from history, and named in
  section 8.3 so the counterparty can take it up.
- **No website is recreated.** Recreating a site is a product decision and is
  not a documentation debt of this repository.

### D-4: the migration note and a producer acceptance check gate the next release

**Ruling: (a).** `docs/adopter-migration.md`, finished against the measured
adopter inventory, and a conforming-producer acceptance check both land before
the next tag.

Two additions the ruling makes explicit:

- **The release candidate is prepared promptly** and does not wait for full
  harness delivery, for aicortex, for spec 102, or for any deferred vocabulary.
- **The candidate carries a genuinely new version.** `0.21.0` must not be the
  identity of two different published behaviors.

### D-5: the drafting baseline for the deferred contracts

**Ruling: adopted as the baseline every section 1.3 draft is written against.**

1. Declarations live in **frontmatter**. No second fenced declaration syntax.
2. Three constraint kinds: **requirement**, **invariant**, **verification**.
3. **Stable unique ids within a spec**, and **qualified** cross-spec references.
4. References point at **validated heading anchors**.
5. **Section digests** are carried alongside full spec identity, not instead
   of it.
6. Verification inputs are **explicitly declared**, never inferred.
7. **No fourth constraint kind** in the initial contract. `constraint` as a
   kind of its own is not adopted.

And the limit on what an obligation asserts, which is part of the baseline and
not a caveat on it:

> An obligation identifies declared requirements and evidence relationships. It
> does **not** establish that a verification result is true, and it does not
> establish that the declarations enumerate every dependency.

Section 1.3 row P2 said ContextClosure needs D-5 settled first. It is settled,
so that blocker is discharged.

### D-6: evidence reuse, and the independence of CI

Set in the same instruction and recorded here because section 1.4's H8 was the
open question it answers.

> Evidence is reusable only when the **subject**, the **tool identity**, the
> **command**, the **configuration** and the **relevant inputs** all match. CI
> verification remains independent.

Consequences: a passing result may be reused across a session only while every
one of those five is unchanged; a rebuilt binary, an edited `spec-spine.toml`,
a different flag or a moved tree each invalidate it. CI never reuses a local
result, and a local result is never reported as a CI result.

H8's disposition therefore moves from **decision** to **decided**: the
governance-side rule is the paragraph above; the orchestration-side scheduling
of when to re-run remains Statecraft's.

## 7. The sequence, as authorized

Spec 100 is still the first work order, for the reasons section 1.1 E1 gives.
What follows it is now an authorized sequence rather than a recommendation,
each item separately reviewable and each spec its own pull request:

1. **Spec 100**, corrected before it is built (section 9 records the
   corrections the owner required).
2. **Spec 101**, independently.
3. **The producer release candidate**: the validated producer mismatch, the
   smallest conforming change, a genuinely new version, `docs/adopter-migration.md`
   finished, and a reviewable publication plan. Prepared, not published.
4. **Spec 103**, tightened before it is built.
5. **The governed-scope spec** (D-2), filed at the next free ordinal.
6. **The section 1.3 drafts**, under D-5's baseline.

> **[superseded 2026-09-22 by D-7]** The paragraph below no longer governs.
> Spec 102 is authorized for implementation under section 10's evaluation; the
> absence of a consumer request is no longer a reason to leave it unbuilt. The
> paragraph is preserved as the record of what was decided on 2026-09-21.

Spec 102 stays deferred until a consumer requires it. It is small, and that is
not a reason to build it.

> **[superseded 2026-09-22 by D-7]** Spec 102 is authorized for implementation
> under section 10's evaluation; the absence of a consumer request is no longer
> a reason to leave it unbuilt.

## 8. The deleted site, audited (D-3)

Bounded and performed 2026-09-21. The tree was extracted from
`373ab506^`, the last commit that contained it: 53 files, of which 34 are
documentation pages. Every page's distinct claims were extracted as literal
tokens and tested against the current documentation surface
(`docs/*.md`, `README.md`, `AGENTS.md`, `CLAUDE.md`, `standards/spec/**`).
The measurement is reproducible from that commit; it is not a reading
impression.

### 8.1 The result, by page group

| Group | Pages | Disposition |
|---|---|---|
| `concepts/*` | 8 | **covered here.** Six carry no claim absent from the current tree. Two carried unit-literal syntax and the manifest-key JSON shape, retained in `docs/configuration.md` and already present in `standards/spec/contract.md` |
| `getting-started/*`, `adoption-guide.md` | 3 | **covered here** by `docs/adoption-guide.md`, which is three times the length of the site's version |
| `extending-and-overlays.md` | 1 | **covered here** by `docs/overlay-contract.md`. Zero absent claims |
| `releasing.md` | 1 | **covered here** by `docs/releasing.md`. Zero absent claims |
| `schema-and-versioning.md` | 1 | **covered here** by `docs/schema-versioning.md`, minus one install-script environment variable now in `docs/adoption-guide.md` |
| `api-reference.md` | 1 | **covered here** by `docs/api.md`. Its two absent tokens are the facade signature and the purity statement, both in `CLAUDE.md` and `docs/design/00-architecture.md` |
| `cli/*` minus `init.md` | 7 | **retained**: this was the real gap. See 8.2 |
| `configuration.md` | 1 | **retained**: this was the other real gap. See 8.2 |
| `faq.md` | 1 | **retained in part**: its two absent items are a waiver-reason convention and a layout key, both now in `docs/cli-reference.md` and `docs/configuration.md` |
| `cli/init.md` | 1 | **obsolete.** `spec-spine init` was removed by spec 092. Not retained; `docs/cli-reference.md` says so under its own heading, so a reader arriving from an old link learns why rather than finding nothing |
| `claude-code/*` | 10 | **handoff to Statecraft.** See 8.3 |

### 8.2 What was retained, and where

Two documents were written, from current source rather than copied forward:

- **`docs/cli-reference.md`**: every verb, its flags, the exit-code contract,
  and the `--json` verdict envelope with its `verb` and `error.kind` closed
  sets. The deleted `cli/` pages were the only per-verb flag reference that
  ever existed here, and seven verbs and subcommands have appeared since they
  were written (`check`, `delta`, `compact`, `config show`, `index owner`,
  `index diagnostics`, `registry plan --next`).
- **`docs/configuration.md`**: every `spec-spine.toml` key with its default and
  what enabling it changes. `[lint]`, `[coverage]`, `[meta]` and
  `layout.state_dir` did not exist when the deleted page was written.

Both are engine documentation and stay here by the ruling. Both were checked
against the binary's own `--help` and against
`crates/spec-spine-types/src/config.rs`, not against the deleted pages, so a
default that changed since deletion is current rather than preserved wrong.

### 8.3 What is handed off, and is not recreated here

The ten `claude-code/` pages documented the distributed kit: installing it,
copying `.claude/`, the `settings.json` hook table, the skills and agents
reference, the session-init protocol, the governed loop and its checkpoints,
and a troubleshooting page. Their subject is the **managed development
environment**, which is Statecraft's by spec 092 and note 07 section 4.

Two things follow, and they are different:

- **Nothing here is recreated.** This repository's own harness instruction
  lives in `AGENTS.md` and `.claude/`, which is its development instruction and
  not a distribution source.
- **The content is available to Statecraft**, in git at `373ab506^` under
  `website/docs/claude-code/`, as source material for its own onboarding
  documentation. Roughly three of the ten pages (`install.md`,
  `adopt-in-your-repo.md`, `overview.md`) describe a kit-copy mechanism
  Statecraft has replaced with content-addressed delivery and should not be
  followed; the other seven describe the loop, the hooks, the skills and the
  failure modes, which survive the mechanism change.

This is a pointer, not a deliverable. spec-spine owes Statecraft nothing
further on it.

### 8.4 What the audit did not find

No engine semantics, API surface or governance rule was found that exists only
on the deleted site and nowhere else after this retention. That is the
audit's *result*, reached by the token comparison above. It is not its
acceptance criterion, and it was not assumed at the start.

## 9. Corrections this note makes to itself

Recorded here rather than by silent edit, because section 2 holds note 08 to
the same standard.

1. **"A human-named draft cannot be built"** was never stated outright and was
   implied by section 1.1's "Build 100. It is the next work order" sitting
   beside four `draft` specs. Stated plainly: in this repository **draft
   readiness is intentional**, and a draft a human has named can be built
   without prior ratification. The lifecycle is draft, build, then separately
   ratify. `registry plan` reporting a draft on `ready` is that cadence
   working, which is exactly what spec 101 writes down.
2. **The matrix counts.** Section 1's five tables hold **51 rows**, not the
   unstated number a reader would infer from "seven dispositions". Recomputed
   as of the decisions above, and using a consistent vocabulary: two labels in
   use (`blocked`, `not a defect`) were outside the declared seven and are
   folded in as `deferred` with a stated blocker and `superseded` respectively,
   with the row text unchanged.

| Disposition | Rows, at filing | Rows, after section 6 |
|---|---|---|
| implemented | 15 | 15 |
| filed | 4 | 4 |
| deferred | 13 (plus 2 labelled `blocked`) | 13, plus 2 blocked on Statecraft, plus the 10 of 1.3 now **draftable under D-5** |
| transferred | 4 | 4 |
| superseded | 7 (plus 2 labelled `not a defect`) | 9 |
| decision | 3 | **0**: H8 answered by D-6, A2 by D-3, A3 by D-2 |
| draftable | 1 | 0: A1 is being written under D-4 |
| **total** | **51** | **51** |

3. **Dated measurements are not current facts.** Sections 3.1 and 3.2 are
   measurements taken on 2026-09-21 against `4ab1b31e`. They are labelled as
   such and must be re-measured before anything is built on them; D-2's ruling
   requires exactly that for section 3.2. A number in this note is evidence of
   what was true at a commit, never a claim about the working tree a later
   reader has.

## 10. Owner decision D-7 (2026-09-22): opportunity-led expansion

### 10.1 The ruling

Recorded as given, in substance:

> The earlier "specify now; implement only for a named consumer need"
> restriction is replaced. An existing consumer request is useful evidence, but
> it is no longer a prerequisite for filing or implementing a spec-spine
> capability. Opportunities are evaluated by the capability they enable; their
> potential value to the Statecraft CLI, the Statecraft platform and other
> adopters; their differentiation and strategic usefulness; their ability to
> support multiple future features; and their architectural fit, dependencies,
> maintenance cost and compatibility. Latent value is a legitimate reason to
> build infrastructure. A plausible opportunity is distinguished from
> demonstrated customer demand, but is not rejected because nobody has
> requested it yet. This includes spec 102.

What it supersedes: section 5's implementation condition, D-1's "schedulable
never, until a consumer names it", section 7's last paragraph, and spec 102's
own D-1 (recorded there as superseded, by its D-2). What it does not change:

- **The producer boundary.** Runtime activation, remote fetching, mutable
  orchestration state, provider execution and operational trust decisions stay
  with their owners (spec 092; note 07). An opportunity that needs one of them
  inside this repository is re-scoped, not built.
- **Transferred items stay transferred** (section 1's C9, H1, H9, A7).
- **D-5's drafting baseline**, which every contract in section 1.3 is written
  against.
- **One spec per implementation pull request, ratification separate.** Filing
  and building remain distinct acts, and a draft is still not ratified by being
  built.
- **The lifecycle value `deferred`** keeps its meaning (spec 035): a decision
  not to schedule. What changed is the reason a spec may be deferred, not the
  state.

### 10.2 The deferred contracts, reconciled

Sections 1.2 and 1.3 were written before the deferred contracts were filed.
Their state on 2026-09-22, before anything below is built:

| Row | Spec | Where it is | State |
|---|---|---|---|
| C2 | 102 | `main` | filed, `draft` / `pending`; contract complete |
| C3 | 103 | PR #301 | built, `draft` / `complete`, outside the 0.22.0 candidate |
| A3 | 105, `governed_scope` enabled | local branch `105-governed-scope-is-enabled-here` (`a48a1691`), not pushed | built against the pre-integration stack; needs re-basing onto `main` and its own pull request |
| P1 | 106, obligations | local branch `specs/deferred-contracts-2026-09-21` (`ed69e00a`), not pushed | filed as a deferred contract; no territory |
| P2 | 107, ContextClosure | same branch | filed as a deferred contract; its open question 1 (where a closure lives) unanswered |
| P3 | 108, WorkScope | same branch | filed, deferred |
| P4 | 109, impact and conflict | same branch | filed, deferred; needs 106 |
| P5 | 110, digest-pinned interface references | same branch | filed, deferred; needs 106's section digests |
| P6 | 111, reviewed move mapping | same branch | filed, deferred |
| P7 | 112, typed overlays | same branch | filed, deferred |
| P8 | 113, waiver lifecycle | same branch | filed, deferred |
| P9 | 114, A10/B23 separation | same branch | filed, deferred; A10 is Statecraft's |
| P10 | 115, bindings mandate | same branch | filed, **deferred by mandate**: the one row whose deferral is the decision, unaffected by D-7 |
| | 116, `L-001` exempts a deferred contract | same branch | built; needed only if the deferred contracts land on `main` while still deferred |

Ordinals 105 and 108 to 116 stay **reserved** for those drafts (spec 118 made
gaps legitimate). Nothing below renumbers or duplicates them: 106 and 107 are
carried from that branch and corrected, not re-filed.

### 10.3 The bounded wave selected under D-7

Three specs, in dependency order, each its own pull request, none part of the
0.22.0 candidate.

**Spec 102, `status` on a ready spec.**

- *Possible:* a consumer reading `registry plan --json` applies an approval
  rule without one `registry show` per entry.
- *Distinctive:* small, but it is the first read-document field added for a
  scheduler rather than for a human, and it fixes the shape before a second
  consumer copies the join.
- *Scenario:* the Statecraft CLI's scheduling stage filters the ready set to
  `approved` entries from one read, in one process, and records the plan
  document it filtered.
- *Hypothetical:* no Statecraft stage consumes it today.
- *Smallest contract:* spec 102 as filed. One member, verbatim, read-schema
  MINOR.
- *Evidence:* ready-set membership and order unchanged (asserted by name); the
  emitted plan conforms at the new version.

**Spec 106, obligations.**

- *Possible:* a requirement, invariant or verification becomes an addressable
  thing (`<spec>#<id>`) with a text, a validated anchor and a digest of the
  section that states it in full. Commit messages, acceptance lines, review
  comments and orchestrators can cite a requirement rather than a paragraph
  number that moves.
- *Distinctive:* this is the substrate every later contract in section 1.3
  builds on (107, 109, 110 and 113 all reference obligation ids). No other
  tool in this family gives a requirement a stable, hash-checkable identity
  tied to the prose it came from.
- *Scenario:* a Statecraft work order records
  `100-a-deleted-path-is-judged-where-it-lived#R-1` and the section digest it
  was issued against; when the work is reviewed, the platform asks whether that
  digest moved and flags the order for re-reading if it did.
- *Hypothetical:* no corpus declares obligations yet, and no consumer reads
  them. The first declarations will be this repository's own.
- *Smallest contract:* an optional frontmatter key, three kinds, per-spec ids,
  validated anchors, per-anchor section digests in the registry shard (additive
  MINOR), explicit verification inputs, withdrawn ids kept as tombstones so
  reuse is detectable without history, and no gate consulting any of it.
- *Evidence:* compile-time validation of every rule; section digests move only
  with their section; a corpus without the key compiles to unchanged
  `shardHash` values; `couple` verdicts unchanged; the conformance test at the
  new registry version.

**Spec 107, ContextClosure, as a pure resolver.**

- *Possible:* a set of specs, sections and obligations that a piece of work is
  answerable to is resolved against the corpus and content-addressed, so "the
  context this was done against has changed" is one digest comparison.
- *Distinctive:* it turns 106's identities into the object an orchestrator
  actually handles, while leaving the orchestration state where it belongs.
- *Scenario:* the Statecraft CLI stores a closure request in a work order,
  calls the resolver when the order is issued and again at review, and compares
  the two digests.
- *Hypothetical:* the request shape is designed here, not taken from a
  consumer. 107's open question 1 (where a closure lives) is answered by the
  boundary: it lives in the consumer's record, and spec-spine resolves it.
- *Smallest contract:* a request document in, a resolved closure with an
  order-independent digest out, through the JSON facade and one CLI verb;
  every reference qualified and validated; inert in every gate.
- *Evidence:* digest stable under reordering, moving with any member's
  content; every unresolvable reference refused by name; gate verdicts
  unchanged.

### 10.4 The dependency sequence after this wave

Not authorized for implementation by this note; recorded so the next wave
starts from an order rather than a list.

1. **105**, `governed_scope` enabled here: already built, needs re-basing and
   its own pull request. Independent of the rest.
2. **109**, impact and conflict sets, declared against 106's ids.
3. **110**, digest-pinned interface references, on 106's section digests. Its
   fetching and trust stay outside.
4. **113**, waiver lifecycle over caller-supplied inputs, citing obligations
   where a waiver names what it waives.
5. **108**, WorkScope, completing spec 072's overlap half.
6. **111** and **112** as consumers of the above appear. **114** is a
   separation record, and **115** stays deferred by mandate.


## 11. The reserved ordinals and the second increment (2026-09-23)

Section 10.2 reserved 105 and 108 to 116 for the deferred contracts on
`specs/deferred-contracts-2026-09-21`, and section 10.4 ordered what came
after the first wave. Their state now, read from each spec's frontmatter on
`main`, not from ordinals:

| Ordinal | Contract | State on `main` | How |
|---|---|---|---|
| 105 | `governed_scope` enabled here | `draft` / `complete` | #311, repaired by #315 (carries 078's acceptance through `amends` + `amends_verification`) |
| 106 | obligations | `draft` / `complete` | #308, carried from the branch and corrected, same id |
| 107 | ContextClosure | `draft` / `complete` | #309, same |
| 108 | WorkScope | `draft` / `complete` | #320. The reserved draft was corrected before the build (its D-1): a scope is a consumer-held document, evaluated against the committed index, compared purely, and never a gate, lock or permission |
| 109 | impact and conflict | `draft` / `complete` | #312 |
| 110 | digest-pinned interface references | `draft` / `complete` | #313 |
| 111 | reviewed move mapping | reserved, not on `main` | unchanged: filed on the branch, deferred |
| 112 | typed overlays | reserved, not on `main` | unchanged |
| 113 | waiver lifecycle | `draft` / `complete` | #318. Corrected before the build (its D-1): the draft required both reporting an unscoped waiver and a byte-identical verdict, which cannot both hold |
| 114 | A10/B23 separation | reserved, not on `main` | unchanged; A10 is Statecraft's |
| 115 | bindings mandate | reserved, not on `main` | deferred by mandate, unaffected by D-7 |
| 116 | `L-001` exempts a deferred contract | reserved, not on `main` | still unneeded: 108 and 113 landed built, not deferred |

No ordinal was renumbered or re-used. Ordinals 117 to 125 are new filings:

- **117 to 121** belong to the 0.22.0 correction work (the release record's
  §§9 to 13) and are `approved`.
- **122** (#310, a commit refuses an unresolved merge or unformatted Rust)
  is corrective. It `extends` spec 094's `.githooks/` and replaced nothing in
  the authorized list; it exists because the first wave's integration
  committed conflict markers and an unformatted change (0.22.0 record §14.8).
- **123** (#317, #319) is corrective: the session hooks name their reader and
  do not report an older in-tree build's verdict as the tree's. It `amends`
  093 for one wording rule.
- **124** (#322) gives the expansion line its own version (0.23.0) and moves
  this repository's floor with it.
- **125** (#321) is corrective: `verify`'s forwarder delivers whole lines, the
  defect that turned the post-merge sweep at `0bf9ff78` red twice.

Section 10.4's order is complete through its fifth item. What remains is its
sixth: **111** and **112** when a consumer of the above appears (D-7 permits
building them sooner on their own merits), **114** as a separation record, and
**115**, deferred by mandate.
