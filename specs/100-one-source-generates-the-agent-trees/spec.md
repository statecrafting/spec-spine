---
id: "100-one-source-generates-the-agent-trees"
title: "One source generates the agent instruction trees"
status: approved
kind: "tooling"
created: "2026-09-16"
summary: >
  Spec 081's own commit added two tracked instruction trees for the non-Claude
  agents, `.agents/skills/` and `.codex/agents/`, and its section 2 reads "No
  new file". Both were produced by a blind `claude` to `Codex` substitution over
  a copy of the Claude trees, which rewrote `.claude/rules/` into a `.Codex/rules/`
  that exists on no filesystem, rewrote the literal URL shape `claude.ai/code/session_`
  the commit skill forbids into `Codex.ai/code/session_`, and collapsed the pair
  "CLAUDE.md and AGENTS.md" into "AGENTS.md and AGENTS.md". Neither tree is
  claimed by any spec, neither is covered by an `[index] extra_hashed_inputs`
  glob, and `.agents/skills/` still carries the five skills spec 081 removed,
  unchanged since that commit. This spec makes both trees generated output of one
  script with a parity test, in the shape `scripts/gen-kit-embedded.py` already
  uses, and claims them. It delivers parity only: no global installation,
  upgrade, pinning, compatibility floor or recorded resolved identity.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "048-kit-ships-the-governed-loop-skills"
  - "057-claimed-but-unwitnessed"
  - "065-init-and-the-kit-are-one-adoption"
  - "081-the-kit-ships-what-the-loop-calls"
establishes:
  # 3.1: the one generator. `scripts/*.py` is already a hashed glob.
  # 3.2, 3.5: the two trees, tracked and unowned since f5efb23. `index owner`
  # answered "(no spec owns this path)" for both when this spec was filed.
  # 3.6: the parity acceptance, in its own file rather than inside the test
  # spec 048 establishes.
extends:
  # 3.1: the generator becomes the sanctioned writer of this repository's skill
  # tree, which spec 048 3.3 established and pins byte-identical to the kit's.
  # No byte of it changes here; what changes is who may write it.
  - { spec: "048-kit-ships-the-governed-loop-skills", unit: ".claude/skills/", nature: additive }
  # 3.5, D-3: the two trees join the hashed-inputs list, because L-008 refuses a
  # claim on a file that is in no content hash.
  - { spec: "057-claimed-but-unwitnessed", unit: "spec-spine.toml", nature: additive }
references:
  # 3.1: read as a generation source and never written. The transform to
  # `.codex/agents/*.toml` is structural; spec 048 keeps owning the prose.
  - { unit: { kind: file, path: ".claude/agents/" }, role: "generation source" }
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/06-harness-and-distribution-2026-09.md" }, role: context }
---

# 100: One source generates the agent instruction trees

## 1. Purpose

### 1.1 Two trees nobody owns, carrying text nobody wrote

Commit `f5efb23` ("feat(081): the kit ships what the loop calls") added
`.agents/skills/` (fifteen `SKILL.md` files) and `.codex/agents/` (four
`.toml` files) to the repository. Spec 081's section 2 reads "No new file".
Measured on 2026-09-16 at `a6ef6e3`, neither tree has been touched since.

Both are copies of the Claude trees with a case-insensitive `claude` to `Codex`
substitution applied to every occurrence, including occurrences that are not
product names. The result is text that no author wrote and that no reader can
follow:

| Source text | What the substitution produced | Why it is wrong |
|---|---|---|
| `.claude/rules/` | `.Codex/rules/` | No such directory exists, here or in any corpus `spec-spine init` scaffolds. The rules are at `.claude/rules/` for every agent that reads them. |
| `claude.ai/code/session_` | `Codex.ai/code/session_` | The commit skill forbids that literal URL shape in a commit message. The rewritten form forbids a shape nothing emits, so the rule is inert in the copy. |
| `CLAUDE.md` and `AGENTS.md` | `AGENTS.md` and `AGENTS.md` | Two reads collapsed into one repeated read. |
| `.claude/agent-memory/` | `.Codex/agent-memory/` | A memory path that resolves nowhere. |

