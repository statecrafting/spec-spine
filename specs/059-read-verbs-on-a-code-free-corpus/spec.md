---
id: "059-read-verbs-on-a-code-free-corpus"
title: "Two read verbs that mislead a specify-first corpus"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "011-index-render-orphans"
  - "032-ownership-coverage"
  - "044-in-progress-is-in-flight"
extends:
  - { spec: "011-index-render-orphans", unit: "crates/spec-spine-core/src/render.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  - { spec: "011-index-render-orphans", unit: "crates/spec-spine-core/tests/render.rs", nature: additive }
  # The partition reads the registry's lifecycle fields (3.1).
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # Both renderings' acceptance, which pinned the old flat shape and the
  # empty-case silence.
  - { spec: "011-index-render-orphans", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Three of the four governed repositories are specify-first: the whole corpus is
  ratified before a line of code exists, and specs live at approved plus pending
  for months. Two read verbs give that corpus an answer that is technically
  correct and practically a lie. `index orphans` reports sixty-four of hqgit's
  sixty-eight specs, because an orphan is a spec claiming nothing that resolves,
  and a spec whose code is not written yet claims nothing that resolves; the
  list is the corpus, so it says nothing. `index coverage --fail-on-untraced`
  on a tree with no discovered package enumerates zero source files, computes
  zero untraced, and exits 0 from inside a CI step named "the whole-tree
  ownership assertion". This spec makes the first partition by the in-flight
  predicate the index already computes, and makes the second refuse an empty
  universe instead of passing it.
---

# 059: Two read verbs that mislead a specify-first corpus

## 1. Purpose

Specify-first is not an edge case. Three of the four repositories the audit
examined work that way: hqgit (68 specs, no code), aicortex (29 specs, no code)
and rahi (22 specs, one wave built). Specs 041 and 044 were written for exactly
this mode. It is the mode a new adopter is in on their first day, and it is the
mode two read verbs handle badly.

**`index orphans` reports the corpus.** An orphan is a spec whose
`implementing_paths` is empty: it claims nothing that resolves to a file. On a
corpus where the code is not written yet, that describes every spec that has not
been built, which on hqgit is sixty-four of sixty-eight. The output is correct
and carries no information, because a list that is nearly the whole corpus
cannot distinguish the case it exists to find: a spec that claims territory
which will never resolve, because the code moved or the claim was wrong.

**`index coverage --fail-on-untraced` passes an empty tree.** The coverage
universe is package-scoped: `enumerate_source_files` walks the discovered
packages, and a repository with no `Cargo.toml` and no `package.json` discovers
none. So `source_files` is 0, `untraced_files()` is 0, `is_fully_claimed()` is
true, and the verb exits 0. An adopter who has wired this into CI under a step
named for the whole-tree ownership assertion has a green check asserting
nothing, and it stays green on the day they add their first source file outside
a package.

The second is the more dangerous of the two, because it fails open. The first
merely wastes a verb. Both come from the same root: a predicate that means one
thing on a built corpus and something else on a corpus that has not been built,
with no way for the reader to tell which they are looking at.

These are items 11 and 12 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/render.rs` | 011 | `orphans` partitions |
| `crates/spec-spine-core/tests/render.rs` | 011 | its acceptance |
| `crates/spec-spine-core/src/coverage.rs` | 032 | the empty-universe case |
| `crates/spec-spine-core/tests/coverage.rs` | 032 | its acceptance |
| `crates/spec-spine-cli/src/cmd_index.rs` | 004 | both renderings |

Neither spec 011 nor spec 032 is edited. 011 requires `orphans` to report the
id-sorted `orphanedSpecs` list, and it still does; this spec adds a partition
over that same list. 032 requires `--fail-on-untraced` to exit 1 unless every
source file has a specific owner, and it still does; this spec adds a refusal
for the case where "every source file" is vacuous.

## 3. Behavior

### 3.1 `orphans` partitions by the in-flight predicate

`index orphans` MUST report two groups rather than one flat list:

```
orphaned (claims nothing that resolves, and is not in flight):
  019-structured-partial-supersedes

in flight (claims nothing that resolves yet; draft or pending):
  061-something-not-built-yet
  062-also-not-built-yet
```

The predicate is the one `index.rs` already computes and spec 044 defined:
in flight is `implementation: complete` being absent and either `status: draft`
or `implementation` in `{pending, in-progress}`. It is the same predicate that
decides whether an unresolved unit is a blocking error or a `W-001` warning, and
using it here means the two verbs cannot disagree about which specs are under
way.

**Decision, 2026-09-07: the lifecycle half is read from the registry.** The
index shard records `spec_status` and not `implementation`, so the committed
index alone cannot answer the predicate. Adding the field would move
`INDEX_SCHEMA_VERSION` and restamp every shard for a read verb's benefit, which
§3.4 forbids. `index orphans` therefore loads both committed artifacts, as
`couple` already does. A spec the registry has no record of is treated as in
flight: a corpus that cannot say otherwise should not have its spec called
abandoned.

The first group is the finding. A spec that is neither draft nor pending nor
in-progress, that claims nothing resolving, is a spec whose author said the work
is done or does not apply while the ledger says nothing of theirs exists. That
is the condition `orphans` was built to surface, and on hqgit it is four specs
rather than sixty-four.

The second group is the corpus's normal state and MUST be reported, not
filtered. Suppressing it would replace a useless answer with an incomplete one,
and a scheduler wanting "what has no code yet" would lose the verb that answers
it. Two named groups let a reader take either.

Under `--json`, the flat array becomes an object with the two arrays as named
members. This is a **breaking** shape change for a consumer parsing the array
directly, and it is the right call: the flat array's meaning is the thing this
spec is fixing, so preserving it would preserve the defect under a compatibility
argument. `INDEX_SCHEMA_VERSION` does not move (no committed artifact changes)
and no verdict envelope is involved (`orphans` emits none); the change is to one
read verb's `--json` output and MUST be called out in the release notes as such.

Ordering inside each group stays id-sorted, as spec 011 §3.3 requires.

**Decision, 2026-09-07: a corpus with no orphans stays silent.** The prose form
prints both groups whenever either has members, so a reader always sees which
side an id fell on. With both empty it prints nothing, as it did before this
spec: two headers and two `(none)` lines would be noise on the answer "nothing
to report", and an existing acceptance test pinned the silence. The `--json`
form is unaffected and always emits both arrays, since a consumer parsing an
object should not have to distinguish "absent" from "empty".

### 3.2 An empty coverage universe is a refusal, not a pass

`index coverage --fail-on-untraced` MUST exit `1` when the coverage universe is
empty, with a message naming the reason:

```
coverage: no source files under any discovered package.
--fail-on-untraced asserts that every source file has a specific owning spec,
and there are none to assert about. Nothing was verified.
```

The flag is an assertion, and an assertion over an empty set is vacuously true,
which is the mathematically correct answer and the operationally wrong one. A
person wiring `--fail-on-untraced` into CI is asserting that the tree is fully
owned. If the tool cannot see the tree, the honest report is that the assertion
did not run, and a CI step that did not run its check should not be green.

Without `--fail-on-untraced`, `coverage` MUST still exit `0` on an empty
universe and report the fact plainly. The report is a read verb, and "no source
files under any discovered package" is a true and useful thing for it to say to
a specify-first corpus. Only the assertion refuses.

An empty universe with packages discovered but no source files in them is the
same refusal for the same reason. What matters is that the denominator is zero,
not why.

### 3.3 The message says which of two things happened

The refusal MUST distinguish, in its text, no discovered packages from
discovered packages containing no source files. They have different fixes:
the first is usually `layout.standalone_rust_workspaces` or
`standalone_npm_packages` not naming a package that exists, and the second is
usually `index.resolver_exclusions` pruning too much.

A refusal that names the condition without naming the likely cause makes the
reader search for both. The audit found adopters deriving this class of thing by
experiment, and this is the cheapest place to stop that.

### 3.4 Nothing committed moves

No shard changes, `INDEX_SCHEMA_VERSION` and `VERDICT_SCHEMA_VERSION` stay
where they are, and `compile --check` stays fresh. Both changes are to read
verbs' output and to one exit code in a case that previously could not fail.

This repository's CI runs `index coverage --fail-on-untraced` and has four
discovered packages with seventy-four source files, so §3.2's refusal is
unreachable here and the step's behavior is unchanged.

## 4. Out of scope

**Widening the coverage universe beyond packages.** hqgit's four claimed scripts
live outside every package and are therefore invisible to coverage as well as
unhashed. That is a real gap, it is the same gap spec 057 measures from the hash
side, and widening the universe changes what every coverage number in every
adopting repository means. It needs its own spec and its own migration note.

**Changing what an orphan is.** A spec claiming nothing that resolves is the
definition spec 011 gave and this spec keeps it. Only the presentation
partitions.

**A `--fail-on-orphans` gate.** `orphans` stays a read verb. With the partition
in place a gate becomes conceivable for the first time, since the first group is
finally a small set, but adding a refusal is a separate decision from making the
report meaningful.

**Reporting in-flight specs anywhere else.** `registry plan` already answers
"what can be worked on" from the scheduling side. This spec makes `orphans`
honest, not a second scheduler.

## 5. Verification

Both changes fail against pre-059 code: `orphans` emitted one flat array, and
`--fail-on-untraced` exited 0 on an empty universe.

Each line below is one command. Spec 049 §3.2 is explicit that a fence's body
line **is** a command, so a trailing `\` continuation is not joined: the
continuation becomes its own fragment and fails. The setup lines here are
therefore single lines, however long.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test render --locked
cargo test -p spec-spine-core --test coverage --locked
# 3.1: `orphans` reports two named groups instead of one flat list.
target/release/spec-spine index orphans --json | python3 -c 'import json,sys; o=json.load(sys.stdin); assert set(o) == {"orphaned", "inFlight"}, o'
# 3.2 + 3.3: the empty-universe fixture. Built once at a fixed path, because
# each line here is its own shell and a `$(mktemp -d)` would not survive to the
# next assertion.
rm -rf "${TMPDIR:-/tmp}/ss059" && mkdir -p "${TMPDIR:-/tmp}/ss059/specs/001-x" && : > "${TMPDIR:-/tmp}/ss059/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n' > "${TMPDIR:-/tmp}/ss059/specs/001-x/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index >/dev/null
# 3.2: without the flag it still exits 0 and reports the fact. The report is a
# read verb, and "no source files" is a true and useful thing to say.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage
# 3.2: with the flag it refuses. An assertion over an empty set is vacuously
# true, and a CI step that did not run its check should not be green.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced ; test $? -eq 1
# 3.3: and the message names which of the two empty cases this is.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced 2>&1 | grep -q 'no package was discovered'
rm -rf "${TMPDIR:-/tmp}/ss059"
# 3.4: this repository's own coverage assertion is unaffected: four packages,
# seventy-four source files, so the refusal is unreachable here.
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine compile --check
```
