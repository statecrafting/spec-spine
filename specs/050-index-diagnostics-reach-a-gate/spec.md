---
id: "050-index-diagnostics-reach-a-gate"
title: "Index diagnostics reach a gate"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "025-unresolved-unit-severity"
  - "037-machine-readable-verdicts"
  - "044-in-progress-is-in-flight"
establishes:
  - "crates/spec-spine-core/src/diagnostics.rs"
  - "crates/spec-spine-core/tests/diagnostics.rs"
extends:
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "specs/025-unresolved-unit-severity/spec.md" }, role: context }
summary: >
  Spec 025 downgrades an unresolved unit to a counted `W-001` (owning unit on an
  in-flight spec) or `W-002` (non-owning `references` unit) instead of failing
  the index, because work under way legitimately claims territory that does not
  exist yet. That downgrade is right and this spec does not touch it. What 025
  never gave the warning is a reader. The codes are deliberately absent from
  `BLOCKING_CODES`, `lint` counts a different set entirely (`L-001`..`L-006`),
  and `index check` prints one word, `fresh`. The only way to see them is to run
  the *writing* command, which mutates the tree, or to read `index render`'s
  markdown, which is the ad-hoc parsing constitution II forbids. So a corpus
  accumulates them without limit while every gate stays green: aicortex carries
  248 that nothing reports. The asymmetry is exact, because error-tier
  diagnostics are gated: `I-003`..`I-009` mark their shard stale and `index
  check` exits 2. This spec gives the warning tier the same reachability without
  giving it the same force. `index check` reports the counts it already holds,
  `--fail-on-unresolved` turns them into exit 1 for a corpus past the
  specify-first stage, and `index diagnostics` lists them as typed output. All
  three read the committed shards and recompute nothing, so the answer always
  describes the ledger the corpus compiled to.
---

# 050: Index diagnostics reach a gate

## 1. Purpose

The indexer records what it could not resolve. Nothing reads it.

`cmd_index.rs` states the principle already, in a comment above the loop that
prints both tiers after a write:

> a warning the operator never sees is a unit that quietly went unresolved

That is true, and the command it guards is the one that mutates the tree. The
command an operator runs to *check* the tree prints `index is fresh` and stops.

### 1.1 The exact shape of the gap

Four surfaces could report an index warning, and none does.

| surface | what it does with `W-001` / `W-002` |
|---|---|
| `spec-spine index` (writing) | prints both tiers to stderr, then returns 0 |
| `spec-spine index check` | ignores them; prints `index is fresh` |
| `spec-spine lint --fail-on-warn` | counts `L-001`..`L-006`; never sees them |
| `spec-spine index render` | shows them, inside a markdown projection |

Only the writing command reports, and reporting is not its job: an operator who
runs it to see the warnings has already changed the tree to find out. Only
`render` shows them without writing, and reading them back out of its markdown
is exactly the ad-hoc parsing constitution II forbids.

### 1.2 The asymmetry is with the error tier, not with severity

Error-tier resolver diagnostics are gated. `BLOCKING_CODES` (`I-003`..`I-009`)
makes a shard carrying one report as stale, so `index check` exits 2 and CI
stops. Spec 025 excluded `W-001` and `W-002` from that list on purpose, and
correctly: an in-flight spec's unresolved claim must not fail the build.

But 025 chose between *blocking* and *nothing*, and nothing is what it got. The
tier exists to say "not yet", and a corpus cannot currently ask how many "not
yet"s it is carrying without writing to disk.

### 1.3 What 248 invisible warnings cost

aicortex carries 248 `W-001`s. Every gate in that repository is green, and the
number is not reported by any of them. Two things follow, and the second is the
worse one:

- The corpus cannot tell whether the number is going up or down. A burndown
  needs a count that a script can read, and the only count available is inside
  a markdown table.
- A spec that goes `complete` while still claiming an unresolved unit turns
  those warnings into hard errors all at once (spec 041), so the debt is
  invisible right up to the moment it blocks a merge.

## 2. Territory

One new core module, `diagnostics.rs`, which loads the committed shards and
answers questions about their recorded diagnostics, plus its fixtures. Additive
edits to `cmd_index.rs` (the `check` arm, one new subcommand), `index.rs`
(making the committed-shard reader crate-visible), the facade, and the
end-to-end tests. `main.rs` is untouched: `IndexAction` lives in `cmd_index.rs`,
so both the new subcommand and the new flag land there.

