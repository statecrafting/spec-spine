---
id: "001-alpha"
title: "Alpha"
status: approved
created: "2026-10-01"
implementation: complete
summary: "The alpha rule | with a pipe."
establishes:
  - { kind: file, path: "docs/alpha.md" }
  - { kind: symbol, id: "demo::alpha" }
references:
  - { unit: { kind: file, path: "docs/beta.md" }, role: "context" }
obligations:
  - id: "V-1"
    kind: verification
    text: "Alpha is checked."
    anchor: "verification"
    inputs:
      - "docs/alpha.md"
  - id: "R-1"
    kind: requirement
    text: "Alpha holds."
    anchor: "3-1-alpha"
intent:
  goal: "say what alpha is"
  non_goals:
    - "everything else"
---
# 001

## 3. Behavior

### 3.1 Alpha

Alpha.

## Verification

```verify:cli
spec-spine registry list
```
