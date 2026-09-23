---
id: "114-authoring-adapters-and-intent-are-separated"
title: "Authoring adapters and intent are separated"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: pending
owner: "The spec-spine Authors"
depends_on:
  - "092-the-engine-ships-governance-not-an-environment"
summary: >
  The feature register carried A10 (authoring-tool adapters) and B23 (an
  intent declaration) as one item. They belong to different owners: A10 is
  environment delivery, which is Statecraft's since spec 092, and B23 is a
  corpus-side declaration, which is spec-spine's. This separates them, records
  what an adapter author can build against, and specifies the smallest
  optional structured intent a spec can carry: a standing goal and its
  non-goals, validated for shape, transported into the registry, and read by
  no gate. The prose stays authoritative; attempt-specific planning stays
  with the consumer.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/intent.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/intent.rs", planned: true }
extends:
  # 3.3: the frontmatter key and the registry member.
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  # 3.4: shape validation at compile.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
references:
  - unit: { kind: file, path: "docs/design/07-statecraft-realignment-2026-09.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "A well-formed intent reaches the declaring spec's registry record with its goal and non-goals preserved as authored text."
    anchor: "3-3-an-intent-is-a-standing-declaration-on-the-spec"
  - id: "R-2"
    kind: requirement
    text: "A malformed intent (an empty or whitespace-only goal or non-goal, a wrong type, or an unknown member) is a compile validation error naming the spec."
    anchor: "3-4-a-malformed-declaration-is-refused-as-a-declaration"
  - id: "I-1"
    kind: invariant
    text: "A well-formed intent, present or absent, changes no verdict and authorizes nothing."
    anchor: "3-5-a-valid-intent-is-not-an-authorization"
  - id: "I-2"
    kind: invariant
    text: "spec-spine builds no adapter, installs nothing and adds no verb that writes a spec for A10."
    anchor: "3-2-what-spec-spine-must-not-build-for-a10"
  - id: "V-1"
    kind: verification
    text: "Round trip, optionality with unchanged shard hashes, each malformed shape refused, and verdict neutrality on equivalent fresh trees."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/intent.rs"
---

# 114: Authoring adapters and intent are separated

## 1. Purpose

Design note 09 §1.3 P9 carries A10 and B23 as one row, and says A10 "is
adjacent to harness delivery, which is now Statecraft's, so it should be
re-scoped before it is re-proposed". The re-scope is this spec, and the answer
is that the row was two items with two owners.

### 1.1 The two, and why pairing them was wrong

**A10, authoring-tool adapters.** Making an editor, an agent harness or an IDE
able to author specs: templates, completions, validation as you type, a "new
spec" command. Every one of those is installed into a developer's
environment.

**B23, an intent declaration.** A structured statement, in the corpus, of what
a spec is for and what it deliberately excludes, so that a reviewer and a tool
read the same declared intent.

They differ on who owns the artifact. A10's artifact is an installed
environment, which spec 092 removed from this engine's scope and design note
07 assigned to Statecraft. B23's artifact is text in a `spec.md`, which is
this corpus's.

## 2. Territory

B23's frontmatter key and its registry member: a new types module
(`crates/spec-spine-types/src/intent.rs`), additive edits to the frontmatter
and registry DTOs and the types export list, shape validation in
`compile.rs`, and a new acceptance file. The registry schema, `version.rs`,
the conformance pins and the verifier fixtures move with the MINOR under the
specs that own them, as `extends` edges the build declares. A10 has no
territory here.

## 3. Behavior

### 3.1 A10 is transferred, and what transfers with it

A10 is **not spec-spine's**. It joins note 09 §1.4's transferred rows, owned
by Statecraft. What spec-spine owes an adapter author is what it already owes
every consumer:

- the **library API** and the `&str -> Result<String, Error>` facade;
- the **authoring templates**, emitted by `scaffold_init_json` as data;
- the **validation verbs**, in particular `compile --spec <id>`, which answers
  "is this one spec well-formed" without writing;
- the **read documents**, versioned, so an adapter parses a contract rather
  than prose.

All four exist. An adapter needs no new engine surface.

### 3.2 What spec-spine MUST NOT build for A10

- No editor plugin, language server or IDE integration.
- No installed file of any kind, in any agent's harness directory.
- No "new spec" command, and no verb that writes a spec. There is no `init`
  (spec 092).
- No generated harness content.

### 3.3 An intent is a standing declaration on the spec

A spec MAY carry one optional frontmatter key:

```yaml
intent:
  goal: "a correct removal stops being refused"
  non_goals:
    - "changing what C-001 means for an unowned deletion"
```

- **`goal`** (required when `intent` is present): one non-empty string, what
  the spec is for.
- **`non_goals`** (optional, default empty): non-empty strings, each a thing a
  reader would reasonably expect and that is deliberately excluded.

No other member is accepted. In particular there is no `approach`: how a
particular attempt means to meet the goal is a property of that attempt, not
of the standing spec, and it belongs in the consumer's work record (a
Statecraft work order), where the attempt lives.

