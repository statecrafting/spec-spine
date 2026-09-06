---
id: "039-declared-state-dir"
title: "`layout.state_dir`: a declared, ungoverned tool-state root"
status: draft
kind: "tooling"
created: "2026-09-05"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "003-conformance-lint"
  - "004-codebase-index"
  - "005-coupling-gate"
  - "032-ownership-coverage"
extends:
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/config.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # The two files 2's territory list omitted; see 5, D-1.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/symbols.rs", nature: additive }
  - { spec: "024-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
references:
  # The bypass floor and the ownership ratchet this key has to agree with, and
  # the design note this is wave 1 of.
  - { unit: { kind: file, path: "specs/009-coupling-floor-claim-precedence/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
summary: >
  A repo governed by spec-spine increasingly hosts tooling that keeps state
  inside it: a build daemon's journal, a local database, captured transcripts,
  attestation bundles. There is nowhere agreed to put any of it. Every such tool
  invents a directory, and every invented directory is then a surprise to the
  gates: `couple` sees changed files no spec claims and refuses the merge under
  `require_ownership`, `index coverage` counts them as untraced debt, and the
  resolver walks them. The adopter's only recourse is to keep appending private
  paths to `bypass_prefixes` and `resolver_exclusions`, one list per concern,
  none of which states what the directory *is*. This spec adds `layout.state_dir`:
  a single declared root, unset by default, that spec-spine agrees to know about
  and refuses to govern. Declared means the gates recognize it (bypassed by
  `couple`, excluded from coverage classification and from the resolver scan, and
  never hashed); ungoverned means spec-spine never reads its contents, never
  writes into it, and refuses a spec that tries to claim a unit inside it, which
  `lint` reports as `L-006`, a contradiction rather than a precedence question
  resolved silently in either direction.
  Unset by default, so no existing repo changes behavior.
---

# 039: `layout.state_dir`

Wave 1 of `docs/design/02-agentic-builder-substrate.md`.

## 1. Purpose

spec-spine already distinguishes two roots. `layout.specs_dir` holds authored
truth and `layout.derived_dir` holds compiler-owned truth. A third kind of file
has no root and keeps arriving: state belonging to a *tool* that operates on the
repo but is not the repo's source and is not spec-spine's output.

Concretely, that is a build daemon's work journal, a local SQLite store, captured
agent transcripts, or a per-spec evidence bundle. It is not authored truth
(nobody writes it by hand), not derived truth (spec-spine does not emit it), and
not source (no spec should claim it).

With nowhere to declare it, each such tool picks a path, and the gates then treat
that path as unexplained source:

| Gate | What it does with an undeclared tool directory |
|---|---|
| `couple` with `require_ownership` | changed files no spec claims: `C-002`, merge refused |
| `index coverage --fail-on-untraced` | counted as untraced debt against the whole-tree ratchet |
| the resolver | walks it, and may resolve a symbol or path inside it |
| content hashing | may fold it into a hash, so unrelated tool writes restale the tree |

Today the adopter answers each of those separately, by appending a private path
to `coupling.bypass_prefixes` and another to `index.resolver_exclusions`, with
nothing recording why either entry is there. Two lists, one intention, and no
statement anywhere that the directory is not source. The next gate added has to
be told a third time.

The problem this key solves is not "how do I silence four warnings". It is that
the config has no vocabulary for *a path that exists and is deliberately not
governed*, and the number of things needing that vocabulary is growing.

## 2. Territory

`config.rs` gains the key. `couple.rs` consults it when deciding bypass;
`coverage.rs` consults it when classifying a source file; `index.rs` consults it
when scanning. `symbols.rs` consults it when walking for symbols and modules,
and `shard.rs` when folding the globally hashed inputs. `lint.rs` gains one
diagnostic. No emitted DTO changes shape, and no schema version moves: this is a
config key and six reads of it.

The last two are additions to this section, made when it was implemented; 5
records why.

## 3. Behavior

### 3.1 Unset by default

`state_dir` defaults to the empty string, meaning **no state root is declared**.
Every behavior below is inert when it is unset, so an existing repo, this one
included, is byte-identical across this change.

A default of `.spec-spine/` was rejected. Silently bypassing a real path in every
adopter's repo, on upgrade, is a change to what the gate refuses, and a gate that
quietly stops refusing something is the one kind of regression this project can
least afford.

### 3.2 Declared: the gates recognize it

When set, the value names one repo-relative directory. For every path beneath it:

- `couple` MUST treat it as bypassed, on the same footing as the built-in floor.
- `coverage` MUST exclude it from classification entirely: it is neither claimed,
  floor-only, nor untraced, because it is not a source file. It MUST NOT be
  counted in the denominator.
- The resolver MUST NOT scan it, so no unit ever resolves to a path inside it.
- It MUST NOT contribute to any content hash, so a tool writing its own state can
  never make the committed ledger stale.

A trailing slash is trimmed, so `state` and `state/` name one root, matching the
handling spec 036 established for `specs_dir`. Prefix matching is
separator-aware: `state` MUST NOT match `stateful/`.

The value MUST NOT overlap `layout.specs_dir` or `layout.derived_dir` in either
direction: it may not equal one, contain one, or sit inside one, and config load
MUST reject such a value with a clean `Error::Config`.

The comparison is against the **resolved** values of those two keys, never
against their defaults. Both are configurable, so an adopter with
`specs_dir = "corpus"` and `state_dir = "corpus/state"` must be refused on
`corpus`, and a check written against the literal `specs` would clear that
configuration and quietly make every `spec.md` under it ungoverned. This is the
same defect spec 036 fixed in `couple.rs`, where two functions spelled the corpus
root as a literal and the gate therefore searched a path the repo did not
contain. It is worth naming here because a validation rule is exactly the kind of
code where a default is easiest to hardcode and hardest to notice. A `state_dir` of `specs` makes every `spec.md` ungoverned; a
`state_dir` of `.` does it to the entire repository. In both cases every gate
keeps exiting 0 while adjudicating nothing, which is the worst failure mode this
project has: not a refusal, but a silent stop. The exclusion is an overlap test
rather than an equality test precisely because the dangerous values are the ones
that contain a root, not the ones that equal it.

The descendant direction is refused for a different reason. A `state_dir` of
`specs/state` under a `specs_dir` of `specs` would put a path inside a governed
root and an ungoverned one at once, and every gate would then need a precedence
rule for a situation with no legitimate use. Refusing the configuration is
cheaper than specifying which root wins, and leaves nothing to get wrong later.

### 3.3 Ungoverned: spec-spine never reads or writes it

spec-spine MUST NOT read the contents of anything under `state_dir` and MUST NOT
create, write or delete anything there. The key exists so that spec-spine and the
tools around it agree on where such state lives; it is not a directory spec-spine
uses.

This keeps the purity invariant intact. A root that spec-spine read from would be
a second input to functions contracted to be pure in `(config, file contents)` of
the corpus, and a root it wrote to would make a read command mutate the tree.

### 3.4 A claim inside `state_dir` is a contradiction, and `lint` says so

If any spec declares a unit whose path falls under `state_dir`, `lint` MUST emit
`L-006`, an error-tier diagnostic naming both the spec and the unit.

`L-006` is the next free code in the `L-` band at the time of filing
(`L-001`..`L-005` are allocated). It is a **provisional** reservation, not a
claim on the namespace: this spec is `implementation: pending`, and another spec
may allocate an `L-` code before it is built. The binding rule is therefore
"the next free code in the band", and `L-006` is what that resolves to today. An
implementer who finds the band has grown takes the next free code and updates
this section rather than colliding, and the §3.6 invariant (no two diagnostics
share a code) is what makes either outcome safe to discover.

The alternative designs both fail. Letting the claim win reintroduces the
claimed-path-overrides-bypass precedence (spec 009) into a directory whose entire
purpose is to be ungoverned, so the declaration would mean nothing. Letting the
bypass win silently discards a claim an author wrote deliberately, which is the
authority-laundering failure mode spec 025 was careful to close: a unit is never
dropped without being reported.

So neither wins, and the corpus is told. The author either moves the file out of
the state root or stops claiming it, and both are one-line fixes once named.

### 3.5 Relationship to the existing lists

`bypass_prefixes` and `resolver_exclusions` are unchanged and remain the right
tool for their own jobs: an adopter-specific exemption for a path that *is*
source (a vendored tree, a generated file), and a scan exclusion for a build
directory. `state_dir` is not a shorthand for setting both. It carries a meaning
neither list can express, namely that the path is present, deliberate, and
outside the governed surface, and it is a single value rather than a list because
a repo with two ungoverned state roots has an organizational problem the config
should not smooth over.

The effects of `state_dir` MUST NOT be overridable or reachable through either
list: no entry in `bypass_prefixes` or `resolver_exclusions` can cancel them, and
setting both lists to cover the same path is not equivalent to declaring it. The
key is its own decision, not sugar for two floor entries.

### 3.6 Tests (minimum)

- Unset: every gate behaves exactly as before (a fixture repo produces identical
  registry, index, coverage and coupling verdicts across the change).
- Set: a changed file under the root does not trip `C-001` or `C-002`.
- Set: coverage excludes the root from both numerator and denominator, so a repo
  at 100% stays at 100% after state files appear. The denominator shrinks by
  exactly the number of files under the root, asserted as a count rather than as
  a percentage: declaring a state root moves a coverage figure, and the movement
  should be auditable rather than merely plausible.
- Set: the resolver does not resolve a unit to a path inside the root, and files
  there do not contribute to the index content hash (writing one does not make
  `index check` stale).
- `state` and `state/` behave identically; `stateful/` is not matched by `state`.
- A spec claiming a unit under the root yields `L-006` at error tier, naming
  the spec and the unit, and `lint` exits 1 on it without `--fail-on-warn`.
- No two lint diagnostics share a code. `L-006` is the next free code at the
  time of writing (`L-001`..`L-005` are allocated in `lint.rs`), but a claim
  about a namespace is worth an assertion rather than a comment, since the band
  may have grown by the time this is implemented.
- Config load rejects, with a clean `Error::Config`, a value overlapping
  `specs_dir` or `derived_dir` in either direction: equal (`specs`, `.derived`),
  an ancestor (`.`, `./`), or a descendant (`specs/state`, `.derived/state`).
  A sibling sharing a prefix (`specs-state`) is accepted, per the
  separator-aware rule. The empty string (unset) is accepted and inert.
- The overlap check reads the resolved layout: under `specs_dir = "corpus"`,
  `state_dir = "corpus/state"` is refused and `state_dir = "specs/state"` is
  accepted, which is the inverse of the default-layout result and fails against
  any implementation that compares with a hardcoded root.

## 4. Out of scope

**Anything about what goes in the directory.** This spec declares a root and its
treatment by the gates. The format of any artifact stored there, in particular a
per-spec attestation bundle, is wave 2 of the design note and is specified
separately.

**Scaffolding it.** `init` does not create the directory or set the key. An
adopter with no such tooling should have no such directory.

**Gitignore management.** Whether the root, or part of it, is committed is the
adopter's decision and their `.gitignore`'s business. spec-spine's treatment is
the same either way, which is the point: a committed evidence bundle and a
gitignored transcript are both ungoverned by the gates.

**Multiple state roots.** One value, deliberately (§3.5).

## 5. Resolved decisions

- **D-1 (2026-09-06): the territory in 2 was two files short of what 3.2
  requires.** 2 named `config.rs`, `couple.rs`, `coverage.rs`, `index.rs` and
  `lint.rs`. Two of 3.2's four MUSTs reach past them. *The resolver MUST NOT
  scan it*: `index.rs` owns one source walk, but the symbol and module indexes
  walk their own trees in `symbols.rs`, so a symbol defined inside the root
  would still have resolved. *It MUST NOT contribute to any content hash*: the
  globally hashed inputs are folded in `shard.rs::global_inputs_hash`, and an
  `extra_hashed_inputs` pattern wide enough to reach into the root is the
  ordinary case rather than an exotic one, since the point of the key is that
  the adopter states the root once. Both files are now claimed by an `extends`
  edge and 2 says so. Rejected: implementing only the four MUSTs the original
  territory covered and recording the other two as limitations, which would
  leave a stated MUST unimplemented and the root scannable by exactly the
  mechanism most likely to reach it.

- **D-2 (2026-09-06): a `state_dir` of `.` is refused by name, not by the
  overlap test.** 3.2 requires refusing a value that contains a governed root,
  and names `.` as the case that ungoverns the entire repository. A string
  comparison does not know the repository root is an ancestor of everything, so
  the overlap test cleared `.` and the first draft of the validation accepted
  it. It is now refused explicitly, with its own message. Rejected: normalizing
  `.` to the empty string, which spells *unset* and would have turned the most
  dangerous value into the silent default.

- **D-3 (2026-09-06): the walkers take the layout, not an extra exclusion
  entry.** The obvious implementation appends the root to
  `index.resolver_exclusions` before walking. It does not work and would be
  wrong if it did: that list matches directory *names* rather than path
  prefixes, so a root of `tool/state` is inexpressible in it, and 3.5 requires
  that neither list can cancel the key's effects. Both walkers take
  `&LayoutConfig` instead, which keeps the two mechanisms visibly separate at
  every call site.

- **D-5 (2026-09-06): every value that would declare nothing is refused, and
  the order of the checks is what makes that true.** Review of the implementing
  PR found `/var/logs` accepted, on the same reasoning as D-2 and D-4: an
  absolute path matches no repo-relative path the gates test. Widening the guard
  then exposed a second case the first fix would have hidden, `/`, which
  normalizes to the empty string and would have been read as *unset*, silently
  turning a value the adopter wrote into no declaration at all.

  So the validation now reads the **raw** value to decide whether a root was
  declared, refuses anything that is not repo-relative before normalizing, and
  only then tests the repository root and the overlaps. Normalizing first is
  what made `/` invisible, and this ordering is the reason it cannot be. The
  three arms share one rationale: a `state_dir` that matches nothing is worse
  than a wrong one, because the gates keep exiting 0 while adjudicating nothing
  and the config claims otherwise.

- **D-7 (2026-09-06): the not-repo-relative check tests segments, not
  prefixes.** Three review rounds found three spellings of one defect: `..`,
  then `/var/logs` and `/`, then `state/..`. Each passed the guard written for
  the previous one, and each was inert for the identical reason, since a
  repo-relative path produced by `rel_posix` carries no `..` and never begins
  with `/`. Patching the third spelling would have invited a fourth. The check
  now refuses any value with a `..` **segment** anywhere, plus any absolute one,
  which is the class rather than its instances.

  The pattern is worth recording beyond this key: a validation rule written
  against the example in front of you tends to encode the example. The general
  question here was always "can this value ever match a path the gates test",
  and asking it directly is what closes the case.

- **D-6 (2026-09-06): `L-006` covers all six ownership-bearing edges, not five.**
  The first implementation walked `establishes`, `extends`, `refines`,
  `co_authority` and `constrains`, and omitted `supersedes`. A partial
  `supersedes` item carries the unit whose authority transfers (spec 019), so it
  claims a path exactly as the others do: a superseding spec could have held a
  claim inside the ungoverned root that `couple` bypasses unconditionally and no
  diagnostic ever named, which is precisely the contradiction this code exists
  to surface. `amends` stays excluded, because its subject is the amended spec's
  `spec.md` rather than an arbitrary unit, and a `spec.md` lives under
  `specs_dir`, which `state_dir` may not overlap.

- **D-4 (2026-09-06): a value escaping the repository is refused, and the
  overlap check stays scoped to the two roots 3.2 names.** Review of the
  implementing PR raised both.

  `../logs` passed validation and would then have matched nothing, because every
  path the gates test is repo-relative and carries no `..`. The gates would
  behave as though no root were declared while the config said one was, which is
  the same failure class as the `.` case in D-2 and is refused the same way:
  silence that claims to be a decision.

  The second was an unchecked overlap with `standalone_rust_workspaces` and
  `standalone_npm_packages`. 3.2 names `specs_dir` and `derived_dir` and only
  those, and the check is left scoped to them deliberately. The two governed
  roots are **spec-spine's own**: a state root swallowing one breaks the tool
  against a corpus the adopter did not intend to ungovern, and no adopter means
  `state_dir = "specs"`. A declared package directory is the adopter's own code,
  and naming it as the state root is a statement about their repository that
  this key exists to let them make. Widening the refusal would be adding a rule
  the spec does not have, to prevent a configuration that is legible on its
  face; the coverage report already shows the denominator that results.
