---
id: "040-amendment-authoring"
title: "An amendment is declared once, in the amending spec"
status: draft
kind: "governance"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "005-coupling-gate"
establishes:
  - { kind: section, file: "standards/spec/contract.md", anchor: "amendment-authoring" }
references:
  # The principle this makes unambiguous, the gate half already specified, and
  # the two precedents that have practised the rule since before it was written.
  - { unit: { kind: file, path: "standards/spec/constitution.md" }, role: context }
  - { unit: { kind: file, path: "specs/025-unresolved-unit-severity/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/036-configured-corpus-root/spec.md" }, role: context }
summary: >
  The corpus has always authored amendments one way: the `amends` edge is
  declared in the amending spec's frontmatter, and the amended spec's `spec.md`
  is left alone. Spec 025 amends 004 without editing it; spec 036 amends 005
  without editing it. The rule is enforced by nothing, stated by nothing, and
  visible only as a pattern in the history, so a reader looking at a diff cannot
  tell a deliberate convention from an omission. That is not hypothetical: the
  review of PR #89 asked six separate times for spec 033 to be edited to record
  that 038 had amended it, withdrew the request once on its own analysis, then
  raised it again, because nothing in the corpus says which answer is right.
  This spec writes it down, in the one-page contract where a reader checking the
  edge vocabulary will find it. It adds no mechanism, because the two mechanisms
  already exist and work: the gate's amends-awareness (005 §3.3) widens the owner
  set so editing the amender clears a violation on the amended spec's territory,
  and `registry relationships` reports `amended_by (incoming)` so the amendment
  is discoverable from the amended spec's side without that spec being touched.
---

# 040: Amendment authoring

## 1. Purpose

Three of this project's rules meet at the `amends` edge, and until now the
meeting point was undocumented.

The constitution says history stays queryable, and that "an amendment patches its
predecessor in place rather than blowing away its history" (V). *Patches in
place* is about the effective contract, not about the file: an amendment narrows
or corrects what its predecessor governs while the predecessor's own text stands
as the record of what was true. Read quickly, though, "patches in place" sounds
like an instruction to edit the predecessor, which is the opposite of what it
means and the opposite of what the corpus does.

`.claude/rules/adversarial-prompt-refusal.md` says never to amend an owning spec
to satisfy a mechanical refresh. That rule governs agents working in this repo
and is not part of the corpus, so it does not answer the question for anyone
reading the specs alone.

Spec 005 §3.3 specifies what the *gate* does with an `amends` edge. It says
nothing about how a human should author one.

So the authoring rule exists in practice, in a harness file, and in the gap
between two specs, but nowhere a reader can cite. The evidence that this is a
real gap rather than a tidiness concern: across seven review rounds on PR #89, an
automated reviewer asked six times for `specs/033-dependency-cycle-refusal/spec.md`
to be edited because spec 038 declared `amends: ["033-dependency-cycle-refusal"]`.
It withdrew the request in round 5 on its own analysis of the mechanism, then
raised it again in round 6. A question that answerable should not have to be
answered from precedent six times.

## 2. Territory

One new section in `standards/spec/contract.md`, the one-page normative summary,
placed immediately after the typed-edge vocabulary so that a reader who has just
met `amends` meets the authoring rule in the same breath.

The constitution is deliberately not edited. Its Principle V is correct as
written; what was missing is the operational consequence, and the contract is
where this corpus states operational consequences.

## 3. Behavior

### 3.1 The rule

An `amends` edge MUST be declared in the amending spec's frontmatter, and only
there. The amended spec's `spec.md` MUST NOT be edited for the purpose of
recording that it has been amended: no back-pointer, no "narrowed by NNN" note,
no forward reference.

This holds regardless of how substantive the amendment is. An amendment that
narrows an explicit exclusion is exactly as silent in its predecessor as one that
corrects a severity tier.

### 3.2 Why the predecessor is not edited

**It destroys the record.** The predecessor's text is evidence of what the corpus
held when it was ratified. Constitution V makes "who established this" and "who
owns it now" separately answerable, and that only works if the earlier document
still says what it said. A spec annotated with every later narrowing stops being
a record and becomes a changelog.

**It duplicates a compiled fact into prose that will rot.** The relationship is
already in the registry, and prose copies of compiled facts drift: the second
amendment forgets to add its note, and the predecessor now lists one of two
narrowings, which is worse than listing neither because it reads as complete.

**It is the move the coherence guard exists to stop.** Editing an older spec so a
newer change reads consistently is the shape of authority laundering, whether the
motive is honest or not. Keeping the rule absolute removes the judgement call at
the moment when the person making it is least able to be objective.

### 3.3 Discovery is a compiled read, not a prose pointer

A reader who wants to know whether a spec has been amended asks the ledger:

```
spec-spine registry relationships 033-dependency-cycle-refusal
  depends_on: 001-compile-registry, 016-short-id-resolution
  amended_by (incoming): 038-registry-plan-ready-set
```

`amended_by (incoming)` is the inbound projection, and it is complete by
construction because it is derived from the edge set rather than maintained by
hand. This is the same argument constitution II makes for every other machine
fact: the typed read is authoritative and the prose copy is not.

### 3.4 The gate half is unchanged

Spec 005 §3.3 already specifies what `couple` does with an `amends` edge: under
the FR-005 strict-expansion guard it *widens* the owner set for a changed
`<specs_dir>/<id>/spec.md`, so editing the amending spec can clear a violation on
the amended spec's territory. The edge is never read as an obligation on the
amended spec, and no waiver is needed for an amendment that touches only the
amender.

This spec adds no mechanism and changes no behavior. Both halves already work;
only the sentence telling an author which one to rely on was missing.

## 4. Out of scope

**Enforcement.** There is nothing to enforce. "Someone edited the predecessor to
mention its successor" is not mechanically distinguishable from any other edit to
that spec, so this rule is a convention backed by review, not a lint code. Adding
a diagnostic that guessed at intent would produce false positives on legitimate
edits to an amended spec, which are ordinary and expected.

**`supersedes`.** Superseding a spec does change the predecessor: `status`
becomes `superseded` and `superseded_by` is set, which is a lifecycle transition
the predecessor genuinely undergoes. `amends` is the edge that leaves its target
alone, and conflating the two is precisely the confusion this spec exists to
prevent. Spec 019's structured partial supersession governs that side.

**Amending the constitution.** Whether an ordinary spec can amend
`standards/spec/constitution.md`, and by what edge given that `amends` resolves
to spec ids and the constitution is not a spec, is a real question this spec does
not answer. It did not need to: the rule stated here is an operational
consequence of Principle V, not a change to it.
