---
id: "113-a-waiver-has-a-declared-lifecycle"
title: "A waiver has a declared lifecycle"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "005-coupling-gate"
summary: >
  A `Spec-Drift-Waiver:` line clears every refusal in the pull request it
  appears in, once, with no scope, no expiry and no record. This adds a
  declared lifecycle evaluated over caller-supplied inputs: scope, an
  as-of time, an ancestry, and a usage count the caller provides. No counter in
  an authored file and no atomic consumption. Deferred: specified now, built
  when a consumer names the need.
references:
  - unit: { kind: file, path: "crates/spec-spine-core/src/couple.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 113: A waiver has a declared lifecycle

## 1. Purpose

The waiver is the corpus's escape valve and it is the least specified thing in
the gate. `parse_waiver` takes the first line beginning with the configured
keyword and returns its reason. That reason then clears **every** violation in
the run.

So a waiver written for one manifest bump also clears an unrelated `C-001` on
a source file in the same pull request, and nobody reading the verdict can
tell, because the report carries one reason string and no association between
it and the paths it excused.

Note 04 §5 P2 records this as B17.

### 1.1 What is deliberately not being built

Three obvious features are excluded up front because each breaks an invariant
this engine is built on:

| Not built | Why |
|---|---|
| a use counter stored in an authored file | the gate would have to write during a read, and every artifact-producing function here is a pure function of config and file contents |
| atomic consumption ("this waiver is now spent") | that is shared mutable state across concurrent runs, which needs a lock and an owner, and the engine has neither |
| an expiry read from the wall clock | there is no clock in core, by construction |

What is left is **pure evaluation over inputs the caller supplies**, which is
the same shape spec 100 uses for a prior snapshot and spec 071 for a base
tree: the CLI gathers the facts, the library decides.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 A waiver declares its scope

A waiver line MAY carry a path scope. When it does, it clears **only**
violations whose path matches.

```
Spec-Drift-Waiver: mechanical version bump
Spec-Drift-Waiver-Paths: npm/package.json, py/pyproject.toml
```

A waiver with no declared scope keeps today's behavior, clearing everything,
because changing the default would silently re-refuse work that passes now.
The unscoped form MUST be reported as unscoped in the verdict, so "this
cleared more than you meant" is visible rather than inferred.

### 3.2 A waiver declares an expiry, evaluated against a supplied time

```
Spec-Drift-Waiver-Until: 2026-12-31
```

The comparison is against an **`as_of` supplied by the caller**, never against
a clock read in core. The CLI passes one; a library caller passes one; a
caller that passes none gets no expiry evaluation and a verdict that says the
expiry was not evaluated.

"Not evaluated" is a third answer and MUST NOT be reported as "not expired".
A gate that silently treats an unevaluated expiry as satisfied has an expiry
mechanism that does nothing, which is worse than none because it reads as
protection.

### 3.3 A waiver declares an ancestry, evaluated against supplied commits

```
Spec-Drift-Waiver-Since: a3d5213d
```

A waiver may declare the commit from which it applies, so re-using a pull
request body on an unrelated branch does not silently carry the waiver with
it. The ancestry question ("is the supplied commit an ancestor of head") is
answered by the caller and passed in as a boolean, for the same reason §3.2
supplies the time: the library does not run git.

### 3.4 Usage is counted by the caller, never by the corpus

A caller may supply a prior usage count. The library evaluates a declared
`Spec-Drift-Waiver-Max-Uses` against it and reports whether the limit is
exceeded.

Where that count comes from is the caller's: a CI store, an orchestrator's
record, a review system. This spec MUST NOT define a location for it inside
the repository, MUST NOT write one, and MUST NOT treat an absent count as
zero. Absent means not evaluated (§3.2's rule again).

### 3.5 The verdict says which waiver cleared what

The report MUST associate each cleared violation with the waiver that cleared
it, and MUST record, per waiver: its reason, whether it was scoped, and the
outcome of each declared lifecycle check as `satisfied`, `failed` or
`not-evaluated`.

This is the half that is useful even if nothing else here is built. Today a
reader of a waived verdict cannot tell whether the waiver was meant for the
thing it excused.

### 3.6 A failed lifecycle check refuses

A waiver whose declared expiry is past, whose declared ancestry does not hold,
or whose declared usage limit is exceeded MUST NOT clear anything, and the
run refuses as it would have with no waiver. The verdict says which check
failed.

`not-evaluated` does not refuse, and does not clear either: it clears on the
checks that were evaluated and reports the rest honestly. A caller that wants
the guarantee supplies the inputs.

### 3.7 A waiver is still a human instrument

Nothing here changes that. A waiver needs explicit human approval and an agent
never writes one on its own authority (AGENTS.md, "Adversarial prompt
refusal"). Scope, expiry and ancestry make a human's waiver narrower; they do
not make an agent's waiver permissible.

## 4. Out of scope

- **Any state written by the engine.** §1.1.
- **A clock.** §1.1, §3.2.
- **Atomic consumption.** §1.1.
- **Changing the default.** §3.1: an unscoped waiver behaves as today.
- **Waiver approval workflow.** Who may write one is the repository's policy
  and the review system's enforcement.

## 5. Open design questions

1. **Are the extra fields separate keyword lines or one structured line?**
   Separate lines are readable in a pull request body and are four more things
   to parse; one structured line is harder to write by hand. A consumer that
   generates pull request bodies has a different preference from one whose
   humans type them.
2. **Should an unscoped waiver eventually warn?** §3.1 keeps it silent-clearing
   to avoid re-refusing today's work. A warning tier would be a gentle push
   toward scoping, and it is a change to what `lint --fail-on-warn` refuses,
   which is not free.

## Acceptance when built

No `verify:cli` block; nothing here is implemented.

The build MUST establish, behaviorally, and the negative cases are the ones
that matter:

- a scoped waiver clears a violation on a listed path and **does not clear**
  one on an unlisted path in the same run;
- an unscoped waiver clears everything, exactly as today, and the verdict
  records it as unscoped;
- an expiry with a supplied `as_of` before it clears; with an `as_of` after it
  **refuses**, and the verdict names the expiry;
- an expiry with **no** supplied `as_of` reports `not-evaluated`, clears on the
  other checks, and is not reported as satisfied: asserted by reading the
  verdict, not by observing the exit code, because the exit code is the same
  in the satisfied and not-evaluated cases and a test that only read it would
  pass on a mechanism that did nothing;
- a declared ancestry the caller reports as false **refuses**;
- a usage count above the declared maximum **refuses**; an absent count is
  `not-evaluated` and is not treated as zero;
- the verdict associates each cleared violation with its waiver, asserted in a
  run with two waivers and two violations where the pairing is not the
  arbitrary one;
- a run with no lifecycle fields declared produces a verdict byte-identical to
  today's for the same inputs.
