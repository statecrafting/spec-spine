---
id: "095-the-corpus-describes-what-exists"
title: "The corpus describes what exists"
status: approved
kind: "governance"
created: "2026-09-20"
summary: >
  Spec 092 removed the kit and obeyed every standing rule while doing it: it
  withdrew claims, declared amendments, replaced acceptance, and edited no
  predecessor's prose. The result is a corpus where 102 unit claims were
  withdrawn from 29 specs, 29 specs no longer run their own acceptance because
  one block stands for all of them, two blocks pass only because the files they
  assert about are gone, and 41 documents describe artifacts that no longer
  exist while reading as live instruction. That is the cost of the rules, paid
  once, and the owner authorized a one-time exception to settle it. This spec is
  the record of that exception: what was deleted, what was merged into what, how
  the ordinals were made contiguous again, and what the rules go back to being
  the moment it is done.
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "037-amendment-authoring"
  - "092-the-engine-ships-governance-not-an-environment"
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
establishes:
  # 3.7: a citation outlives the document it cites, and after a collapse and a
  # renumber the ordinal alone misleads. This is the file that answers both.
  - "docs/corpus-map.md"
extends:
  # 3.5: the exemption ledger and the ordinal it closes at.
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: corrective }
# 3.5: the ledger's closing ordinal is a number spec 089's acceptance pins, and
# this change moves it. The correction is that number and nothing else.
amends:
  - "089-nothing-reruns-a-merged-acceptance"
references:
  - { unit: { kind: file, path: "docs/design/07-statecraft-realignment-2026-09.md" }, role: context }
---

# 095: The corpus describes what exists

## 1. Purpose

### 1.1 What obeying the rules cost

Spec 092 is a large removal performed correctly. Measured on 2026-09-20 on the
branch that implemented it:

| | |
|---|---|
| unit claims withdrawn | 102, across 29 specs |
| specs no longer running their own acceptance | 29 (one block stands for all) |
| blocks passing only because their subject is gone | 2 |
| documents naming a removed path in prose | 41 |
| specs left owning nothing | 1 |
| deletions the coupling gate still refuses | 7 |

None of that is a defect in the code. It is the corpus describing a product that
no longer exists, in documents that read as instruction, because the rules that
kept spec 092 honest (spec 037: an amendment is declared once, in the amending
spec, and the amended file is not edited) also prevent anyone from saying so in
the file a reader opens.

### 1.2 The exception, and its shape

The owner authorized a one-time exception to those rules, for this change only:
delete the specs whose whole subject is gone, collapse the scattered families
into one document each, and renumber the survivors so the ordinals are
contiguous again.

The exception is bounded in four ways, and each is asserted in §5:

1. **It is one change.** After it merges, spec 037 governs again with no
   residue: the next removal declares an amendment and edits nobody.
2. **Nothing is weakened.** Every `MUST` of every collapsed spec is carried into
   its successor or named in that successor's "what is not carried" table with
   the file whose deletion removed its subject.
3. **Nothing is hidden.** Every removed spec is named in a successor's
   `supersedes`, so the graph answers "where did 071 go".
4. **History is not rewritten.** The documents stay in git; this is a change to
   what the corpus holds, not to what it held.

### 1.3 The six this removes outright

- `006-init-scaffold`
- `029-claude-code-skill-kit`
- `065-init-and-the-kit-are-one-adoption`
- `100-one-source-generates-the-agent-trees`
- `113-the-scaffolded-protocol-is-the-gate-the-kit-ships`
- `115-the-kit-ships-no-claim-an-adopter-cannot-resolve`

They are not named in a `supersedes` edge: the edge takes a spec id, the corpus
refuses an id it cannot resolve (`L-004`), and after the collapse these
documents are in git rather than in `specs/`. `docs/corpus-map.md` is the map.

## 2. Territory

This spec establishes `docs/corpus-map.md` (§3.7). Beyond that it is a record
and an authority: the deletion set, the merge map and the renumber map, plus the
rules the exception suspends and the moment they resume.

The territory it disposes of is the 27 spec directories §3 names, and the
ordinals of the 93 that survive.

## 3. Behavior

