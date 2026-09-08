---
id: "053-depends-on-ordinal-monotonicity"
title: "A dependency points backward, and the corpus can say so"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "003-conformance-lint"
  - "033-dependency-cycle-refusal"
  - "038-registry-plan-ready-set"
extends:
  # The conformance lint gains one opt-in code. Spec 003 owns the lint module
  # and its severity tiers; it says nothing about which conventions are
  # checkable, so a new code is surface added rather than behavior changed.
  # 003's spec.md is not edited (spec 040).
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  # One new opt-in table in the config model.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  # `LintConfig` joins the crate root's re-export beside every sibling table
  # (2.1). One line, and the only reason it is a crossing at all is that the
  # re-export list lives in the crate root rather than in `config.rs`.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
establishes:
  # Created by this spec (2), so claimed by it: the corpus had no dedicated
  # lint test file, and `L-007`'s acceptance is the reason to open one.
  - "crates/spec-spine-core/tests/lint.rs"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Three adopters independently wrote the same ninety-line `scripts/spec-dag.sh`
  to assert one thing the tool does not: that a `depends_on` entry names a lower
  ordinal than the spec declaring it. Spec 033 already ships the harder half of
  that graph check, refusing a cycle at compile time, and ordinal monotonicity
  is the cheap total order that makes a cycle impossible by construction rather
  than detectable after the fact. This spec adds `L-007`, an opt-in conformance
  lint gated on a new `[lint] require_ordinal_monotonic_depends_on` knob,
  defaulting off so no existing corpus changes verdict. It is deliberately a
  lint and not a `V-` code: a forward dependency is a corpus convention an
  adopter may reasonably not hold, not a structural defect that makes the
  registry unreadable.
---

# 053: A dependency points backward, and the corpus can say so

## 1. Purpose

`depends_on` is the scheduling edge. `registry plan` (spec 038) reads it to
partition the corpus into what can be worked on now and what is blocked, and
spec 033 refuses a cycle in it at compile time, because a cyclic scheduling
graph has no answer and a walk over it does not terminate.

What the tool has never checked is direction. Nothing stops
`012-index-hash-slices` from declaring `depends_on: ["044-in-progress-is-in-flight"]`,
and nothing about that entry is structurally wrong: the target resolves, the
graph stays acyclic, `plan` reports 012 as blocked and moves on.

It is still almost always a mistake, and three adopters thought so independently.
hqgit, aicortex and rahi each carry the same ninety-line `scripts/spec-dag.sh`,
and each wrote it for this one assertion. That is the audit's ordinary signal
for a missing feature: not one adopter with an unusual need, but three arriving
at the same script without talking to each other.

The reason a forward dependency is a mistake is that ordinals encode filing
order. A spec filed later was written knowing the corpus that existed when it
was filed; a spec filed earlier was not. An edge from earlier to later says the
earlier spec's schedule depends on work whose shape was unknown when it was
written, which is either a renumbering nobody finished, a copied frontmatter
block, or a genuine inversion the author should turn into the other edge. Every
one of those is worth a line of output.

Ordinal monotonicity is also strictly stronger than the property spec 033
defends. A graph whose every edge decreases a total order cannot contain a
cycle, so a corpus that holds this convention gets acyclicity as a theorem
rather than as a detection. 033 stays exactly as it is: it is the check for
corpora that do not hold this convention, and it is the reason this one can be
opt-in without leaving anybody unprotected.