The compiler records a well-formed intent on the spec's registry record as
`intent: { goal, nonGoals }`, the strings exactly as authored after YAML
scalar parsing. A spec without the key has no `intent` member, so its record
is unchanged. `registry show` prints it.

This is a registry schema MINOR (additive).

### 3.4 A malformed declaration is refused as a declaration

The compiler MUST refuse, as a validation error on the declaring spec:

- an `intent` that is not a mapping, or that lacks `goal`;
- a `goal` or any `non_goals` entry that is not a string, or is empty or
  whitespace only;
- a member other than `goal` and `non_goals`.

This is the same kind of refusal every malformed frontmatter key receives. It
judges whether the declaration is well formed, never whether it is true,
achieved or wise, and it is the only verdict intent can move (§3.5).

### 3.5 A valid intent is not an authorization

For a well-formed intent, presence, absence and content change no verdict:
`compile`, `check`, `lint`, `index coverage` and `couple` reach the same
answers on equivalent fresh trees with and without it. No verb refuses for
its absence, no verb reads it to decide anything, and it grants, scopes or
waives nothing. An intent-aware gate would be refusing on the strength of a
sentence nobody verified.

An edit to the intent is an edit to `spec.md`, so it moves that spec's
content hash like any other edit; that is freshness, not a verdict reading
the intent.

### 3.6 The prose stays authoritative, and disagreement is surfaced, not resolved

Every spec already has `## 1. Purpose` and `## 4. Out of scope`. The intent
does not replace them and MUST NOT: prose says why, and is what a human needs.
Where the two disagree, **the prose governs** and the intent is the thing to
correct.

The engine does not detect disagreement and does not pick a side. It surfaces
the two together: the intent is authored in the same file as the prose, so any
change to either is one diff a reviewer reads; `registry show` prints the
intent beside the spec's summary; and the registry already records a digest
for each section (spec 106), so a consumer that stored an intent together with
the digests of the sections it restates can see when that prose moved after
the intent was read.

## 4. Out of scope

- **Everything in §3.2.**
- **Any gate on intent.** §3.5.
- **Attempt-specific planning.** §3.3: it is the consumer's.
- **Replacing or checking the prose.** §3.6.
- **Scheduling A10.** It is Statecraft's to propose.

## 5. Resolved decisions

**D-1 (2026-09-23, finalized before the build: the empty-goal contradiction).**
The reserved draft required both that "no verb's verdict changes" with intent
and that an empty `goal` be a validation error, and a validation error is a
changed `compile` verdict. The two are separated: validating that a
declaration is well formed (§3.4) is not behavioral enforcement of what it
says (§3.5). A malformed intent is refused like any malformed key; a valid one
moves nothing.

**D-2 (2026-09-23, `approach` removed).** The draft's own open question 2
observed that `approach` reads as the change's and `non_goals` as the spec's.
The boundary answers it: attempt state belongs to the consumer (spec 092,
note 07), so the standing contract carries only `goal` and `non_goals`.

**D-3 (2026-09-23, duplication is accepted with a rule).** The draft asked
whether B23 is worth a second place for facts the prose already states. It is
built as the smallest optional form under D-7, with §3.6's rule that the prose
governs and with disagreement surfaced to a reviewer rather than resolved by
the engine.

**D-4 (2026-09-23, A10 is recorded as transferred).** Note 09's transferred
rows gain A10 when this spec is built, with §3.1 as what Statecraft can build
against.

## Verification

Written to fail against the tree it is filed on: the test target does not
exist.

```verify:cli
# 3.3 - 3.5: round trip, optional with unchanged shard hashes, every malformed
# shape refused by name, and verdict neutrality on equivalent fresh trees.
cargo test -p spec-spine-core --test intent --locked
cargo build --release --locked -p spec-spine-cli
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
