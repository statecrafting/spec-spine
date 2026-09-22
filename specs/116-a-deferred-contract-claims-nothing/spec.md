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
  statement by claiming nothing. A spec deliberately not scheduled is in the
  same position and is not exempt, so filing a contract before there is a
  consumer for it makes `lint --fail-on-warn` red. The exemption is extended
  to `implementation: deferred` and re-arms the moment the spec is scheduled.
amends:
  - "003-conformance-lint"
amends_sections: ["l-001"]
extends:
  - spec: "003-conformance-lint"
    paths:
      - "crates/spec-spine-core/src/lint.rs"
      - "crates/spec-spine-core/tests/lint.rs"
    nature: additive
references:
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 116: A deferred contract claims nothing

## 1. Purpose

The owner adopted a rule on 2026-09-21 (design note 09 §5, §6 D-1):

> Specify now, implement only for a named consumer need.

Acting on it means filing contracts that are complete and deliberately not
scheduled. Ten were filed the same day, specs 106 to 115. Every one of them
claims no territory, because a contract nobody is building must not hold a
claim on files nobody is writing, and a speculative `establishes` raises an
unresolved-unit diagnostic that `check --fail-on-unresolved` refuses.

All ten then raise `L-001`, and `lint --fail-on-warn` is in this repository's
gate chain. Following the adopted rule turns the gate red.

### 1.1 The exemption L-001 already has, and why this is the same case

`lint.rs` exempts a `superseded` or `retired` spec from `L-001`, and the
comment beside it states the reason:

> `L-001` exists to catch an author who wrote a spec and forgot to say what it
> governs; a document whose authority has been transferred by an explicit
> `supersedes` edge, or withdrawn, is making a correct statement when it claims
> nothing.

A spec marked `implementation: deferred` is making the same correct statement.
It is not an author who forgot; it is an author who wrote down a contract and
recorded, in the one field the corpus has for it, that nothing is scheduled.

A retroactive-origin spec is exempt for a third version of the same reason.
This adds the fourth case that was always in the set and was not in the code,
because until the rule was adopted the case did not arise.

### 1.2 Why the answer is not "give them planned territory"

`planned: true` (spec 063) is the corpus's way to declare territory before it
is written, and it is the wrong instrument here for two reasons.

