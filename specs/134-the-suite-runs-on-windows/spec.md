---
id: "134-the-suite-runs-on-windows"
title: "The suite runs on Windows"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  spec-spine ships a Windows binary and proves its shard trees byte-identical
  on Windows, but CI only ever built there: the workspace test suite ran on
  Linux alone, so a Windows behavior difference could ship with every check
  green. A `test (windows)` job now runs `cargo test --workspace` on
  `windows-latest` as a check of its own, required through `ci-gate`. A test
  that cannot run on Windows is gated or quarantined in its source with a
  tracked finding, never skipped by the workflow, and Windows results are
  reported apart from build results.
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "094-one-gate-and-the-boundaries-it-holds"
extends:
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".github/workflows/ci.yml", nature: additive }
  # 3.2: the POSIX-shell suites, gated to Unix with a tracked finding each.
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: "crates/spec-spine-core/tests/ai_review_policy.rs", nature: additive }
  - { spec: "117-the-gate-resolves-the-binary-it-documents", unit: "crates/spec-spine-core/tests/gate_binary.rs", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_hooks.rs", nature: additive }
  - { spec: "122-a-commit-refuses-an-unresolved-merge", unit: "crates/spec-spine-core/tests/commit_boundary.rs", nature: additive }
  - { spec: "123-a-startup-verdict-names-its-reader", unit: "crates/spec-spine-core/tests/reader_identity.rs", nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/windows-findings.md" }, role: context }
---

# 134: The suite runs on Windows

## 1. Purpose

Draft; completed with the first Windows run's results.

## Verification

```verify:cli
grep -q 'name: test (windows)' .github/workflows/ci.yml
grep -q 'needs: \[test, test_windows,' .github/workflows/ci.yml
```
