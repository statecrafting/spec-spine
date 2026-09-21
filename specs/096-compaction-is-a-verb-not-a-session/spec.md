---
id: "096-compaction-is-a-verb-not-a-session"
title: "Compaction is a verb, not a session"
status: draft
kind: "tooling"
created: "2026-09-20"
summary: >
  Spec 095 removed 27 specs and renumbered 93, which meant rewriting about six
  thousand references by hand. Five distinct defects were introduced doing it,
  every one a rewrite rule that matched more or less than its author meant, and
  every one silent: the corpus compiled, the gate passed, and the citations
  pointed at the wrong documents. Four were caught only because spec 089 built a
  sweep that runs every acceptance block, and the fifth was caught by the sweep
  finding a sixth. A corpus whose ids are the tool's own vocabulary should not
  need a session to renumber itself. This spec adds `spec-spine compact`: a plan
  in, a rewritten corpus out as data, idempotent, refusing every ambiguity the
  five defects came from, and reporting what it changed rather than leaving a
  six-thousand-line diff to read.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "001-compile-registry"
  - "015-short-id-resolution"
  - "089-nothing-reruns-a-merged-acceptance"
  - "095-the-corpus-describes-what-exists"
establishes:
  - { kind: file, path: "crates/spec-spine-core/src/compact.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/compact.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_compact.rs", planned: true }
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/corpus-map.md" }, role: context }
---

# 096: Compaction is a verb, not a session

## 1. Purpose

### 1.1 Five defects, one family

Spec 095's rewrite touched 2,910 prose citations, 3,491 full-id references, the
frontmatter of 93 specs, 32 command arguments, 12 acceptance blocks, four
manifests, the sweep's ledger and the harness. Doing it by hand produced five
defects, measured on 2026-09-20:

| # | Defect | What it did |
|---|---|---|
| 1 | The map was keyed on the three-digit **ordinal**, and three new specs carried provisional ids colliding with three real ones. Last write won. | 355 citations across 34 files pointed at the wrong document |
| 2 | A **bare short id** in a command (`registry show 111`, `spec-spine verify 044`) carries no slug and no `spec` before it, so no rule matched it. | 32 acceptance commands addressed the wrong spec or none |
| 3 | The citation rule matched `spec 022` **inside** `--spec 022-index-sharding`, because the hyphen after the digits is a word boundary, so every `--spec <full-id>` was rewritten **twice**. | 18 lines across 7 specs named a third, unrelated spec |
| 4 | The rewrite was scoped to acceptance blocks with a **non-greedy fence regex**, and one block greps for the literal opening fence, which closed the match early. | one file was rewritten in half, and the second pass then double-applied |
| 5 | An **absence assertion** read a message with the repository path interpolated into it, and the sweep names its temporary directory after the spec id. | a spec named `...-is-not-a-stale-shard` failed an assertion about the word `stale` |

Every one is the same shape: a pattern that matched more, or less, than its
author meant. Every one was silent. `compile`, `index`, `lint`, `couple` and
`check` were all green with 355 citations pointing at the wrong document,
because a citation is prose to the compiler.

### 1.2 What caught them, and what that cost

