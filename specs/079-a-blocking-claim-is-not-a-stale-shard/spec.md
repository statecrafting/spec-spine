---
id: "079-a-blocking-claim-is-not-a-stale-shard"
title: "A blocking claim is not a stale shard"
status: approved
kind: "tooling"
created: "2026-09-15"
summary: >
  `check_index_freshness` folds two different refusals into one verdict: a
  committed shard whose bytes moved, and a spec whose claimed unit does not
  resolve. Both arrive as `Freshness::Stale`, so `check` prints
  "codebase-index: STALE (run `spec-spine index`)" and `index check` prints
  "index is STALE (run `spec-spine index` to refresh)" for a corpus whose
  shards are byte-exact. The named remedy provably does not work: reproduced at
  0.19.0, `index` exits 0, writes the shard, and `check` still exits 2, because
  re-indexing cannot create a file a spec claims. An adopter documenting exit 2
  as staleness follows that advice indefinitely. This spec keeps the refusal
  and the exit code exactly as they are and changes what the two freshness
  verbs say: a blocking resolution diagnostic is reported as its own class,
  names the owning spec and unit, and says regeneration will not clear it. The
  mixed case reports both and says which half regeneration addresses.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "004-codebase-index"
  - "044-index-diagnostics-reach-a-gate"
  - "062-one-name-one-freshness-verb"
  - "069-the-committed-index-is-compared-not-trusted"
