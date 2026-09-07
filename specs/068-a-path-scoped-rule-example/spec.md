---
id: "068-a-path-scoped-rule-example"
title: "A path-scoped rule the kit actually ships"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "029-claude-code-skill-kit"
  - "047-harness-rules-name-the-legitimate-edits"
  - "048-kit-ships-the-governed-loop-skills"
extends:
  - { spec: "029-claude-code-skill-kit", unit: "kit/.claude/rules/", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/README.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: ".claude/rules/governed-artifact-reads.md" }, role: context }
summary: >
  The kit's three rules are unconditional: every one loads in every session
  regardless of what the session touches. Claude Code supports `paths:`
  frontmatter that loads a rule only when a matching file is touched, and the
  kit's README lists that pattern under "intentionally excluded" while hqgit
  carries three good ones, including per-file scoping. The exclusion was a
  judgement made before any adopter had tried it, and an adopter has now tried
  it and kept it. This spec ships one path-scoped rule, scoped to the derived
  artifact tree, whose content is the one instruction that is only ever relevant
  when someone has a compiled artifact open: do not hand-edit it. One rule, as
  a worked example of the mechanism, with the README's exclusion replaced by a
  description of when to reach for it.
---

# 068: A path-scoped rule the kit actually ships

## 1. Purpose

`kit/.claude/rules/` holds three files: `orchestrator-rules`,
`governed-artifact-reads`, `adversarial-prompt-refusal`. All three load in every
session, always, because that is the only mode the kit uses.

Claude Code supports another. A rule file with `paths:` frontmatter loads only
when the session touches a matching file. The kit's README lists that pattern
under **intentionally excluded**, which was a defensible call when the kit was
first assembled and nobody had used one. It is no longer the state of the
evidence: hqgit carries three path-scoped rules, including one scoped to a
single file, and kept them.

The cost of the omission is not that adopters cannot write one. It is that the
kit teaches, by example, that rules are unconditional, and the README actively
says the alternative was excluded rather than saying when to use it. An adopter
reading both concludes the mechanism is discouraged.

There is also a fit. Unconditional rules are the right shape for the three the
kit ships, which are about how to work at all. The rule that does not fit that
shape is the one about compiled artifacts: "do not hand-edit a derived artifact,
and do not parse one with `jq`" is advice that matters only when somebody has a
derived artifact open, and it currently loads in every session about anything.
That is a small waste of context and, more to the point, it is the worked
example sitting in the kit already.

This is item 10 of the adopter audit's ranked backlog for the kit.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `kit/.claude/rules/` | 029 | one new rule file |
| `kit/README.md` | 029 | the exclusion becomes guidance |

The new rule file is created by the implementing change and claimed there. The
three existing rules are **not** edited: spec 047 tuned their wording across the
scaffold constants, the kit copy and this repository's copy, and reopening them
here would put a fourth hand on text three specs have settled.

## 3. Behavior

### 3.1 One path-scoped rule

The kit MUST ship exactly one rule file carrying `paths:` frontmatter, scoped to
the derived artifact tree:

```markdown
---
paths:
  - ".derived/**"
---

# Derived artifacts are compiler output
...
```

Its content is the artifact-specific half of what
`governed-artifact-reads.md` says: a derived artifact is compiler output,
hand-editing one is a workflow violation, reading one with `jq` or `sed` is
equally forbidden, and the way to read it is a `spec-spine` subcommand. Parsing
the **output** of a subcommand is a typed read and is allowed, which is the
clarification spec 047 added and which is the sentence adopters most needed.

The `paths` glob MUST be derived from `layout.derived_dir` when the kit is
installed through `init --with-kit` (spec 065), the same way every other
scaffolded file is config-aware. A repository with a non-default derived
directory gets a rule that matches it.

**One, not three.** The value here is the worked example and the fit; a kit that
shipped a directory of conditional rules would be making adopters read scoping
decisions that are theirs to make. hqgit has three because hqgit has three
domains that want them.

### 3.2 The unconditional rule stays

`kit/.claude/rules/governed-artifact-reads.md` MUST remain unconditional and
MUST keep its full content.

This is the part worth getting right. The path-scoped rule is not a replacement
and MUST NOT be described as one. A rule that loads only when `.derived/**` is
touched cannot prevent the mistake it is about, because the mistake is reaching
for `jq` **instead of** the subcommand, and an agent that reaches for `jq` may
never touch a path the glob matches. The unconditional rule is what prevents
that. The scoped rule is what reinforces it at the moment somebody has the file
open.

Saying this in the rule file itself is required, not optional. A reader
comparing the two files will otherwise conclude one is redundant, and the
redundant-looking one is the one that does the work.

### 3.3 The README describes when to scope

`kit/README.md` MUST replace the "intentionally excluded" entry for `paths:`
frontmatter with a short description of when a scoped rule is right:

- **unconditional** when the rule is about how to work at all: the loop, the
  refusals, the standing constraints;
- **scoped** when the rule is only actionable while a particular kind of file is
  open, and when loading it always would spend context on a case most sessions
  never reach;
- and the caveat from §3.2: a scoped rule cannot prevent a mistake whose whole
  shape is not touching the path.

It MUST name the shipped rule as the example and MUST note that hqgit's
three, including a per-file scoping, are the field evidence.

### 3.4 It is exercised here

Spec 046 found three kit hooks that wrote when they should read, and the reason
was that this repository had no `.claude/settings.json` and never ran them. Spec
051 made the harness run the verbs it ships.

So this repository MUST carry the same path-scoped rule under its own
`.claude/rules/`, and `tests/kit_skills.rs` (or its successor) MUST assert that
the kit's copy and this repository's copy agree, as it already does for the
unconditional three. A rule shipped to adopters and never loaded here is the
third instance of a mistake this corpus has now made twice.

### 3.5 Nothing mechanical changes

No code path reads a rule file. No committed artifact changes shape and no
schema version moves. `kit/` and `.claude/rules/` are claimed territory, so the
new files are claimed and the index gains their paths; the implementing change
regenerates and commits the shards.

## 4. Out of scope

**Rewriting the three unconditional rules.** §2. Specs 047 and 048 settled them.

**Splitting `governed-artifact-reads.md`.** §3.2. It keeps its full content and
the scoped rule reinforces rather than replaces it.

**A rules taxonomy.** One example and three sentences of guidance. A general
framework for when to scope a rule is a document nobody asked for.

**Path-scoped agents or skills.** The `paths:` mechanism is a rule feature and
this spec uses it as one.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test kit_skills --locked
# The kit ships exactly one path-scoped rule, and it is scoped to .derived.
test "$(grep -rl '^paths:' kit/.claude/rules/ | wc -l | tr -d ' ')" = "1"
grep -rq '.derived' kit/.claude/rules/
# The unconditional rule kept its full content, including the 047 clarification.
grep -q 'typed read' kit/.claude/rules/governed-artifact-reads.md
# The README describes when to scope instead of excluding the pattern.
grep -q 'paths:' kit/README.md
! grep -q 'intentionally excluded.*paths' kit/README.md
# This repository loads what it ships, and the shards were regenerated.
test "$(grep -rl '^paths:' .claude/rules/ | wc -l | tr -d ' ')" = "1"
target/release/spec-spine index check
```