### 3.1 The deletion set: six specs with no successor requirement

These six describe a surface spec 092 removed entirely. Each is superseded by
this spec, and no requirement of any of them survives, because each requirement
was about a file that is gone.

| Spec | Subject |
|---|---|
| 006-init-scaffold | the `spec-spine init` command |
| 029-claude-code-skill-kit | the `kit/` bundle |
| 065-init-and-the-kit-are-one-adoption | `init --with-kit` |
| 100-one-source-generates-the-agent-trees | generating `.agents/` and `.codex/` |
| 113-the-scaffolded-protocol-is-the-gate-the-kit-ships | the scaffolded `AGENTS.md` |
| 115-the-kit-ships-no-claim-an-adopter-cannot-resolve | `# Spec:` headers in kit scripts |

Two properties of these six **do** survive and MUST be held elsewhere, because
their subject is the library producer rather than the command:

- what the scaffold emits and what it must not (006, 065): spec 092 §3.2 and
  §3.3, already;
- that no file the scaffold produces carries a claim header resolving in no
  corpus (115): asserted in `tests/gate.rs`, already.

### 3.2 The merge map: twenty-one specs into two

| Into | From |
|---|---|
| 093-the-harness-this-repository-runs | 046, 047, 048, 051, 063, 068, 071, 072, 078, 080, 081, 082, 099, 104, 110, 116 |
| 094-one-gate-and-the-boundaries-it-holds | 020, 064, 089, 090, 114 |

Each successor MUST name every predecessor in `supersedes`, MUST carry every
surviving requirement, and MUST carry a table of what it does not carry with the
reason. Both do.

### 3.3 The renumber: ordinals become contiguous

After §3.1 and §3.2 the corpus has 92 surviving documents and three new ones.
The survivors MUST be renumbered to `000` through `091` **in their existing
order**, and the three new specs take `092`, `093`, `094`. Chronology is
preserved: a spec filed earlier keeps a lower ordinal than one filed later.

Every reference to a renumbered spec MUST move with it, in all of:

- frontmatter (`depends_on`, `amends`, `amends_verification`, `supersedes`,
  `superseded_by`, and the `spec:` of every `extends` / `refines` / `constrains`
  / `co_authority` item);
- `## Verification` commands, which name spec ids and spec paths;
- prose, in both the `NNN-slug` and the "spec NNN" forms;
- `// Spec:` claim headers in source files;
- `[package.metadata.spec-spine].spec` in every manifest;
- `scripts/verify-sweep.sh`'s exemption ledger and its closing ordinal;
- documentation, the harness, and the design notes.

A rewrite MUST NOT touch a three-digit token that is not a spec reference. The
corpus is full of them: `V-016`, `C-001`, `L-008`, `W-001`, `0.18.0`, `2026`,
dates, line numbers, and citations of **another** project's specs ("OAP spec
177"). The rewrite MUST therefore be driven by the map, matching only a full
`NNN-slug` id whose slug is a real spec's, or a `spec NNN` / `specs NNN` form
whose ordinal is in the map and which is not preceded by another project's name.

### 3.4 The map outlives the documents

`docs/corpus-map.md` MUST carry, for every removed spec, its id, the title it
carried and the spec that holds its requirements now; and for every surviving
spec, its old ordinal and its new one.

It exists because a citation outlives the document it cites. Git history, merged
pull requests, release notes and four adopter repositories all name spec ids
this change removes or moves, and none of them is rewritten. After the renumber
a bare old ordinal is worse than a dangling one: it resolves, to the wrong
document. The map is how any of those citations is read.

The removed specs MUST NOT be named in a `supersedes` edge. The edge takes a
spec id, `lint` reports an unresolvable one as `L-004`, and `lint --fail-on-warn`
is in this repository's gate; relaxing that lint to accommodate a one-time
collapse would trade a standing check for a convenience. Each successor names
its predecessors in prose (§1.3 of each), and the map is the machine-readable
half.

### 3.5 The ledger's boundary moves with the ordinals

