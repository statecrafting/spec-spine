---
id: "116-a-deferred-contract-claims-nothing"
title: "A deferred contract claims nothing"
status: draft
kind: "tooling"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "003-conformance-lint"
summary: >
  `L-001` warns when a spec declares no ownership edge, and exempts a
  superseded, retired or retroactive one because each is making a correct
  statement by claiming nothing. A spec deliberately not scheduled
  (`implementation: deferred`) is in the same position and is not exempt, so
  the one shape the corpus contract leaves a deferred contract, no territory,
  makes `lint --fail-on-warn` red. The exemption is extended to `deferred`,
  covers the declaration and never a claim, and re-arms the moment the spec
  is scheduled.
amends:
  - "003-conformance-lint"
amends_sections: ["l-001"]
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/deferred_contract.rs" }
extends:
  - spec: "003-conformance-lint"
    paths:
      - "crates/spec-spine-core/src/lint.rs"
    nature: additive
references:
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
  - unit: { kind: file, path: "standards/spec/contract.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "A spec whose implementation is deferred does not raise L-001 for declaring no ownership edge."
    anchor: "3-1-a-deferred-spec-is-exempt-from-l-001"
  - id: "R-2"
    kind: requirement
    text: "Any implementation value other than deferred, including absent and n-a, leaves L-001 exactly as it was."
    anchor: "3-3-the-exemption-re-arms-on-scheduling"
  - id: "I-1"
    kind: invariant
    text: "A unit a deferred spec names is held to every diagnostic about that unit; the exemption covers the declaration, never a claim."
    anchor: "3-2-the-exemption-covers-the-declaration-never-a-claim"
  - id: "V-1"
    kind: verification
    text: "Exemption, re-arming for every scheduled value, n-a and absent, and a deferred claim still diagnosed, each on a scratch corpus that compares one changed field against a fixed base."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/deferred_contract.rs"
---

# 116: A deferred contract claims nothing

## 1. Purpose

The corpus contract (`standards/spec/contract.md`, "Lifecycle as scheduling")
gives `implementation: deferred` a meaning: a decision not to schedule. Design
note 09 D-7 (2026-09-22) kept that value and its meaning while changing the
reasons a spec may be deferred (note 09 §10.1).

A deferred contract has exactly one honest shape for its territory: none. It
must not claim a file nobody is writing. A speculative `establishes` on a path
that does not exist raises an unresolved-unit diagnostic that
`check --fail-on-unresolved` refuses, and `planned: true` on a file that
already exists raises `L-012`.

A spec with no ownership edge raises `L-001`, and `lint --fail-on-warn` is in
this repository's gate chain. So the lifecycle value the contract offers
cannot be used in the only shape it can honestly take without turning the gate
red. That is not hypothetical: on 2026-09-21 ten contracts were filed deferred
on `specs/deferred-contracts-2026-09-21`, and all ten raised `L-001` (D-1).

### 1.1 The exemption L-001 already has, and why this is the same case

`lint.rs` exempts a `superseded` or `retired` spec from `L-001` (spec 092
§3.11), and the comment beside it states the reason: `L-001` exists to catch
an author who wrote a spec and forgot to say what it governs, and a document
whose authority was transferred or withdrawn is making a correct statement
when it claims nothing. A retroactive-origin spec is exempt for a third
version of the same reason.

A spec marked `implementation: deferred` is making the same correct
statement. It is not an author who forgot; it is an author who recorded, in
the one field the corpus has for it, that nothing is scheduled.

### 1.2 Why the answer is not "give them planned territory"

`planned: true` (spec 063) declares territory before it is written. For a
deferred contract whose eventual territory is a file that already exists, a
planned flag raises `L-012` ("it now resolves: drop the flag"). For the rest,
a planned claim on a file nobody will write is a standing reservation of
territory by a document nobody is building, which makes the file read as that
spec's to anyone reading the frontmatter.

### 1.3 Why it is still needed after the reserved contracts were built

When this spec was first drafted, the ten contracts that tripped `L-001` were
its motivation. By 2026-09-23 six of them were built and the remaining four
are scheduled, so none is deferred on `main`. The defect is not in those ten
documents; it is that the contract's lifecycle table and the lint disagree
about a value both keep. The next deferred contract reproduces it. The
evidence therefore moves from this corpus to a fixture (D-5).

## 2. Territory

`L-001`'s exemption test in `crates/spec-spine-core/src/lint.rs` (an
`extends` of spec 003), and its acceptance in a new test file,
`crates/spec-spine-core/tests/deferred_contract.rs`.

It amends spec 003, because it changes what an approved spec says the lint
refuses. The amendment is declared in this spec's frontmatter and spec 003's
document is not edited (spec 037).

## 3. Behavior