extends:
  # 3.2, 3.5: the partition exists here already and is flattened away.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  # 3.3: `index check`'s verdict line.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  # 3.3: `check`'s index half.
  - { spec: "062-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: corrective }
  # §5: the four cases and the regeneration regression.
  - { spec: "069-the-committed-index-is-compared-not-trusted", unit: "crates/spec-spine-core/tests/index_body.rs", nature: additive }
  # §5: the same assertions at the verbs, and the `--json` parity check.
  - { spec: "034-machine-readable-verdicts", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  # 3.2, D-5: `check_report_full`, the one seam that carries the partition to
  # `check` without a second index run. Additive: `check_report` keeps its
  # signature and its payload.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
references:
  # Note 06 is deliberately not a typed unit here: it is on the documentation
  # branch (PR #214) and not yet on the default branch, and a `references` edge
  # to a file that does not exist raises W-002 on every gate run.
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 079: A blocking claim is not a stale shard

## 1. Purpose

### 1.1 One verdict, two refusals, one remedy

`index.rs::check_index_freshness` computes two independent facts and returns
them as one value. First it collects the spec shards carrying an error-tier
resolver diagnostic (spec 044's `BLOCKING_CODES`, `I-003` to `I-009`) into a
`BTreeSet`; then it compares the committed shard bytes against the recompute
(spec 069 §3.1). Both sets are flattened into one `Vec<String>` and handed to
`drift_verdict`, which renders `Freshness::Stale { expected, actual }` with
`actual` reading `N stale shard(s):` over the joined lines.

The blocking lines keep a class token, `blocking-diagnostics <file>`, so the
distinction survives into the text. Nothing above that line reads it. Both
verbs render the verdict by its variant alone, and `Stale` has exactly one
remedy attached to it.

### 1.2 The remedy does not work, and the tree says so

Reproduced on 2026-09-15 with a `0.19.0` binary, in a scratch corpus created by
`spec-spine init`, with one spec claiming `src/nothing.rs` while declaring
`implementation: complete`:

```
$ spec-spine check
spec-registry: fresh
codebase-index: STALE (run `spec-spine index`)
1 stale shard(s):
  blocking-diagnostics by-spec/001-missing-territory.json
$ echo $?
2
$ spec-spine index && spec-spine check
indexed 1 package(s), 2 mapping(s) -> .../.derived/codebase-index (1 error diagnostic(s), 0 warning(s))
codebase-index: STALE (run `spec-spine index`)
$ echo $?
2
```

`index` exits 0 and writes the shard. The committed tree is then byte-exact,
and `check` still refuses, because the diagnostic is recomputed from the corpus
on every run and `src/nothing.rs` still does not exist. The instruction the
operator was given is not merely unhelpful; following it to completion leaves
the same output.

`index check` says the same thing in its own words (`index is STALE (run
`spec-spine index` to refresh)`), and both feed the session harness: the kit's
`SessionStart` hook matches `codebase-index: STALE` and reports "STALE, run
spec-spine index" in the session banner.

This is the shape a downstream consumer already hit. The finding routed here
from aicortex on 2026-09-12 (grand-refactor AI-08, case N6) records an adopter
whose `CLAUDE.md` reads exit 2 as staleness "and nothing else" since 0.18.0, so
a caller branching on the code prints a remedy for a defect re-indexing cannot
fix. Design note 05 §9.3 carries the disposition this spec implements.

**Where the cited sections live.** Note 05 §9 and design note 06 are on the
documentation branch of PR #214 and are not on the default branch at the time
this draft is filed. Every citation of them below is a citation of that branch.
Nothing in this spec depends on that PR merging: the defect, the reproduction
and the acceptance criteria stand on the corpus at `3bc004b`.

### 1.3 What is wrong is the description, not the decision

The gate refuses, and it is right to refuse. A spec claiming a unit that does
not resolve is a real defect and `I-004` is the right code for it. Nothing in
this spec makes the corpus easier to pass.

The defect is that the refusal is described as something it is not, and the
description carries an action. Staleness means "the committed artifact is
behind the source, regenerate it"; an unresolved claim means "the source and
the tree disagree about what exists". Those need different work from different
people, and one of them is not addressed by running a command.

## 2. Territory

No new files. Six existing units, each already owned:

| Unit | Owner | This spec |
|---|---|---|
| `crates/spec-spine-core/src/index.rs` | 004 establishes | `extends`, corrective: carry the partition instead of flattening it |
| `crates/spec-spine-cli/src/cmd_index.rs` | 004 establishes | `extends`, corrective: `index check`'s verdict line |
| `crates/spec-spine-cli/src/cmd_check.rs` | 075 establishes | `extends`, corrective: `check`'s index half |
| `crates/spec-spine-core/tests/index_body.rs` | 086 establishes | `extends`, additive: the four cases, and the regeneration regression |
| `crates/spec-spine-cli/tests/cli.rs` | 037 establishes | `extends`, additive: the same at the verbs, plus `--json` parity |
| `crates/spec-spine-core/src/lib.rs` | 001 establishes | `extends`, additive: `check_report_full`, the seam that hands `check` the partition from the one index run it already makes (D-5) |

## 3. Behavior

### 3.1 The exit codes do not move

`check` and `index check` MUST exit exactly as they do today for every input.
A blocking resolution diagnostic MUST still produce exit `2`; a stale shard
tree MUST still produce exit `2`; the two together MUST still produce exit `2`.
`--fail-on-unresolved` keeps its own meaning over the warning tier (`W-001`,
`W-002`) and its exit `1`, and spec 044 §3.3's precedence (staleness outranks
unresolution for that flag) is untouched.

Spec 069 §3.1 states "the existing blocking-diagnostic refusal stays. The exit
code is unchanged: 2 when anything drifted", and this spec holds that sentence
true. **Whether an `I-004` refusal should instead exit `1` is a separate
decision** (design note 05 §9.4, R-4). It would be a contract change needing an
`amends` edge on 086, and it is deliberately not taken here: a message can be
corrected without renegotiating a code adopters branch on, and doing both at
once would make the compatibility question ride on a fix that has none.

### 3.2 The two refusals are distinguished at the source

`check_index_freshness` already computes the blocking set separately before
folding it into the drift vector. That partition MUST be carried to the
reporting layer as data rather than re-derived, and MUST NOT be recovered by
parsing rendered text or by indexing a second time.

The resulting report MUST let a caller answer three questions without
inspecting prose: are any committed shards stale, are any claims unresolved,
and which spec and unit does each unresolved claim belong to.

Per-shard reporting is unchanged: spec 028 §3.3 makes it contractual that each
drifted shard occupies its own line carrying its drift class, and spec 069 §3.2
extends that to the index. Every line this spec emits keeps that shape.

### 3.3 What the verbs say

**Blocking only** (no shard bytes moved). The verbs MUST NOT report staleness
and MUST NOT name regeneration. The report MUST:

- classify the refusal as an unresolved claim, distinct from staleness;
- state that regenerating the index does not clear it;
- name, for each blocking diagnostic, its code, the owning spec id, and the
  unit;
- point at `spec-spine index diagnostics` for the full list when the report is
  capped.

**Stale only.** Unchanged, wording included. A caller that reads staleness
today MUST read staleness after this spec.

**Mixed** (shards moved *and* a claim does not resolve). Both MUST be reported,
neither elided, and the report MUST say that regeneration addresses the stale
shards and not the unresolved claims. A reader who regenerates and re-runs
MUST find the second half still refusing, and MUST have been told so in
advance.

**Healthy.** Unchanged.

### 3.4 Naming the contradiction, without naming a way out of it

When the spec owning a blocking diagnostic declares `implementation: complete`,
the report MUST say that the spec records the work as done while the claimed
unit is absent. That is the fact the operator needs and the one the current
output hides.

The report MUST NOT suggest removing or narrowing the claim, and MUST NOT
suggest `planned: true`, as a way to make the refusal go away. Spec 063 §3.3
already refuses `planned: true` beneath `implementation: complete` as `L-011`
at the error tier, on the grounds that completion and planned territory
contradict each other in the spec's own frontmatter; a message proposing that
pairing would be proposing a lint violation.

When the owning spec does **not** declare `implementation: complete`, the
report MAY name `planned: true` (spec 063 §3.2) as the declared way to hold
territory that is not written yet. That is a legitimate in-flight state under
specs 023, 041 and 044, and no contradiction exists to report.

In neither case does the tool choose which side of the disagreement is wrong.
It states that the spec and the tree disagree, names both, and stops.

### 3.5 The harness contract this changes, and the one it does not

The kit's `SessionStart` hook matches `codebase-index: STALE` in `check`'s
output and reports "STALE, run spec-spine index". After this spec that match
still fires for the stale and mixed cases, which is correct. In the
blocking-only case it will not fire, and the hook's fallback reports
`unknown (check exit 2)`.

That is a deliberate, stated trade: "unknown" is uninformative, "STALE, run
spec-spine index" is false, and this spec does not edit either
`settings.json`, because both are hashed governance inputs whose edit restales
every shard in the corpus and belongs with the `Stop` hook work in 3.5.1
rather than riding on a message fix.

**The follow-up is named, not implied.** Both hooks are carried forward as
`statecrafting/spec-spine#216`, filed 2026-09-16, which records what remains on
each hook, why neither is done here, and that the misleading-remediation
problem is therefore not closed by this spec. It is named as an issue rather
than as a design-note row because the note that would hold the row (note 06, and
note 05 §9) is on PR #214's branch and not on the default branch, and this spec
is not to depend on that PR merging (1.2).

#### 3.5.1 The `Stop` hook is not fixed by this spec

`kit/settings.json`'s `Stop` hook runs `"$sc" check >/dev/null 2>&1` and, on
any nonzero, prints `[freshness] STALE: run \`spec-spine compile\` and
\`index\` ...`. It discards the output this spec corrects, so it will keep
printing the same wrong remedy at the end of every response, and it will keep
printing it for exit `1` (validation, or a refused unresolved unit) and exit
`3` (I/O, parse, schema, config) as well.

**This spec does not fix that, and no one may report the misleading-remediation
problem as closed on the strength of it.** The hook's verdict is design note 06
§3.9's open question H-6 (does it refuse, or advise?), tracked as
`statecrafting/spec-spine#216`, and its own future spec:
it edits a hashed governance file in two pinned copies plus the generated
`kit_embedded.rs`, it needs a policy decision this spec has no standing to
take, and its blast radius reaches every adopter who re-copies the kit.
Section 6 records this as an explicit carry-forward rather than as a
limitation buried in prose.

## 4. Functional requirements

- **FR-001.** `check` and `index check` MUST produce, for every input, the exit
  code they produce before this spec.
- **FR-002.** The blocking set and the stale set MUST reach the reporting layer
  as distinct data, without re-indexing and without parsing rendered text.
- **FR-003.** In the blocking-only case neither verb may print a staleness
  verdict, and neither may name `spec-spine index`, `spec-spine compile` or any
  other regeneration command as the remedy.
- **FR-004.** In the blocking-only case each reported diagnostic MUST carry its
  code, its owning spec id and its unit.
- **FR-005.** In the blocking-only case the report MUST state that regenerating
  does not clear the refusal.
- **FR-006.** In the mixed case both classes MUST be reported, and the report
  MUST attribute regeneration to the stale class only.
- **FR-007.** When the owning spec declares `implementation: complete`, the
  report MUST name the contradiction between that claim and the absent unit,
  and MUST NOT offer `planned: true` or the removal of the claim as a remedy.
- **FR-008.** The stale-only and healthy reports MUST be unchanged, wording
  included.
- **FR-009.** `check --json`'s envelope MUST keep `schemaVersion` `0.4.0` and
  its current member set, types and nesting. No field is added, removed or
  retyped by this spec.

## 5. Acceptance criteria

- **AC-1 (blocking only).** A corpus whose committed shards are byte-exact and
  whose spec claims a missing unit: `check` exits 2, does not print `STALE`
  on the index line, names `I-004`, the spec id and the unit, and says
  regeneration will not clear it. Same for `index check`.
- **AC-2 (regeneration regression).** From AC-1's state, running `index` exits
  0 and `check` still exits 2 with the same class of message. The message after
  regeneration is still accurate: it does not claim the tree is stale, and it
  does not claim anything was repaired. This is the case the current output
  gets wrong, so it is asserted directly rather than inferred from AC-1.
- **AC-3 (stale only).** A corpus with a drifted shard and no blocking
  diagnostic reports staleness exactly as it does today, names the shard with
  its class, and points at regeneration. Exit 2.
- **AC-4 (mixed).** A corpus with both reports both, attributes regeneration to
  the stale half only, and exits 2. After `index`, the blocking half still
  refuses and the stale half is gone.
- **AC-5 (healthy).** A corpus with neither is `fresh` on both halves, exit 0,
  output unchanged.
- **AC-6 (completion claim).** With `implementation: complete`, the report
  names the contradiction. With an owning spec that does **not** declare
  completion and still blocks (`status: approved`, `implementation: deferred`),
  the same refusal is reported without a contradiction it never made. The
  in-flight arm is asserted too: `status: draft` with `implementation:
  in-progress` produces no blocking diagnostic at all under specs 023/041/044,
  which is why it cannot be the negative (D-5).
- **AC-7 (no weakening offered).** No output path in the blocking case contains
  a suggestion to remove the claim or to set `planned: true` beneath
  `implementation: complete`.
- **AC-8 (`--json` parity).** `check --json` over a fixture is byte-identical
  before and after this spec, and its `schemaVersion` is `0.4.0`.
- **AC-9 (`Stop` hook untouched).** `kit/settings.json` and
  `.claude/settings.json` are **byte-identical to a fixed base**, commit
  `3bc004bf5f0fb30b9c2127c1d2269face10fb37b` (the default-branch commit this
  branch was cut from), and the `Stop` hook still contains its generic
  `[freshness] STALE` line. The fixed base is what makes this an assertion
  rather than a restatement: a `grep` for the line passes over an edited file
  that still contains it, so it could not tell an untouched hook from a
  rewritten one (D-6). Asserted so that the carry forward in 3.5.1 is mechanical
  rather than a promise.

## 6. Out of scope

**The exit code.** Whether an `I-004` refusal belongs under exit 1 rather than
2 is R-4, needs an `amends` on 069 §3.1, and is a separate decision. 3.1 holds
the current codes.

**The `Stop` hook, and the `SessionStart` hook.** 3.5 and 3.5.1, carried
forward as `statecrafting/spec-spine#216`. Both are hashed governance files in
two pinned copies plus the generated `kit_embedded.rs`; both need the policy
decision design note 06 records as H-6.
The misleading remediation is **not** fully retired by this spec, and 3.5.1
says so in the spec's own text so that a later reader cannot mistake this for
the whole fix.

**`Error::Stale`'s wording at the freshness guard.** `couple`, `index
coverage` and `index owner` refuse through `Error::Stale`, whose `Display` in
`crates/spec-spine-types/src/error.rs` reads "index is stale: expected
content-hash ...". It carries no false remedy, only a false label, and that
file is claimed by the tier-1 bootstrap spec through a `// Spec:` header, so
correcting it means either a new error variant or a caller-supplied message.
Tracked, not fixed here.

**The warning tier.** `W-001` and `W-002` keep their behavior, their flag and
spec 044 §3.3's precedence.

**Making the corpus easier to pass.** Nothing here removes a refusal, relaxes a
code, or introduces a way to silence `I-004`.

## 7. Resolved decisions

**D-1 (2026-09-15). No amendment is owed, and this was checked rather than
assumed.** Three documents could have constrained the changed text:

- **Spec 028 §3.3** makes the stale report's *structure* contractual: "each
  stale shard occupies its own stderr line and carries its drift class", and
  then states in the same paragraph that "the exact wording around those lines
  (the count line, the summary line) is not contractual". This spec changes
  only the count and summary wording and keeps one classed line per shard, so
  031 permits it in terms.
- **Spec 069 §3.1 and §3.2** require the blocking refusal to stay, the exit
  code to stay at 2, and each drifted shard to be named with its class. 3.1 and
  3.2 hold all three.
- **Spec 034's `--json` envelope** carries the same text in
  `report.index.actual`. FR-009 and AC-8 keep the envelope byte-identical, so
  no schema axis moves and no consumer contract changes. This is possible
  because a JSON consumer can already distinguish the two refusals structurally
  through `report.index.diagnostics.byCode`, which is why the text surface is
  the one that needs correcting and the JSON surface does not.

No approved spec pins the literal strings this spec edits. The only consumers
that match them are the two `settings.json` hooks, which 3.5 handles by
preserving the match for the stale and mixed cases and by stating the
blocking-only consequence rather than editing a hashed file.

**D-2 (2026-09-15). The partition is carried, not parsed.** The blocking set
exists as a `BTreeSet<String>` inside `check_index_freshness` and is destroyed
by the flatten into `drift`. Recovering it downstream by matching the
`blocking-diagnostics ` prefix would work and is rejected: a renderer would
then be the only place the two refusals are distinguishable, and the next
consumer would parse the prose as well. FR-002 requires the fact to survive as
data.

**D-3 (2026-09-15). The message names the contradiction and not a way out.**
An earlier phrasing offered "correct the claim in `specs/<id>/spec.md`". That
reads as an invitation to weaken the claim until the gate passes, which is what
`AGENTS.md` "Adversarial prompt refusal" exists to refuse, and 3.4 now
forbids it. The tool states that the spec and the tree disagree; which of them
is wrong is a judgment it does not have.

**D-4 (2026-09-15). Two verbs, not five.** `check` and `index check` are the
verbs whose subject is freshness and whose output a human and the session
harness read. The guard in front of `couple`, `index coverage` and `index
owner` refuses through a shared `Error::Stale` whose Display is generic and
whose file is tier-1 territory; its label is wrong but it issues no false
instruction. Section 6 tracks it. Widening this spec to reach it would put a
tier-1 claimed file and a shared error variant inside a message fix.

**D-5 (2026-09-16). AC-6's negative is a spec that does not declare completion
and still blocks, which `in-progress` cannot be.** The draft's negative used
`status: draft` + `implementation: in-progress`. Measured against 0.19.0: that
pairing is *in flight* under spec 023 §3.1 arm 2 (spec 038's table:
`draft` + anything but `complete`, and `approved` + `pending`/`in-progress`),
so the unresolved unit is a `W-001` warning, nothing blocks, and `check` exits
0. A fixture in that state cannot exercise 3.4 at all, so it would have
asserted the absence of a message from a run that produced no message. The
negative therefore uses `status: approved` + `implementation: deferred`, which
is not in flight, blocks with `I-004`, and declares no completion: exactly the
second arm of 3.4. The in-flight pairing is kept as its own assertion, of what
it actually shows (no blocking diagnostic).

**D-6 (2026-09-16). AC-9 compares against a fixed base, not against a pattern.**
The draft asserted the `Stop` hook by `grep`-ing each `settings.json` for its
`[freshness] STALE` line. That passes over a file this spec had rewritten as
long as the line survived somewhere, so it could not distinguish an untouched
hook from an edited one, which is the only thing AC-9 exists to say. It now
diffs both files against `3bc004bf5f0fb30b9c2127c1d2269face10fb37b`, the
default-branch commit this branch was cut from. A fixed commit rather than
`origin/main`: the base must not move under a later merge, or the assertion
weakens silently as the branch ages. The `grep` is kept beside it, so a reader
sees both that nothing moved and what did not move.

**D-7 (2026-09-16). The partition reaches `check` through a new seam in
`lib.rs`, not through a widened `CheckReport`.** `check`'s payload is built by
`check_report`, and `CheckReport` **is** the `--json` envelope, which FR-009
freezes. Widening it would move the JSON surface this spec says it does not
move; calling a second core function from the CLI would index twice, which
FR-002 forbids. `check_report_full` returns the same report plus the partition
from the same run, and `check_report` becomes a wrapper over it, so no caller
and no payload changes. The cost is one additive `extends` edge on spec 001's
`lib.rs`, which §2 carries.

## Verification

> **Superseded acceptance (2026-09-21).** This block no longer runs.
> `092-the-engine-ships-governance-not-an-environment` declares this spec in
> `amends_verification`, so `spec-spine verify 079` builds its plan from that
> spec's block and names the substitution in `acceptanceFrom` (spec 082 3.2
> and 3.4).
>
> The commands below are kept **verbatim** and are not corrected, even where a
> path in one no longer exists. A predecessor is amended, never edited (spec
> 037 3.1): this block is the record of what was asserted when this spec was
> ratified, and the amending spec is where the assertion lives now.

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture corpus under `${TMPDIR:-/tmp}/ss098` carries the
state instead.

**Fail-first evidence.** Run against this tree the block fails at command 10,
the first AC-1 assertion: today's `check` names no diagnostic code at all, it
prints `codebase-index: STALE (run \`spec-spine index\`)` for a corpus whose
shards are byte-exact, and it names the remedy that AC-2 proves does not work.
Every setup line before it exits 0, so the failure is the behavior and not the
fixture. The `cargo test` lines are **not** fail-first: the cases in §5 do not
exist at the parent commit, so those suites pass vacuously until the build adds
them.

```verify:cli
cargo build --release --locked
cargo test -p spec-spine-core --test index_body --locked
cargo test -p spec-spine-cli --test cli --locked
# Fixture: committed shards byte-exact, one spec claiming a unit that is absent.
rm -rf "${TMPDIR:-/tmp}/ss098" && mkdir -p "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" init >/dev/null
sed -i.bak 's/REPLACE-WITH-DATE/2026-09-15/' "${TMPDIR:-/tmp}/ss098"/specs/000-bootstrap/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/000-bootstrap/spec.md.bak
printf '%s\n' '---' 'id: "001-missing-territory"' 'title: "A claim with no code"' 'status: draft' 'implementation: complete' 'created: "2026-09-15"' 'summary: >' '  Claims a file that does not exist while declaring itself complete.' 'establishes:' '  - "src/nothing.rs"' '---' '' '# 001: A claim with no code' > "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
# AC-1 / FR-001: the refusal and the exit code are unchanged.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 2
# AC-1 / FR-004: the code, the owning spec and the unit are named.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q '001-missing-territory'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'src/nothing.rs'
# AC-1 / FR-003: the index half reports no staleness and names no regeneration.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'spec-spine index'
# AC-1 / FR-005: it says regenerating will not clear this.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'regenerat'
# AC-6 / FR-007: the contradicted completion claim is named.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'complete'
# AC-7 / FR-007: no way out is offered. `planned: true` under `complete` is
# L-011 (spec 063 §3.3), and narrowing the claim is not the tool's call.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'planned: true'
# AC-1: the registry half is untouched.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-registry: fresh'
# AC-1 at the primitive verb.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check >/dev/null 2>&1; test $? -eq 2
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index check 2>&1 | grep -q 'to refresh'
# AC-8 / FR-009: the --json envelope is unchanged in version, members and
# nesting, and already separates the two refusals structurally.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check --json > "${TMPDIR:-/tmp}/ss098"/check.json 2>/dev/null; test -s "${TMPDIR:-/tmp}/ss098"/check.json
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss098/check.json'));assert d['schemaVersion']=='0.4.0';assert sorted(d)==['exitCode','ok','report','schemaVersion','verb'];assert sorted(d['report'])==['index','registry'];assert sorted(d['report']['index'])==['actual','diagnostics','expected','fresh','unwitnessed'];assert d['report']['index']['diagnostics']['byCode']=={'I-004':1};assert d['exitCode']==2 and d['ok'] is False"
# AC-2: the regression. Regenerating exits 0, repairs nothing, and the verb
# still refuses with an accurate message.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 2
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
# AC-4: mixed. Move a committed shard's bytes while the blocking claim stands.
python3 -c "import json,glob;p=sorted(glob.glob('${TMPDIR:-/tmp}/ss098/.derived/codebase-index/by-spec/*.json'))[0];d=json.load(open(p));d['shardHash']='0'*64;json.dump(d,open(p,'w'),indent=2,sort_keys=True);open(p,'a').write('\n')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 2
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'STALE'
# AC-4 / FR-006: regeneration is attributed to the stale half only.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qiE 'only the stale|stale shard\(s\) only|not the unresolved'
# AC-4: after regenerating, the stale half is gone and the blocking half stands.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep 'codebase-index:' | grep -q 'STALE'
# AC-6 in-flight arm (D-5): `draft` + `in-progress` is in flight under specs
# 023/038/041, so the unit is a `W-001` warning, nothing blocks, and `check`
# exits 0. Asserted for what it shows, which is why it is not the negative.
sed -i.bak 's/implementation: complete/implementation: in-progress/' "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md.bak
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 0
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
# AC-6 negative: a spec that blocks without declaring completion is refused for
# the same code, and is not accused of contradicting a claim it never made.
sed -i.bak -e 's/status: draft/status: approved/' -e 's/implementation: in-progress/implementation: deferred/' "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md && rm -f "${TMPDIR:-/tmp}/ss098"/specs/001-missing-territory/spec.md.bak
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -qi 'complete'
# AC-3: stale only. Satisfy the claim, regenerate, then move a shard's bytes.
mkdir -p "${TMPDIR:-/tmp}/ss098"/src && printf '%s\n' '// placeholder' > "${TMPDIR:-/tmp}/ss098"/src/nothing.rs
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
python3 -c "import json,glob;p=sorted(glob.glob('${TMPDIR:-/tmp}/ss098/.derived/codebase-index/by-spec/*.json'))[0];d=json.load(open(p));d['shardHash']='0'*64;json.dump(d,open(p,'w'),indent=2,sort_keys=True);open(p,'a').write('\n')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check >/dev/null 2>&1; test $? -eq 2
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'codebase-index: STALE'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-spine index'
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'I-004'
# AC-5: healthy. Regenerate; both halves fresh at exit 0.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'spec-registry: fresh'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss098" check 2>&1 | grep -q 'codebase-index: fresh'
# AC-9 (D-6): neither settings.json is edited by this spec. Compared against a
# fixed base, the default-branch commit this branch was cut from, so the
# assertion cannot weaken as the branch ages; the pattern check is kept beside
# it so a reader sees what it is that did not move.
git diff --quiet 3bc004bf5f0fb30b9c2127c1d2269face10fb37b -- kit/settings.json .claude/settings.json
grep -q 'freshness. STALE' kit/settings.json
grep -q 'freshness. STALE' .claude/settings.json
rm -rf "${TMPDIR:-/tmp}/ss098"
```