The substitution is not a transform with a rule behind it; it is a defect. This
spec's position is that a generated tree is a **projection of its source**, and
that the only text a generator may change is text whose form the destination
requires.

### 1.2 The drift the absence of a generator produced

Measured on 2026-09-16, against `kit/.claude/skills/`, which spec 048 section 3.3
and spec 081 section 3.1 make this repository's skill source:

- `.agents/skills/` holds **fifteen** skills. Spec 081 reduced the set to
  **ten**. `cleanup`, `implement-plan`, `refactor-claude-md`, `research` and
  `validate-and-fix` were deleted from the kit and from `.claude/skills/` and
  remain here.
- Of the ten that survived, **nine** differ from the kit copy. `spec` is the
  only one still byte-identical, and only because it happens to contain no
  substitutable token.
- `shepherd` differs by 105 diff lines: every change specs 082, 085 and 099
  made to the shipped skill is missing from this copy.

Nothing detects any of this. No spec claims either tree, so `couple` has no
owner to name; no `extra_hashed_inputs` glob covers either tree, so no shard
stales when their bytes change; and `C-002` reaches only
`coverage.rs::SOURCE_EXTS`, which does not include `.md` or `.toml`, so the
ownership ratchet is silent as well. All fifteen `.agents/skills/` files were
once deleted by accident with `couple` reporting no drift, which is the
measurement design note 05 section 3 records.

### 1.3 Why generation rather than deletion

Design note 05 section 3 B5 records the maintainer's ruling of 2026-09-12:
generate the supported trees from one source with a parity test, in the shape
`kit_embedded.rs` already uses, **not** delete the tree. Spec 082 D-3 left the
tree alone for the stated reason that adopting it would be "a decision about
whether that tree should exist at all, and that decision is not this spec's".
It is this spec's, and the decision is already recorded: the trees stay, and
they stop being hand-maintained.

### 1.4 What this spec is not

Design note 05 section 3 B5 is explicit and this spec repeats it: **B5 is
parity, and parity only.** Shipping this delivers no global installation, no
`repo upgrade`, no version pinning, no compatibility floor and no recorded
resolved identity. Those are design note 06 sections 3.2 and 3.4, they are
unfiled, and nothing here may be reported as having delivered them.

## 2. Territory

One new script, one new test, two claimed trees, one configuration edit:

| Unit | Edge | Why |
|---|---|---|
| `scripts/gen-agent-trees.py` | `establishes` | The one generator. |
| `.agents/skills/` | `establishes` | Generated output, unowned until now. |
| `.codex/agents/` | `establishes` | Generated output, unowned until now. |
| `crates/spec-spine-core/tests/agent_trees.rs` | `establishes` | The parity acceptance. |
| `.claude/skills/` | `extends` 048, additive | Becomes generated output. No byte changes. |
| `spec-spine.toml` | `extends` 057, additive | The hashed-input globs the claims oblige. |
| `.claude/agents/` | `references` | Read as a generation source, never written. |

`.codex/hooks.json` is spec 099's and is untouched. `kit/**` is untouched:
the kit is the source, not a destination.

## 3. Behavior

### 3.1 One script writes every generated agent tree

`scripts/gen-agent-trees.py` MUST be the only sanctioned writer of the trees in
the table below, and MUST derive each destination file from exactly one source
file:

| Destination | Source | Transform |
|---|---|---|
| `.claude/skills/<name>/SKILL.md` | `kit/.claude/skills/<name>/SKILL.md` | identity |
| `.agents/skills/<name>/SKILL.md` | `kit/.claude/skills/<name>/SKILL.md` | identity |
| `.codex/agents/<name>.toml` | `.claude/agents/<name>.md` | structural, section 3.3 |

