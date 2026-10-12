---
id: "001-alpha"
title: "An implemented spec"
status: approved
implementation: complete
created: "2026-10-10"
summary: "The fixture's implemented spec: it owns one source file and declares a passing acceptance."
establishes:
  - "src/alpha.rs"
obligations:
  - { id: "R-1", kind: requirement, text: "The fixture holds.", anchor: "behavior" }
---

# 001: An implemented spec

## Behavior

The fixture holds.

## Verification

```verify:cli
true
```
