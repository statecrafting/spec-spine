---
id: "052-couple-names-the-crossing"
title: "The coupling gate names the crossing"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
  - "037-machine-readable-verdicts"
  - "050-index-diagnostics-reach-a-gate"
extends:
  # The refusal's rendering, and the owner set it carries. Spec 005 3.5 states
  # that an owned path with no owner edit is `C-001`; it states nothing about
  # the message text or the resolution footer, so this is surface added to the
  # gate, not behavior changed. 005's spec.md is not edited (spec 040).
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
  # The shared `Violation` DTO gains one optional field, populated by `couple`
  # alone. Registry emission is byte-identical (3.4), so no schema file and no
  # schema version moves.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/registry.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: ".claude/rules/adversarial-prompt-refusal.md" }, role: context }
summary: >
  The coupling gate has always named the specs that own a drifted path, and has
  never named the mechanism for crossing into their territory. Its resolution
  footer offers two doors: edit an owning spec, or waive. For an author who has
  legitimately touched a unit somebody else owns, both are shut. Editing another
  spec to clear a refusal is the illegitimate edit the corpus rules forbid, and
  the waiver is a human instrument that no adopter has ever used and that an
  unattended session is forbidden on every path. The third door, an `extends`
  edge declared in the author's own spec, is the corpus's actual answer and the
  gate never mentions it. This spec makes the refusal name it: three doors in
  author order, and when the diff edits exactly one spec.md, the concrete
  `extends` block to paste into that file. It also promotes the owner set from
  prose into an optional `owners` field on the violation, so an orchestrator
  reading `couple --json` learns the owner as data instead of by regexing
  English. No committed artifact and no schema version moves.
---

# 052: The coupling gate names the crossing

## 1. Purpose

`spec-spine couple` refuses a changed path whose owning spec did not change, and
it has named the owners in that refusal since the gate was first built:

```
C-001 'crates/spec-spine-core/src/index.rs' changed without an authoring
      edit to any owning spec (004-codebase-index)
```

Then it tells the reader what to do about it, and this is where it fails them:

```
Resolve by editing an owning spec's spec.md, or add a
'Spec-Drift-Waiver:' line to the PR body.
```

Two doors, and for the case that produces most of these refusals, both are shut.

The author is usually building their own spec and has reached into a file
another spec owns. Door one, editing `004-codebase-index/spec.md`, is exactly
the edit `.claude/rules/adversarial-prompt-refusal.md` forbids: changing what a
spec requires, mid-build, to clear a mechanical refusal. Door two, the waiver,
is a human instrument. The adopter audit found it configured, hook-gated and
documented in all four governed repositories and **used in none of them**, and
forbidden to the machine on every path by the one that runs unattended.

The corpus has a third door and the gate does not mention it. An `extends` edge,
declared in the author's own spec, names the other spec and the unit being
touched, makes the author a legitimate owner of that unit, and therefore clears
the gate on the next run. It amends nobody, it needs no waiver, and it leaves a
typed record of the crossing in the ledger, which is the whole point of the
authority graph. It is how claude-observatory carries 86 cross-territory edits.

The cost of not saying so is measured. claude-observatory's spec 016 D-12
records four sessions and $17.44 spent proving a wall that one line of prompt
would have avoided. The refusal was correct every time; it simply never told the
reader that a legitimate crossing existed, so the reader concluded there was
none and kept trying the two doors that were shut.

This is item 3 of the adopter audit's ranked backlog for the tool. Its own
sentence states the whole requirement: "The gate knows the owner; it should say
so and name the corpus mechanism for crossing territory." Half of that is
already true. This spec finishes it, and makes the half that is true readable by
a machine as well as by a person.

## 2. Territory

Nothing here is new surface. Five files, all owned elsewhere, all extended
additively:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-types/src/registry.rs` | 000 | `Violation` gains an optional `owners` field |
| `crates/spec-spine-core/src/couple.rs` | 005 | `C-001` populates `owners` |
| `crates/spec-spine-cli/src/cmd_couple.rs` | 005 | the resolution footer |
| `crates/spec-spine-core/tests/couple.rs` | 005 | the `owners` field's acceptance |
| `crates/spec-spine-cli/tests/couple.rs` | 005 | the footer's acceptance |

Spec 005 §3.5 states that an owned path with no owner edit is a `C-001`
violation. It states nothing about the message text, the footer, or the fields a
violation carries, so every change below adds surface rather than changing what
005 requires. No `amends` edge is declared and 005's `spec.md` is not edited.

## 3. Behavior

### 3.1 `C-001` carries its owners as data

`Violation` MUST gain an `owners` field: a list of spec ids, defaulting to
empty, omitted from serialization when empty.

`couple_with` MUST populate it on every `C-001` violation with exactly the owner
set that violation's message names, in the same sorted order. No other code path
populates it: `V-*`, `L-*`, `I-*` and `C-002` violations carry an empty list and
therefore serialize exactly as they do today.

The field exists because the owner set is a fact the gate computed and then
destroyed by formatting it into English. A consumer of the spec 037 `--json`
envelope, an orchestrator deciding whether a refusal is a crossing it should
handle or a drift it should escalate, currently has to regex a prose sentence to
recover it. That is the same complaint the audit's item 6 raises about consumers
rebuilding path-to-spec maps from raw edges, and this is its cheapest instance:
the answer is already in hand at the moment the violation is constructed.

`C-002` deliberately keeps an empty `owners` list. It fires precisely when no
spec specifically claims the path, so there is no owner to name, and the floor
specs its message reports are not owners in the sense this field means.

### 3.2 The footer names three doors

When the report contains at least one `C-001`, the resolution footer MUST name
three resolutions, in the order an author should consider them:

1. **Edit the owning spec**, when that spec is the one being authored.
2. **Declare an `extends` edge** in the author's own spec, naming the owning
   spec and the unit, for territory the author does not own. The footer MUST
   state that this amends nobody and needs no waiver.
3. **Add a waiver line**, described as what it is: a human instrument requiring
   explicit approval, not a flag an unattended session sets for itself.

Door two MUST be rendered as pasteable YAML, not described in prose. An author
who has to translate "declare an extends edge" into frontmatter syntax has been
handed a second problem, and the syntax is the part the audit found adopters
getting wrong.

The existing `C-002` guidance ("claim the path in a spec's owning edge") is
unchanged in substance and joins the same numbered rendering. A report
containing only `C-002` violations MUST render the footer it renders today.

### 3.3 The guidance is specific when the diff makes it specific

When the diff contains **exactly one** `<specs_dir>/<id>/spec.md` path, the
footer MUST name that spec and emit a concrete `extends` block for the violating
paths:

```
spec 052-couple-names-the-crossing is the only spec.md edited in this diff,
and owns none of the paths above. Cross into the owning territory by
declaring, in specs/052-couple-names-the-crossing/spec.md:

  extends:
    - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