The script MUST accept `--check`, which writes nothing and exits non-zero
naming every destination whose bytes differ from what it would write. `--check`
is what the test and a reviewer run; the bare invocation is what an author runs
after editing a source.

The script MUST remove a destination file that no source file maps to, so a
skill deleted from the kit is deleted from every generated tree. This is the
clause that would have removed the five skills of section 1.2 when spec 081
removed them.

### 3.2 A generated tree is a projection, never a substitution

The script MUST NOT rewrite any identifier, path, product name or URL in the
text it copies. Specifically, it MUST NOT rewrite `claude` to `Codex` in any
casing, and it MUST NOT rewrite `.claude/` to anything.

The rules a skill cites live at `.claude/rules/`, which is where
`spec-spine init` scaffolds them and where they are on this filesystem, for
every agent that reads them. A path is a fact about a filesystem, not a brand.
The three other rewrites section 1.1 tabulates have no defensible form at all.

The generated trees MUST therefore contain no occurrence of `.Codex/`, and no
occurrence of `Codex.ai`.

### 3.3 The `.codex/agents` transform is structural and nothing else

A Claude agent definition is markdown with a YAML frontmatter block. A Codex
agent definition is TOML. The script MUST produce, from
`.claude/agents/<name>.md`:

```toml
name = "<the frontmatter `name`>"
description = "<the frontmatter `description`>"
developer_instructions = """
<the markdown body, verbatim, with the frontmatter block removed>"""
```

The keys Codex has no use for (`tools`, `model`, `safety_tier`, `mutation`,
`memory`) are dropped rather than translated: this spec does not invent a Codex
meaning for a Claude field.

The body is carried **verbatim**, and a TOML multi-line basic string can only
carry it verbatim when it holds neither of the two sequences the format would
reinterpret. The script MUST refuse, naming the source file, when the body:

- contains a backslash, which a basic string decodes as an escape. This is the
  subtler of the two: `\n` written as text in an instruction file decodes to a
  newline, and a backslash before a line ending swallows the line break.
- contains a `"""` sequence, which ends the string early wherever it appears,
  the closing delimiter included.

A body **ending** in `"` or `""` is **not** refused. TOML 1.0 allows one or two
quotation marks immediately before the closing delimiter, so `abc"` emits as
`"""abc""""` and decodes back to `abc"`. Three in a row are the only
unencodable case and the clause above already covers them, wherever they sit.

Refusing is correct because no body contains either sequence today (measured
2026-09-16), so an escape path would be untested code on a route nothing
exercises, and a wrong escape produces TOML that parses into something other
than the source text. With both refused, the decode is provably the identity,
which is what lets section 3.6 assert round-trip fidelity.

`.claude/agents/` is a **source** and is not written by the script. Spec 048
keeps owning its prose. What this spec adds is an obligation running the other
way: an edit to `.claude/agents/<name>.md` leaves `.codex/agents/<name>.toml`
stale, and section 3.6's test is what says so.

### 3.4 The skill set of the generated trees is the kit's

`.agents/skills/` MUST hold exactly the skills `kit/.claude/skills/` holds,
which spec 081 section 3.1 fixes at ten. The five skills of section 1.2 are
removed by this change.

### 3.5 A claimed tree is a hashed tree

All three generated trees, `.claude/skills/*/SKILL.md`,
`.agents/skills/*/SKILL.md` and `.codex/agents/*.toml`, MUST be added to
`[index] extra_hashed_inputs` in `spec-spine.toml`. `L-008` (spec 057) refuses
a claim on a file that is in no content hash, and these are governance files of
exactly the kind the glob remedy is right for: a handful, rarely changed, and a
change to one should stale the ledger. The cost is that every shard is
restamped by this change, which is the documented price and not a surprise.

