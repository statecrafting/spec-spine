---
id: "065-init-and-the-kit-are-one-adoption"
title: "Init and the kit are one adoption"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
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
no IO, so the kit's contents are embedded as `const`s exactly as the JSON
Schemas and the constitution template are. A test MUST assert that the embedded
constants and the checked-in `kit/` tree agree, so a change to one cannot
silently diverge from the other. That is the same shape as the conformance test
pinning DTOs against embedded schemas, and it is what keeps `kit/` the editable
source rather than a copy nobody remembers to update.

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

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test scaffold --locked
cargo test -p spec-spine-cli --test init --locked
# Plain init writes an AGENTS.md with the protocol, and no kit.
tmp=$(mktemp -d) && target/release/spec-spine --repo "$tmp" init >/dev/null \
  && grep -q 'New Sessions' "$tmp/AGENTS.md" \
  && grep -q 'Working the backlog' "$tmp/AGENTS.md" \
  && test ! -d "$tmp/.claude/skills"
# --with-kit installs the kit at the adopter's own paths.
tmp2=$(mktemp -d) && target/release/spec-spine --repo "$tmp2" init --with-kit >/dev/null \
  && test -f "$tmp2/.claude/skills/build/SKILL.md" \
  && test -f "$tmp2/.claude/settings.json" \
  && test ! -d "$tmp2/kit"
# The scaffolded repository satisfies its own gate on the first run.
target/release/spec-spine --repo "$tmp2" compile >/dev/null
target/release/spec-spine --repo "$tmp2" index >/dev/null
target/release/spec-spine --repo "$tmp2" lint --fail-on-warn
# The README says how the kit is installed.
grep -q 'init --with-kit' kit/README.md
```
