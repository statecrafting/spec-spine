---
id: "061-the-scaffold-ships-what-adopters-wrote"
title: "The scaffold ships what every adopter wrote by hand"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "006-init-scaffold"
  - "039-declared-state-dir"
  - "043-governance-document-gaps"
extends:
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "standards/spec/templates/constitution-template.md" }, role: context }
  - { unit: { kind: file, path: "spec-spine.toml" }, role: exemplar }
summary: >
  Four scaffold defects the audit measured, all in one generator. `init` writes
  no `.gitignore`, so every adopter independently learns that
  `.derived/**/build-meta.json` carries a wall clock and dirties the tree, and
  spec 039's `state_dir` has the same missing half: the live failure it fixed
  was a permanently dirty tree, not a classification. The scaffolded
  `spec-spine.toml` emits five knobs; adopters needed thirteen, and aicortex
  annotated each of its own with the diagnostic code that knob drives, which is
  the template to copy. And `CONSTITUTION_TEMPLATE` in `scaffold.rs` is still
  the two-bullet stub spec 043 complained about, while the real thirty-four-line
  document sits in `standards/spec/templates/constitution-template.md`: adopters
  get the stub, and two of them deleted it. This spec fixes all four in
  `scaffold_init`, which stays a pure function of `Config` performing no IO.
---

# 061: The scaffold ships what every adopter wrote by hand

## 1. Purpose

`spec-spine init` is the first thing an adopter runs and the last thing anyone
looks at afterwards. The audit looked at what four adopters had to add to it,
and found the same four additions.

**No `.gitignore`.** The scaffold writes nine files and none of them is one.
`.derived/**/build-meta.json` is the single non-deterministic artifact in the
system: it carries `builtAt`, it is excluded from every determinism and golden
check, and it is regenerated on every run. Every adopter discovers by
experiment that it must be gitignored, because until it is, the working tree is
dirty after every compile. The tool knows the path: it is `derived_dir` plus a
filename the tool itself chose.

Spec 039's `state_dir` has the identical missing half. 039 declared a root that
no content hash reaches, so a tool writing its own state cannot stale the
ledger. The failure it was written for was a **permanently dirty tree**, and a
declared-but-untracked-and-unignored state root is still permanently dirty. The
classification landed; the `.gitignore` line did not.

**Five knobs out of thirteen.** `config_toml` emits `[manifest]`, `[domains]`,
`[kind]`, `[layout]` (three of its ten keys) and `[coupling]` (two of its five).
Absent entirely: `index.extra_hashed_inputs`, `index.resolver_exclusions`,
`index.slices`, `layout.state_dir`, `layout.schemas_dir`,
`layout.standalone_rust_workspaces`, `layout.standalone_npm_packages`,
`coupling.require_ownership`, `coupling.auto_waive_dependency_only`,
`frontmatter.extra_known_keys`, `provenance.uri_schemes` and `[branding]`. Every
one of those is a knob an adopter reached for, and the scaffolded file gives no
hint any of them exists. aicortex's own config annotates each knob with the
diagnostic code it drives, which is the presentation worth copying: a knob whose
comment says which refusal it changes is a knob an adopter can reason about.

**The constitution stub.** Spec 043 §1.4 complained about it and did not fix it.
`CONSTITUTION_TEMPLATE` is still seven lines: a title, two sentences, and two
`<principle>` placeholders. The real template, thirty-four lines with the tier
statement, the normative hierarchy and the amendment clause, lives at
`standards/spec/templates/constitution-template.md` and is not what `init`
writes. Two adopters deleted the stub rather than fill it in.

