---
id: "108-a-work-scope-is-declared"
title: "A work scope is declared"
status: approved
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "072-two-ready-specs-can-collide"
summary: >
  Spec 072 reports that two ready specs claim overlapping territory, at spec
  granularity. What was left of WorkScope is the path-level half: one piece of
  work declares which paths it expects to change alone, which it expects to
  change alongside named other specs, and which it only reads. A scope is a
  document the consumer holds, like a context closure (spec 107). spec-spine
  evaluates it against the committed ownership index, reporting undeclared
  crossings, unowned paths and wrong sharing declarations, and compares two
  scopes for conflicting intentions. Both answers are reads. Nothing here
  locks, reserves, excludes or permits anything, and no gate reads a scope.
establishes:
  - { kind: file, path: "crates/spec-spine-core/src/scope.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/scope.rs" }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_scope.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/scope.rs" }
extends:
  # 3.6: the facade and the CLI entry point.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  # 3.6: the documentation a consumer reads. `docs/cli-reference.md` is edited
  # too, but claimed by no spec before or after this one, so it is left
  # unclaimed here rather than adding it to `[index] extra_hashed_inputs`
  # purely to clear `L-008` for a claim this change does not need to make.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  # 3.3, 3.5: the read axis moves for the two new documents, with its pin and its table.
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: context }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every mutable path owned by a spec other than the scope's own, and not declared shared, is reported as an undeclared crossing naming both."
    anchor: "3-3-evaluation-reports-what-the-declaration-and-the-ownership-disagree-on"
  - id: "R-2"
    kind: requirement
    text: "Two scopes conflict exactly when a path one changes is changed or read by the other; two scopes that only read a path do not conflict."
    anchor: "3-5-two-scopes-are-compared"
  - id: "R-3"
    kind: requirement
    text: "An evaluation over a stale index is refused with the staleness exit before any path is resolved."
    anchor: "3-4-an-evaluation-over-a-stale-index-is-refused"
  - id: "I-1"
    kind: invariant
    text: "No gate verdict reads a scope, and evaluating or comparing one writes nothing and grants nothing."
    anchor: "3-7-a-scope-is-not-a-gate-a-lock-or-a-permission"
  - id: "V-1"
    kind: verification
    text: "Each finding and each conflict kind is asserted positively and negatively, and every refusal has its exit code through the shipped binary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/scope.rs"
      - "crates/spec-spine-cli/tests/scope.rs"
---

# 108: A work scope is declared

## 1. Purpose

Note 04 §4.6 proposed a WorkScope carrying two things: a statement of which
paths a task may change, and a way to see that two tasks collide.

**The second half exists at spec granularity.** Spec 072 reports, from
`registry plan`, that two ready specs claim overlapping territory. What is
left is the path-level half: for one piece of work, which paths it expects to
change **alone**, which it expects to change **alongside** named other specs,
and which it only **reads**, checked against who actually owns each path.
Without it, "may I touch this file" is answered by reading ownership and
guessing about concurrency, and "somebody changed what I was reasoning about"
is not answerable at all.

### 1.1 Why it is built now (design note 09, D-7)

Opportunity-led, not requested. No consumer reads a scope today, and nothing
here depends on one appearing.

- *Capability.* A consumer can state, per task, the paths it intends to
  change, share and read, have spec-spine tell it where that statement
  disagrees with the corpus's ownership, and ask whether two tasks' statements
  collide.
