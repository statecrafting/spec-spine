---
id: "065-init-and-the-kit-are-one-adoption"
title: "Init and the kit are one adoption"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "006-init-scaffold"
  - "029-claude-code-skill-kit"
  - "043-governance-document-gaps"
  - "061-the-scaffold-ships-what-adopters-wrote"
extends:
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-cli/src/cmd_init.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-cli/tests/init.rs", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/README.md", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # The kit's skills, agents and settings are the source the embedded module is
  # generated from, so they join the hashed-input set: a change to one must
  # stale the ledger rather than leave the compiled-in copy silently ahead.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "spec-spine.toml", nature: additive }
establishes:
  # Created by this spec (3.2): the generated module and its generator.
  - "crates/spec-spine-core/src/kit_embedded.rs"
  - "scripts/gen-kit-embedded.py"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/adoption-guide.md" }, role: context }
summary: >
  Adoption is two halves that nothing joins. `spec-spine init` writes a corpus,
  a constitution, a contract, templates and three agent rules. `kit/` adds the
  session protocol, the hooks, the agents and fifteen skills. The scaffold does
  not mention the kit, the kit is not installable by any verb, and `AGENTS.md`,
  which every skill in the kit reads first and which the three rewriting
  adopters each wrote from nothing, is written by neither. Spec 043 named this
  gap G5 and did not close it. This spec adds `spec-spine init --with-kit`,
  emitting the kit's files through the same `Scaffold` data path as everything
  else, and makes plain `init` write an `AGENTS.md` naming the governed loop,
  because a repository with rules and no protocol is the configuration three
  adopters had to repair by hand.
---

# 065: Init and the kit are one adoption

## 1. Purpose

Ask what a new adopter has to do, and the answer is a sequence nobody wrote
down.

`spec-spine init` gives them a corpus, `standards/`, a bootstrap spec, and three
`.claude/rules/` files. That is a governed repository with no instructions for
working in it.

`kit/` gives them the instructions: `AGENTS.md` with the New Sessions protocol
and the Working the backlog protocol, `settings.json` with four hooks, four
agents, fifteen skills. It is a directory in this repository's git tree. There
is no verb that installs it. An adopter copies it, or does not know it exists.

Nothing connects the two. `init`'s output never mentions the kit. The kit's
README assumes a corpus that `init` produced but cannot produce one. And
`AGENTS.md`, which is the cross-agent authority every skill reads first and the
file the AAIF standard defines, is written by neither half: three adopters each
wrote one from nothing, and each wrote a "Working the backlog" section, which is
why spec 047 had to add that section to the kit after the fact.

