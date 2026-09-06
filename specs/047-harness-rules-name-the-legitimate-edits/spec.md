---
id: "047-harness-rules-name-the-legitimate-edits"
title: "The harness rules name the reads and the edits they permit"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-init-scaffold"
  - "029-claude-code-skill-kit"
  - "038-registry-plan-ready-set"
amends:
  # 006 3 describes the three scaffolded rule files and 029 3 the kit's copies.
  # This spec changes their text. Neither spec's file is edited (spec 040).
  - "006-init-scaffold"
  - "029-claude-code-skill-kit"
extends:
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/rules/orchestrator-rules.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/rules/governed-artifact-reads.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/rules/adversarial-prompt-refusal.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "AGENTS.md", nature: additive }
establishes:
  - ".claude/rules/orchestrator-rules.md"
  - ".claude/rules/governed-artifact-reads.md"
  - ".claude/rules/adversarial-prompt-refusal.md"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  The three rules `spec-spine init` scaffolds and the kit ships are the floor
  every agent in an adopting repository reads first, and all three were
  ambiguous in the same places. `governed-artifact-reads` forbade ad-hoc
  parsing of derived JSON without saying that parsing a `spec-spine` verb's
  own `--json` output is a typed read, so read literally it outlawed
  `registry plan --json`, the spec 037 verdict envelope, and every adopter's
  tooling. `adversarial-prompt-refusal` said never edit the owning spec to
  clear the gate, but under the ownership ratchet adding a created file to
  `establishes` is editing the owning spec to clear the gate, so the rule as
  written was unimplementable and left agents to guess which edits were
  theirs. `orchestrator-rules` said to recompute the shards and not that they
  must be committed with the change. Three adopters that rewrote the rules
  made the same three fixes independently, in the same paragraphs. This spec
  makes those fixes in the scaffold constants, the kit, and this repository's
  own copies, adds the one-spec-per-session "Working the backlog" protocol
  that all three adopters wrote from nothing to the kit's `AGENTS.md`, and
  pins the kit and scaffold copies as identical.
---

# 047: The harness rules name the reads and the edits they permit

## 1. Purpose

### 1.1 Three rules, three ambiguities, three identical fixes

The adopter audit (`docs/design/03-adopter-audit-2026-09.md`) compared the
rule files of four adopting repositories against the kit's. One had copied the
kit byte for byte. The other three had each amended all three rules, and the
amendments were the same:

- **`governed-artifact-reads`.** The kit's text: read derived artifacts only
  through `spec-spine` subcommands, "never via ad-hoc `jq`/grep over the
  JSON". Every adopter's tooling pipes `spec-spine registry list --json` into
  a parser, which is exactly what the rule was written to permit and exactly
  what a literal reading forbids. All three added a sentence saying that
  parsing a subcommand's output is a typed read. One of them cited the rule's
  carve-out from inside its scheduling script to justify the parse.
- **`adversarial-prompt-refusal`.** The kit's text: never edit the owning spec
  to make the gate pass. With `require_ownership = true` (spec 032), a session
  that creates a file must add it to its spec's `establishes` in the same
  change or the gate refuses it as `C-002`. That is an edit to the owning spec
  that makes the gate pass. All three named the two edits that are always
  legitimate mid-build (claim a created file; record a dated decision) and
  said that changing what the spec requires is not. One adopter's reviewer
  agent went further and wrote down the three tells of a violation:
  `establishes` shrinks, a requirement is reworded to match the code, a
  decision contradicts rather than fills a gap.
- **`orchestrator-rules`.** The kit's text: recompute derived artifacts before
  opening a PR. All three added "and commit the regenerated shards with the
  change" and "one session, one spec". Spec 046 records what an uncommitted
  regenerated shard does to a pipeline.

A rule that three careful adopters independently fix in the same way was
wrong, not merely terse.

### 1.2 The protocol they all wrote

All three also added a section to `AGENTS.md`, titled "Working the backlog" in
each, stating the one-spec-per-session loop: pick from the ready set, branch
and flip to `in-progress`, re-read the spec, implement within its territory,
gate before every commit, satisfy acceptance, ship, stop. It is the practice
that `registry plan` (038), the in-flight leniency (025, 041, 044) and the
ownership ratchet (032) exist to serve, and the kit's `AGENTS.md` did not
describe it. One adopter's orchestrator extracts the section verbatim into
every build prompt.