No DTO changes: `Diagnostic` and `Diagnostics` already exist and are already
committed inside every shard. Nothing about what the indexer *records* changes,
and no shard changes shape. This spec is entirely about reading what is already
written down.

## 3. Behavior

### 3.1 `index check` reports the counts it already holds

`index check` MUST report the diagnostics recorded in the committed shards,
whether the index is fresh or stale.

Prose form appends the counts to the existing verdict line:

```
index is fresh (248 warning(s), 0 error(s): 248 W-001)
```

The per-code breakdown omits zero entries here exactly as it does in the JSON,
so a corpus with no `W-002` does not print `0 W-002`. An earlier draft of this
example printed the zero and contradicted the rule stated two paragraphs below
it; the example was the defect, not the rule.

A corpus with none MUST keep printing the bare `index is fresh`, so that a
clean tree reads exactly as it does today.

Under `--json` the report gains a `diagnostics` member: `{ "warnings": n,
"errors": n, "byCode": { "W-001": n, ... } }`, with `byCode` omitting zero
entries so the shape does not grow with codes a corpus does not have.

**`compile --check`'s payload MUST NOT change.** Index diagnostics have no
meaning for the registry, so its verdict must not acquire a member that is
permanently zero. It keeps rendering the bare freshness object through the CLI's
`freshness_report` helper.

`index check` therefore does not share that helper: it composes an
`IndexCheckReport` (the freshness members plus `diagnostics`) in **core**, and
both the CLI's `--json` arm and the `check_freshness_json` facade serialize that
one type. The composition has to live in core because spec 037 pins the two
against each other, and a payload built twice is a payload that drifts. An
earlier draft of this section had `index check` extending `freshness_report`'s
output in the CLI; 037's parity test refused it, correctly.

### 3.2 `--fail-on-unresolved` is opt-in

`index check --fail-on-unresolved` MUST exit 1 when the committed index records
any `W-001` or `W-002`, and 0 otherwise.

Opt-in, because specs 025 and 044 exist precisely to let a corpus that ratifies
before it builds stay green while the work is openly under way. Making this the
default would refuse the workflow those two specs were written to permit. A
corpus past that stage sets the flag; one in the middle of it does not.

When the flag refuses, the prose verdict line MUST say so. `is fresh` stays true
on its own axis, and a line reading only that, while the process exits 1,
invites a reader to take it for a pass. Prose is not the machine surface: spec
037 exists so a consumer branches on the exit code or the `--json` envelope,
both unambiguous here. This is one clause, not a second contract.

The flag name says `unresolved` rather than `warn` because that is what the two
codes mean: a declared unit resolved to nothing. It deliberately does not read
`--fail-on-warn`, which is `lint`'s flag over a different code set, and the two
must not look interchangeable.

### 3.3 Staleness outranks unresolution

When the committed index is stale **and** carries unresolved warnings, `index
check --fail-on-unresolved` MUST exit 2, not 1.

A stale index's diagnostics describe a tree that no longer exists, so reporting
them as the reason for refusal would name the wrong problem. Refresh first, then
ask. The counts are still reported in both forms, marked as belonging to the
stale ledger, because suppressing them would hide the only number the operator
came for.

### 3.4 `index diagnostics` lists them

`spec-spine index diagnostics [--json]` MUST list the diagnostics recorded in
the committed shards: code, severity tier, path, message, and owning spec id,
sorted by spec id then code then path.

It is a read verb beside `index coverage` and `index orphans`, and it exists so
that a consumer never has to parse `index render`'s markdown to reach a
structured fact. It writes nothing and always exits 0 when it can read the
shards; the refusal lives on `check`, not here.

### 3.5 Everything reads the committed shards

None of 3.1 to 3.4 MAY re-run the indexer. All three read the committed shard
set, which already carries every diagnostic verbatim.

Reading them costs one pass over the shard set that `check_index_freshness`
has already made and does not hand back. Folding the two into a single read
means refactoring the staleness gate itself, which spec 004 owns and which every
other gate depends on; that is not worth a constant factor on a set of small
per-spec JSON files, in a verb that already hashes every input. **Decision,
2026-09-06:** the second read stays, and the counting path folds directly over
the shards rather than over a listing it would allocate and drop.