- *Potential downstream value.* An orchestrator (the Statecraft CLI or
  platform, or another adopter's) deciding which two tasks may proceed side by
  side has a path-level answer to put beside spec 072's spec-level one, and a
  reviewer has a record of what a task said it would touch.
- *Compatibility.* New surfaces only: a module, a facade pair, one CLI
  command group and one read-document MINOR. No frontmatter key, no registry or
  index change, no verdict change.
- *What it does not claim.* §3.7.

### 1.2 Not an obligations dependency

A scope is about paths and the specs that own them, which the corpus already
models; nothing in it needs an obligation id. The two were proposed in the same
wave, which is not a dependency.

## 2. Territory

Establishes the scope module, its CLI command, and their tests. Extends the
facade, the CLI entry point, the read axis and the consumer documentation, as
the build records in its frontmatter.

## 3. Behavior

### 3.1 A scope lives with its consumer, and spec-spine evaluates it

The draft declared a scope in frontmatter and left open where it lives. It
lives where spec 107 §3.1 put a closure: in the consumer's record. A scope is a
property of one piece of work, and a work record is orchestration state, which
spec 092 keeps out of this repository. There is no `work_scope` frontmatter
key, and no spec's frontmatter changes.

A scope is a JSON document:

```json
{
  "id": "W-1",
  "ownSpec": "100-a-deleted-path-is-judged-where-it-lived",
  "mutable": ["crates/spec-spine-core/src/couple.rs"],
  "shared": [
    { "path": "crates/spec-spine-types/src/version.rs", "with": ["074-a-governed-read-names-its-version"] }
  ],
  "readOnly": ["crates/spec-spine-cli/src/cmd_delta.rs"]
}
```

- **`ownSpec`** (required) names the spec this work executes, by the corpus's
  one spec-id policy (a full id, or a short form resolving to exactly one).
- **`mutable`** are paths this work expects to change and expects no
  concurrent work to change.
- **`shared`** are paths it expects to change knowing that work under the
  specs named in `with` may change them too. `with` is non-empty.
- **`readOnly`** are paths it depends on and does not change.
- **`id`** is optional free text, carried through unchanged.

`mutable`, `shared` and `readOnly` each default to empty; at least one path in
total MUST be named. A path is repo-relative POSIX; an entry ending in `/`
names a subtree. An absolute path, a `..` segment, an empty entry, any other
member, or one path (or one subtree and a path inside it) under two roles MUST
be refused as a parse error (exit 3). A path named twice under one role
appears once.

### 3.2 Paths are resolved the way `index owner` resolves them

Each entry's owners are the specs the committed index says own it, by the
function `index owner` and `couple` already share (spec 048). A subtree entry's
owners are every spec that owns the subtree, anything inside it, or a subtree
containing it. `ownSpec` and every `with` entry MUST resolve to a spec in the
registry; every one that does not is named in one refusal (`NotFound`, exit 1),
as spec 107 §3.2 does for a closure.

A path need not exist: a scope may declare a file the work will create, whose
owners are then the specs whose claims cover it.

### 3.3 Evaluation reports what the declaration and the ownership disagree on

The evaluation is a read document (spec 074) carrying the scope's `id` and
resolved `ownSpec`, one entry per declared path with its `role` and resolved
`owners` (and `with` for a shared path), the index's aggregate content hash as
`indexHash`, and `findings`, each a warning with a code, the path, and the
names it disagrees about:

| Code | When |
|---|---|
| `S-001` unowned | a `mutable` or `shared` path no spec owns: territory to claim, or a path outside the corpus |
| `S-002` undeclared crossing | a `mutable` path owned by a spec other than `ownSpec`, naming `ownSpec` and those owners. The corpus's answer to a crossing is an `extends` edge, and a scope that hides one is worse than no scope |
| `S-003` sharing mismatch | a `shared` path whose `with` set differs from its owners other than `ownSpec`, naming both sets |

`readOnly` paths are resolved and reported, and raise no finding: who owns
something a task only reads is information, not a disagreement.

Findings are sorted by path, then code. The evaluation exits 0 whenever it
produced a document, findings or not: it is a report, not a gate (§3.7).

### 3.4 An evaluation over a stale index is refused

The owners a scope is checked against are the committed index's. An evaluation
over an index that no longer matches the tree would check a declaration
against ownership nobody can see, so the facade and the CLI MUST check index
freshness first and refuse a stale index with the staleness exit (2), as
spec 107 §3.5 refuses a stale registry.

### 3.5 Two scopes are compared

Given two scopes, a consumer can ask whether they conflict. Two entries
overlap when they name the same path or one names a subtree containing the
other. For each overlapping pair:

| Kind | One scope | The other |
|---|---|---|
| `both-mutable` | `mutable` | `mutable` |
| `mutable-shared` | `mutable` | `shared` |
| `changed-under-read` | `mutable` or `shared` | `readOnly` |

Two `shared` entries do not conflict (each declared it expects the other), and
two `readOnly` entries do not conflict. The answer is a read document naming
both scopes' `id` and `ownSpec` as declared and each conflict's kind and both
entries, sorted. Comparison reads no ledger: it is a pure function of the two
documents, which are validated as in §3.1, and it exits 0 whether or not they
conflict.

This is spec 072's collision question asked at path granularity and between
declared intentions rather than between ready specs. It does not replace
072's report and does not read it.

### 3.6 The surfaces

- `spec_spine_core::scope::evaluate_scope(...)`, pure over the registry, the
  committed index and the document; and `spec_spine_core::scope::compare_scopes(a, b)`,
  pure over two documents.
- `spec_spine_core::scope_json(config_json, repo_root, scope_json)` and
  `spec_spine_core::scope_compare_json(a_json, b_json)`: the facade, `&str` in
  and `String` out; the first checks index freshness and reads the committed
  ledger.
- `spec-spine scope evaluate --scope <FILE | -> [--json]` and
  `spec-spine scope compare <A> <B> [--json]` (one of which may be `-`).
  Without `--json`, one line per path and per finding, or per conflict.

`docs/api.md` and `docs/cli-reference.md` document the document, both answers
and the facade.

### 3.7 A scope is not a gate, a lock or a permission

A scope establishes that somebody **declared** this partition for this work,
and what the ownership index said about each path when it was evaluated.

It does **not** establish, and nothing here claims:

- **that the declaration is complete.** Work may change a path its scope never
  named. `couple` judges what was changed; a scope is what was intended.
- **exclusion.** Declaring a path `mutable` reserves nothing and prevents no
  other work from changing it. Two scopes may both declare it; §3.5 reports
  that, and deciding which proceeds is an orchestration decision (spec 092).
- **locking, holding or release.** Nothing is written, so there is nothing to
  hold. A scope that could be held would be a lock with no owner and no
  release.
- **permission.** Declaring a path grants nothing. Ownership and the coupling
  gate are unchanged.

No verb changes its verdict because of a scope: `couple`, `check`, `lint` and
`index coverage` never read one. The gate judges the diff against the corpus,
and a second, weaker statement of intent must never be able to clear a path the
corpus refuses, nor refuse one it clears. "The scope said I could" is the most
obvious wrong thing to build next, and it would turn a planning aid into an
ownership bypass. Evaluating and comparing write nothing.

## 4. Out of scope

- **Any gate, lock, reservation or permission.** §3.7.
- **Scheduling.** Which of two conflicting scopes proceeds is Statecraft's.
- **Re-doing spec 072's collision report.** §3.5.
- **Storing scopes.** The consumer holds them (§3.1).

## 5. Resolved decisions

**D-1 (2026-09-23, filed draft corrected before the build).** The draft
declared a scope in frontmatter under `work_scope`, with its location as open
question 1. That question is answered by the boundary, as spec 107 D-1
answered it for closures: a scope is a document the consumer holds and
spec-spine evaluates, so the frontmatter key is dropped and no spec's
frontmatter changes. The draft's warnings "at compile time" become the
evaluation's findings (§3.3), and its "none changes `lint`'s exit code"
becomes §3.7's stronger statement that no gate reads a scope at all. Its open
question 2 (whether `readOnly` is worth declaring) is answered by keeping it
and raising no finding on it: its value is §3.5's `changed-under-read`, the
"somebody changed what I was reasoning about" case, which needs it declared.
`changed-under-read` extends the draft's list, which named only
mutable-and-read-only, to a `shared` entry as well: a shared path is still one
the other work changes. The draft's owner-lookup wording ("the compiler MUST
resolve its owners") is kept as §3.2, now against the committed index.

**D-2 (2026-09-23, build).** §3.3 does not say what a `mutable` or `shared`
path that resolves to no owner at all reports beyond `S-001`, and whether
`S-002`/`S-003` also fire on top of it. The smallest faithful reading: `S-001`
is the whole answer for an unowned path. A `mutable` entry checks `S-001`
first and only computes the crossing (`S-002`) when owners is non-empty; a
`shared` entry checks `S-001` first and only compares the declared `with`
against the owners (`S-003`) when owners is non-empty. An unowned `shared`
path with a non-empty `with` therefore reports only `S-001`, not also a
sharing mismatch against the empty owner set: the more fundamental problem
(nobody owns this yet) is the one finding, not two that both restate it.

**D-3 (2026-09-23, build: fail-first measured).** Both new test files
(`crates/spec-spine-core/tests/scope.rs`, `crates/spec-spine-cli/tests/scope.rs`)
were run against a tree with `scope.rs` and `cmd_scope.rs` removed and the
`lib.rs`/`main.rs`/`version.rs`/`read.rs` edits reverted (the frontmatter and
docs edits are inert either way). The core suite failed to compile (every new
name unresolved); the CLI suite compiled, since `main.rs` no longer referenced
`cmd_scope`, and failed at runtime instead (`error: unrecognized subcommand
'scope'`, exit 3, on 8 of 9 tests; the ninth, which itself expects exit 3,
passed by coincidence). Both are genuine fail-first results: neither test file
passes against unbuilt code.

## Acceptance

The build MUST establish, behaviorally, with the negative cases alongside the
positive ones:

- a scope whose paths all resolve evaluates with no finding;
- a `mutable` path owned by another spec, not declared `shared`, produces
  `S-002` naming both; the same path declared `shared` with the right `with`
  produces nothing;
- a `mutable` path no spec owns produces `S-001`;
- a `shared` entry whose `with` disagrees with the owners produces `S-003`
  naming both sets;
- a `readOnly` path owned by another spec produces no finding;
- an unknown `ownSpec` or `with` spec is refused with exit 1 naming every one;
  a malformed document (unknown member, `..`, absolute path, no path at all,
  one path under two roles) with exit 3; a stale index with exit 2;
- two scopes declaring the same path `mutable` conflict as `both-mutable`; a
  subtree against a file inside it overlaps; `mutable` against `shared` and
  changed against `readOnly` conflict; two scopes sharing only a `readOnly`
  path, or only a `shared` path, do not;
- `couple`'s verdict on the same diff is byte-identical with and without a
  scope document committed in the tree that declares a changed path
  `readOnly`, and `check` and `lint` are unchanged;
- the facade and the CLI agree, and the emitted documents carry the new read
  axis version.

## Verification

Written to fail against the tree it is filed on: nothing named here exists.

```verify:cli
cargo build --release --locked
# 3.1 - 3.5, 3.7: evaluation, comparison, refusals, gate neutrality.
cargo test -p spec-spine-core --test scope --locked
# 3.4, 3.6: the shipped verbs, including stdin, the stale refusal and exit codes.
cargo test -p spec-spine-cli --test scope --locked
# A scope over this corpus evaluates, and one of its own paths is a crossing.
sh -c 'printf "%s" "{\"ownSpec\":\"108\",\"mutable\":[\"crates/spec-spine-core/src/couple.rs\"]}" | ./target/release/spec-spine scope evaluate --scope - --json | grep -q "\"S-002\""'
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