### 3.1 A deferred spec is exempt from L-001

A spec whose `implementation` is `deferred` MUST NOT raise `L-001` for
claiming no territory.

### 3.2 The exemption covers the declaration, never a claim

Exactly as the superseded exemption does: a deferred spec that **does** name
a unit is still held to every diagnostic about that unit. `L-012`, the
unresolved-unit diagnostics and the coupling gate all see it, because the unit
is written down.

This is the sentence that keeps the exemption from becoming a way to park
claims outside the lint.

### 3.3 The exemption re-arms on scheduling

When a deferred spec's `implementation` moves to `pending`, `in-progress` or
`complete`, or the field is removed, `L-001` applies again immediately.
Nothing remembers that the spec was once deferred.

`n-a` is deliberately **not** added. `n-a` is a ratified record that owns
nothing on purpose, which is a different statement from "not scheduled".
Adding it would be a separate change with its own spec.

### 3.4 Nothing else about the lint moves

No other `L-` code changes, no tier changes, no message changes for a spec
that still raises `L-001`, and the exemption adds no configuration key. A knob
would be a way for a corpus to turn `L-001` off generally, and that is not
what this is. The exemption also does not hide the spec: a deferred spec is
still listed by `registry list` and `registry status-report`, and
`registry plan` still reports it as not schedulable.

## 4. Out of scope

- **Exempting `n-a`.** §3.3.
- **A configuration knob.** §3.4.
- **Changing what `deferred` means** anywhere else. The lifecycle table
  already says a deferred spec is not schedulable; this acts on it.
- **Any warning other than `L-001`.** A deferred spec's other diagnostics are
  unchanged, and this spec is not a way to silence them.

## 5. Resolved decisions

**D-1 (2026-09-21, first build: the fail-first evidence was that corpus).**
On `specs/deferred-contracts-2026-09-21`, before the change,
`lint --fail-on-warn` emitted exactly ten `L-001` warnings, one per deferred
contract. After it, zero. Commit `981ddb76` on that branch holds the first
build; it never reached `main`.

**D-2 (2026-09-21, the re-arming assertion compares against a fixed base).**
The re-arming test writes one spec file, asserts the exemption, then rewrites
the same file with `pending`, `in-progress` and `complete` in turn and asserts
the warning returns each time, and finally with the field absent. A test that
only asserted the exemption would pass on an implementation that suppressed
`L-001` unconditionally.

**D-3 (2026-09-21, the claim half is asserted through the index).** A
deferred spec claiming a nonexistent path does not raise `L-008` (that code is
about content hashes); the index's unresolved-unit diagnostic is the mechanism
that holds such a claim, so that is what the test asserts. The build asserts
the specific code and the path, not a prefix match (D-6).

**D-4 (2026-09-21, recorded then, since resolved elsewhere).** The first build
found `make gate` resolving `spec-spine` on `PATH`. That was a defect in a
file spec 092 owns and was not fixed here; spec 117 later fixed binary
selection. Nothing in this spec depends on it.

**D-5 (2026-09-23, reconciliation: the evidence is a fixture).** No spec on
`main` is deferred (§1.3), so the corpus can no longer serve as the negative
control. The acceptance lives in a new test file so that its verification
block fails on a tree without the build (the test target does not exist),
and every case runs on a scratch corpus.

**D-6 (2026-09-23, reconciliation: the first build's claim assertion was too
loose).** It accepted any code beginning `W-` or `I-`, which a regression
emitting an unrelated diagnostic would satisfy. The build asserts the
unresolved-unit code the index emits for that claim and the path it names.

**D-7 (2026-09-23, build: fail-first and mutation evidence).** The five
cases in `tests/deferred_contract.rs` were run before the exemption was
written: the exemption, re-arming and "no other diagnostic moves" cases
failed (3 of 5), and the two guard cases (`n-a`, and a deferred claim still
diagnosed) passed, as they must on a lint that exempts nothing. With the
exemption in place all five pass. Two mutants were then run against the
suite: one that stops `L-001` firing at all fails the re-arming, `n-a` and
"no other diagnostic" cases; one that also exempts `n-a` fails the `n-a`
case. The re-arming loop covers `pending`, `in-progress`, `complete` and an
absent field, and then returns to `deferred`.

## Verification

Written to fail against the tree it is filed on: the test target does not
exist.

```verify:cli
# 3.1 - 3.3: exemption, re-arming for pending, in-progress, complete and
# absent, n-a unchanged, and a deferred claim still diagnosed.
cargo test -p spec-spine-core --test deferred_contract --locked
# 3.4: the rest of the gate is unmoved on this corpus.
cargo build --release --locked -p spec-spine-cli
./target/release/spec-spine lint --fail-on-warn
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