Spec 043 named this G5 and did not close it. The audit lists it as item 3 of the
kit backlog. It is the difference between a tool that scaffolds a corpus and a
tool that scaffolds an adoption.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/scaffold.rs` | 006 | the kit files as scaffold data |
| `crates/spec-spine-cli/src/cmd_init.rs` | 006 | the `--with-kit` flag |
| `crates/spec-spine-core/tests/scaffold.rs` | 006 | acceptance |
| `crates/spec-spine-cli/tests/init.rs` | 006 | end-to-end acceptance |
| `kit/README.md` | 029 | says how it is installed |

Spec 006 owns the scaffold and requires that `init` return files as data which
the CLI writes. That contract is what makes this change small: the kit becomes
more entries in the same `Vec<ScaffoldFile>`, written by the same code, with the
same `overwrite: false` semantics.

## 3. Behavior

### 3.1 Plain `init` writes an `AGENTS.md`

`scaffold_init` MUST emit `AGENTS.md`, unconditionally, whether or not
`--with-kit` is given.

It is not a kit extra. It is the cross-agent authority that every governed
repository needs and that this repository, the kit, and all four adopters treat
as the first file an agent reads. A scaffold that writes three `.claude/rules/`
files and no protocol has written the constraints without the procedure.

The scaffolded `AGENTS.md` MUST contain:

- **New Sessions**, the init protocol, with the parallel reads named as the
  `spec-spine` verbs they are, `compile --check` and `index check` and not their
  writing forms, and with the freshness verdicts spelled out per exit code;
- **Working the backlog**, the pick-branch-implement-gate-verify-ship-stop
  sequence spec 047 added to the kit;
- **The gate chain**, in order, once, as the definition every skill refers to;
- **the ask-the-version precondition** from spec 063.

Config-aware, as everything in the scaffold is: the specs directory, the derived
directory and the binary invocation come from `Config` so a non-default layout
scaffolds coherently.

### 3.2 `init --with-kit`

`init` MUST accept `--with-kit`, adding the kit's files to the same `Scaffold`:
`.claude/settings.json`, `.claude/agents/*`, `.claude/skills/*/SKILL.md`,
`.mcp.json`, and, when spec 064 has landed, `Makefile` and the
`.github/workflows/govern.yml` starting point.

They are emitted at the adopter's own paths, not under a `kit/` directory. The
`kit/` prefix is this repository's storage location for the templates; an
adopter's `.claude/skills/build/SKILL.md` is where the file has to be to work.

The three `.claude/rules/` files that plain `init` already writes are **not**
duplicated. They are the same three files the kit carries, which spec 047 kept
in sync across the scaffold constants, the kit copy, and this repository's copy.

**Purity holds.** `scaffold_init` stays a pure function of `Config` performing
no IO, so the kit's contents are embedded as `const`s. A test MUST assert that
the embedded constants and the checked-in `kit/` tree agree, so a change to one
cannot silently diverge from the other, and it is what keeps `kit/` the editable
source rather than a copy nobody remembers to update.

**Decision, 2026-09-07: the constants are generated, not hand-written, and the
analogy to the JSON Schemas does not hold.** The schemas are `include_str!`'d
because they live **inside** `spec-spine-types`. `kit/` is at the repository
root, outside every crate, and `cargo package` ships only what is under a
package root: a published `spec-spine-core` built with `include_str!("../../../kit/…")`
would not contain the kit at all, so `--with-kit` would write nothing for every
adopter who installed from crates.io.

The two ways out were moving `kit/` inside the crate, or copying its bytes into
a generated module. Moving it would churn five specs' territory (029, 046, 048,
051, 064 all name `kit/` paths), change where adopters and documentation look,
and rewrite every path in three test files, to save a generated artifact. So the
bytes are copied: `scripts/gen-kit-embedded.py` writes
`crates/spec-spine-core/src/kit_embedded.rs`, and the agreement test names the
regeneration command in its failure message. Thirty files, 141 KB, marked
`@generated` and never edited by hand.

**Decision, 2026-09-07: two files the kit stores are not files an adopter
receives.** `kit/README.md` documents the kit rather than being part of it, and
`kit/.gitattributes-stanza` is a block to append to an existing file rather than
a file to write. `kit/AGENTS.md` is a third: §3.1 requires the scaffold to emit a
**config-aware** `AGENTS.md` unconditionally, and §3.2's list of kit files does
not include one, so the adopter gets the generated one, whose corpus and derived
paths match their layout. Two more are remapped rather than stripped:
`kit/settings.json` is flat storage for `.claude/settings.json`, and
`kit/govern.yml` for `.github/workflows/govern.yml`.

### 3.3 It refuses to clobber

Every kit file keeps `overwrite: false`, like every other scaffolded file. An
adopter running `--with-kit` in a repository that already has
`.claude/skills/build/SKILL.md` is told the file exists, not silently
overwritten. `--force` behaves for these files exactly as it does for the other
ten.

This is the case that matters most for `AGENTS.md`. An adopter who has written
their own is the common case in a repository that has been worked in, and
overwriting a cross-agent authority document would destroy project-specific
protocol that no backup makes obvious.

### 3.4 The scaffolded repository is immediately governed

After `init --with-kit`, the repository MUST satisfy its own gate:
`compile`, `index`, `lint --fail-on-warn` all clean, exactly as
`tests/scaffold.rs` already asserts for the plain scaffold.

The end-to-end test MUST scaffold with `--with-kit` into a temporary directory
and run the whole chain, because the kit adds hooks and skills that reference
verbs and paths, and a scaffold that produces a repository failing its own gate
on the first run is worse than no scaffold.

### 3.5 The README says how it is installed

`kit/README.md` MUST state that `spec-spine init --with-kit` installs it, that
`kit/` is the source of those templates, and what the difference is between
plain `init` and `--with-kit`, in one short section near the top.

The README today describes the kit's contents to a reader who has already found
the directory. The one thing it cannot currently tell them is how to get it.

## 4. Out of scope

**Updating an installed kit.** `--with-kit` installs; it does not diff, merge or
migrate an adopter's edited skills against a newer kit. That is a real need with
a real design (a three-way merge against the version they installed) and it
requires recording which version they installed, which nothing does yet.

**Making the kit's contents adopter-specific.** The skills ship as written. Spec
048 settled where project-specific material goes: `AGENTS.md`, not the skill
files.

**A `spec-spine kit` verb.** `init --with-kit` covers install. A verb family for
listing, diffing or updating kit files is the update story above.

**The Makefile and workflow themselves.** Spec 064.

## 5. Verification

Each line is one command (spec 049 §3.2). Every assertion fails against pre-065
code: `--with-kit` was an unknown argument and plain `init` wrote no
`AGENTS.md`.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test scaffold --locked
cargo test -p spec-spine-cli --test init --locked
# 3.2: scaffold a fresh adopter with the harness.
rm -rf "${TMPDIR:-/tmp}/ss065" && mkdir -p "${TMPDIR:-/tmp}/ss065" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" init --with-kit >/dev/null
# 3.1: the protocol is there, and names the non-writing reads.
test -f "${TMPDIR:-/tmp}/ss065/AGENTS.md"
grep -q 'spec-spine compile --check' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# 3.2: the harness landed at the adopter's own paths, not under `kit/`.
test -f "${TMPDIR:-/tmp}/ss065/.claude/skills/build/SKILL.md"
test -f "${TMPDIR:-/tmp}/ss065/.claude/settings.json"
test -f "${TMPDIR:-/tmp}/ss065/.github/workflows/govern.yml"
test ! -d "${TMPDIR:-/tmp}/ss065/kit"
# 3.4: and the scaffolded repository satisfies its own gate on the first run.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" lint --fail-on-warn
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" index check
# 3.2: plain `init` writes the protocol and not the harness.
rm -rf "${TMPDIR:-/tmp}/ss065b" && mkdir -p "${TMPDIR:-/tmp}/ss065b" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065b" init >/dev/null && test -f "${TMPDIR:-/tmp}/ss065b/AGENTS.md" && test ! -d "${TMPDIR:-/tmp}/ss065b/.claude/skills"
# 3.5: the README says how it is installed.
grep -q 'init --with-kit' kit/README.md
rm -rf "${TMPDIR:-/tmp}/ss065" "${TMPDIR:-/tmp}/ss065b"
```
