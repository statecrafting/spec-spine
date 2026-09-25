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
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "094-one-gate-and-the-boundaries-it-holds"
  - "134-the-suite-runs-on-windows"
extends:
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  # `jsonschema` without its HTTP resolver (a dev-dependency).
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/Cargo.toml", nature: corrective }
references:
  - { unit: { kind: file, path: "deny.toml" }, role: context }
  - { unit: { kind: file, path: "CONTRIBUTING.md" }, role: context }
---

# 135: The toolchain floor and the dependencies are checked

## 1. Purpose

Draft; completed with the first CI run's results (the measured `rust-version`
floor).

## Verification

```verify:cli
grep -q 'name: build · test (rust-version)' .github/workflows/ci.yml
grep -q 'name: cargo-deny' .github/workflows/ci.yml
grep -q 'needs: \[test, test_windows, msrv, deny,' .github/workflows/ci.yml
test -f deny.toml
```