`scripts/verify-sweep.sh` carries the closed exemption ledger of the 48 specs
that predate executable acceptance, "closed at ordinal 48". Five of those 48 are
deleted or merged away, so the ledger MUST be rewritten to the 43 survivors,
under their new ids, with the closing ordinal set to the new ordinal of the
first spec that was never exempt. The four properties spec 089 §3.4 requires of
the ledger (individually named, closed, only shrinks, live) are unchanged and
MUST still hold.

Two of spec 089's own assertions read the boundary and are corrected to the new
one, which is §3.6's "only a path spelling changed" case applied to a number:
the reported `ledgerClosedAt`, and the refusal message naming the closing
ordinal. Its dangling-exemption fixture moves below the new boundary too, or the
ledger refuses it as *closed* and the message under test never appears.

**One assertion of spec 089 cannot pass until this merges, and is left
failing.** It runs the sweep against `origin/main` with the built-in ledger, to
prove the shipped ledger works on a trusted, already-merged revision. The
ledger this change writes describes a corpus that exists only on this branch,
so against `origin/main` every entry is dangling and the sweep refuses, exit 3,
correctly. On the default branch after this merges, `origin/main` is this
corpus and the assertion holds again. Rewriting it to read `HEAD` would trade
away the one thing it tests, which is that the ledger is true of a merged
revision rather than of the branch that wrote it.

### 3.6 What the exception does not permit

- **Weakening a requirement.** A clause is carried or its subject is gone. There
  is no third option, and §5 asserts the count.
- **Deleting a document without a successor.** Every removed spec is named in
  some `supersedes`.
- **Editing a surviving spec's requirements.** The renumber rewrites
  **references**; it MUST NOT change what any surviving spec requires. The only
  other edit permitted to a survivor is dropping a frontmatter edge whose target
  no longer exists, repointed to the successor where one exists.
- **Rewriting history.** No git history is altered.

### 3.7 The rules resume

The moment this change merges, spec 037 governs unchanged: an amendment is
declared once, in the amending spec, and the amended file is not edited. A
future removal takes the path spec 092 took and accepts the cost §1.1 measures,
or files its own exception and argues for it. This spec is not a precedent for
"the corpus may be rewritten when it gets untidy"; it is the record of one
authorized settlement of one architectural break.

## 4. Out of scope

- **Repairing the blocks spec 092 left passing vacuously.** Those belong to the
  specs that own them, and the sweep spec 089 built is what schedules the audit.
  The two the collapse removes (074's and 078's kit assertions) are gone with
  their subject; the rest are unchanged.
- **The seven deletions the coupling gate refuses.** Spec 092 §3.11.1 measures
  them and D-11 records why the fix is its own spec.
- **Renumbering anything but specs.** Diagnostic codes, schema versions and
  release versions are untouched.
- **Any change to the engine.** This change edits documents, their references,
  and the two generated trees that follow from them.

## 5. Resolved decisions

D-1 (2026-09-20, collapse rather than retire-in-place). Marking the 21 merged
specs `superseded` and leaving them on disk was the smaller change and it keeps
the reading problem: a reader looking for what the push gate refuses still finds
three documents, two of which open with a page about `kit/settings.json`. The
owner asked for the corpus to describe what exists.

D-2 (2026-09-20, contiguous ordinals). Gaps are not a defect on their own; spec
095's and 117's reserved gaps were deliberate. What made contiguity worth the
rewrite is that after removing 27 of 120 the gaps stop being reservations and
start being scar tissue, and every one of them is a question a future reader
asks and cannot answer from the corpus. The cost is a mechanical rewrite of
roughly six thousand references, which §3.3 bounds and §5's acceptance checks.

D-3 (2026-09-20, chronological order is preserved). Renumbering by subject
(grouping the coupling specs, then the index specs) was considered. It reads
better in a listing and destroys the one thing an ordinal reliably tells you
here: what was known when. Every `depends_on` in this corpus points backward,
and `[lint] require_ordinal_monotonic_depends_on` exists to check it.

D-4 (2026-09-20, the three new specs go at the end). 092, 093 and 094 are filed
now, so they sort after everything filed before them, exactly as an ordinary new
spec would. Interleaving them with the specs they replace would put a document
written today at an ordinal that says 2026-06.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured on 2026-09-20 at the parent of this branch:
`registry show 094` is red (not found, exit 1); `ls specs/ | wc -l` is 120; the
27 directories §3.1 and §3.2 name all exist; `registry list --ids-only` shows
gaps at 117 and, after spec 092's own build, nowhere else.

