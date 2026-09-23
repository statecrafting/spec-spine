---
id: "113-a-waiver-has-a-declared-lifecycle"
title: "A waiver has a declared lifecycle"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
establishes:
  # 3.1 to 3.6: the declaration, its parsing and its evaluation.
  - { kind: file, path: "crates/spec-spine-core/src/waiver.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/waiver.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/waiver.rs" }
extends:
  # 3.3, 3.6, 3.8: the gate evaluates every declared waiver and reports it.
  - { spec: "005-coupling-gate", unit: { kind: file, path: "crates/spec-spine-core/src/couple.rs" }, nature: additive }
  - { spec: "005-coupling-gate", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_couple.rs" }, nature: additive }
  - { spec: "005-coupling-gate", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "005-coupling-gate", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  # 3.8: the verdict envelope's MINOR, with its pins.
  - { spec: "034-machine-readable-verdicts", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "034-machine-readable-verdicts", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  - { spec: "034-machine-readable-verdicts", unit: { kind: file, path: "crates/spec-spine-cli/tests/cli.rs" }, nature: additive }
  # The documentation a consumer reads.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
summary: >
  A `Spec-Drift-Waiver:` line clears every refusal in the pull request it
  appears in, with no scope, no expiry and no record of what it excused. A
  waiver may now declare a path scope, an expiry, an ancestry and a use limit
  on lines of its own. The gate evaluates each declared check over inputs the
  caller supplies (an as-of date, an ancestry answer, a prior use count) and
  reports it as satisfied, failed or not evaluated, never guessing a missing
  input. A failed check makes the waiver clear nothing; a scoped waiver clears
  only its paths; and the verdict says which waiver cleared which violation.
  Nothing is counted, consumed, stored or authorized by the engine, and an
  undeclared waiver behaves exactly as before.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "A scoped waiver clears only violations on the paths it lists, and a violation it does not cover still refuses."
    anchor: "3-2-a-waiver-may-declare-its-scope"
  - id: "R-2"
    kind: requirement
    text: "A declared check whose input was not supplied is reported as not evaluated, never as satisfied, and an absent use count is never treated as zero."
    anchor: "3-4-a-missing-input-is-not-evaluated-never-satisfied"
  - id: "R-3"
    kind: requirement
    text: "A waiver with any failed lifecycle check clears nothing, and the verdict names the failed check."
    anchor: "3-5-a-failed-check-clears-nothing"
  - id: "R-4"
    kind: requirement
    text: "The verdict associates every cleared violation with the one waiver that cleared it."
    anchor: "3-6-the-verdict-says-which-waiver-cleared-what"
  - id: "I-1"
    kind: invariant
    text: "The engine reads no clock, runs no git, writes no usage record and consumes nothing; every lifecycle input is supplied by the caller."
    anchor: "3-7-what-the-engine-does-not-do"
  - id: "I-2"
    kind: invariant
    text: "A run with no waiver declared produces the same report bytes it produced before this spec; only the envelope's schemaVersion moves."
    anchor: "3-8-compatibility"
  - id: "V-1"
    kind: verification
    text: "Each lifecycle check is asserted satisfied, failed and not evaluated from the verdict, the pairing is asserted non-positionally, and disabling the checks or the scope fails the suite."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/waiver.rs"
      - "crates/spec-spine-cli/tests/waiver.rs"
---

# 113: A waiver has a declared lifecycle

## 1. Purpose

The waiver is the corpus's escape valve, and it is the least specified thing in
the gate. `parse_waiver` takes the first line beginning with the configured
keyword and returns its reason. That reason then clears **every** violation in
the run.

So a waiver written for one manifest bump also clears an unrelated `C-001` on
a source file in the same pull request, and nobody reading the verdict can
tell, because the report carries one reason string and no association between
it and the paths it excused. Note 04 §5 P2 records this as B17. The 0.22.0
version bump (#291) is the live case: its approved waiver named two manifests
in prose and, as parsed, would have cleared anything.

### 1.1 Why it is built now (design note 09, D-7)

Opportunity-led, not requested.

- *Capability.* A human writing a waiver can make it narrower than "everything
  in this pull request": these paths, until this date, on this history, this
  many times. The verdict then shows exactly what each waiver excused.
- *Potential downstream value.* A reviewer reads which violation a waiver
  cleared instead of inferring it. An orchestrator (the Statecraft CLI or
  platform) that keeps its own record of waiver use can supply the count and
  the date and get a verdict that honours them, without spec-spine becoming
  the store.
- *Compatibility.* Additive: new optional lines in the pull request body, new
  optional CLI flags and facade members, and one verdict-envelope MINOR whose
  new member is omitted when no waiver is declared.

### 1.2 What is deliberately not being built

| Not built | Why |
|---|---|
| a use counter stored in an authored file | the gate would have to write during a read, and every artifact-producing function here is a pure function of config and file contents |
| atomic consumption ("this waiver is now spent") | that is shared mutable state across concurrent runs, which needs a lock and an owner, and the engine has neither |
| an expiry read from the wall clock | there is no clock in core, by construction |
| approval | who may write a waiver is the repository's policy and the review system's enforcement |

What is left is **pure evaluation over inputs the caller supplies**, which is
the shape spec 100 uses for a prior snapshot: the CLI gathers the facts, the
library decides.

## 2. Territory

Extends spec 005's coupling module and CLI command, the facade and the CLI
entry point, the verdict axis, and the consumer documentation, as recorded in
the build's frontmatter.

## 3. Behavior

### 3.1 The declaration

A waiver is still a line beginning, after leading whitespace, with the
configured `[coupling] waiver_keyword` (default `Spec-Drift-Waiver:`) and a
non-empty reason. It MAY be followed by lifecycle lines, each naming the key
that the keyword spells without its trailing colon, a suffix, and a colon:

```
Spec-Drift-Waiver: mechanical version bump
Spec-Drift-Waiver-Paths: npm/package.json, py/pyproject.toml
Spec-Drift-Waiver-Until: 2026-12-31
Spec-Drift-Waiver-Since: a3d5213d
Spec-Drift-Waiver-Max-Uses: 1
```

- A lifecycle line belongs to the nearest waiver line above it. A body may
  declare several waivers; each lifecycle line narrows only its own.
- `-Paths:` is a comma-separated list of repo-relative paths; an entry ending
  in `/` names a subtree, and an entry matches the way a claim does. Repeated
  `-Paths:` lines add to one list.
- `-Until:` is a date, `YYYY-MM-DD`, inclusive.
- `-Since:` is a commit, 4 to 40 hexadecimal characters.
- `-Max-Uses:` is a positive integer.
- `-Until:`, `-Since:` and `-Max-Uses:` may each appear once per waiver.

Separate lines, not one structured line (the draft's open question 1): each is
readable in a pull request body, and a human types them. A malformed value, or
a repeated single-valued key, is not a parse error of the run: it is a failed
check on that waiver (§3.5), named with what was wrong, because the gate
reached a verdict and the defect is in one declaration.

A lifecycle line with no waiver line above it narrows nothing. It is reported
in the verdict as unattached, so a key written before its waiver is visible
rather than silently ignored.

### 3.2 A waiver may declare its scope

A waiver with a `-Paths:` line clears **only** violations whose path matches
one of its entries. A violation it does not cover is not cleared by it.

A waiver with no `-Paths:` line keeps today's behavior and clears every
violation, because changing the default would silently re-refuse work that
passes now. The verdict MUST report it as unscoped, so "this cleared more
than you meant" is visible rather than inferred. Whether an unscoped waiver
should eventually warn (the draft's open question 2) is left undecided: it
would change what `lint --fail-on-warn` refuses, which is not free.

### 3.3 Declared checks are evaluated against supplied inputs

| Check | Declared by | Input the caller supplies | Satisfied when |
|---|---|---|---|
| `expiry` | `-Until:` | an as-of date, `YYYY-MM-DD` | as-of is on or before the declared date |
| `ancestry` | `-Since:` | for that commit, whether it is an ancestor of the head under judgment | the answer is true |
| `uses` | `-Max-Uses:` | for that waiver's `id` (§3.6), how many runs it has already cleared, excluding this one | the count is below the maximum |

The library never computes an input. The CLI supplies them as follows:

- the as-of date only from `--waiver-as-of <YYYY-MM-DD>`, never from the
  clock: an expiry the operator did not date is not evaluated;
- ancestry by asking `git merge-base --is-ancestor <since> <head>` for each
  declared `-Since:`, since the CLI already reads the range from git: exit 0
  is true, exit 1 is false, and any other answer (an unknown commit, a shallow
  clone) supplies nothing;
- a use count only from `--waiver-uses <ID>=<N>`, repeatable.

The facade takes the same three inputs as data.

### 3.4 A missing input is not evaluated, never satisfied

Each declared check is reported as `satisfied`, `failed` or `not-evaluated`.
A check whose input was not supplied is `not-evaluated`, and MUST NOT be
reported or treated as satisfied. An absent use count is `not-evaluated`, and
MUST NOT be treated as zero.

`not-evaluated` does not refuse and does not fail the waiver: the waiver
clears on the checks that were evaluated, and the verdict says which were not.
A caller that wants the guarantee supplies the input. A gate that silently
treated an unevaluated expiry as satisfied would have an expiry mechanism that
does nothing, which is worse than none because it reads as protection.

### 3.5 A failed check clears nothing

A waiver with any `failed` check (an expiry before the as-of date, an ancestry
answered false, a use count at or above the maximum, or a malformed or
repeated declaration) MUST NOT clear anything. The run refuses as it would
have without that waiver, unless another waiver clears the violation, and the
verdict names each failed check with its declared value, its input and why.

### 3.6 The verdict says which waiver cleared what

Each violation is cleared by at most one waiver: the first, in declaration
order, whose checks did not fail and whose scope covers the violation's path.
The run blocks when any violation is not cleared.

The couple report (the `couple --json` envelope's `report`, and
`couple_json`'s answer) gains `waivers`, one entry per declared waiver in
declaration order, carrying:

- `id`: `sha256:` over the declaration (reason, sorted paths, until, since,
  max-uses), the key a caller counts uses against;
- `reason`, and `scoped` with `paths` when scoped;
- `checks`: each declared check with `check`, `declared`, `input` when one was
  supplied, `outcome`, and `detail` when failed;
- `effective`: whether no check failed;
- `clears`: each violation it cleared, by `code` and `path`.

It gains `unattachedWaiverLines` when §3.1 found any. The existing `waiver`
member keeps its meaning, "this run was waived": it carries the first
effective waiver's reason when every violation is cleared, and is absent
otherwise.

### 3.7 What the engine does not do

No clock is read and no `git` runs in the library. Nothing is written: no use
count, no record that a waiver was used, no mark that it is spent. A use limit
evaluated here is only as good as the count the caller supplied, and two
concurrent runs given the same count both see the same answer; atomic
consumption is the caller's to build if it needs it (§1.2).

A waiver remains a human instrument. It needs explicit human approval, and an
agent never writes one on its own authority (`AGENTS.md`, "Adversarial prompt
refusal"). Scope, expiry, ancestry and a use limit make a human's waiver
narrower; they do not make an agent's waiver permissible, and a verdict that
reports a waiver `effective` says nothing about whether it was authorized.

### 3.8 Compatibility

- A run with no waiver declared produces the same report bytes it produced
  before this spec (`couple_json`'s answer, and the envelope's `report`):
  `waivers` and `unattachedWaiverLines` are omitted when empty. The envelope's
  `schemaVersion` is the one member that moves, as spec 100's did.
- A run whose one waiver declares no lifecycle line decides exactly as before
  (same exit code, same `violations`, same `waiver`) and its report gains the
  `waivers` block reporting it unscoped, which §3.2 requires.
- The dependency-only auto-waiver (specs 005, 027) is reported the same way,
  as one unscoped waiver.
- `parse_waiver`, the `Waiver` type and every existing `couple*` signature are
  unchanged; the new behavior is reached through new functions and optional
  request members.
- The verdict envelope's `schemaVersion` moves `0.5.0` to `0.6.0`, an additive
  MINOR.

## 4. Out of scope

- **Any state written by the engine, a clock, atomic consumption.** §1.2.
- **Changing the default.** §3.2: an unscoped waiver behaves as today.
- **Approval.** §3.7.
- **Naming obligations a waiver waives.** Note 09 §10.4 suggested a waiver cite
  obligation ids (spec 106). Nothing here needs it, and a key for it can be
  added the way §3.1's keys are.

## 5. Resolved decisions

**D-1 (2026-09-23, filed draft corrected before the build).** The draft's last
acceptance line required a run "with no lifecycle fields declared" to produce
a byte-identical verdict, while its §3.1 required an unscoped waiver to be
reported as unscoped in the verdict. Both cannot hold for a run with a plain
waiver. Kept: §3.2's reporting, which is the point of the spec. Byte identity
is kept for every run that declares no waiver (§3.8), which is almost every
run, and a run with a plain waiver keeps its decision exactly. The draft's
open question 1 is answered with separate lines (§3.1); question 2 stays open
(§3.2). Four points the draft left unstated are fixed: how several waivers and
several violations pair (§3.6), what a use count counts (prior clearing runs,
excluding this one), how a malformed declaration is treated (a failed check,
§3.5), and where the CLI's inputs come from (§3.3: an explicit flag for the
date and the count, git for ancestry, never the clock).

**D-2 (2026-09-23, build).** Decisions the contract left to the build:

- *The payload, not the envelope, is what stays byte-identical.* The draft and
  D-1 said "verdict"; the envelope's `schemaVersion` moves to `0.6.0` on every
  run, so the invariant is stated over the report (`couple_json`'s answer and
  the envelope's `report`), which is what spec 100 kept for its own MINOR.
  I-2 and §3.8 say so.
- *The as-of date is validated by the pure core too*, not only by the
  freshness-guarded entry: a caller passing `23/09/2026` straight to
  `couple_with_prior_waived` gets the usage error (exit 3), rather than a
  string comparison that would decide an expiry on character order.
- *The CLI's ancestry question* passes `--end-of-options` before its operands,
  as `merge_base` does, and asks only for a `-Since:` shaped like a commit, so
  nothing a pull request body says reaches git as an option.
- *The waiver `id`* is `sha256:` over named pieces with the corpus's one hash
  construction, so the order lines were written in and repeated `-Paths:`
  lines do not move it and every declared value does. Scoped-to-nothing and
  unscoped have different ids.
- *Prose.* One plain waiver renders exactly the lines it rendered before;
  anything more adds a block per waiver (its `id`, which `--waiver-uses`
  takes, its scope, `effective` or `REFUSED`, what it cleared, each check) and
  one line per unattached lifecycle line. A refusal counts and lists only the
  violations no waiver cleared, and its resolution footer is computed over
  those.

**D-3 (2026-09-23, build: fail-first by mutation).** The new test files call
functions that do not exist before this build, so run against the prior tree
they fail to compile, which proves nothing. Measured instead against two
mutations of `waiver.rs`: with every lifecycle check discarded, 9 of 22
library cases and 4 of 9 CLI cases fail; with scope ignored (every effective
waiver clears every violation), 4 and 2 fail. The rest assert compatibility
and parsing that neither mutation touches.

## Acceptance

The build MUST establish, behaviorally, with the negative cases that matter:

- a scoped waiver clears a violation on a listed path and **does not clear**
  one on an unlisted path in the same run, which refuses;
- an unscoped waiver clears everything, exactly as before, and the verdict
  records it as unscoped;
- an expiry with an as-of on or before it clears; with an as-of after it
  **refuses**, and the verdict names the expiry;
- an expiry with **no** as-of reports `not-evaluated` and clears on the other
  checks, asserted by reading the verdict and not the exit code, because the
  exit code is the same when satisfied and when not evaluated and a test that
  only read it would pass on a mechanism that did nothing;
- a declared ancestry answered false **refuses**; through the CLI, a `-Since:`
  that is an ancestor of head clears and one that is not refuses;
- a use count at the declared maximum **refuses**; below it clears; an absent
  count is `not-evaluated` and is not treated as zero;
- a malformed value and a repeated single-valued key each fail their waiver;
- two waivers and two violations pair by scope, not by position: the first
  waiver's scope covers the second violation and the second waiver's the
  first;
- an unattached lifecycle line is reported and narrows nothing;
- a run with no waiver produces the same report bytes as the same run before
  this change, and a run with one plain waiver decides identically;
- the facade and the CLI agree, and the envelope carries verdict `0.6.0`.

## Verification

Written to fail against the tree it is filed on: nothing named here exists.

```verify:cli
cargo build --release --locked
# 3.1 - 3.6, 3.8: the library's evaluation, pairing and compatibility.
cargo test -p spec-spine-core --test waiver --locked
# 3.3, 3.6: the shipped verb, its flags, git ancestry and the envelope.
cargo test -p spec-spine-cli --test waiver --locked
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
