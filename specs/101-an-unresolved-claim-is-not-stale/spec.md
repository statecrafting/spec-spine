---
id: "101-an-unresolved-claim-is-not-stale"
title: "An unresolved claim exits as a validation failure"
status: draft
kind: "tooling"
created: "2026-09-16"
summary: >
  Spec 098 stopped `check` calling an unresolved claim "stale" and stopped it
  prescribing `spec-spine index`, a remedy that provably does not work. It
  deliberately left the exit code alone, because 086 3.1 pins "the exit code is
  unchanged: 2 when anything drifted" and adopters branch on it. The result is
  a verb whose prose says "this is not staleness" while its exit code says
  staleness, and exit 2 is the code an adopter's CLAUDE.md maps to "run
  spec-spine index". This spec moves an unresolved claim to exit 1, the
  validation code, by an `amends` edge on 086. A spec claiming a unit that does
  not resolve is a corpus that does not describe its tree, which is a validation
  failure and not a ledger that has fallen behind. No message changes: 098
  already made every report line correct.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "075-one-name-one-freshness-verb"
  - "086-the-committed-index-is-compared-not-trusted"
  - "098-a-blocking-claim-is-not-a-stale-shard"
amends:
  # 086 3.1's closing sentence, and 098 3.1's own MUST. Both are approved and
  # both state the rule normatively, so both are amended (spec 040). 098 3.1
  # names only 086 as needing the edge; that sentence undercounts itself.
  - "086-the-committed-index-is-compared-not-trusted"
  - "098-a-blocking-claim-is-not-a-stale-shard"
