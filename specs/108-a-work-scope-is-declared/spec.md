---
id: "108-a-work-scope-is-declared"
title: "A work scope is declared"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "072-two-ready-specs-can-collide"
summary: >
  Spec 072 already reports that two ready specs claim overlapping territory,
  which was half of what WorkScope proposed. What is left is the other half: a
  declaration of what one piece of work may change exclusively, what it shares,
  and which spec answers for each. Deferred: specified now, built when a
  consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 108: A work scope is declared

## 1. Purpose

Note 04 §4.6 proposed a WorkScope carrying two things: a statement of which
paths a task may change, and a way to see that two tasks collide.

**The second half exists.** Spec 072 reports, from `registry plan`, that two
ready specs claim overlapping territory. That is why this spec is narrower
than the note it comes from, and why its `depends_on` names 072: what is left
builds on the collision report rather than replacing it.

What is left is the scope document itself: for one piece of work, which paths
are **mutable** by it exclusively, which are **shared** with concurrent work,
and which spec answers for each. Without it, "may I touch this file" is
answered by reading ownership and guessing about concurrency.

### 1.1 Not an obligations dependency

This spec does not `depends_on` spec 106. A scope is about paths and the specs
that own them, which the corpus already models; nothing in it needs an
obligation id. The two were proposed in the same wave, which is not a
dependency.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 A scope is a declared partition of paths

A WorkScope MUST be declared in frontmatter, under one key, with no second
fenced syntax.

```yaml
work_scope:
  id: "W-1"
  own_spec: "100-a-deleted-path-is-judged-where-it-lived"
  mutable:
    - "crates/spec-spine-core/src/couple.rs"
    - "crates/spec-spine-cli/src/cmd_couple.rs"
  shared:
    - { path: "crates/spec-spine-types/src/version.rs", with: ["034-machine-readable-verdicts"] }
  read_only:
    - "crates/spec-spine-cli/src/cmd_delta.rs"
```

- **`own_spec`** names the spec this work is executing. It MUST exist.
- **`mutable`** are paths this work expects to change and expects no
  concurrent work to change.
- **`shared`** are paths it expects to change and knows another spec's work
  may also change. Each entry names the other specs.
- **`read_only`** are paths it depends on and does not change. Declaring them
  is what lets a consumer notice that somebody else changed the thing this
  work was reasoning about.

### 3.2 Every path is checked against ownership, and a mismatch is reported

For each declared path, the compiler MUST resolve its owners the way
`index owner` does, and report:

- a `mutable` path **no spec owns**, which is either territory to claim or a
  path outside the corpus;
- a `mutable` path owned by a spec **other than** `own_spec`, and not
  declared `shared`, which is an undeclared crossing: the corpus's answer to
  it is an `extends` edge, and a scope that hides it is worse than no scope;
- a `shared` path whose `with` list does not match its actual owner set.

These are **reports**, at warning tier. They are not refusals: see §3.4.

### 3.3 What a scope establishes, and what it does not

A scope establishes that somebody **declared** this partition for this work.

It does **not** establish:

- **that the declaration is complete.** Work may touch a path the scope never
  named, and nothing here prevents that. `couple` is what judges what was
  actually changed; a scope is what was intended.
- **that concurrent work will respect it.** A scope is one task's statement
  about itself. Two scopes can both declare the same path `mutable` and
  neither is wrong on its own terms; spec 072's collision report and §3.5 are
  what surface the pair.
- **any permission.** Declaring a path `mutable` grants nothing. Ownership
  and the coupling gate are unchanged.

### 3.4 A scope is not a gate

No verb refuses because of a scope. In particular `couple` MUST NOT consult
one: the gate judges the diff against the corpus, and a second, weaker
statement of intent must never be able to clear a path the corpus refuses, nor
refuse one it clears.

The reason to state this as a MUST NOT rather than an omission is that "the
scope said I could" is the most obvious wrong thing to build next, and it
would convert a planning aid into an ownership bypass.

### 3.5 Two scopes are comparable

Given two declared scopes, a consumer MUST be able to ask whether they
conflict, and get a structured answer: the paths that are `mutable` in both,
and the paths `mutable` in one and `shared` or `read_only` in the other.

This is the half spec 072 does at spec granularity, at path granularity and
between declared intentions rather than between ready specs. It is a read,
never a refusal, for §3.4's reason.

## 4. Out of scope

- **Any gate or permission.** §3.3, §3.4.
- **Scheduling.** Deciding which of two conflicting scopes proceeds is an
  orchestration decision and, since spec 092, Statecraft's.
- **Locking or reservation** of any kind. Nothing here writes state, and a
  scope that could be "held" would be a lock with no owner and no release.
- **Re-doing spec 072's collision report.** §1.

## 5. Open design questions

1. **Where does a scope live?** Like a closure (spec 107 §5), it is more
   naturally a property of a work order than of a spec, and this repository
   has no work-order record. A consumer answers this first.
2. **Is `read_only` worth its cost?** It is the member most likely to go stale
   and the one whose warnings would be noisiest. It is included because the
   "somebody changed what I was reasoning about" case is real; a consumer may
   find it is not worth declaring.

## Acceptance when built

No `verify:cli` block; nothing here is implemented.

The build MUST establish, behaviorally:

- a scope whose paths all resolve compiles and appears in the registry shard;
- a `mutable` path owned by another spec, not declared `shared`, produces the
  undeclared-crossing warning naming both specs;
- a `mutable` path no spec owns produces the unowned warning;
- a `shared` entry whose `with` list disagrees with the resolved owner set
  produces the mismatch warning naming both lists;
- none of the above changes `lint`'s exit code without `--fail-on-warn`, and
  none changes `couple`'s verdict at all on the same diff (§3.4);
- two scopes declaring the same path `mutable` are reported as conflicting,
  and two scopes sharing only a `read_only` path are not.
