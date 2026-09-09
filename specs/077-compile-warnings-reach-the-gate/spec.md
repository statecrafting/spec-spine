---
id: "077-compile-warnings-reach-the-gate"
title: "Compile warnings reach the gate"
status: draft
kind: "tooling"
created: "2026-09-08"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "001-compile-registry"
  - "003-conformance-lint"
  - "031-registry-freshness-check"
  - "037-machine-readable-verdicts"
  - "050-index-diagnostics-reach-a-gate"
  - "064-the-kit-ships-the-composite-gate"
  - "075-one-name-one-freshness-verb"
extends:
  # 3.2 the flag, the severity gate that reads it, and the facade.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/tests/compile.rs", nature: additive }
  # 3.3 the forward into the compile half: the only form CI ever calls.
  - { spec: "075-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: additive }
  # 3.4 the registry half carries its own warning tally, beside the index half's.
  - { spec: "050-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-core/src/diagnostics.rs", nature: additive }
  # 3.5 the gate chain, in every place it is written down.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/Makefile", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/govern.yml", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: ".github/workflows/ci.yml", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
  # 3.5 the protocol. The root-AGENTS.md attribution is inherited, not endorsed:
  # see the 2026-09-08 decision in section 5.
  - { spec: "029-claude-code-skill-kit", unit: "AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/AGENTS.md", nature: additive }
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: ".claude/skills/", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/skills/", nature: additive }
  - { spec: "051-harness-runs-the-verbs-it-ships", unit: "crates/spec-spine-core/tests/kit_skills.rs", nature: additive }
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: additive }
  - { spec: "067-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
summary: >
  `V-010`, a `depends_on` naming a spec that does not exist, is the only
  warning-tier code the compiler emits, and no verb in the gate chain can be
  told to refuse it. `lint --fail-on-warn` cannot reach it: lint's codes are
  disjoint from compile's by design, and `lint()` keeps only the registry from
  the compile it runs. So a dangling dependency compiles clean, lints clean at
  every severity, and reaches the default branch unread. This spec adds
  `--fail-on-warn` to `compile` and forwards it through `check` into the compile
  half, the shape spec 050's `--fail-on-unresolved` already uses for the index
  half. The flag gates the CLI's exit code only; `validation.passed` keeps the
  meaning spec 001 fixed for it, and `V-010` keeps its warning tier, because a
  corpus that files specs forward must be able to name a dependency it has not
  written yet. Zero `V-010`s exist today: the gap is structural, and this spec
  closes it before it costs something.
---

# 077: Compile warnings reach the gate

## 1. Purpose

The compiler emits seventeen validation codes. Sixteen are error tier, and an
error fails the build at the first step of the gate chain. One, `V-010`, is a
warning, and nothing in the chain can be told to refuse it.

Before this spec, a corpus with a dangling `depends_on` reports:

```
spec-spine compile                             -> compiled 77 spec(s), 1 warning(s)
spec-spine check --fail-on-unresolved          -> 0
spec-spine lint --fail-on-warn --fail-on-info  -> 0 error(s), 0 warning(s), 0 info
spec-spine couple                              -> 0
```

Every step is green and the edge is broken. The warning is printed by the one
verb whose output nobody reads twice, and then it is gone.

**`lint --fail-on-warn` is not the missing gate, and cannot become it.** Spec
003 makes the lint's `L-` codes "disjoint from compile's structural `V-` codes",
and the implementation is faithful to that: `lint()` calls `compile()` and binds
only `.registry`, discarding every violation the compile produced. That is the
correct architecture, not an oversight. The two families answer different
questions, and merging them would put corpus conventions and structural validity
behind one flag. The consequence is simply that the `--fail-on-warn` a reader
already knows is the wrong flag for this, and there is no right one.

**The gap is one code wide and it is worth closing anyway.** `V-010` is the
whole warning tier, so a reader could reasonably ask why a flag is needed for a
single diagnostic that nobody has tripped. Two reasons. The first is that the
tier is a shape, not a count: the next warning-tier code, and spec 033 shows how
readily one is proposed, inherits the same invisibility, and it will inherit it
silently. The second is that a broken `depends_on` is precisely the defect that
surfaces late and elsewhere, in a consumer ordering work by a graph it does not
own, against a corpus whose author saw a green gate. Spec 033 wrote that
sentence about cycles and then fixed cycles at error tier. The dangling case was
left where it was, correctly, and left unreachable, which was not a decision
anyone made.

## 2. Territory

This spec adds no file. Every unit it touches belongs to another spec and is
reached by an `extends` edge, which is why the frontmatter has no `establishes`
list. The conformance lint's `L-001` is satisfied by the `extends` edges, which
`has_ownership_edge` counts.

**The amends audit, performed against the specs' words rather than the size of
the change.** Four specs looked like candidates and none is owed an amendment:

- **001** states, normatively, that "`validation.passed` is false iff any
  error-tier violation is present." A `--fail-on-warn` that flipped that field
  would contradict it outright. Section 3.2 therefore gates the **CLI exit
  code** and leaves the field alone. That is not a workaround; it is the
  arrangement spec 003 already uses, whose implementation says severity gating
  "is applied by the CLI; this layer just produces the diagnostics." Copying a
  shape the corpus already sanctions means no stated behavior changes.
- **003** owns the lint and its `--fail-on-warn`. This spec neither changes that
  flag nor makes lint aware of `V-` codes. It borrows the pattern; it does not
  touch the code.
- **075** requires `check` to accept `--fail-on-unresolved` and forward it to
  the index half. A second forwarded flag is additive to that requirement.
  Its section 3.3 pins the exit-code fold `3` over `1` over `2` over `0`; a
  refused warning is a `1`, which lands **inside** that order rather than
  altering it, so the fold is unchanged and no amendment is owed for it either.
- **031** makes the registry stale report's structure contractual. Section 3.4
  adds a tally beside that report without reshaping it.

**The `extends` edge naming root `AGENTS.md` is inherited and is known to be
wrong.** Spec 029 establishes exactly one unit, `kit/`. Root `AGENTS.md` is
outside that subtree and no spec in the corpus establishes it. Seven approved
specs already extend 029 for it. This spec matches them rather than
unilaterally disagreeing, and records why in section 5 so the correction is a
decision someone makes rather than a discrepancy someone finds. `kit/AGENTS.md`
is a different matter and is correctly attributed: it falls inside 029's `kit/`
subtree.

## 3. Behavior

### 3.1 `V-010` keeps its warning tier

The tier MUST NOT change. Escalation is the caller's decision, taken with a
flag, and never the compiler's.

This is load-bearing rather than conservative. This repository, and the
`specify-first` workflow the docs describe, file specs forward: a spec is
written, and may name a `depends_on` target that is filed after it. Spec 016's
short-id resolution and spec 053's ordinal-monotonicity lint both assume that
authoring order. At error tier, `compile` would refuse the corpus the moment an
author wrote the edge, and `/spec` could not file anything that pointed at work
not yet filed. The warning tier is what makes the forward-filing workflow
possible, and the defect was never the tier.

### 3.2 `compile --fail-on-warn` gates the exit code, not the verdict

`compile` MUST accept `--fail-on-warn`. With it, the CLI MUST exit `1` when the
compile produced any warning-tier violation, and MUST otherwise exit as it does
today.

`1` is the correct code and not a new meaning for it: the exit-code contract
already spends `1` on "validation failure", and `lint --fail-on-warn` and
`check --fail-on-unresolved` both already refuse with it.

The flag MUST NOT change `validation.passed`, the registry field, or any
emitted byte. A run with the flag and a run without it MUST write identical
shards. What the flag changes is the exit code and nothing else, which keeps
spec 001 section 3.2 true as written and keeps the determinism claim untouched.

The flag MUST be accepted on every form of the verb: the writing form,
`--check`, and spec 056's `--spec`. Spec 037 section 4 withholds `--json` from
the writing form because that form's verdict is deliberately not
machine-readable; that restriction is about what is **written**, and an exit
code is not written output. A caller who regenerates and wants the regeneration
refused on a warning is asking a coherent question, and the local gate chain
runs the writing form.

The library SHOULD expose the warning count on the compile outcome rather than
requiring each caller to re-filter violations by severity, so the CLI, the
facade and `check` read one number.

### 3.3 `check --fail-on-warn` forwards into the compile half

`check` MUST accept `--fail-on-warn` and forward it to its compile half, exactly
as it forwards `--fail-on-unresolved` to its index half. Refusal MUST exit `1`,
composed through the existing fold rather than beside it.

**This section is the one that closes the gap.** CI does not run `compile`. The
`self_governance` job runs `check --fail-on-unresolved` in place of `compile`
and `index`, because a gate must never repair the tree it is judging, and spec
075 moved every consumer onto the composed verb precisely so the protocol would
stop naming the primitives. A `--fail-on-warn` that existed only on `compile`
would therefore be unreachable from the only chain that runs on a pull request,
and this spec would ship a flag that closes nothing.

The two flags MUST remain independent. `--fail-on-warn` governs the compile
half's warning tier; `--fail-on-unresolved` governs the index half's
unresolved-unit diagnostics. Neither implies the other, and a caller MUST be
able to pass either alone.

### 3.4 The composed report says which half refused

The registry half of `check`'s report MUST carry its warning tally, so the verb
can decide without recompiling and can say why it refused.

Today the composed verb discards violations entirely and tells the reader, in
its own output, to "run `spec-spine compile --check` for the violations". That
is acceptable for a verdict that only reports freshness. It is not acceptable
for one that can now **refuse** on a warning: an exit `1` a reader cannot
attribute to a tree, let alone to a code, sends them to the wrong verb.

The report line MUST name the count and the tree. Naming each individual
violation on the composed verb is not required; the pointer to
`compile --check` remains correct for the detail, and section 3.3's forwarding
does not change what that primitive prints.

Per spec 037 the tally MUST appear in the `--json` envelope's `report`, and the
flag MUST change what is written and never what is decided: the exit code MUST
be identical with and without `--json`.

### 3.5 Every written form of the gate chain adopts the flag

The gate chain is written down in seven places, and they MUST agree: `AGENTS.md`
and `kit/AGENTS.md`, `kit/Makefile`, `kit/govern.yml`, this repository's
`.github/workflows/ci.yml`, and the inlined floors in both skill trees.

The chain's freshness step MUST become
`spec-spine check --fail-on-unresolved --fail-on-warn`.

Spec 051's subset assertion MUST continue to hold: `kit_skills.rs` asserts each
skill's inlined floor is a subset of `AGENTS.md`'s list, so the list and every
skill that inlines it change together or that test goes red. Spec 064's
`kit_gate.rs` walks `kit/Makefile`'s gate against `AGENTS.md`'s list in order,
so the step MUST keep its position in the sequence; only its flags change.

The generated `kit_embedded.rs` MUST be regenerated by
`scripts/gen-kit-embedded.py`, never hand-edited, and `tests/scaffold.rs` MUST
continue to assert the embedded copy and `kit/` agree.

`docs/adoption-guide.md` MUST document the flag on both verbs, since an adopter
whose corpus is not forward-filed may reasonably want it on from day one.

### 3.6 The refusal is proven on a corpus that trips it

A test MUST compile a fixture corpus containing a dangling `depends_on` and
assert three things: that the run without the flag exits `0`, that the run with
it exits `1`, and that the shards written are byte-identical in both cases.

The third assertion is the one that matters and the one a reader would skip. It
is what makes section 3.2's "changes the exit code and nothing else" an
enforced claim rather than a stated intention.

## 4. Out of scope

**Re-tiering `V-010` to error.** Refused in 3.1, and named here so it is not
proposed as the simpler fix later. It is simpler, and it breaks forward filing,
which is how this repository and the `specify-first` workflow both operate.

**`compile --fail-on-info`.** The compiler emits no info-tier code. Adding the
flag would ship a knob that governs the empty set, and the day an info-tier `V-`
code exists is the day to decide what it means to refuse one.

**Teaching `lint` to see `V-` codes.** The disjointness is spec 003's stated
design, the families answer different questions, and merging them would put
corpus conventions and structural validity behind one flag. A reader who wants
both refused runs both verbs, which the gate chain already does.

**Turning the flag on for adopters by default.** The chain the kit ships turns
it on because this corpus wants it on. `compile` without flags keeps today's
behavior exactly, so an adopter mid-migration, with a corpus that is legitimately
forward-filed and noisy, is not broken by upgrading.

**A phantom-`extends` check.** Section 2 documents that no validation confirms an
`extends` unit appears in the target spec's territory, which is how root
`AGENTS.md` came to be attributed to a spec that never claimed it. That is the
same class of leak spec 034 closed for `references`, now on `extends`, and it is
a corpus-wide validation change with its own blast radius. It belongs in its own
spec, filed against its own evidence, not smuggled in behind a CLI flag.

## 5. Resolved decisions

**2026-09-08: the flag gates the exit code, and `validation.passed` is left
alone.** The first shape considered was making `--fail-on-warn` promote warnings
to errors, which is the shorter implementation and reads naturally. It
contradicts spec 001 section 3.2 in one sentence, and it would make a registry
shard's contents depend on a CLI flag, which the determinism claim forbids.
Gating at the CLI is what spec 003 already does for `L-` codes, so the pattern
was available and did not need inventing.

**2026-09-08: the flag must reach `check`, or the spec ships nothing.** The
finding this spec came from named `compile --fail-on-warn` alone. Reading
`ci.yml` showed the `self_governance` job runs `check` and never `compile`, and
spec 075 deliberately moved every consumer off the primitives, so a flag on
`compile` alone would be unreachable from the only chain that runs on a pull
request. The scope grew by one section as a result, and the spec would have been
decorative without it.

**2026-09-08: `1`, not a new exit code.** A distinct code for "refused on a
warning" was considered and dropped. The exit codes are a stable contract mapped
in one place, `1` already means validation failure, and both existing
`--fail-on-X` flags already refuse with it. Section 3.4's report line is what
disambiguates, which is the same answer spec 075 reached when `1` became
reachable from two conditions on `check`.

**2026-09-08: the root `AGENTS.md` edge is inherited knowingly, not
endorsed.** Spec 029 establishes only `kit/`, and root `AGENTS.md` sits outside
that subtree with no establisher anywhere in the corpus, yet specs 047, 048,
051, 063, 072, 074 and 075 all extend 029 for it. Nothing validates the claim:
`V-017` checks an `extends` unit only against **planned** units, and no check
confirms the unit appears in the target's territory, so the misattribution
survived seven ratifications without ever failing a gate. Correcting it here
would mean either amending approved spec 029 to establish a file it does not
mention, or filing a harness spec that establishes it. Both are human decisions
and neither is this spec's to take, and a single spec disagreeing with seven
siblings would be worse than the consistent error. The edge is therefore
written as the precedent has it, with this entry as the record. The cost today
is accuracy only: coverage ignores markdown, and the coupling gate reads the
file as claimed either way.

**2026-09-08: zero occurrences is the right time to file this.** The corpus has
no `V-010` today, so the flag will refuse nothing on the day it lands. That is
the argument for it rather than against it: the gate is being closed while the
change is a flag and a test, instead of during the pull request where the first
dangling edge is discovered by a consumer that cannot order its work.

## Verification

Each line below is one command: spec 049 section 3.2 makes a fence's body line a
command, so no line may depend on a variable another line set.

Every assertion that can fail first does, and the ones that cannot are named
here rather than left to look stronger than they are.

`cargo test` with a filter matching nothing exits `0`, so a block built from
name filters would pass on an unbuilt spec and assert nothing. The test lines
below therefore name whole files: they run today's suites, which pass, and carry
this spec's new cases once those are written.

The lines that do fail against pre-077 code are the two flag invocations, which
exit `3` under spec 063's usage-error mapping, and the five greps. Each grep
matches the **composed** flag string, never the bare `--fail-on-warn`: `lint
--fail-on-warn` is already in `kit/Makefile`, `kit/govern.yml` and `ci.yml`, so
a grep for the bare form would pass today and prove nothing about this spec.
The two skill trees have no grep line; section 3.5 routes them through spec
051's subset assertion in `kit_skills.rs`, which is a test line and therefore
one of the ones that does not fail first.

```verify:cli
cargo build --release --locked
# 3.2 the flag exists on the primitive, in its writing and checking forms.
./target/release/spec-spine compile --check --fail-on-warn
# 3.3 and it is reachable from the composed verb, which is the form CI runs.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
# 3.2, 3.4, 3.6 the gating, the tally, and the byte-identical-shards assertion.
cargo test -p spec-spine-core --test compile --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.5 spec 051's subset assertion still holds after the gate list changed.
cargo test -p spec-spine-core --test kit_skills --locked
# 3.5 spec 064's in-order walk of kit/Makefile against the AGENTS.md list.
cargo test -p spec-spine-core --test kit_gate --locked
# 3.5 065's generator keeps the embedded copy in step with kit/.
cargo test -p spec-spine-core --test scaffold --locked
# 3.5 every written form of the chain names the composed flag.
grep -qF 'check --fail-on-unresolved --fail-on-warn' AGENTS.md
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/AGENTS.md
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/Makefile
grep -qF 'check --fail-on-unresolved --fail-on-warn' kit/govern.yml
grep -qF 'check --fail-on-unresolved --fail-on-warn' .github/workflows/ci.yml
```
