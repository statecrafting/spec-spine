---
id: "078-the-protocol-has-an-owner"
title: "The protocol has an owner"
status: draft
kind: "governance"
created: "2026-09-08"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "029-claude-code-skill-kit"
  - "048-kit-ships-the-governed-loop-skills"
  - "051-harness-runs-the-verbs-it-ships"
  - "057-claimed-but-unwitnessed"
  - "075-one-name-one-freshness-verb"
establishes:
  # The cross-agent protocol file, claimed by the spec whose whole subject it
  # is. Already in `[index] extra_hashed_inputs`, so the claim is witnessed by
  # a content hash and draws no `L-008` (spec 057).
  - "AGENTS.md"
summary: >
  Root `AGENTS.md` is the cross-agent authority: the protocol `/prime`
  executes, the gate list every skill inlines a subset of, and the document
  spec 051's parity test compares this repository's CI against. No spec in the
  corpus establishes it. Seven approved specs reach it with an `extends` edge
  naming spec 029, but 029 establishes exactly one unit, `kit/`, and root
  `AGENTS.md` is outside that subtree, so the ledger records an owner that
  never established the file. That is not a defect: `extends` means "adds
  surface to a predecessor", so those edges are the corpus's ordinary
  convention, and measurement confirms it. What is genuinely missing is
  narrower. The protocol file has no `establishes` claim at all, so the ledger
  has no direct answer to who owns it. This spec supplies one, and requires the
  file to name the spec that governs it, the same "a file naming its own spec"
  move the comment headers make for source. It clarifies the ledger only; it
  changes no behavior and amends nobody.
---

# 078: The protocol has an owner

## 1. Purpose

`AGENTS.md` is the most load-bearing prose file in this repository. It is the
cross-agent authority under the AAIF standard, the protocol `/prime` executes
step by step, the canonical spelling of the gate chain that `kit_skills.rs`
asserts every skill's inlined floor is a subset of, and the list spec 051's
parity test compares CI's governance commands against.

No spec establishes it.

Seven approved specs (047, 048, 051, 063, 072, 074 and 075) declare an
`extends` edge naming spec 029 with unit `AGENTS.md`. Spec 029's `establishes`
list has exactly one entry, `kit/`. Root `AGENTS.md` is not inside that
subtree; `kit/AGENTS.md` is, and that one is correctly attributed. So seven
specs record a claim about a file that their named target never claimed.

**The misattribution is invisible by construction.** `V-004` refuses an edge
naming a spec that does not exist, and `V-017` refuses an `extends` on a unit
another spec has only *planned*. Neither asks the obvious question: does the
target's territory contain the unit at all? Nothing does. The edge therefore
compiles, lints clean at every severity, passes the coupling gate and survives
ratification, seven times over.

**What it costs today is accuracy, not safety.** The coupling gate reads root
`AGENTS.md` as owned, because an `extends` edge carrying a unit puts the
extending spec into that unit's owner set, so an edit to the protocol still has
to arrive with a spec that changed. Coverage ignores it, since markdown is not
in `SOURCE_EXTS`. Nothing is unguarded. What is wrong is the ledger's answer to
"who owns this file", and a ledger whose whole product is that answer should
not be wrong about its own protocol document.

## 2. Territory

This spec establishes one unit, root `AGENTS.md`, and nothing else.

**The claim owes no `amends`, and that is a fact about spec 029 rather than a
convenience.** An amendment is owed to a spec whose stated behavior a later
spec changes. 029 states nothing about root `AGENTS.md`: it does not claim it,
require anything of it, or mention it in its territory. Claiming a file its
alleged owner never claimed contradicts nothing that spec says. This is the
rare case where the correct edge is no edge at all.

**`kit/AGENTS.md` is untouched.** It falls inside 029's `kit/` subtree and is
correctly attributed there. The two files are near-copies and it would be easy
to sweep both; only one is wrong.

**The seven existing edges are correct and are left alone.** They were first
read as a misattribution to be migrated. Measurement over the corpus refuted
that: a large minority of unit-carrying `extends` edges name a target that
never established the unit, across more than fifty specs, and `extends` is the
sole claim for about a fifth of this repository's source files. `edges.rs`
defines the edge as "adds surface to a predecessor", which is what those seven
do. There is nothing to repoint and no debt to retire.

## 3. Behavior

### 3.1 The claim

Root `AGENTS.md` MUST be established by this spec.

It MUST be claimed as a bare `file` unit. A `section` unit per heading was
considered and rejected in section 5: the protocol's headings are edited as a
document, and per-heading claims would multiply the coupling surface without
telling anyone anything they do not already know.

The unit is already covered by `[index] extra_hashed_inputs`, so the claim is
witnessed by a content hash and draws no `L-008` under spec 057. No
configuration change is required, and none is permitted by this spec: adding a
pattern to `extra_hashed_inputs` restales every shard in the ledger, and this
spec has no reason to.

### 3.2 The file names the spec that governs it

`AGENTS.md` MUST carry a line naming this spec as its owner.

This is the same move the `// Spec:` comment headers make for source files, and
it is made for the same reason: a file naming the spec that already governs it
records something that was true and never written down. A reader who opens the
protocol should not have to compile the registry to learn which spec they are
editing under.