`.claude/skills/` is included even though its source `kit/.claude/skills/*/*.md`
is already hashed, because the two are reachable independently. An edit that
goes through the kit stales the ledger at the source; a direct edit to
`.claude/skills/<name>/SKILL.md` matches no glob at all, and `C-002` does not
reach it either, since `.md` is outside `coverage.rs::SOURCE_EXTS`. Without its
own entry, the one generated tree a Claude session actually reads would be the
only one whose corruption stales nothing, with §3.6's test as the sole detector
and only when it is run.

### 3.6 The acceptance compares the trees, not the script

`crates/spec-spine-core/tests/agent_trees.rs` MUST assert, reading the
repository's own files rather than re-running the generator:

- every `kit/.claude/skills/<name>/SKILL.md` is byte-identical to
  `.claude/skills/<name>/SKILL.md` and to `.agents/skills/<name>/SKILL.md`;
- the three skill directories hold the same set of names, and that set is spec
  081 section 3.1's ten;
- every `.claude/agents/<name>.md` has a `.codex/agents/<name>.toml`; that file
  parses as TOML, its `name` and `description` are the source frontmatter's, and
  its `developer_instructions` **round-trips**: the decoded value equals the
  source body with the frontmatter block removed, byte for byte. Asserting the
  decoded value rather than the file text is what catches section 3.3's escape
  hazard, because a body that TOML reinterprets produces a file that still looks
  right and decodes wrong.
- no generated file in any of the three trees contains `.Codex/` or
  `Codex.ai`. Asserting it on `.claude/skills/` as well as the other two is
  not redundant with the byte-identity assertion above: if a banned string
  ever reached the kit source, identity would still hold and only this
  assertion would name what went wrong.

A test that re-runs the generator and compares its output to itself proves
nothing about what is committed, which is the failure mode the committed
`.derived/` trees exist to avoid. The assertion is against the bytes on disk.

## 4. Out of scope

- **Distribution.** Whether `init --with-kit` writes `.agents/skills/` or
  `.codex/agents/` into an adopter's repository. The kit ships
  `.claude/skills/` and `kit_embedded.rs` is generated from `kit/**`; extending
  that to the non-Claude trees is design note 06 sections 3.2 and 3.4, with
  open questions H-1 to H-3 unanswered. Section 1.4.
- **`.claude/agents/` and `kit/.claude/agents/` differing.** They differ
  deliberately: the kit's are adopter-generic, this repository's name its own
  crates. That is spec 048's design and this spec reads it rather than changing
  it.
- **The markdown blind spot in `C-002`.** Spec 097 shipped the opt-in
  `[coverage] governed_scope` and this repository has not enabled it. Claiming
  these trees here does not enable it and does not depend on it; the hashed-input
  globs of section 3.5 are what make a change to them visible.
- **Whether `.codex/hooks.json` fires at all.** Codex loads hooks from
  `CODEX_HOME`, so a project-local `.codex/hooks.json` may never run. That is a
  question about spec 099's territory and about distribution, and it is not
  answered by making a sibling tree generated.
- **`/shepherd`'s `issues/<n>/comments` gap and its green-path routing.** Design
  note 05 section 3 B4 and spec 082 D-3's last paragraph. Both are edits to the
  skill source, which this spec only copies.

## 5. Resolved decisions

D-1 (2026-09-16, why `kit/.claude/skills/` is the skill source rather than
`.claude/skills/`). They are byte-identical and spec 048 section 3.3 pins them
so, which makes the choice free today. The kit copy is the one spec 081 section
3.1 counts, the one `kit_embedded.rs` is generated from, and the one
`extra_hashed_inputs` already covers, so it is the copy that is already
load-bearing in three places. Naming the other one would create a second source
of truth for the same bytes.

D-2 (2026-09-16, why `.claude/agents/` is the `.codex/agents/` source rather
than `kit/.claude/agents/`). They are not identical, and the existing
`.codex/agents/*.toml` were projected from this repository's copy: the
architect definition names `crates/spec-spine-core` and `crates/spec-spine-cli`,
which the kit's generic copy does not. Switching the source would rewrite four
files with adopter-generic prose in a repository that is not an adopter.

