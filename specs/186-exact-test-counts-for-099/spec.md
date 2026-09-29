---
id: "186-exact-test-counts-for-099"
title: "Exact test counts for 099"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 099's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 099's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "099-a-merged-acceptance-is-asked-again"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "099-a-merged-acceptance-is-asked-again"
amends:
  - "099-a-merged-acceptance-is-asked-again"
---

# 186: Exact test counts for 099

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 099's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 099:66 | core `gate` | `acceptance` | 5 |

"Block line" counts lines inside 099's `verify:cli` fence. The names each
line ran:

- `gate` `acceptance`: `the_acceptance_workflow_calls_the_scripts_and_names_the_trusted_ref`, `the_acceptance_workflow_carries_a_read_only_token_and_no_secret`, `the_acceptance_workflow_is_outside_the_required_check`, `the_acceptance_workflow_is_reachable_from_no_pull_request`, `the_acceptance_workflow_writes_nothing_back`.

## 2. Territory

This spec establishes nothing. It edits 099 only to add spec 082 §3.4's
superseded-acceptance note above 099's block.

## 3. Behavior

### 3.1 099's block is carried

`spec-spine verify 099` MUST run this
spec's block, which carries 099's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

099 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 099's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 099-a-merged-acceptance-is-asked-again (amends_verification); each loose test count names its tests (153 3.2) ----
cargo build --release --locked
# --- 3.1: what the workflow can read, and what it can block ---
test -f .github/workflows/acceptance.yml
test -f .github/workflows/acceptance.yml && ! grep -qE '^[[:space:]]*(pull_request|pull_request_target|merge_group):' .github/workflows/acceptance.yml
grep -qE '^[[:space:]]*schedule:' .github/workflows/acceptance.yml
grep -qE '^[[:space:]]*workflow_dispatch:' .github/workflows/acceptance.yml
! grep -qF 'acceptance' .github/workflows/ci.yml
# --- 3.4: the decision about the token ---
grep -qF 'contents: read' .github/workflows/acceptance.yml
# Comment lines excluded: the workflow explains at length why it has no
# secret, and a grep that read that sentence would refuse the file for saying
# so. The same exclusion is in the suite's copy of this assertion.
test -f .github/workflows/acceptance.yml && ! grep -v '^[[:space:]]*#' .github/workflows/acceptance.yml | grep -qE 'secrets:|secrets\.'
# --- 3.5 and D-4: the trusted ref is named, and the sweep's exit is the step's
grep -qF -- '--trusted-ref origin/main' .github/workflows/acceptance.yml
grep -qF 'exit $rc' .github/workflows/acceptance.yml
# --- 3.7: the instrument reports, it does not repair ---
test -f .github/workflows/acceptance.yml && ! grep -v '^[[:space:]]*#' .github/workflows/acceptance.yml | grep -qE 'git (push|commit)|gh (pr|issue) create'
# --- 3.2: the scope is a script, and it refuses a question it cannot ask ---
test -x scripts/acceptance-scope.sh
scripts/acceptance-scope.sh --head HEAD >/dev/null 2>&1; test $? -eq 3
scripts/acceptance-scope.sh --base nope-not-a-rev --head HEAD >/dev/null 2>&1; test $? -eq 3
# --- 3.2 on a fixture: an owned change selects its spec ---
rm -rf "${TMPDIR:-/tmp}/ss099" "${TMPDIR:-/tmp}/ss099-owned.txt" "${TMPDIR:-/tmp}/ss099-unowned.txt" "${TMPDIR:-/tmp}/ss099-doc.txt"
mkdir -p "${TMPDIR:-/tmp}/ss099/specs/000-alpha" "${TMPDIR:-/tmp}/ss099/src"
printf '[layout]\nspecs_dir = "specs"\nderived_dir = ".derived"\n' > "${TMPDIR:-/tmp}/ss099/spec-spine.toml"
printf -- '---\nid: "000-alpha"\ntitle: "Alpha"\nstatus: approved\ncreated: "2026-09-21"\nsummary: "s"\nimplementation: complete\nestablishes:\n  - "src/a.rs"\n---\n\n# 000: Alpha\n\n## 1. Purpose\n' > "${TMPDIR:-/tmp}/ss099/specs/000-alpha/spec.md"
printf 'fn a() {}\n' > "${TMPDIR:-/tmp}/ss099/src/a.rs"
printf 'fn b() {}\n' > "${TMPDIR:-/tmp}/ss099/src/b.rs"
git -C "${TMPDIR:-/tmp}/ss099" init -q
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm base
target/release/spec-spine compile --repo "${TMPDIR:-/tmp}/ss099" > /dev/null
target/release/spec-spine index --repo "${TMPDIR:-/tmp}/ss099" > /dev/null
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm derived
printf '// owned change\n' >> "${TMPDIR:-/tmp}/ss099/src/a.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm owned
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-owned.txt"
grep -qx '000-alpha' "${TMPDIR:-/tmp}/ss099-owned.txt"
# --- 3.2: and an unowned change selects nothing, which is a pass, not a sweep
printf '// unowned change\n' >> "${TMPDIR:-/tmp}/ss099/src/b.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm unowned
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-unowned.txt"
test ! -s "${TMPDIR:-/tmp}/ss099-unowned.txt"
# --- 3.2: a change to a spec's own document selects that spec ---
printf 'A sentence.\n' >> "${TMPDIR:-/tmp}/ss099/specs/000-alpha/spec.md"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm doc
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-doc.txt"
grep -qx '000-alpha' "${TMPDIR:-/tmp}/ss099-doc.txt"
# 3.2: a stale ledger is a question that cannot be asked, not an empty answer.
# Found by this block: an earlier draft wrote its output INTO the fixture repo,
# the next commit carried it, and the scope of an unindexed file refused.
printf 'fn c() {}\n' > "${TMPDIR:-/tmp}/ss099/src/c.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm stale
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" >/dev/null 2>&1; test $? -eq 3
rm -rf "${TMPDIR:-/tmp}/ss099" "${TMPDIR:-/tmp}/ss099-owned.txt" "${TMPDIR:-/tmp}/ss099-unowned.txt" "${TMPDIR:-/tmp}/ss099-doc.txt"
# --- 3.6: spec 089's line, corrected rather than deleted ---
grep -qF '.claude/skills/' specs/089-nothing-reruns-a-merged-acceptance/spec.md
grep -qF 'workflows/acceptance.yml' specs/089-nothing-reruns-a-merged-acceptance/spec.md
# --- the workflow's shape, asserted in the suite as well as here ---
# exact on a Unix sweep host with default features: gate has one `#[cfg(unix)]` test (spec 153 3.3).
sh -c 'cargo test -p spec-spine-core --locked --test gate -- --exact the_acceptance_workflow_calls_the_scripts_and_names_the_trusted_ref the_acceptance_workflow_carries_a_read_only_token_and_no_secret the_acceptance_workflow_is_outside_the_required_check the_acceptance_workflow_is_reachable_from_no_pull_request the_acceptance_workflow_writes_nothing_back 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss099-g.txt"
# --- the governed loop, over the corpus this spec is part of ---
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
