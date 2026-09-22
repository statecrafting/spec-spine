---
id: "111-a-move-is-a-reviewed-mapping"
title: "A move is a reviewed mapping"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "100-a-deleted-path-is-judged-where-it-lived"
summary: >
  The gate passes `--no-renames`, so a move arrives as a deletion and an
  addition and each half is judged where it is true. That is correct and it is
  not the whole story: nothing records that the two halves are one act. This
  declares the mapping as authored, reviewed data, never as a similarity
  heuristic. Deferred: specified now, built when a consumer names the need.
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
---

# 111: A move is a reviewed mapping

## 1. Purpose

Note 04 records A07: git similarity is a suggestion, never authority. Spec 071
takes that position and spec 100 §3.6 keeps it, judging a rename's delete half
at the snapshot it lived in and its add half at head, with no pairing by any
heuristic.

The result is right and it loses something. After a move, nothing in the
corpus says the old path and the new path are the same thing. A reader asking
"where did `couple.rs`'s bypass logic go" gets an answer from git's similarity
index, which is a guess, or from a human's memory.

### 1.1 Why the dependency is spec 100

Spec 100 §3.6 is what makes a mapping additive rather than a replacement: it
fixes that the two halves are judged independently and forbids any heuristic
pairing, so a declared mapping is a **record on top of** two correct verdicts
rather than a way to change either. Building this before 100 would have meant
designing against semantics that were still wrong. That is a real ordering
dependency and note 09 P6 names it.

It does not depend on obligations, scopes or closures.

## 2. Territory

None, on spec 106 §2's terms.

## 3. Behavior

### 3.1 A mapping is authored, in frontmatter

```yaml
moves:
  - from: "kit/skills/build/SKILL.md"
    to: ".claude/skills/build/SKILL.md"
    kind: relocated
    reviewed_by: "the spec that declares this move"
  - from: "crates/spec-spine-core/src/kit_embedded.rs"
    to: null
    kind: removed
    answered_by: "092-the-engine-ships-governance-not-an-environment"
```

- **`from`** is the path as it was, and MUST NOT need to exist at head.
- **`to`** is the path as it is, or `null` for a removal.
- **`kind`** is `relocated` (same content, new path), `split` (one path became
  several, `to` is a list), `merged` (several became one), or `removed`.
- **`answered_by`** names the spec that answers for the path now, for a
  removal.

The mapping is declared by the spec performing the move, which is the spec
that knows.

### 3.2 No similarity, ever

This spec MUST NOT introduce rename detection, similarity scoring, content
comparison between `from` and `to`, or any inference that pairs a deletion
with an addition. It MUST NOT cause `--no-renames` to be dropped.

A mapping is true because an author wrote it and a reviewer read it. A
mapping that a tool guessed is a different artifact with a different
reliability, and the moment the two are stored in one field nobody can tell
which they are holding.

### 3.3 A mapping changes no verdict

`couple` MUST NOT consult a mapping. The delete half still needs the spec that
owned the old path to author the removal, and the add half still needs the
spec that owns the new path to claim it (spec 100 §3.6). A declared move does
not clear either.

The reason is the one §3.2 gives from the other side: if a mapping could clear
a deletion, then writing a mapping would be a way to move a governed file out
from under its owner, and the only thing standing between that and an
ownership bypass would be review discipline.

### 3.4 What a mapping is for

Reading. Specifically:

- `index owner <old-path>` can say "no spec owns this path; spec X declared it
  moved to Y on this date", instead of "(no spec owns this path)".
- A citation repair like spec 098's has a machine-readable source instead of
  a hand-built table.
- `docs/corpus-map.md`, which spec 096 already emits for spec ids, has a path
  analogue.

### 3.5 Mappings are validated for shape, not for truth

The compiler MUST report:

- a `to` path that does not exist at head, for `kind: relocated`;
- a `from` path that still exists at head, for `kind: removed`;
- an `answered_by` naming a spec that does not exist.

At warning tier. It MUST NOT attempt to verify that the content moved, which
is §3.2's boundary.

## 4. Out of scope

- **Rename detection.** §3.2.
- **Changing any verdict.** §3.3.
- **Retroactive mappings** for moves already performed. Every historical move
  could be declared and none will be; the value is in the next one.
- **Spec-id mapping.** `docs/corpus-map.md` and spec 096 already do that, for
  ids. This is paths.

## 5. Open design questions

1. **Does a mapping belong in the moving spec or in a corpus-wide map?** In the
   spec it is discoverable from the change; in a map it is discoverable from
   the path. §3.4's first use case wants the second, which suggests the
   compiler should build the map from the declarations, and that is a design
   decision with a cost.
2. **How long does a mapping stay?** A path that moved twice has two hops, and
   a reader wants the transitive answer. Transitivity needs cycle handling and
   a decision about how far back the chain is kept.

## Acceptance when built

No `verify:cli` block; nothing here is implemented.

The build MUST establish, behaviorally:

- a well-formed `moves` declaration compiles and reaches the registry shard;
- a `relocated` entry whose `to` does not exist warns, naming both paths;
- a `removed` entry whose `from` still exists warns;
- an `answered_by` naming a missing spec is a validation error;
- `index owner` on a mapped old path reports the mapping;
- **`couple` reaches exactly the same verdict, on a diff that deletes `from`
  and adds `to`, with and without the mapping declared**, including that the
  deletion still refuses when the owning spec is not edited. This is the
  assertion that keeps §3.3 from decaying into a bypass, and it must be a
  refusal case and not only a clearance case;
- `--no-renames` is still passed, asserted by a fixture where git's similarity
  index would pair two files and the gate still reports two paths.