> **Superseded acceptance (2026-09-22).** This block no longer runs.
> `118-the-renumber-is-history-not-a-standing-rule` declares this spec in
> `amends_verification`, so `spec-spine verify 095` builds its plan from that
> spec's block and names the substitution in `acceptanceFrom` (spec 082 3.2
> and 3.4). The repository owner ruled that 3.3's renumber is a historical
> transformation, not a standing contiguity rule
> (`docs/release-candidate-0.22.0.md` 10.1); this note is the one edit to this
> file that ruling and spec 082 3.4 require.
>
> The commands below are kept **verbatim** and are not corrected: the amending
> spec's own acceptance asserts that the superseded predicate is still here,
> which is how the corpus proves this spec was amended rather than edited
> (spec 037 3.1).

```verify:cli
cargo build --release --locked
# 3.1, 3.2: none of the twenty-seven remains, by directory and by registry.
! test -e specs/006-init-scaffold
! test -e specs/029-claude-code-skill-kit
! test -e specs/046-kit-hooks-read-never-write
! test -e specs/064-the-kit-ships-the-composite-gate
! test -e specs/100-one-source-generates-the-agent-trees
! test -e specs/116-shepherd-reads-every-reviewer
# 3.3: the ordinals are contiguous, with no gap and no duplicate.
target/release/spec-spine registry list --ids-only > "${TMPDIR:-/tmp}/ss095-ids.txt"
python3 -c "ids=[l.strip() for l in open('${TMPDIR:-/tmp}/ss095-ids.txt') if l.strip()]; o=[int(i[:3]) for i in ids]; assert o==sorted(o); assert len(set(o))==len(o); assert o==list(range(len(o))), o[:5]; print(len(o),'specs, contiguous')"
rm -f "${TMPDIR:-/tmp}/ss095-ids.txt"
# 3.3: every reference resolves. A dangling id is a compile error (V-004 family),
# and an unresolved unit is what `check` refuses, so a green pair is the proof.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
# 3.3: and no removed id is cited outside the specs that account for it and the
# map that resolves it. The four are 092, which removed the surface, and the
# three filed here. Read per id, so a failure names the one at fault.
# 096 joined that set the day it was filed: its plan example quotes two of these
# ids, which is a document recording the removal and not a live citation. The
# range is spelled as an explicit character class rather than `09[0-9]` so a
# later spec citing a removed id is still caught.
sh -c 'for id in 006-init-scaffold 029-claude-code-skill-kit 046-kit-hooks-read-never-write 064-the-kit-ships-the-composite-gate 100-one-source-generates-the-agent-trees 116-shepherd-reads-every-reviewer; do if grep -rlF "$id" specs crates .claude/skills .claude/agents AGENTS.md CLAUDE.md README.md 2>/dev/null | grep -qv "^specs/09[23456]-"; then echo "still cited: $id"; exit 1; fi; done; exit 0'
# 3.2 and 3.4: every removed spec is named by a successor and is in the map.
test "$(grep -cE '^\| `[0-9]{3}-[a-z0-9-]+` \|' docs/corpus-map.md)" -ge 27
sh -c 'n=0; for f in specs/*/spec.md; do n=$((n + $(grep -cE "^- \`[0-9]{3}-[a-z0-9-]+\`$" "$f"))); done; test "$n" -eq 27 || { echo "predecessors named: $n, want 27"; exit 1; }'
# 3.4: the sweep's ledger is live, closed, and every entry names a real spec.
grep -qE '^readonly LEDGER_CLOSED_AT=[0-9]+$' scripts/verify-sweep.sh
bash -n scripts/verify-sweep.sh
# 3.5: nothing in the tree still names the removed product.
! test -e kit
! test -e .agents
! test -e .codex
# And the whole loop, over the rewritten corpus.
target/release/spec-spine index coverage --fail-on-untraced
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
cargo test --workspace --locked
```