extends:
  # 3.1: the composed exit code.
  - { spec: "075-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: corrective }
  # 3.2: the same fold at the primitive.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  # 3.4: the acceptance, alongside spec 098's own cases. Attributed to 098
  # rather than to any other owner of this file: these tests sit beside 098's
  # blocking-claim cases and amend four of its assertions, so 098 is the
  # crossing a reader is actually making.
  - { spec: "098-a-blocking-claim-is-not-a-stale-shard", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 101: An unresolved claim exits as a validation failure

## 1. Purpose

### 1.1 The verb contradicts itself

Reproduced on 2026-09-16 with the `0.19.0` in-tree binary, in a corpus built
by `spec-spine init` holding one spec that claims `src/nothing.rs` while
declaring `implementation: complete`:

```
$ spec-spine check
spec-registry: fresh
codebase-index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1 spec(s), which is not staleness
  I-004 001-missing-territory: spec '001-missing-territory' file unit 'src/nothing.rs' does not exist
    001-missing-territory declares `implementation: complete` while the unit it claims is absent: the spec and the tree disagree about what exists
  regenerating the index does not clear this: each diagnostic is recomputed from the corpus on every run
$ echo $?
2
```

Every line of that report is correct: spec 098 fixed the prose. The last line
is the exit code, and it says the one thing the four lines above it deny. Exit
2 is staleness, and staleness is the refusal whose remedy is regeneration.

### 1.2 Why the code matters more than it looks

An exit code is the surface a consumer branches on when it does not read prose.
Design note 05 §9.3 records the consequence measured against an adopter:
a `CLAUDE.md` documenting exit 2 as staleness will print "run `spec-spine
index`" forever, because the verb keeps spending the staleness code on a
refusal regeneration cannot clear. The advice loop 098 removed from the verb's
own output survives in every consumer that reads only the code.

Exit 1 is already defined as validation failure, and spec 075 §3.3
already ranks it above 2 with the reasoning this spec extends: "staleness is
not meaningful against a corpus that does not validate". A spec claiming a unit
that does not resolve is precisely a corpus that does not describe its tree.
The code that names that is 1.

### 1.3 Why this needs an amendment rather than a fix

Spec 086 §3.1 states, of `index check`: "The existing
blocking-diagnostic refusal stays. The exit code is unchanged: 2 when anything
drifted." That sentence is approved and adopters branch on it. Spec 098
§3.1 held to it deliberately and filed the question forward; design note
05 §9.4 carries it as R-4, "does an `I-004` refusal move to exit 1?",
with "Human; needs an `amends` on 086" beside it.

The decision was taken on 2026-09-16: it moves. This spec records that by an
`amends` edge, which changes the rule without editing the approved file
(spec 040). Nothing in `specs/086-the-committed-index-is-compared-not-trusted/spec.md` is
touched.

## 2. Territory

No new file. Two exit-code folds and their acceptance:

| Unit | Edge | Why |
|---|---|---|
| `086` | `amends` | Section 3.1's closing sentence. |
| `crates/spec-spine-cli/src/cmd_check.rs` | `extends` 075, corrective | The composed fold. |
| `crates/spec-spine-cli/src/cmd_index.rs` | `extends` 004, corrective | The primitive's fold. |
| `crates/spec-spine-cli/tests/cli.rs` | `extends` 098, additive | The acceptance. |

No message, no report line and no `--json` member changes.

## 3. Behavior

### 3.1 `check` spends 1 on an unresolved claim

`spec-spine check` MUST exit `1` when the index half holds at least one
blocking claim, whatever else it holds. The fold in `cmd_check.rs` reads the
blocking set that `check_report_full` already returns beside the report, so the
verb decides from the partition spec 098 §3.2 built and not from the
composed `fresh` flag, which cannot tell the two refusals apart.

The order of spec 075 §3.3 is unchanged and already accommodates this:
`3` dominates `1` dominates `2` dominates `0`. A corpus with a blocking claim
**and** a stale shard therefore exits `1`, and the report still names both
halves and still attributes regeneration to the stale one alone.

This replaces the closing sentence of spec 086 §3.1. Under spec 040 the
replacement text lives here, and 086 is not edited. Where 086 reads:

> The existing blocking-diagnostic refusal stays. The exit code is unchanged: 2
> when anything drifted.

it now reads:

> The existing blocking-diagnostic refusal stays. Drift alone exits 2. A
> blocking diagnostic exits 1, the validation code, because a spec claiming a
> unit that does not resolve describes a corpus that does not match its tree,
> and regenerating cannot clear it. A tree holding both exits 1, under spec 075
> §3.3's order.

The rest of 086 §3.1 stands: the three drift classes, the `--slice` sidecar
comparison and the byte comparison itself are untouched, and this spec changes
nothing about **what** `index check` detects, only which code it spends.

Spec 098 §3.1 states the same rule in its own words and as its own MUST, so it
is amended too. Where 098 reads:

> `check` and `index check` MUST exit exactly as they do today for every input.
> A blocking resolution diagnostic MUST still produce exit `2`; a stale shard
> tree MUST still produce exit `2`; the two together MUST still produce exit
> `2`.

it now reads:

> `check` and `index check` MUST exit exactly as they do today for every input
> that carries no blocking resolution diagnostic. A stale shard tree MUST still
> produce exit `2`. A blocking resolution diagnostic exits `1`, and the two
> together exit `1`, which spec 101 decided and this spec deliberately did not.

The rest of 098 §3.1 stands, including the `--fail-on-unresolved` sentence and
spec 050 §3.3's precedence, both of which this spec leaves alone. 098's closing
paragraph, which forecasts the decision and names the `amends` edge it would
need, is left as the accurate record it is: it names 086 and not itself, which
is the one thing about it this spec has to correct by also amending 098.

### 3.2 `index check` spends 1 on the same fact

`spec-spine index check` MUST make the same move, for the same reason. The two
verbs answer the same question about the same tree and a caller must not have
to know which one it invoked to know what a code means.

### 3.3 Nothing else moves

- Every report line stays exactly as spec 098 left it. This spec changes no
  wording, because there is no wording left to correct.
- The `--json` envelope keeps its version, members and nesting. Its `exitCode`
  member carries the new code, which is the point; `report.index.fresh` and
  `report.index.actual` are unchanged.
- `--fail-on-unresolved` is a different axis and is untouched: it refuses on the
  warning-tier `W-001` / `W-002` diagnostics, which are a spec legitimately
  claiming territory it has not written yet (specs 025 and 044). Spec 050
  §3.3's rule that staleness outranks unresolution refers to that flag
  and still holds.
- A corpus with no blocking claim keeps exit 2 for a stale shard, wording
  included. A caller that reads staleness today reads it after.

### 3.4 The acceptance is the reproduction

`crates/spec-spine-cli/tests/cli.rs` MUST assert, against the corpus helper
spec 098's cases already build:

- `check` on a blocking-only corpus exits `1`, and its report still says
  `UNRESOLVED CLAIM` and still does not say `STALE` on the index line;
- `index check` on that corpus exits `1`;
- `check` on a corpus with a blocking claim **and** a stale shard exits `1`,
  with both halves still named;
- `check` on a stale-only corpus still exits `2`, wording included;
- `check --json` on a blocking corpus carries `exitCode` 1 and its members are
  otherwise unchanged.

The first three fail against pre-101 code, which exits 2 on all three. The
fourth and fifth are the regression half and pass at the parent commit by
construction; they are here because the value of this change is that it moves
one code and no other.

## 4. Out of scope

- **`report.index.actual`'s wording.** For a blocking-only corpus it reads
  `1 stale shard(s):\n  blocking-diagnostics by-spec/<id>.json`, which is the
  compatibility shim spec 098 FR-009 deliberately held still. It stays held.
  Spec 098 records why the JSON surface did not need the correction the prose
  did: a consumer can already separate the two refusals through
  `report.index.diagnostics.byCode`, which carries `I-004` directly. Moving the
  exit code does not weaken that discriminator.
- **The hooks.** Spec 099 made both session hooks read the report rather than
  the code, so neither depends on which code the verb spends. They inherit this
  change without an edit, which is what that design bought.
- **`W-001` / `W-002` and `--fail-on-unresolved`.** Section 3.3.
- **Every adopter's own `CLAUDE.md`.** An adopter documenting exit 2 as
  staleness now has a correct document for the first time, but the release note
  is the instrument, as it was in specs 089 and 099.

## 5. Resolved decisions

D-1 (2026-09-16, why 1 rather than a new code). Exit codes are a stable
contract with four rungs and adopters branch on them. A fifth rung would make
every existing consumer's `case` statement incomplete, which is a worse break
than moving a refusal between two rungs that both already mean "refused". Exit
1 already means validation failure and this is one.

D-2 (2026-09-16, why a blocking claim outranks a stale shard rather than the
reverse). Spec 075 §3.3 already ranks 1 above 2 and gives the reason:
staleness is not meaningful against a corpus that does not validate. A tree
whose spec claims a file that does not exist cannot be made correct by
regenerating, so reporting the half that regeneration fixes as the headline
would send the operator to the remedy that does not work, which is the defect
spec 098 removed from the prose.

D-4 (2026-09-16, why spec 098 is amended as well as spec 086). Added during
the build, from a review finding. Spec 098 §3.1 is titled "The exit codes do
not move" and states the rule as its own MUST rather than merely citing 086's,
so it is a second approved document carrying the behavior this spec changes,
and spec 040 requires the edge to name every such document. 098's own forecast
of this decision says it "would be a contract change needing an `amends` edge
on 086", naming one spec where two were needed: an amendment that landed on 086
alone would have left 098's MUST standing unamended against the code.

D-3 (2026-09-16, why both verbs move together). A caller that runs `check`
composes two trees; one that runs `index check` reads one. Neither difference
bears on what an unresolved claim is. Leaving the primitive at 2 would mean the
composed verb and the verb it composes disagree about the same corpus, which is
the class of defect spec 075 exists to remove.

## Verification

Each line is one command (spec 049 §3.2).

The first line is the regression half of §3.3, run against this repository's
own corpus, which holds no blocking claim: a clean tree still exits 0. It
cannot fail on the defect and is here to fail on an over-broad fix, which is
the failure mode a change to an exit-code fold has.

The second and third lines are the fail-first evidence. Their filter selects
tests that do not exist at the parent commit, where `cargo test` runs zero of
them and reports `ok`: a filter matching nothing passes, so the count is
asserted before the suite is trusted. Each selected test builds its own scratch
corpus, because the case this spec changes cannot be constructed in a
repository whose own corpus validates.

```verify:cli
target/release/spec-spine check
sh -c 'n=$(cargo test -p spec-spine-cli --test cli --locked spec101_ 2>&1 | grep -c "^test spec101_"); test "$n" -ge 5 || { echo "expected at least 5 spec101_ tests, ran $n"; exit 1; }'
cargo test -p spec-spine-cli --test cli --locked spec101_
```
