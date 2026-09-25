---
id: "150-a-change-runs-the-acceptance-it-can-break"
title: "A change runs the acceptance it can break"
status: draft
kind: "process"
created: "2026-09-25"
summary: >
  Specs 144 and 137 each broke an approved spec's acceptance (046 and 089), and
  nothing noticed until the pre-release sweep of `e440b714`, after both had
  merged; spec 146 repaired them. The sweep runs only on a merged revision
  (specs 089, 099), and no pre-merge step runs any acceptance other than the
  changed spec's own. This spec adds a pre-merge acceptance run: a selector
  that names every spec whose acceptance a change can break, and a step that
  runs those blocks on the pull request's head before it merges, with the
  counts recorded in the pull request. Measured on 136 to 146, a selector keyed
  on references alone would not have been narrower than the corpus, and would
  have caught 046 only through its generic `lint` line, so the selector falls
  back to the whole corpus when engine source changes.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "089-nothing-reruns-a-merged-acceptance"
  - "099-a-merged-acceptance-is-asked-again"
  - "146-carried-acceptance-follows-139-and-144"
extends:
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/test-verify-sweep.py", nature: additive }
  - { spec: "064-compile-warnings-reach-the-gate", unit: "AGENTS.md", nature: additive }
  - { spec: "064-compile-warnings-reach-the-gate", unit: ".claude/skills/", nature: additive }
references:
  - { unit: { kind: file, path: ".github/workflows/acceptance.yml" }, role: context }
---

# 150: A change runs the acceptance it can break

## 1. Purpose

### 1.1 The miss

| Broken block | Broken by | How | Found |
|---|---|---|---|
| 046 | 144 (#365) | 046's block lints this corpus from a scratch root whose `specs` is a link out of that root; 144 refuses such a link (exit 2) | sweep of `e440b714`, after merge |
| 089 | 137 (#361) | 089's block asserts the ledger reports 011 `exempt`; 137 removed 011's line | same |

Both pull requests ran the full gate and their own spec's acceptance. Neither
ran 046's or 089's. `verify` is outside the gate chain on purpose (spec 043),
and the sweep refuses any revision that is not already on the trusted ref
(089 §3.7, 099 §3.1), so nothing could have run them before merge.

### 1.2 What a reference selector would have caught

Measured on 2026-09-25 over the 147 effective plans (`verify <id> --plan`, so
carried blocks included) at `7ce4935f` plus the ratifications, against the
specs, rules, messages, paths and test files 136 to 146 changed:

- 129 of 147 plans reference something the range changed: 63 directly (the
  block changed, or names a changed rule, message or test function), 54 by
  running a changed test file or suite, 12 incidentally. 14 reference nothing;
  4 declare nothing (the legacy ledger).
- 68 plans run `check`, `lint` or `index check` against this repository, so any
  change to those verbs reaches them whatever they assert.
- 089 is caught by path (144's range changed `scripts/verify-sweep.sh`). 046
  is caught only through that generic line: its block names no rule, message or
  path of 144; it broke through a link in its own fixture.

A selector keyed on references therefore selects close to the whole corpus for
any engine change, and is narrower only for changes outside engine source.

## 2. Territory

- `scripts/verify-sweep.sh`: `--affected-by <base>`, and a pre-merge mode.
- `scripts/test-verify-sweep.py`: its cases.
- `AGENTS.md` "Working the backlog" step 6 and the `/ship` and `/shepherd`
  skills: the step and where its result is recorded.

## 3. Behavior

### 3.1 The selector

`verify-sweep.sh --affected-by <base>` MUST select, for the change
`<base>...<rev>`:

- every spec, when the change touches engine source (`crates/*/src/**`,
  `crates/*/schemas/**`), `scripts/verify-sweep.sh`, `Cargo.toml` or
  `Cargo.lock`;
- otherwise every spec whose effective plan names a spec id whose `spec.md`
  changed, a changed path, or a changed test file, plus every spec whose
  `spec.md` changed.

The report MUST say which rule selected each spec.

### 3.2 It runs before merge, on the pull request's head

Before a pull request that changes anything but documentation merges, the
session driving it MUST run `verify-sweep.sh --affected-by <base>` on the
pull request's head and record the head SHA and the counts in the pull
request. A `failed` or `not-run` outcome blocks the merge until it is fixed or
a spec is filed for it; the step never marks a block exempt.

Where it runs is the owner's decision (D-1), because each option changes a
trust boundary spec 099 set:

- **A. The driving session, locally** (recommended). The session authored the
  branch, so the sweep's trusted ref is the branch head itself
  (`--trusted-ref <head>`), an explicit override 089 §3.7 already allows. No
  workflow changes; a stranger's pull request is never swept by CI.
- **B. CI on the merge queue** (`merge_group`): it runs only on a change a
  maintainer has queued, which is closer to 099's rule, but costs the queue
  about 45 minutes per engine change, sequentially.

### 3.3 The post-merge sweep stays

The acceptance workflow on the default branch (spec 099) and the pre-release
sweep are unchanged. This step moves detection earlier; it does not replace
the release verdict.

## 4. Out of scope

- Selecting by diagnostic code or message text: measured in 1.2 as adding
  nothing the path and engine-source rules do not already select.
- Parallelizing the sweep.

## 5. Resolved decisions

**D-1 (open, owner): where the pre-merge run executes.** Option A or B in 3.2.

## Verification

```verify:cli
# 3.1: the selector's cases, including the whole-corpus fallback on engine source.
sh -c 'python3 scripts/test-verify-sweep.py 2>&1 | grep -q "affected_by"'
sh -c 'scripts/verify-sweep.sh --help 2>&1 | grep -q -- "--affected-by"'
# 3.2: the step is where the loop is.
grep -q -- '--affected-by' AGENTS.md
```