This is item 4 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-types/src/config.rs` | 000 | a new `[lint]` table with one boolean |
| `crates/spec-spine-core/src/lint.rs` | 003 | the `L-007` check |

The corpus has no dedicated lint test file today: `L-006`'s acceptance lives in
`tests/coverage.rs` and the scaffold's in `tests/scaffold.rs`. The implementing
change creates `crates/spec-spine-core/tests/lint.rs` and claims it there, which
is one of the two always-legitimate mid-build edits
(`.claude/rules/adversarial-prompt-refusal.md`): a file you created is added to
the spec you are implementing, in the same change.

Spec 003 owns the lint and defines its severity tiers (error always, warning
under `--fail-on-warn`, info under `--fail-on-info`). It enumerates the codes
that existed when it was written and states no closed set, so adding a code is
additive surface. Spec 033 owns the cycle refusal and is untouched: this spec
adds a second, independent, opt-in check over the same edge and changes nothing
about the first.

### 2.1 One line in the crate root

**Decision, 2026-09-07.** `crates/spec-spine-types/src/lib.rs` re-exports every
`Config` table by name, and `LintConfig` joins that list. The table would be
reachable without it (`config` is a `pub mod`), so this is not a correctness
need; it is that a new public table absent from the list every sibling appears
in is a wart a reader trips on. Declared as an `extends` edge rather than left
undone or waived.

## 3. Behavior

### 3.1 One new opt-in knob

`Config` MUST gain a `[lint]` table carrying a single boolean,
`require_ordinal_monotonic_depends_on`, defaulting to `false`.

The table is new rather than a field on an existing one because there is no
existing home that is not a lie. `[frontmatter]` is the authored grammar's
configuration and this is not a grammar rule; `[coupling]` is the PR-time gate
and this never runs there. A `[lint]` table names what the knob actually gates,
and it is where a future opt-in conformance convention belongs, so the next one
does not have to relitigate this.

Every table in `Config` carries `#[serde(default, deny_unknown_fields)]`, so a
`spec-spine.toml` written before this spec parses unchanged and reads the
default. `CONFIG_VERSION` does not move: it versions the config schema, and an
added optional table with a default is the additive case that a version exists
to make safe rather than to record.

Defaulting off is not timidity. The convention is real but not universal: a
corpus that files by domain rather than by date, or one that renumbered once and
lives with the result, holds a coherent position this lint would spam. The
adopters who want it asked for it by writing a script; a knob is how they stop
maintaining that script, and it is not a verdict on anybody who did not.

### 3.2 `L-007`

When the knob is on, `lint` MUST emit one `L-007` diagnostic, at **error**
severity, for each `depends_on` entry whose ordinal is greater than or equal to
the ordinal of the spec declaring it:

```
L-007  spec '012-index-hash-slices' depends_on '044-in-progress-is-in-flight',
       which is not a lower ordinal (044 >= 012): a dependency points backward
       in filing order
```

Error tier, so the knob alone decides whether the corpus is held to this. A
warning tier would have meant two knobs (this one and `--fail-on-warn`) for one
decision, and an adopter who turned this on turned it on to be refused. It
matches `L-006`, the other check whose whole purpose is to refuse.

The diagnostic's `path` MUST be the declaring spec's `spec_path`, as every other
`L-` code sets it, so an editor jumping to the diagnostic lands on the file that
has to change.

When the knob is off, `L-007` MUST NOT be emitted at all. Not emitted at info
tier, not emitted and filtered: a corpus that has not opted in sees byte-identical
lint output before and after this spec.

### 3.3 The ordinal is the leading digits, and absence is not a violation

A spec's ordinal is the leading decimal digit run of its `id`, parsed as an
integer. `053-foo` yields 53. `007-bar` yields 7, and compares equal to a
hypothetical `7-bar`: the comparison is numeric, not lexical, so a corpus that
outgrows three digits and files `1001-foo` orders correctly rather than sorting
`1001` below `999`.

An id with **no** leading digit run has no ordinal. Both the declaring spec and
the target are checked, and if either lacks an ordinal, `L-007` MUST NOT fire for
that pair. Silence is correct here and a diagnostic would not be. The corpus does
not require numeric ids: `V-001` requires only that the directory equal the id,
and `V-004`'s duplicate-prefix check reads the first three characters whatever
they are. A corpus with `auth-login` and `auth-logout` is well-formed, and
telling it that `auth-login` has no ordinal is telling it that it does not hold a
convention it never claimed. If such a corpus turns this knob on, it gets silence,
which is the honest answer to "are these edges monotonic" when the order is
undefined.