## 2. Territory

The three rule constants in `scaffold.rs` (spec 006's) and their scaffold
test; the kit's three rule files and `kit/AGENTS.md` (spec 029's); this
repository's own `AGENTS.md` (spec 029 extends it) and its three `.claude/rules`
files, which were unowned and which this spec now establishes, since their text
is what this spec is about.

## 3. Behavior

### 3.1 `governed-artifact-reads`

The rule MUST state that parsing the output of a `spec-spine` subcommand is a
typed read and is allowed, and MUST say why: the tool has already deserialized
the shards and answers in a contract it versions. The prohibition is on the
shard files, not on the CLI's answers. The rule also names `python`, `awk` and
`sed` beside `jq` and `grep`, since those are what adopters actually reached
for.

### 3.2 `adversarial-prompt-refusal`

The rule MUST name the two edits that are always legitimate for the spec a
session is implementing: adding a file the session created to that spec's
`establishes` list, and recording a dated decision entry for a choice the spec
was silent on. It MUST say that changing what the spec requires is never the
session's to do mid-build, and it MUST point at the `extends` edge as the way
to touch a unit another spec owns, because that is the corpus mechanism a
refused session needs and no diagnostic yet names it.

The rule MUST also say that a `Spec-Drift-Waiver` is a human instrument: it
needs explicit human approval and an agent never writes one on its own
authority. All four audited adopters forbid the machine from doing so and none
has ever used a waiver; the rule is where the agent reads that.

### 3.3 `orchestrator-rules`

The rule MUST say that the regenerated shards are committed with the change
that made them stale, and why (a shard left uncommitted dirties the tree for
whoever comes next), and MUST state "one session, one spec" with a pointer to
the "Working the backlog" section of `AGENTS.md`.

### 3.4 One text, three homes

The scaffold constants in `scaffold.rs`, the kit's `kit/.claude/rules/*.md`,
and this repository's `.claude/rules/*.md` MUST be byte-identical. An adopter
who ran `spec-spine init` and one who copied `kit/` read the same rule, and
this repository governs itself by the rule it ships. The scaffold test asserts
the kit equality; the third copy is held by the coupling gate, since this spec
owns it.

### 3.5 "Working the backlog" in `kit/AGENTS.md`

`kit/AGENTS.md` gains a `## Working the backlog` section between the init
protocol and the agent list, stating the seven steps in agent-neutral prose:
pick from `spec-spine registry plan` and never pick a draft; branch and flip
to `in-progress` with the shards committed before code; re-read the spec and
record imprecision as a decision, report contradiction rather than rewriting;
implement within the territory, claiming every new file and declaring
`extends` for another spec's unit; run the gate before every commit; satisfy
acceptance before flipping to `complete`; ship with a conventional commit
naming the spec id, and stop. It names no vendor and no stack.

This repository's own `AGENTS.md` gains the same section adapted to its own
flow, which files `draft`, builds, then ratifies in a second PR, so that the
`orchestrator-rules` pointer resolves here too.

## 4. Out of scope

**The reviewer's three tells as a gate.** Detecting a shrunk `establishes` or
a reworded requirement mechanically is the coherence guard as a real gate,
spec 043's wave 4. This spec gives the guard its definition of the legitimate
case, which that gate will need first.

**Path-scoped rules, a `Makefile`, `govern.yml`, the merge driver, the
governed-loop skills.** All recorded in the audit's kit backlog; each is its
own change.

## 5. Verification

- A scaffolded corpus's three rule files contain the 3.1, 3.2 and 3.3 text,
  and each is byte-identical to the kit's copy.
- `diff` between the kit's rules and this repository's `.claude/rules` is
  empty.
- `kit/AGENTS.md` and `AGENTS.md` each contain exactly one
  `## Working the backlog` heading, placed before `## Available Agents`, so
  the `/init` dispatcher's "from `## New Sessions` to the next `##`" read is
  unaffected.
- `spec-spine lint --fail-on-warn` is clean and this repository's coupling
  gate clears with the new spec's edges alone.