```

The condition is sound without further tests. Clearance is "any one owner's
`spec.md` is in the diff", so if the single edited spec owned a violating path
that path would already be cleared. Every `C-001` that survives to the footer is
therefore a path the edited spec does not own, which is exactly the crossing
case. The id comes from the path (`spec_id_for_spec_md_path`), so a spec being
filed for the first time, which no committed shard knows about yet, is named
correctly too.

With zero or two or more edited `spec.md` paths, the footer MUST fall back to
the generic form of 3.2. Two edited specs is a legitimate shape (an amendment
pair) and the gate has no basis for guessing which one should declare the edge;
inventing an answer there would be worse than the generic text.

One `extends` item is emitted per violating path, in the report's existing
sorted-by-path order. Where a path has several owners the item names the first
owner id in sorted order, since clearing requires only one; the emitted unit is
the file-shorthand form. Both are a correct starting point rather than the only
correct answer, and a narrower `section` or `symbol` unit remains available to
an author who wants the claim tighter. The footer MUST NOT claim otherwise.

The output is a pure function of the report and the diff. No new input is read,
nothing is written, and the same inputs produce the same bytes.

### 3.4 Nothing committed moves

No committed artifact changes and no schema version moves.

`Violation` appears in the registry schema and the registry shard schema, both
with `additionalProperties: false`. Because `owners` is omitted when empty and
registry violations never carry owners, every emitted shard is byte-identical
and continues to validate against the unchanged schema. `REGISTRY_SCHEMA_VERSION`
therefore stays at `1.1.0` and the schema files are not edited: they describe
the artifact, and the artifact gained nothing.

`VERDICT_SCHEMA_VERSION` stays at `0.2.0`. Spec 050 §3.6 settled this: adding a
member to one verb's `report` payload is additive and must not move a constant
that versions the envelope, because a consumer of a different verb cannot
observe the change. That decision was recorded so that the next spec to add a
payload field would not have to re-derive it. This is that spec, and it follows
it rather than reopening it.

### 3.5 The JSON form carries data, not prose

The `--json` envelope gains `owners` on each `C-001` and nothing else. The
three-door footer is a CLI rendering and MUST NOT appear in the envelope.

Spec 037 draws this line already, and `cmd_couple.rs` records it: the envelope
carries the reasons a consumer needs, and the prose form's breakdown is a
rendering of the same fields rather than a second fact. Guidance text in the
envelope would be a third representation of the same `owners` list, immediately
stale against the prose, and useless to the machine that has the list.

Exit codes are untouched. Every code is identical before and after this spec,
with and without `--json`. This spec changes what a refusal says, never what it
decides.

## 4. Out of scope

**Making `extends` clear the gate.** It already does, by making the extending
spec a legitimate owner of the unit. Nothing in the clearance algorithm changes
here. This spec closes a documentation gap in the refusal, not a gap in the
mechanism, and the reason the gap was expensive is precisely that the mechanism
worked perfectly and silently.

**Writing the `extends` edge for the author.** The gate prints the block; a
person or a session pastes it. A gate that edited the frontmatter of the spec it
is judging would be repairing the tree it is meant to refuse, which is the same
error spec 046 found in three of the kit's hooks and which `compile --check`
exists to avoid on the registry side.

**An owner-of-path query.** The audit's item 6 (`registry owner <path>`, or
`index coverage --by-path --json`) is the general form of the question this spec
answers only at the moment of a refusal. It deserves its own spec, its own
verb and its own output contract. `owners` on the violation is not a substitute
and does not pre-empt the design.

**Naming the owner on `C-002`.** There is none by construction. The floor specs
`C-002` reports are the reason the path is debt rather than the reason it is
refused, and putting them in an `owners` field would make the field mean two
different things depending on the code.

**Guidance for the `--paths-from` mode.** It carries no diff and no spec.md
edits, so 3.3's specific form never triggers there and the generic footer
applies. That is correct rather than a limitation: without a diff there is no
"the spec you are authoring" to name.

**Per-payload schema versioning.** Spec 050 §3.6 declined to open that axis and
this spec does not reopen it.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test couple --locked
cargo test -p spec-spine-cli --test couple --locked
# The envelope version did not move for a payload addition (3.4, spec 050 3.6).
test "$(target/release/spec-spine couple --base HEAD~1 --head HEAD --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["schemaVersion"])')" = "0.2.0"
# Every committed registry shard still validates against the unchanged schema,
# and is byte-identical to what the corpus compiles to (3.4).
target/release/spec-spine compile --check
```