D-3 (2026-09-16, why a glob rather than a section unit for section 3.5). Spec
074 section 3.2 makes the two remedies a real choice: a glob folds the file into
the global scalar and restamps every shard, which is right for a few governance
files and wrong for a tree of source. Fourteen instruction files that change
only when a skill changes are the first case, and the sibling entries for
`kit/.claude/skills/*/*.md` and `.codex/hooks.json` are already there.

D-4 (2026-09-16, why the script refuses a body TOML would reinterpret rather
than escaping it). Section 3.3. No body contains a backslash or a `"""` today,
so an escape path would be untested code on a route nothing exercises, and a
wrong escape produces TOML that parses into something other than the source
text. A refusal is loud, correct, and cheap to replace with a specified escape
the day a body needs one.

Both clauses moved during the build, in opposite directions. The backslash
clause was **added**: the first draft refused only quote sequences, which left
the escape that changes a body's meaning without changing its appearance. A
separate refusal of a body **ending** in `"` was **removed**, because its
stated reason was false. TOML 1.0 permits one or two quotes before the closing
delimiter, so such a body is valid and unambiguous; verified against a TOML
parser on 2026-09-16, where one and two trailing quotes decode exactly and only
three fail. Keeping a refusal whose justification is wrong is worse than not
having it: the next reader either trusts a false claim about the format or
removes the clause without knowing which cases it was really holding.

D-5 (2026-09-16, why a basic string rather than a TOML literal string). A
literal (`'''`) string processes no escapes, which would make the three
refusals of section 3.3 unnecessary. Two reasons not to: the four committed
files already use a basic string, so switching rewrites bytes for no behavioral
gain; and a literal string has its own unencodable sequences (`'''`, a trailing
`'`), so the refusal does not disappear, it only changes which characters it
names. With the refusals in place the two encodings are equivalent, and the one
already on disk wins.

## Verification

Each line is one command (spec 049 section 3.2). The first five fail against
pre-100 code: the script does not exist, `.agents/skills/` holds fifteen
directories nine of which differ from the kit, and both generated trees contain
`.Codex/`. The `cargo test` line also fails at the parent commit, because the
test target `agent_trees` does not exist there and `cargo test --test` refuses
an unknown target rather than passing vacuously.

```verify:cli
# 3.1: the generator is present and every generated tree is what it would write.
python3 scripts/gen-agent-trees.py --check
# 3.4: exactly the ten skills of spec 081 3.1, in the generated tree.
test "$(ls -1d .agents/skills/*/ | wc -l | tr -d ' ')" = 10
# 3.1, 3.6: the skill bytes are the kit's, in both generated trees.
for s in build code-review commit next prime setup shepherd ship spec verify; do cmp "kit/.claude/skills/$s/SKILL.md" ".agents/skills/$s/SKILL.md" || exit 1; cmp "kit/.claude/skills/$s/SKILL.md" ".claude/skills/$s/SKILL.md" || exit 1; done
# 3.2: the substitution is gone and does not come back.
sh -c '! grep -rqF ".Codex/" .agents/skills .codex/agents'
sh -c '! grep -rqF "Codex.ai" .agents/skills .codex/agents'
# 3.6: the acceptance.
cargo test -p spec-spine-core --test agent_trees --locked
```

D-6 (2026-09-16, why `.claude/skills/` is hashed although its source already
is). Added during the build, from a review finding. The first draft of §3.5
listed only the two trees this spec claims, on the reasoning that an edit to a
generation source is caught at the source. That reasoning covers edits that
travel through the kit and misses the ones that do not: `.claude/skills/` is a
real directory a session can edit directly, and such an edit matched no glob
and no `C-002` check. The three generated trees now have one enforcement story
instead of two.
