---
id: "039-declared-state-dir"
title: "`layout.state_dir`: a declared, ungoverned tool-state root"
status: draft
kind: "tooling"
created: "2026-09-05"
implementation: pending
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
when scanning and when hashing. `lint.rs` gains one diagnostic. No emitted DTO
changes shape, and no schema version moves: this is a config key and four reads
of it.

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
`L-006`, an error-tier diagnostic naming both the spec and the unit. It takes the
next free code in the `L-` band (`L-001`..`L-005` are allocated).

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

`state_dir` MUST NOT be removable by any list: it is not additive to the bypass
floor, it is its own decision.

### 3.6 Tests (minimum)

- Unset: every gate behaves exactly as before (a fixture repo produces identical
  registry, index, coverage and coupling verdicts across the change).
- Set: a changed file under the root does not trip `C-001` or `C-002`.
- Set: coverage excludes the root from both numerator and denominator, so a repo
  at 100% stays at 100% after state files appear.
- Set: the resolver does not resolve a unit to a path inside the root, and files
  there do not contribute to the index content hash (writing one does not make
  `index check` stale).
- `state` and `state/` behave identically; `stateful/` is not matched by `state`.
- A spec claiming a unit under the root yields `L-006` at error tier, naming
  the spec and the unit, and `lint` exits 1 on it without `--fail-on-warn`.
- The key is rejected with a clean `Error::Config` if it names the corpus root or
  the derived root, which would make two roots contradict.

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
