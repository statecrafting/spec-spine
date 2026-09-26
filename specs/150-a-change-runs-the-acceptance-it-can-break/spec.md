---
id: "150-a-change-runs-the-acceptance-it-can-break"
title: "A change runs the acceptance it can break"
status: approved
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
implementation: in-progress
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

The report MUST say which rule selected each spec (`engine-source`,
`changed-spec`, `names-spec`, `names-path` or `names-test`).

`scripts/test-verify-sweep.py` MUST carry one case per rule, named
`test_affected_by_engine_source_selects_every_spec`,
`test_affected_by_a_changed_spec_selects_itself`,
`test_affected_by_a_named_spec_is_selected`,
`test_affected_by_a_named_path_is_selected` and
`test_affected_by_a_named_test_is_selected`, and a sixth,
`test_affected_by_an_unrelated_spec_is_not_selected`, proving the selector is
narrower than the corpus outside engine source.

### 3.2 It runs before merge, on the pull request's head

Before a pull request merges, unless its diff touches only `docs/`,
`website/`, or Markdown files outside `specs/`, the session driving it MUST
run `verify-sweep.sh --affected-by <base>` on the
pull request's head and record the head SHA and the counts in the pull
request. A `failed` or `not-run` outcome blocks the merge until it is fixed or
a spec is filed for it; the step never marks a block exempt.

It runs in the driving session, locally (D-1). The session authored the
branch, so the sweep's trusted ref is the branch head itself
(`--trusted-ref <head>`), an explicit override 089 §3.7 already allows. No
workflow changes, and a stranger's pull request is never swept by CI.

### 3.3 The post-merge sweep stays

The acceptance workflow on the default branch (spec 099) and the pre-release
sweep are unchanged. This step moves detection earlier; it does not replace
the release verdict.

## 4. Out of scope

- Selecting by diagnostic code or message text: measured in 1.2 as adding
  nothing the path and engine-source rules do not already select.
- Parallelizing the sweep.

## 5. Resolved decisions

**D-1 (2026-09-25, owner): the pre-merge run executes in the driving session,
locally.** Each option changes a trust boundary spec 099 set. Chosen: the
session that authored the branch runs the sweep against its own head, with
`--trusted-ref <head>` visible in the report; it costs that session about 45
minutes per engine change and changes no workflow. Considered and not taken:
CI on the merge queue (`merge_group`), which runs only on a change a maintainer
has queued and so stays closer to 099's rule, but costs the queue about 45
minutes per engine change, sequentially; its runner cost could be carried by
self-hosted runners paid with cloud credits. That option stays open for a later
spec if session time becomes the constraint; adopting it is a change to the
check suite and so the owner's.

**D-2 (2026-09-25, build): `Cargo.toml` in the engine-source rule is the
workspace manifest and each `crates/<crate>/Cargo.toml`.** 3.1 names
`Cargo.toml` without a path; a crate's own manifest changes what that crate
builds, so it selects every spec too. `Cargo.lock` is the root one.

**D-3 (2026-09-25, build): how a plan "names" something.** The plan is the
text `verify <id> --plan` prints at the revision under test. It names a spec
when it contains the full id, or the id's three-digit ordinal as a token not
joined to a letter or digit (`verify 046`, the short form spec-spine
resolves); a path when it contains the changed path as written from the
repository root; a test when a line runs `cargo test` whose `-p`/`--package`
(if any) names the changed file's crate and whose `--test` (if any) names its
target, where `crates/<crate>/tests/<name>.rs` is target `<name>` and any
other file under `tests/` (a fixture, a shared module) reaches every target
of the crate. Each selected spec reports one rule, the first that
matched in the order engine-source, changed-spec, names-spec, names-path,
names-test.

**D-4 (2026-09-25, build): what the selector refuses.** A plan it cannot read
is a spec it cannot rule out, so `--affected-by` refuses (exit 3) rather than
leave it out; `--only` and `--affected-by` together are refused as two
selections; a base that does not resolve or shares no history with `--rev`
is refused before anything is created. An empty selection is a true answer:
nothing runs and the sweep exits 0.

**D-5 (2026-09-25, build): the report.** An `--affected-by` run writes report
schema 1.2.0: 1.1.0 plus `affectedBy` (base, its revision, the merge base,
the changed paths, corpus size, number selected) and a `selectedBy` rule on
each row, and a "Selection" table in `sweep.md`. Every other run writes the
1.1.0 report it wrote before, unchanged.

## Verification

```verify:cli
# 3.1: the suite passes, and each of the six named selector cases ran and passed.
sh -c 'O="${TMPDIR:-/tmp}/ss150.out"; python3 scripts/test-verify-sweep.py > "$O" 2>&1; rc=$?; r=0; for t in test_affected_by_engine_source_selects_every_spec test_affected_by_a_changed_spec_selects_itself test_affected_by_a_named_spec_is_selected test_affected_by_a_named_path_is_selected test_affected_by_a_named_test_is_selected test_affected_by_an_unrelated_spec_is_not_selected; do grep -q "^$t .* ok$" "$O" || { echo "missing or failed: $t"; r=1; }; done; rm -f "$O"; test $rc -eq 0 && test $r -eq 0'
sh -c 'scripts/verify-sweep.sh --help 2>&1 | grep -q -- "--affected-by"'
# 3.2: the step is where the loop is.
grep -q -- '--affected-by' AGENTS.md
```