These are item 15 of the audit's tool backlog and items 5, 6 and 14 of its kit
and scaffold backlog. They are one change because they are one function.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/scaffold.rs` | 006 | a tenth file, a fuller config, the real template |
| `crates/spec-spine-core/tests/scaffold.rs` | 006 | acceptance for each |

Spec 006 owns the scaffold and requires that `init` return files as data
(`Scaffold`) which the CLI writes, and that the generator be pure. Both hold
after this change: one more `ScaffoldFile` in the vector, two constants
rewritten. 006's `spec.md` is not edited.

## 3. Behavior

### 3.1 The scaffold writes a `.gitignore`

`scaffold_init` MUST emit a tenth file, `.gitignore`, containing at minimum the
two lines the tool's own behavior requires:

```
# spec-spine writes wall-clock metadata here; it is the one non-deterministic
# artifact and is excluded from every determinism and golden check.
<derived_dir>/**/build-meta.json
```

and, when `layout.state_dir` is non-empty, the state root.

Both paths are computed from `Config`, so a non-default `derived_dir` or
`state_dir` scaffolds coherently, exactly as `config_toml` is already
config-aware.

**It does not ignore the shard trees.** Whether `.derived/spec-registry/` and
`.derived/codebase-index/` are committed is the adopter's decision, and both
answers are legitimate: this repository commits them and gates on their
freshness, and an adopter who does not want that regenerates in CI instead. The
scaffold MUST NOT decide it, and the emitted file MUST carry a comment saying
so, naming both options. Ignoring them by default would silently opt every new
adopter out of the freshness gate that is one of the system's two central
mechanisms.

`ScaffoldFile.overwrite` stays `false` for it, like every other scaffolded file,
so `init` in a repository that already has a `.gitignore` does not clobber it.
`init --force` behaves for this file exactly as it does for the other nine.
Merging into an existing `.gitignore` is deliberately not attempted: a
three-way merge of an adopter's ignore file is a different feature, and getting
it subtly wrong would be worse than a clear "this file already exists".

### 3.2 The scaffolded config shows every knob

`config_toml` MUST emit every table and every key of `Config`, with each key set
to its actual default, and each accompanied by a one-line comment saying what it
does and, where one exists, which diagnostic code it drives:

```toml
[coupling]
# The PR-body waiver keyword (the reason follows the colon). A waiver is a
# human instrument: it needs explicit human approval, and an agent never
# writes one on its own authority.
waiver_keyword = "Spec-Drift-Waiver:"
# Adopter entries are ADDITIVE to the built-in floor and cannot remove one.
# `spec-spine config show` prints the merged list the gate matches against.
bypass_prefixes = []
# C-002: refuse a changed source file that no spec specifically claims.
# Off by default: a corpus adopts this once its coverage debt is retired.
require_ownership = false
```

Commented-out versus emitted is the one judgement here, and the rule is: a key
whose default is the right starting value is **emitted** at that value, and a
key that only makes sense once the adopter has something to put in it
(`extra_known_keys`, `slices`, `standalone_*`, `provenance.uri_schemes`) is
emitted **commented out** with a one-line example. An adopter should be able to
uncomment rather than invent syntax, and should not be handed a file full of
empty tables that look like they mean something.

The emitted file MUST still parse to a `Config` equal to `Config::default()`
modulo the values the CLI substitutes from the invoked configuration, and the
scaffold test MUST assert that. A documented config that has drifted from the
defaults it documents is worse than none, and this is the check that keeps the
comments honest as `Config` grows.

### 3.3 The real constitution template

`CONSTITUTION_TEMPLATE` MUST be replaced with the content of
`standards/spec/templates/constitution-template.md`, the thirty-four-line
document with the tier statement, the normative hierarchy, the amendment clause
spec 043 made writable, and the seam saying which principles are the adopter's
own.

Spec 043 §3.4 already updated `CONSTITUTION`, the scaffolded constitution
itself, for exactly these things. The **template** was left behind, which is why
this defect survived a spec that diagnosed it. The two constants now say the
same thing in the two registers they are for, and the scaffold test MUST assert
that the emitted template states an amendment mechanism, mirroring the
assertion 043 added for the constitution.

The template stays a template: placeholders where an adopter's own principles
go, not this corpus's five principles copied into every repository.

### 3.4 Purity and determinism

`scaffold_init` stays a pure function of `Config` and performs no IO.

In particular, §3.3 MUST NOT read
`standards/spec/templates/constitution-template.md` from disk. The library is
self-contained by construction, as the embedded JSON Schemas are: the content
becomes a `const` in `scaffold.rs`. A test MUST assert the constant and the
checked-in template agree, so the two cannot drift, which is the same shape as
the conformance test that pins DTOs against the embedded schemas.

The scaffolded corpus MUST still compile and lint clean, which
`tests/scaffold.rs` already asserts and which the new `.gitignore` and the
fuller config must not break.

## 4. Out of scope

**Writing `AGENTS.md` or shipping the kit.** `init --with-kit` is spec 065.
This spec fixes the files `init` already writes.

**A `Makefile` or a CI workflow.** Spec 064.

**Merging into an existing `.gitignore`.** §3.1.

**Deciding whether adopters commit their shard trees.** §3.1. The scaffold
presents both options and takes neither.

**Adding knobs.** Every key emitted is one `Config` already has. Making the
scaffold honest about the current surface is separable from growing it.

**Fixing `IndexConfig::default().extra_hashed_inputs`.**

**Decision, 2026-09-07.** Writing §3.2's exhaustive config surfaced that the
**default value itself** carries the broken glob form spec 057 found in this
repository's file: `["standards/**", ".github/workflows/**"]` matches
directories and therefore no files, so every adopter relying on the default
hashes nothing extra and does not know it. 057 fixed this corpus's
`spec-spine.toml` and did not look at the default behind it.

Not fixed here. Changing it would restale the committed index of every adopter
who relies on the default, which is a behavior change to a shipped default and
belongs in its own spec with its own release note, not inside a scaffold
change. What this spec does instead is stop the silence: the key is emitted at
its real (broken) value with a comment naming the trap and the working form, so
the next adopter reads it rather than deriving it.

**Decision, 2026-09-07: an emitted table header is a value.** The first cut of
§3.2 emitted `[provenance.uri_schemes]` uncommented above its commented
examples, which declares an **empty** table and overrides that key's non-empty
default. The round-trip assertion §3.2 requires caught it immediately, which is
the argument for that assertion. A table whose keys are all commented out has
its header commented out too.

## 5. Verification

Each line below is one command: spec 049 §3.2 makes a fence's body line a
command, so a trailing `\` continuation becomes its own fragment and fails. The
scaffold is materialized once at a fixed path, because each line is its own
shell and a `$(mktemp -d)` would not survive to the next assertion.

Every assertion fails against pre-061 code: there was no `.gitignore`, the
config emitted five knobs, and the template was a two-bullet stub.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test scaffold --locked
# Materialize a fresh adopter once.
rm -rf "${TMPDIR:-/tmp}/ss061" && mkdir -p "${TMPDIR:-/tmp}/ss061" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss061" init >/dev/null
# 3.1: the one non-deterministic artifact is ignored...
grep -q 'build-meta.json' "${TMPDIR:-/tmp}/ss061/.gitignore"
# ...and the shard trees deliberately are not, with the choice explained.
grep -q 'freshness gate' "${TMPDIR:-/tmp}/ss061/.gitignore"
# 3.2: the knobs adopters reached for are named, with the codes they drive.
grep -q 'require_ownership' "${TMPDIR:-/tmp}/ss061/spec-spine.toml"
grep -q 'resolver_exclusions' "${TMPDIR:-/tmp}/ss061/spec-spine.toml"
grep -q 'C-002' "${TMPDIR:-/tmp}/ss061/spec-spine.toml"
# 4: and the glob trap is named where it bites, rather than left to be derived.
grep -q 'matches DIRECTORIES' "${TMPDIR:-/tmp}/ss061/spec-spine.toml"
# 3.2: the emitted config is the configuration it documents. `config show`
# reads it back through the same loader every verb uses.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss061" config show >/dev/null
# 3.3: the template is the real document, not the two-bullet stub.
grep -q 'Amendment' "${TMPDIR:-/tmp}/ss061/standards/spec/templates/constitution-template.md"
grep -q 'Normative hierarchy' "${TMPDIR:-/tmp}/ss061/standards/spec/templates/constitution-template.md"
# 3.4: the scaffolded corpus still compiles and lints clean.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss061" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss061" lint --fail-on-warn
rm -rf "${TMPDIR:-/tmp}/ss061"
```