The line MUST name the spec by path (`specs/078-the-protocol-has-an-owner/spec.md`),
so it is greppable and so a rename of the directory breaks it visibly rather
than silently. It MUST NOT be a `// Spec:` header: that grammar is defined for
the extensions in `SOURCE_EXTS`, markdown is not among them, and inventing a
markdown dialect of it would create a second claim mechanism the indexer does
not read.

### 3.3 Existing edges are not touched

This spec MUST NOT edit the frontmatter of any other spec.

An `extends` edge naming root `AGENTS.md` MAY name any spec whose surface it
adds to, as it always could. Adding an `establishes` claim does not make the
existing edges wrong, and this spec creates no obligation to repoint them: the
edge type means "adds surface to a predecessor", and a claim on the file is a
separate statement from the edges that extend it.

There is deliberately no migration, no debt entry and no follow-on. An earlier
draft of this section prescribed all three, on the belief that the existing
edges were misattributed; section 5 records why that was withdrawn.

## 4. Out of scope

**Validating that an `extends` unit appears in its target's territory.**
Proposed while this spec was drafted and since **refuted** rather than
deferred. Such a check would refuse a large minority of the corpus's edges
across more than fifty specs, contradict the edge's documented meaning, and
strip the only claim about a fifth of this repository's source files carry. It
is named here so it is not proposed again as an obvious missing gate. If
visibility into the shape is ever wanted, it belongs as a tier on the
`index coverage` read verb and never as a gate.

**Claiming the other unowned governance files.** This spec fixes the file it
names and does not go looking. A sweep of every unclaimed root-level document
is a coverage question, and `index coverage` is the verb that would answer it
properly, for source files, which these are not.

**Changing what `AGENTS.md` says.** The protocol's content is edited by the
specs that change the protocol. This spec adds one ownership line and touches
nothing else in the document, so that its diff is reviewable as a ledger
correction rather than a protocol change.

## 5. Resolved decisions

**2026-09-08: a new spec establishes it, rather than amending 029.** Two routes
existed. Amending 029 to establish a file it never mentions would retrofit a
claim onto a spec about the Claude Code kit, whose subject is `kit/` and whose
author did not think about the root protocol file. Filing a spec whose subject
*is* the protocol file puts the claim where a reader would look for it, and is
purely additive. The second route also required editing an approved spec, which
is the act the coherence guard reserves for a human.

**2026-09-08: a bare `file` unit, not per-heading `section` units.** Spec 022
made keypath section anchors available, and `AGENTS.md` has stable headings, so
per-section claims were possible. They were rejected because the protocol is
edited as a document: a change to the gate list routinely touches the New
Sessions section and the Working the backlog section together, and per-section
ownership would report that as two units without adding a fact anyone needs.
The one heading that genuinely is a contract, `## New Sessions`, is already
protected by spec 075 §3.1, which requires it not to change.

**2026-09-08: the seven edges were called a misattribution, and the count says
they are the convention.** This spec was drafted believing that an `extends`
edge naming a unit outside its target's territory was a defect, that the seven
specs extending 029 for root `AGENTS.md` had inherited one, and that a
follow-on spec should add a check and migrate them. Measurement through typed
reads refuted all three at once:

```
unit-carrying extends edges                        435
  target never establishes the unit    a large minority, 50+ specs
tracked source files                                95
  established directly                              71
  claimed only through an extends edge              17
```

`edges.rs` defines the edge as "adds surface to a predecessor", and that is
exactly what those edges do. The follow-on spec was dropped, section 3.3's
migration was withdrawn, and section 4 records the check as refused rather
than deferred.

Kept as a correction rather than deleted, because the belief reached three
artifacts before anyone counted, and the transferable lesson is procedural:
measure how common a pattern is, through a typed read, before filing a spec
that would validate against it. What survives of this spec is the part the
measurement never touched, which is that the protocol file had no
`establishes` claim and now has one.

**2026-09-08: the ownership line is prose, not a `// Spec:` header.** The
header grammar is defined over `SOURCE_EXTS`, which is rs, ts, tsx, js, jsx, go,
py and sh. Markdown is not in it, and adding it would mean the coverage
denominator grows to include every markdown file in the repository, which is a
large and unrelated change to what `index coverage` measures. A prose line
costs nothing and reads better in a document meant for humans and agents both.

## Verification

Each line below is one command: spec 049 section 3.2 makes a fence's body line
a command, so no line may depend on a variable another line set.

The greps fail against pre-078 code and the registry read does not: once this
spec is filed, the corpus reports the claim whether or not section 3.2 has been
done. It is included because it is the assertion that matters, and naming which
lines cannot fail first is better than omitting them to keep the block looking
uniformly strict.

```verify:cli
cargo build --release --locked
# 3.1 the ledger answers the question this spec exists to fix.
./target/release/spec-spine registry show 078-the-protocol-has-an-owner
# 3.2 the protocol names the spec that governs it, by path.
grep -qF 'specs/078-the-protocol-has-an-owner/spec.md' AGENTS.md
# 3.2 and it is prose, not a header grammar the indexer does not read here.
! grep -qE '^//\s*Spec:' AGENTS.md
# 3.1 the claim is witnessed: AGENTS.md is a hashed input, so no L-008.
./target/release/spec-spine lint --fail-on-warn
# 2. kit/AGENTS.md was correctly attributed and is left alone.
! grep -qF 'specs/078-the-protocol-has-an-owner/spec.md' kit/AGENTS.md
```
