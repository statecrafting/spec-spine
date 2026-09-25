---
id: "135-the-toolchain-floor-and-dependencies-are-checked"
title: "The toolchain floor and the dependencies are checked"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  The workspace declares `rust-version = "1.85"` and nothing ever built it
  there; the code uses let-chains, stable only since 1.88, so the declaration
  was false. Dependencies were never checked for advisories, licenses or
  sources, and the dev tree carried two open RustSec advisories through
  `jsonschema`'s HTTP resolver, which no test uses. CI now builds and tests
  at the declared `rust-version`, read from `Cargo.toml`, and runs
  `cargo deny check` against a committed `deny.toml`; both are required through
  `ci-gate`. `jsonschema` drops its default features, which removes the whole
  HTTP and TLS dev tree and both advisories, and the declared `rust-version`
  is corrected to the measured floor. `CONTRIBUTING.md` states the loop.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "094-one-gate-and-the-boundaries-it-holds"
  - "134-the-suite-runs-on-windows"
extends:
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  # `jsonschema` without its HTTP resolver (a dev-dependency).
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/Cargo.toml", nature: corrective }
  # The MSRV statement.
  - { spec: "105-governed-scope-is-enabled-here", unit: "CLAUDE.md", nature: corrective }
  # 3.1: the let-chain collapses clippy asks for once the floor is 1.90.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: corrective }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/seal.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: corrective }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/manifest.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/sections.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/symbols.rs", nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: corrective }
  # A test name that still said "three" after spec 132 moved the code to 4.
  - { spec: "067-a-short-id-names-the-same-spec-at-every-verb", unit: "crates/spec-spine-cli/tests/spec_id.rs", nature: corrective }
references:
  - { unit: { kind: file, path: "deny.toml" }, role: context }
  - { unit: { kind: file, path: "CONTRIBUTING.md" }, role: context }
---

# 135: The toolchain floor and the dependencies are checked

## 1. Purpose

### 1.1 The declared floor was false

`Cargo.toml` declares `rust-version = "1.85"`, `CLAUDE.md` repeats it, and
crates.io publishes it to every consumer. Nothing ever built at it. The first
run of the job this spec adds (#354, run 1) refused at resolution:
`tree-sitter 0.27.0` and `tree-sitter-language 0.1.8`, both normal
dependencies and pinned exact, require rustc 1.90, and the `icu_*` crates in
the dev tree require 1.86. The source also uses let-chains in six places,
stable only since 1.88. A consumer on 1.85 through 1.89 got a resolution error
naming a transitive crate, not the declared floor.

### 1.2 Dependencies were never checked

The first `cargo deny check` found two open RustSec advisories,
RUSTSEC-2026-0258 (`h2` 0.4.15) and RUSTSEC-2026-0285 (`rustls` 0.23.41). Both
came in through `jsonschema`'s default HTTP resolver, a dev-dependency no
test uses: every schema the conformance tests load is embedded.

## 2. Territory

`.github/workflows/ci.yml` is established by 094; the two jobs are an additive
`extends`. `crates/spec-spine-core/Cargo.toml`'s dev-dependency, `CLAUDE.md`'s
MSRV sentence and the clippy-collapsed sites below are `extends` crossings on
their owners. `deny.toml` and `CONTRIBUTING.md` are referenced rather than
claimed: a claimed file must be hashed (L-008), and neither should restamp
every shard when it changes.

## 3. Behavior

### 3.1 The declared floor is built and tested

CI MUST run `cargo +<rust-version> test --workspace --locked`, with the version
read from `Cargo.toml` rather than restated in the workflow, as a job named
`build · test (rust-version)`, required through `ci-gate`.

`rust-version` MUST be the floor the locked tree actually needs: `1.90`.

Raising it switches on clippy's let-chain `collapsible_if` suggestions, which
are gated on the declared floor. The sites are collapsed mechanically
(`cargo clippy --fix`), and nothing else about them changes.

### 3.2 The dependencies are checked

CI MUST run `cargo deny check` against the committed `deny.toml`, as a job
named `cargo-deny`, required through `ci-gate`:

- advisories: any vulnerability or unsoundness notice fails, and a yanked
  crate fails;
- licenses: an explicit allow-list of the licenses the tree uses today;
- bans: a wildcard version requirement fails, and duplicate versions warn;
- sources: crates.io only.

`jsonschema` MUST be a dev-dependency without its default features. That
removes the HTTP and TLS dev tree and both advisories without an MSRV-breaking
bump: a MSRV-aware resolution caps `rustls` below its fix.

### 3.3 Contributors read the loop

`CONTRIBUTING.md` MUST state the loop `AGENTS.md` defines (spec first, one spec
per pull request, the gate, ship then ratify) and what `ci-gate` requires.

## 4. Out of scope

**`SECURITY.md`.** It names a private reporting channel the owner chooses;
until then `CONTRIBUTING.md` says so.

**Namespacing diagnostic codes by tool.** A rename of every `V-`, `L-`, `C-`,
`I-`, `W-` code is a contract change for every consumer that branches on them;
it is presented to the owner as a choice, not decided here.

## 5. Resolved decisions

**D-1 (2026-09-24): correct the floor rather than lower the tree.** Holding 1.85
would mean unpinning tree-sitter's exact pin (a determinism input) and removing
the let-chains. The declaration was the false part.

**D-2 (2026-09-24): drop the resolver rather than bump through it.** A
`rustls` fix exists only above the MSRV-aware cap. The resolver itself was
unused, so removing it closes both advisories and shrinks every dev build.

## Verification

```verify:cli
# 3.1 and 3.2: both jobs exist under their names and the gate requires them.
grep -q 'name: build · test (rust-version)' .github/workflows/ci.yml
grep -q 'name: cargo-deny' .github/workflows/ci.yml
grep -q 'needs: \[test, test_windows, msrv, deny,' .github/workflows/ci.yml
# 3.1: the floor the locked tree needs, and nothing restates it in CI.
grep -q '^rust-version = "1.90"$' Cargo.toml
! grep -q '1\.90' .github/workflows/ci.yml
# 3.2: the resolver that carried the advisories is gone from the tree.
grep -q 'jsonschema = { version = "0.49", default-features = false }' crates/spec-spine-core/Cargo.toml
! grep -q '^name = "reqwest"$' Cargo.lock
test -f deny.toml
# 3.3
grep -q 'ci-gate' CONTRIBUTING.md
```
