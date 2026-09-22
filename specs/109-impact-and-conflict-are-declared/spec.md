---
id: "109-impact-and-conflict-are-declared"
title: "Impact and conflict are declared"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "106-obligations-are-declared-constraints"
summary: >
  A spec can say which specs it extends and amends. It cannot say which
  requirements a change is expected to affect, or which requirements it
  knowingly conflicts with. Both are declared against obligation ids, which is
  why this depends on spec 106 and on nothing else. Deferred: specified now,
  built when a consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 109: Impact and conflict are declared

## 1. Purpose

Note 04 §5 P2 records A04, A05 and B03: a change should be able to state what
it is expected to affect, and where it knowingly disagrees with something
already written down.

The corpus has edges, and an edge is coarse. `extends` says "this spec adds
surface to that one"; it does not say which of that spec's requirements are
touched. `amends` says "this spec changes that one"; `amends_sections`
narrows it to a heading, which is closer, and still not to a requirement.

### 1.1 Why the obligations dependency is real

An impact set is a set of **things impacted**, and the thing worth naming is a
requirement, not a file and not a whole spec. Without spec 106's ids there is
nothing to point at, so the declaration would degrade to the edge vocabulary
that already exists. That is a semantic dependency and it is the only one
this spec declares.

It does not depend on ContextClosure, WorkScope, move mappings, overlays,
waiver lifecycle or bindings.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 Two declarations, in frontmatter

```yaml
impacts:
  - obligation: "005-coupling-gate#R-3"
    nature: refines
    note: "the owner set for a deleted path is resolved at a prior snapshot"
conflicts:
  - obligation: "071-a-change-is-classified-under-the-bases-rules#R-2"
    reason: "071 refuses a stale base index; this gate falls back to a report"
    resolution: "deliberate divergence, recorded in this spec's D-2"
```

Both keys are optional, both are frontmatter, and there is no second fenced
syntax (spec 106 §3.1).

### 3.2 An impact names an obligation, qualified and resolving

Each `impacts` entry MUST name a **qualified** obligation reference that
resolves (spec 106 §3.3), and a `nature` from a closed set:

| `nature` | Meaning |
|---|---|
| `refines` | the obligation still holds, more precisely |
| `extends` | the obligation still holds, over more surface |
| `supersedes` | the obligation is replaced by one this spec declares |
| `informs` | the obligation is unaffected but a reader of it should know |

`supersedes` MUST name, in the same entry, the obligation of **this** spec
that replaces it. A supersession with no successor is a deletion wearing a
different word, and spec 106 §5 leaves deletion unsettled on purpose.

### 3.3 A conflict is declared, not resolved

Each `conflicts` entry MUST name a qualified, resolving obligation, a
`reason`, and a `resolution` that is one of:

| `resolution` | Meaning |
|---|---|
| `deliberate` | the divergence is intended and the reason says why |
| `unresolved` | the contradiction is known and nobody has decided |
| `pending: <spec-id>` | a named spec is expected to settle it |

An `unresolved` conflict MUST be reported by `lint` at warning tier and MUST
NOT refuse. A corpus is allowed to know it contradicts itself; what it is not
allowed to do is know silently, which is the state every such contradiction is
in today.

### 3.4 What these establish, and what they do not

They establish that a spec **declares** an impact or a conflict.

They do **not** establish:

- **that the impact set is complete.** Nothing computes what a change actually
  affects, and a consumer MUST NOT read "not in the impact set" as "not
  affected". This is spec 106 §3.7's limit and it bites hardest here, because
  an impact set is exactly the artifact somebody will want to use as a blast
  radius.
- **that a declared conflict is the only one.** Two specs can contradict each
  other with neither saying so.
- **that a `deliberate` conflict is correct.** It records that an author chose
  it. A reviewer still has to agree.

### 3.5 Nothing gates on either

`couple` MUST NOT consult impacts or conflicts, and no verb refuses on their
account except the validation errors §3.2 and §3.3 impose on malformed or
dangling references.

An impact-aware gate is the obvious next thought and it is the wrong one while
§3.4 holds: refusing a change because its declared impact set looked
incomplete would be refusing on the strength of something nothing can verify.

## 4. Out of scope

- **Computing impact** from the dependency graph, the index, or a diff. A
  computed impact set is a different artifact and conflating it with a
  declared one is how a declaration acquires authority it was never given.
- **Resolving a conflict.** §3.3 records; a human or a later spec decides.
- **Any gate.** §3.5.
- **Obligation deletion semantics**, which spec 106 §5 leaves open and §3.2
  works around by requiring a successor.

## 5. Open design questions

1. **Is `conflicts` a property of a spec or of a pair?** Declared in one
   spec's frontmatter it is one-sided: the other spec says nothing. A
   two-sided record would need a home neither spec owns.
2. **Should an `unresolved` conflict expire?** A warning that has been there
   for a year is furniture. A deadline would need an authored date and a clock,
   and the engine has no clock by construction; a caller-supplied "as of" is
   the shape spec 113 uses for waivers and could be reused here.

## Acceptance when built

No `verify:cli` block; nothing here is implemented.

The build MUST establish, behaviorally:

- valid `impacts` and `conflicts` compile and reach the registry shard;
- an unqualified or dangling obligation reference in either key is a
  validation error naming the reference;
- a `nature: supersedes` entry with no successor obligation in the declaring
  spec is a validation error;
- a `resolution: unresolved` conflict produces exactly one `lint` warning and
  does not change `lint`'s exit code without `--fail-on-warn`;
- a `resolution: pending: <id>` naming a spec that does not exist is a
  validation error;
- `couple` reaches the same verdict on the same diff with and without both
  keys declared (§3.5).