It applies to `check_freshness_json` as well as to the CLI, so a binding calling
the facade in a loop pays it too. That is the right place for the cost while it
exists: a caller with a hot path has the typed API, where `check_index_freshness`
and `committed_counts` are separate calls it can order as it likes, while the
facade's contract is one verdict per call. `check_registry_freshness_json`
already documents the same trade-off against `compile_json`.

This keeps `index check` the cheap gate it is, and it keeps the answer honest:
the counts describe the ledger the corpus actually compiled to, not a fresh
computation that might differ from what is committed. When those two disagree,
that disagreement is staleness, and 3.3 already says what to do about it.

### 3.6 The envelope version does not move

Adding a member to one verb's `report` payload is additive and MUST NOT change
`VERDICT_SCHEMA_VERSION`.

That constant versions the *envelope*: `schemaVersion`, `verb`, `ok`,
`exitCode`, and the presence of `report` or `error`. Spec 037 scopes it that
way, and its MINOR rule names `error.kind` and (since spec 049) `verb` tokens,
both envelope members. A payload is the verb's own shape, and versioning every
payload addition through the envelope would make the constant move for reasons a
consumer of a different verb cannot observe.

**Decision, 2026-09-06.** If per-payload versioning is ever wanted it needs its
own axis, one per verb, and this spec does not open that. Recorded because the
question is genuinely ambiguous in 037's text and the next spec to add a payload
field should not have to re-derive the answer.

## 4. Out of scope

**Reclassifying the error tier.** `I-003`..`I-009` reach the operator as
*staleness* (exit 2, the shard listed as "blocking diagnostics"), which
conflates "a declared unit does not resolve" with "the committed ledger is out
of date". Those are different problems with different fixes, and the code is
arguably wrong. It is left alone here: adopters branch on 2, changing it amends
spec 004, and this spec is about the tier that reaches nobody at all rather than
the tier that reaches somebody under an odd name. Named so it is on the record.

**The claimed-file-unit hashing gap, which bounds what this gate promises.**
A shard's hash covers its `spec.md`, its span-backing source files, and the
global-inputs scalar. `index.rs::span_files_for_mapping` collects backing files
for `section`, `symbol` and `module` units only, so a plain `file` unit's
content is in no shard hash: the claimed file can be rewritten and `index check`
still reports fresh. Reproduced while building this spec, on a corpus whose
spec claims `crates/a/src/lib.rs`; rewriting that file left the verdict at
`is fresh`, while editing the `spec.md` restaled it immediately.

This is the adopter audit's item 3.9 and it is not fixed here. It bounds this
spec honestly rather than silently: `--fail-on-unresolved` refuses a *recorded*
unresolved unit, and recording happens when the index is regenerated. It does
not promise that the committed ledger has noticed every change to a claimed
file, because no part of `index check` promises that today. Closing the gap
means either folding claimed file units into the shard hash or linting for a
claimed unit outside every hashed input, both of which change spec 004's
staleness contract and belong in their own spec.

**Making unresolved warnings block by default.** 3.2 explains why. That decision
belongs to a corpus, not to the tool.

**A waiver for warnings.** The coupling gate's waiver is a human instrument for
a refusal that is wrong in a specific case. `--fail-on-unresolved` is opt-in, so
a corpus that cannot yet satisfy it simply does not set it, and a per-warning
escape hatch would add a second, weaker waiver vocabulary for no gain.

**Folding index diagnostics into `lint`.** The two read different inputs: `lint`
reads the corpus, the indexer reads the corpus against the code tree. Merging
their outputs would make `lint` depend on a committed index and turn one cheap
corpus check into a tree-wide one.

**Changing what the indexer records.** No new code, no new tier, no change to
`classify_unresolved`. This spec adds readers, not diagnostics.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test diagnostics --locked
cargo test -p spec-spine-cli --locked
# This corpus carries no unresolved units, so the strict flag passes here.
target/release/spec-spine index check --fail-on-unresolved
# A clean corpus keeps the bare verdict line (3.1).
test "$(target/release/spec-spine index check)" = "index is fresh"
# The read verb answers, and answers as JSON (3.4).
target/release/spec-spine index diagnostics
test "$(target/release/spec-spine index diagnostics --json | python3 -c 'import json,sys; print(len(json.load(sys.stdin)))')" = "0"
# The envelope version did not move for a payload addition (3.6).
test "$(target/release/spec-spine index check --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["schemaVersion"])')" = "0.2.0"
```