Four of the five were caught by running the whole corpus's declared acceptance
(spec 089's sweep) and reading the failures. The fifth was found while fixing
the fourth. Each cycle is a forty-minute sweep on a loaded machine, and the
sweep's value here was entirely accidental: it exists to notice a **merged**
acceptance going stale, not to review a rewrite.

Defects 1, 2 and 3 are each a one-line rule change in a program. A program that
made the rewrite once, correctly, would have cost nothing and caught all three
at the point of writing.

### 1.3 What is, and is not, mechanizable

Spec 095's work divides cleanly, and this spec claims only the mechanical half.

| Step | This spec |
|---|---|
| deciding which specs are removed or merged | **no**: judgement, and it read 205 `MUST` clauses to make it |
| writing the successor documents | **no**: authoring |
| withdrawing a unit claim from frontmatter | **yes**, once decided |
| computing the map and rewriting every reference | **yes**, and this is where all five defects were |
| emitting the map document | **yes** |
| verifying the result | already spec 089's |

## 2. Territory

| Path | What |
|---|---|
| `crates/spec-spine-core/src/compact.rs` | the plan, the map, the rewrite, as a pure function |
| `crates/spec-spine-core/tests/compact.rs` | the rules, each with the defect it prevents |
| `crates/spec-spine-cli/src/cmd_compact.rs` | reads the plan, writes the files, reports |

`lib.rs` gains `compact_json`, the JSON facade every other capability has.
`main.rs` gains the subcommand.

## 3. Behavior

### 3.1 A plan in, a corpus out as data

`compact(cfg, corpus, plan) -> Compaction` MUST be a pure function of its
arguments: no filesystem writes, no clock, no `git`. It returns the rewritten
files as data, the same shape the governance scaffold returns, and the CLI
writes them. The engine's central invariant is unchanged.

The **plan** is authored, not inferred, because §1.3 says which half is
judgement. It names, for each spec leaving the corpus, the spec that answers for
it afterwards:

```yaml
remove:
  - { spec: "029-claude-code-skill-kit", answered_by: "095-the-corpus-describes-what-exists" }
  - { spec: "046-kit-hooks-read-never-write", answered_by: "093-the-harness-this-repository-runs" }
renumber: contiguous     # or `none`, to remove without compacting the ordinals
```

`compact` MUST refuse a plan naming a spec the corpus does not have, or an
`answered_by` it does not have, before it rewrites anything.

### 3.2 The map is computed, and an ordinal collision is a refusal

The survivors MUST be renumbered in their existing order, so a spec filed
earlier keeps a lower ordinal. The map MUST be keyed on the **full id**, never
on the ordinal.

Where two entries would share an ordinal, `compact` MUST refuse and name both.
Defect 1 was a last-write-wins on exactly that collision, and last-write-wins is
never the right answer to an ambiguity in an identifier.

### 3.3 Three reference shapes, and what each excludes

A reference is rewritten if and only if it is one of these, and the exclusions
are as normative as the forms:

1. **A full id**, `NNN-slug`, matched against the map's keys. Longest key first,
   so no key shadows a longer one.
2. **A prose citation**, `spec NNN` or `specs NNN`, any case, where `NNN` is a
   mapped ordinal. It MUST NOT match when the digits are followed by `-` or a
   word character: those digits belong to a full id, which form 1 has already
   handled, and matching them again is defect 3. It MUST NOT match when preceded
   by another project's name (`OAP spec 177`).
3. **A bare short id as a command argument**, following `registry show`,
   `registry relationships`, `verify`, `index owner`, `attest --spec`,
   `compile --spec`, `verify-attestation --spec` or `delta`. This is defect 2.
   It MUST NOT be applied to a command carrying `--repo`: that command addresses
   a fixture corpus whose ids are its own, not this corpus's.

Nothing else is rewritten. A three-digit token that is not one of these is left
alone, and the corpus is full of them: `V-016`, `C-001`, `L-008`, `W-001`,
`0.18.0`, years, counts and line numbers.

### 3.4 Applying twice is applying once

`compact` MUST be **idempotent**: applying its output to its output MUST produce
no further change. This is stated as a requirement rather than left to follow
from §3.3 because it is the property defect 3 violated, it is cheap to assert,
and an assertion of it fails loudly where the rule's subtlety does not.

### 3.5 A block is found by its fence, line by line

Where a rule applies only inside a `verify:cli` block, the block MUST be located
by scanning lines, opening on a line that is exactly the opening fence and
closing on a line that is exactly the closing fence. A regex over the whole
document MUST NOT be used: a block that greps for the literal fence closes a
non-greedy match early and silently halves the rewrite, which is defect 4.

### 3.6 The report is the review

`compact` MUST return, per file, every rewrite it made: the old text, the new
text, and which of §3.3's three forms produced it. A six-thousand-line diff is
not reviewable, and the question a reviewer has is not "what changed" but "was
each change the right rule".

The counts MUST be reported per form, so a form that fired zero times in a
corpus that obviously contains it is visible. Defect 2 was exactly a form that
fired zero times.

### 3.7 The map document

`compact` MUST emit the map document spec 095 §3.4 establishes: every removed id
with the spec that answers for it, and every survivor's old ordinal beside its
new one. A citation outlives the document it cites, and after a renumber a bare
old ordinal is worse than a dangling one, because it resolves.

### 3.8 What it refuses

`compact` refuses, exit 3, before writing anything:

- a corpus that does not compile: a rewrite of a corpus whose ids are not yet
  valid produces ids that are differently invalid;
- a plan naming a spec that is not there (§3.1);
- an ordinal collision (§3.2);
- a reference it cannot classify: a `NNN-slug` token whose ordinal is mapped but
  whose slug matches no spec, which is either a typo or a citation of a
  document that never existed, and guessing between those is not the tool's.

The CLI MUST refuse a dirty working tree unless `--force` is given: the output
is a rewrite of the tree, and an uncommitted edit underneath it is unreviewable.

`--plan` prints the map and the report and writes nothing, which is how a
rewrite of this size is read before it is taken.

## 4. Out of scope

- **Deciding what to remove or merge.** §1.3. The plan is authored.
- **Writing the successor specs.** Authoring.
- **A Statecraft workflow around it.** Orchestrating a corpus cleanup (propose,
  review, apply, verify) is a workflow, and workflows are Statecraft's after
  spec 092. What belongs here is the mechanism the workflow calls. The division
  is the same one §1.3 draws: judgement outside, determinism inside.
- **Measuring the debt that motivates a compaction.** `index diagnostics`,
  `lint`, `index coverage` and `verify --plan` already answer most of it, and
  the rest (which acceptance blocks read a path that no longer exists, which
  specs own nothing) is a reporting spec of its own.
- **Rewriting history, or anything outside the working tree.** Citations in git
  history, merged pull requests and adopter repositories are not rewritten;
  §3.7's map is how they are read.

## 5. Resolved decisions

D-1 (2026-09-20, this is spec-spine's, not Statecraft's). The subject is spec
ids, which are this tool's own vocabulary, and the operation is a deterministic
function of the corpus. Statecraft owns the environment and the workflow around
a cleanup; it should no more implement a renumber than it should implement
`compile`. §4 draws the line.

D-2 (2026-09-20, files-as-data rather than a writing engine). `compact` returns
the rewritten corpus and the CLI writes it, which is the shape the governance
scaffold already has and which keeps the library a pure function of `(config,
file contents)`. It also makes `--plan` free: the same call, nothing written.

D-3 (2026-09-20, the plan is authored). Inferring which specs should be merged
from the corpus was considered and refused. Spec 095's merge map came from
reading 205 normative clauses and deciding which survived a deletion; nothing in
the ledger encodes that. A tool that guessed would produce a plan a human must
check clause by clause, which is the work it claimed to save.

D-4 (2026-09-20, the five defects are named in the acceptance, not only here).
Each of §1.1's five gets a test that fails against the rule that produced it.
A spec written from a post-mortem is worth what its acceptance can still catch a
year later, and the prose above will not fail.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured 2026-09-20 at this spec's parent: `registry
show 096` is red (not found, exit 1); `compact.rs` does not exist in either
crate; `spec-spine compact --help` is an unrecognised subcommand (exit 3).

```verify:cli
cargo build --release --locked
# 2: the territory exists.
test -f crates/spec-spine-core/src/compact.rs
test -f crates/spec-spine-core/tests/compact.rs
test -f crates/spec-spine-cli/src/cmd_compact.rs
# 3: the verb exists, and reads before it writes.
target/release/spec-spine compact --help > "${TMPDIR:-/tmp}/ss096-help.txt" 2>&1
grep -qF -- '--plan' "${TMPDIR:-/tmp}/ss096-help.txt"
rm -f "${TMPDIR:-/tmp}/ss096-help.txt"
# 3.1 - 3.8: every rule, and 5's five defects each with a case that fails
# against the rule that produced it. A non-zero pass count, so a filter that
# matched nothing cannot pass for a run (spec 084 D-7).
cargo test -p spec-spine-core --test compact --locked > "${TMPDIR:-/tmp}/ss096-t.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-t.txt"
rm -f "${TMPDIR:-/tmp}/ss096-t.txt"
# 1.1 defect 1: an ordinal collision is refused, and both entries are named.
cargo test -p spec-spine-core --test compact --locked collision > "${TMPDIR:-/tmp}/ss096-1.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-1.txt"
# 1.1 defect 2: a bare short id in a command is rewritten, and one behind
# `--repo` is not.
cargo test -p spec-spine-core --test compact --locked short_id > "${TMPDIR:-/tmp}/ss096-2.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-2.txt"
# 1.1 defect 3, and 3.4: applying the output to the output changes nothing.
cargo test -p spec-spine-core --test compact --locked idempotent > "${TMPDIR:-/tmp}/ss096-3.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-3.txt"
# 1.1 defect 4: a block that greps for its own fence is rewritten whole.
cargo test -p spec-spine-core --test compact --locked fence > "${TMPDIR:-/tmp}/ss096-4.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-4.txt"
rm -f "${TMPDIR:-/tmp}/ss096-1.txt" "${TMPDIR:-/tmp}/ss096-2.txt" "${TMPDIR:-/tmp}/ss096-3.txt" "${TMPDIR:-/tmp}/ss096-4.txt"
# 3.1, D-2: the library writes nothing. A source read, because the contract is
# what a caller relies on and a doc comment does not fail.
! grep -qE 'fs::(write|create|remove)' crates/spec-spine-core/src/compact.rs
# 3.6: the report names the form that produced each rewrite, per file.
cargo test -p spec-spine-core --test compact --locked report > "${TMPDIR:-/tmp}/ss096-5.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss096-5.txt"
rm -f "${TMPDIR:-/tmp}/ss096-5.txt"
# Declared and read through the CLI.
target/release/spec-spine registry show 096 --json > "${TMPDIR:-/tmp}/ss096-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss096-show.json')); assert d['id'] == '096-compaction-is-a-verb-not-a-session', d"
rm -f "${TMPDIR:-/tmp}/ss096-show.json"
# The governed loop and the stack's own gate.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```