Equality is a violation, not a pass. Two specs cannot share an ordinal (`V-004`
refuses it), so an entry whose ordinal equals the declaring spec's is a spec
depending on itself, which `L-007` catches as a side effect and which is worth
catching however it arrives.

### 3.4 It is a lint, not a validation

`L-007` MUST NOT become a `V-` code and MUST NOT fail `compile`.

The `V-` codes are structural: a duplicate id, a dangling `superseded_by`, a
`depends_on` cycle. Each of them describes a registry that cannot be read
correctly, and each fires unconditionally because no adopter can coherently
opt out of being readable. A forward dependency is none of that. The registry
compiles, the shard is valid, `plan` gives a correct answer, and the only thing
wrong is a convention the corpus may or may not hold. That is the definition of
the lint's job, and spec 003 drew the line in exactly this place.

`compile --check` therefore stays byte-identical, no shard changes, and no
schema version moves.

## 4. Out of scope

**Turning it on in this repository.** The knob ships off in `spec-spine.toml`,
and whether this corpus holds the convention is a separate decision from whether
the tool can check it. Whether it *does* hold it is not a matter of opinion, so
§5 asks rather than asserting: it runs the corpus through the lint with the knob
on, from a scratch root that symlinks `specs/`, leaving the repository's own
configuration untouched. As of this spec the answer is zero diagnostics.

**Fixing a violation.** The lint names the edge; a person decides whether the
answer is a renumbering, a deleted entry, or an inverted edge. A tool that
rewrote frontmatter to satisfy its own lint would be repairing the tree it is
meant to judge, which is the error spec 046 found in the kit's hooks.

**Ordering any other edge.** `extends`, `refines`, `amends`, `supersedes` and
`constrains` all legitimately point in either direction: an amendment names an
older spec, and a `constrains` edge on a reserved unit may well name a newer one.
Only `depends_on` carries scheduling semantics, and only scheduling has a reason
to respect filing order.

**Replacing spec 033.** The cycle refusal stays unconditional and unchanged. It
is what protects a corpus that has not opted in, and it is the reason this check
can be opt-in at all.

**The `&id[..3]` slicing in `detect_duplicates`.** `V-004` reads the first three
characters of an id by byte index, which is not guaranteed to be a character
boundary for a non-ASCII id and would panic rather than diagnose. This spec's
ordinal parser is character-safe by construction, so it does not inherit the
defect, but it also does not fix it: that is a panic-on-user-input defect in
spec 001's territory and it deserves its own spec rather than a drive-by patch
inside a lint change.

## 5. Verification

The first three commands fail against pre-053 code: the test target does not
exist, and `[lint]` is an unknown table to a `Config` whose every table is
`deny_unknown_fields`, which is a hard config error (exit 3) rather than a
silently ignored key.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.1: the knob exists and parses.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" && printf '[lint]\nrequire_ordinal_monotonic_depends_on = true\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint
# 3.1: and a config written before this spec still parses, reading the default.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" && printf '[layout]\nspecs_dir = "specs"\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint
# 4: this corpus is asked, not assumed. A scratch root symlinks `specs/` so the
# repository's own `spec-spine.toml` is neither read nor written; the knob is on
# only inside the scratch root. Zero diagnostics means the convention holds.
tmp=$(mktemp -d) && ln -s "$PWD/specs" "$tmp/specs" && printf '[lint]\nrequire_ordinal_monotonic_depends_on = true\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint | grep -q '^lint: 0 error'
# 3.4: a lint is not a validation. The committed shards are byte-identical and
# the corpus this repository ships still lints clean with the knob off.
target/release/spec-spine compile --check
target/release/spec-spine lint --fail-on-warn
```
