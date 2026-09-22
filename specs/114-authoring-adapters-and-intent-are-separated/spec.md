---
id: "114-authoring-adapters-and-intent-are-separated"
title: "Authoring adapters and intent are separated"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "092-the-engine-ships-governance-not-an-environment"
summary: >
  The feature register carries A10 (authoring-tool adapters) and B23 (an
  intent manifest) as one wave item. They belong to different owners: A10 is
  environment delivery, which is Statecraft's since spec 092, and B23 is a
  corpus-side declaration that is spec-spine's. This separates them and states
  what each owner is left holding. Deferred: specified now, built when a
  consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/07-statecraft-realignment-2026-09.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 114: Authoring adapters and intent are separated

## 1. Purpose

Note 09 §1.3 P9 carries A10 and B23 as one row, and says A10 "is adjacent to
harness delivery, which is now Statecraft's, so it should be re-scoped before
it is re-proposed". Doing that re-scope is this spec, and the answer is that
the row was two items.

### 1.1 The two, and why pairing them was wrong

**A10, authoring-tool adapters.** Making an editor, an agent harness or an
IDE able to author specs: templates, completions, validation as you type, a
"new spec" command. Every one of those is a thing installed into a developer's
environment.

**B23, an intent manifest.** A declaration, in the corpus, of what a change is
trying to achieve, so that a reviewer and a tool can read the intent rather
than infer it from the diff.

They were paired because both are "about authoring". They differ on the only
question that matters here: **who owns the artifact**. A10's artifact is an
installed environment, which spec 092 removed from this engine's scope and
note 07 §4 assigned to Statecraft. B23's artifact is text in a `spec.md`,
which is this corpus's.

A single row invited a single owner, and the wrong one either way.

## 2. Territory

None, on spec 106 §2's terms. B23's eventual territory is a frontmatter key
and is claimed by the spec that builds it.

## 3. Behavior

### 3.1 A10 is transferred, and what transfers with it

A10 is **not spec-spine's** and this spec MUST NOT be read as scheduling it
here. It joins note 09 §1.4's transferred rows.

What spec-spine owes an adapter author is what it already owes every consumer,
and it is worth naming so "transferred" does not read as "abandoned":

- the **library API** and the `&str -> Result<String, Error>` facade, which is
  the surface an adapter wraps;
- the **authoring templates**, emitted by `scaffold_init_json` as data;
- the **validation verbs**, in particular `compile --spec <id>`, which
  answers "is this one spec well-formed" without writing, and is exactly what
  an editor integration needs;
- the **read documents**, versioned, so an adapter parses a contract rather
  than prose.

All four exist. An adapter needs no new engine surface, which is the second
reason A10 was never engine work.

### 3.2 What spec-spine MUST NOT build for A10

- No editor plugin, language server, or IDE integration.
- No installed file of any kind, in any agent's harness directory.
- No "new spec" command. There is no `init` (spec 092) and there is no verb
  that writes a spec.
- No generated harness content. The kit is gone and `scaffold_init_json`
  emits governance starter content only.

This list exists because A10 is the proposal most likely to be re-derived as
"just a small template command", and a small template command is an
initializer.

### 3.3 B23 is spec-spine's, and is a declaration

An intent manifest declares, in frontmatter, what a change is for:

```yaml
intent:
  goal: "a correct removal stops being refused"
  non_goals:
    - "changing what C-001 means for an unowned deletion"
  approach: "judge a deleted path at the snapshot preceding its segment"
```

- **`goal`** is one sentence.
- **`non_goals`** are the things a reader would reasonably expect and that are
  deliberately excluded. This is the member with the most value: the corpus
  already has a `## 4. Out of scope` heading in every spec, and it is prose.
- **`approach`** is one sentence on the mechanism.

Frontmatter, one key, no second fenced syntax (spec 106 §3.1).

### 3.4 What an intent manifest establishes, and what it does not

It establishes that the author **declared** this goal and these non-goals.

It does not establish that the change achieves the goal, that it respects the
non-goals, or that the goal is the right one. Nothing checks any of the three
and nothing could.

Its value is that a reviewer and a tool read the same declared intent, and
that a later reader can tell a deliberate omission from an oversight, which is
what `non_goals` is for.

### 3.5 Nothing gates on intent

No verb refuses because of an intent manifest, and none refuses for its
absence. The key is optional.

An intent-aware gate would be refusing on the strength of a sentence nobody
verified, which §3.4 says is all this is.

### 3.6 The relationship to the existing prose

Every spec in this corpus already has `## 1. Purpose` and `## 4. Out of
scope`. B23 does not replace them and MUST NOT: prose says why, and a
manifest says what, and the first is what a human needs.

If B23 is ever built, the honest framing is that it makes two facts already in
the prose machine-readable, and the open question in §5 is whether that is
worth a second place for them to disagree.

## 4. Out of scope

- **Everything in §3.2.**
- **Any gate.** §3.5.
- **Replacing the prose headings.** §3.6.
- **Scheduling either item.** This spec separates ownership; it does not
  schedule.

## 5. Open design questions

1. **Is B23 worth the duplication?** `goal` and `non_goals` restate `## 1` and
   `## 4`. Two places for one fact is two places to disagree, and the corpus
   has met that failure (a spec's prose describing behavior its acceptance did
   not have). A consumer that can say what it would do with the structured
   form settles this; nobody has.
2. **Does an intent manifest belong to a spec or to a change?** A spec is a
   standing document and a change is an event. `non_goals` reads as the spec's;
   `approach` reads as the change's.
3. **Who re-proposes A10?** It is Statecraft's, and this spec does not propose
   it on their behalf. §3.1 records what they can build against.

## Acceptance when built

No `verify:cli` block; nothing here is implemented, and A10's half is not
this repository's to implement at all.

A build of B23 MUST establish, behaviorally:

- a well-formed `intent` key compiles and reaches the registry shard with
  `goal`, `non_goals` and `approach` preserved verbatim;
- the key is optional: a corpus with none compiles to shards whose
  `shardHash` values are unchanged;
- no verb's verdict changes on the same tree with and without it, including
  `couple` and `lint --fail-on-warn`;
- an `intent` with an empty `goal` is a validation error, because a declared
  goal that says nothing is worse than an absent key.

A10 has no acceptance here. If Statecraft builds an adapter, the assertion
spec-spine owes is that §3.1's four surfaces are the ones it needed, which is
answered by their integration and not by a test in this repository.