For most of the ten, the eventual territory is a file that **already exists**:
a frontmatter grammar change lands in `frontmatter.rs`, a lint in `lint.rs`.
Marking an existing file `planned` raises `L-012` ("it now resolves: drop the
flag"), which is a warning, so the gate is red either way.

And for the rest, a planned claim on a file nobody will write this quarter is
a standing reservation of territory by a document with no consumer. It makes
the file look like that spec's when somebody reads the frontmatter, which is
the same objection specs 103 and 104 recorded when they dropped claims that
bought nothing.

## 2. Territory

`L-001`'s exemption test in `crates/spec-spine-core/src/lint.rs`, and its
acceptance in `crates/spec-spine-core/tests/lint.rs`.

It amends spec 003, because it changes what an approved spec says the lint
refuses. The amendment is declared in this spec's frontmatter and spec 003's
document is not edited (spec 037).

## 3. Behavior

### 3.1 A deferred spec is exempt from L-001

A spec whose `implementation` is `deferred` MUST NOT raise `L-001` for
claiming no territory.

Every other `L-001` input is unchanged: a spec with any `status` and any other
`implementation` value, including absent, still raises it when it declares no
ownership edge.

### 3.2 The exemption covers the declaration, never a claim

Exactly as the superseded exemption does: a deferred spec that **does** name a
unit is still held to every diagnostic about that unit. `L-012`, the
unresolved-unit diagnostics, and the coupling gate all see it, because the
unit is written down.

This is the sentence that keeps the exemption from becoming a way to park
claims outside the lint.

### 3.3 The exemption re-arms on scheduling

When a deferred spec's `implementation` moves to `pending`, `in-progress` or
`complete`, `L-001` applies again immediately. Nothing remembers that the spec
was once deferred.

That is what makes this a ratchet rather than a hole: the moment somebody
schedules one of these contracts, the corpus is back to asking what it
governs, which is the question `L-001` exists to ask.

`n-a` is deliberately **not** added to the exemption. `n-a` means a ratified
spec that owns nothing on purpose, which is a different statement from "not
scheduled", and nothing measured asks for it. Adding it later is a MINOR
change to a lint's behavior with its own spec.

### 3.4 Nothing else about the lint moves

No other `L-` code changes, no tier changes, no message changes for a spec
that still raises `L-001`, and the exemption adds no configuration key. A knob
would be a way for a corpus to turn `L-001` off generally, and that is not
what this is.

## 4. Out of scope

- **Exempting `n-a`.** §3.3.
- **A configuration knob.** §3.4.
- **Changing what `deferred` means** anywhere else. The lifecycle table in
  `standards/spec/contract.md` already says a `deferred` spec is not
  schedulable; this adds no new meaning, it acts on the existing one.
- **The ten contracts themselves.** They are specs 106 to 115 and each is its
  own document.

## 5. Resolved decisions

**D-1 (2026-09-21, build: the fail-first evidence is this corpus).** Before
this change `lint --fail-on-warn` on this branch emitted exactly ten `L-001`
warnings, one per deferred contract, and the gate's lint step was red. After
it, zero. The negative control is not a fixture; it is the ten documents the
adopted rule produced.

**D-2 (2026-09-21, build: the re-arming assertion compares against a fixed
base).** `scheduling_a_deferred_spec_re_arms_l001` writes one spec file, asserts
the exemption, then rewrites the same file with `pending`, `in-progress` and
`complete` in turn and asserts the warning returns each time, and finally with
the field absent. A test that only asserted the exemption would pass on an
implementation that suppressed `L-001` unconditionally, which is the hole this
spec spends its §3.3 promising not to be.

**D-3 (2026-09-21, build: the claim half is asserted through the index, not
the lint).** §3.2 says the exemption covers the declaration and never a claim.
The obvious assertion was that a deferred spec claiming a nonexistent path
still raises `L-008`; it does not, because `L-008` is about a claimed path
being inside a content hash and the default configuration hashes no `src/`
path either way. The unresolved-unit diagnostic the index records is the
mechanism that actually holds such a claim, so that is what the test asserts.
The first version of the test was wrong about which code fires and would have
passed for the wrong reason had the assertion been looser.

**D-4 (2026-09-21, build: `make gate` was reading a stale binary, and the
Makefile documents an order it does not implement).** Found while gating this
change: `make gate` reported ten `L-001` warnings after the exemption was
built and the in-tree binary reported none. `SPEC_SPINE ?= spec-spine`
resolves on `PATH`, which on this machine is `~/.cargo/bin/spec-spine 0.20.0`.
The Makefile's own header says the resolution is "`$SPEC_SPINE`, then
`./target/release`, then ...", which is not what the assignment does. That is
a defect in a file spec 092 owns and is **not fixed here**: it is outside this
spec's territory, and fixing it under cover of a lint change would be the
mid-build scope creep the corpus refuses. It is reported as an open
contradiction. The correct invocation meanwhile is
`make gate SPEC_SPINE=./target/release/spec-spine`.

## Verification

Behavioral, and the load-bearing assertion is the negative one: an exemption
that could not be re-armed would be a hole rather than a ratchet, so §3.3 is
asserted by flipping a deferred spec and watching the warning come back.

Written to fail against the tree this spec is filed on: the ten deferred
contracts raise ten `L-001` warnings today and the gate's lint step is red.

```verify:cli
# 3.1 and 3.3: the unit tests, which build a corpus, flip one field and
# compare the diagnostics. The named cases are:
#   - a_deferred_spec_with_no_territory_is_exempt_from_l001
#   - scheduling_a_deferred_spec_re_arms_l001
#   - a_deferred_spec_that_names_a_unit_is_still_diagnosed
#   - an_n_a_spec_with_no_territory_still_raises_l001
cargo test -p spec-spine-core --test lint --locked
# 3.1 end to end: this corpus carries ten deferred contracts that claim
# nothing, and the lint is clean. Before this build it emitted ten warnings.
./target/release/spec-spine lint --fail-on-warn
# 3.4: the rest of the gate is unmoved.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine index coverage --fail-on-untraced
```
