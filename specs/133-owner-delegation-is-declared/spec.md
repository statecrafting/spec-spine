---
id: "133-owner-delegation-is-declared"
title: "Owner delegation is declared"
status: draft
kind: "governance"
created: "2026-09-24"
summary: >
  What an agent may decide in this repository without asking was spread across
  the owner's session instructions and never written where every agent reads.
  `AGENTS.md` gains an "Owner delegation" section that states it: agents decide,
  record and report reversible choices inside an adopted direction,
  relocation-only changes, test and evidence design, repository hygiene, and
  unambiguous corrections of internal inconsistencies; ratification, waivers,
  publication, spending, trust roots and signing, changes to the gate, check
  suite or acceptance authority, anything visible outside the repository or
  affecting another adopter, and deleting anything remote stay with the owner.
  Every handoff ends with one decision table. The section widens no authority
  the standing rules withhold.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "093-the-harness-this-repository-runs"
extends:
  - { spec: "093-the-harness-this-repository-runs", unit: "AGENTS.md", nature: additive }
references:
  - { unit: { kind: file, path: "specs/093-the-harness-this-repository-runs/spec.md" }, role: context }
intent:
  goal: "every agent working here reads, in one place, which decisions it takes and reports and which it stops and presents"
  non_goals:
    - "widening any authority the standing rules withhold: waivers stay a human instrument and approved specs are still never edited to make code pass (3.2)"
    - "delegation in any other repository (4)"
---

# 133: Owner delegation is declared

## 1. Purpose

Sessions in this repository run long and mostly unattended. The line between
what an agent decides and what it stops for was stated in the owner's session
instructions, differently each time, and nowhere an agent is required to read.
The cost ran both ways: sessions stopped to ask about a reversible choice the
owner had already adopted the direction for, and a session could in principle
read a broad instruction as covering a publication or a waiver it did not.

The owner stated the delegation on 2026-09-24. This spec puts it in `AGENTS.md`,
which every agent loads, beside the standing rules it must not contradict.

## 2. Territory

`AGENTS.md` is established by 093. This spec adds one section to it through an
additive `extends`. It does not amend 093: §4.10 fixes the content of the
`## Rules` section, and this is a separate section that changes none of it.

## 3. Behavior

### 3.1 The section

`AGENTS.md` MUST carry a `## Owner delegation` section, naming this spec by
path, with three parts:

- **Agents decide, record and report afterwards:** reversible choices inside an
  adopted direction; relocation-only changes; test and evidence design;
  repository hygiene; unambiguous corrections of internal inconsistencies.
  "Record" MUST be defined as a dated decision entry in the owning spec, or the
  pull request body where no spec owns the choice; "report" as the next handoff
  naming it.
- **Reserved to the owner:** ratification; waivers; publication; spending money
  or provider usage; trust roots and signing; changes to the gate, the check
  suite or acceptance authority; anything visible outside this repository or
  affecting another adopter; deleting anything remote. An agent reaching one
  MUST stop that line of work, continue independent work, and present the
  choice.
- **Every handoff ends with one decision table:** item, options, recommended
  default, consequence of the default.

### 3.2 It widens nothing

The section MUST say that it widens no authority the standing rules withhold.
A waiver remains a human instrument an agent never writes, and an approved spec
is still never edited to make code pass. Where the delegation and a rule seem to
disagree, the rule governs.

## 4. Out of scope

**Other repositories.** An adopter's delegation is its own; the scaffold does
not emit this section, and Statecraft owns the environment an adopter's agents
run in (092).

**Enforcement.** Nothing mechanical checks that an agent stayed inside the
delegation. The gate, the waiver rule and the ratification step already hold
the reserved decisions that have a mechanical form; the rest is instruction.

## 5. Resolved decisions

**D-1 (2026-09-24): a section, not a rule.** The four `## Rules` are the
standing rules 093 §4.10 fixes by name, and adding a fifth would amend it. The
delegation also changes more often than a rule should: it records the owner's
current posture.

**D-2 (2026-09-24): merged by the owner.** Changing what agents may decide is a
change to acceptance authority in the sense of the section itself, so this
pull request is prepared for the owner to merge rather than merged by the agent
that wrote it.

## Verification

```verify:cli
# 3.1: the section exists, names this spec, and carries its three parts.
grep -q '^## Owner delegation$' AGENTS.md
grep -q 'specs/133-owner-delegation-is-declared/spec.md' AGENTS.md
grep -q '^\*\*Agents decide, record and report afterwards:\*\*$' AGENTS.md
grep -q '^\*\*Reserved to the owner:\*\*$' AGENTS.md
grep -q '^\*\*Every handoff ends with one decision table\*\*' AGENTS.md
# 3.2: it widens nothing.
grep -q 'no authority the rules above withhold' AGENTS.md
# The four standing rules are untouched (093 4.10).
sh -c 'n=$(awk "/^## Rules/{f=1;next} /^## /{f=0} f && /^### /" AGENTS.md | wc -l | tr -d " "); test "$n" -eq 4'
```
