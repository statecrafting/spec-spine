---
id: "172-exact-test-counts-for-084"
title: "Exact test counts for 084"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 084's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 084's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
amends:
  - "084-an-acceptance-outlives-the-output-it-was-written-against"
---

# 172: Exact test counts for 084

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 084's. 084 holds 044, so its plan resolves here too; the superseded note of 044 names this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 084:54 | core `verify` | `spec103_` | 5 |

"Block line" counts lines inside 084's `verify:cli` fence. The names each
line ran:

- `verify` `spec103_`: `spec103_a_cycle_does_not_hang_verify`, `spec103_a_spec_with_no_amender_runs_its_own_block`, `spec103_a_withdrawn_amender_does_not_hold_the_acceptance`, `spec103_resolution_follows_the_chain`, `spec103_verify_runs_the_replacement_block`.

## 2. Territory

This spec establishes nothing. It edits 084 only to add spec 082 §3.4's
superseded-acceptance note above 084's block, and the note of 044 to name this spec.

## 3. Behavior

### 3.1 084's block is carried

`spec-spine verify 084` and `verify` of 044 MUST run this
spec's block, which carries 084's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

084 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 044 keeps its note and gains one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 084's plan and that of 044 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 084-an-acceptance-outlives-the-output-it-was-written-against (amends_verification), and through it 044; each loose test count names its tests (153 3.2) ----
# --- spec 044's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test diagnostics --locked
cargo test -p spec-spine-cli --locked
# This corpus carries no unresolved units, so the strict flag passes here.
target/release/spec-spine index check --fail-on-unresolved
# Scratch: each line is its own shell, so a verb's stdout is carried in a file
# rather than a variable. The redirect leaves the verb's own exit status as the
# line's status, which a pipeline into `head` or `python3` would not (3.2, D-4).
rm -rf "${TMPDIR:-/tmp}/ss106" && mkdir -p "${TMPDIR:-/tmp}/ss106"
# 3.1 (amended by 3.2): a clean corpus keeps the bare verdict line. Spec 050's
# unwitnessed-claims line is a second line about a different subject, so the
# assertion reads the verdict line rather than the whole of stdout.
target/release/spec-spine index check > "${TMPDIR:-/tmp}/ss106/check.txt"
test "$(head -1 "${TMPDIR:-/tmp}/ss106/check.txt")" = "index is fresh"
# 3.4: the read verb answers, and answers as JSON. Spec 074 wrapped the payload
# in an `items` envelope, so the count is of `items` and not of the document.
target/release/spec-spine index diagnostics
target/release/spec-spine index diagnostics --json > "${TMPDIR:-/tmp}/ss106/diagnostics.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss106/diagnostics.json')); assert len(d['items'])==0, d"
# 3.3: the envelope version is compared across two verbs whose payloads differ,
# not pinned to a literal. This is a consistency check: it witnesses one
# envelope constant rather than per-payload versioning, and it does NOT prove
# that a payload addition left the version unchanged, since a bump made for a
# payload reason would move both verbs together. The payload difference is
# asserted as a named verb token and a named member, not as a key-set
# inequality, which would be the same calendar shape on another axis (D-6).
# The counter-verb is anchored on spec 000, the tier-1 bootstrap spec, because
# `compile --spec` needs an id that exists and 000 is the one id whose removal
# would end the corpus rather than move this line (D-8).
target/release/spec-spine index check --json > "${TMPDIR:-/tmp}/ss106/check.json"
target/release/spec-spine compile --spec 000 --json > "${TMPDIR:-/tmp}/ss106/compile.json"
python3 -c "import json; a=json.load(open('${TMPDIR:-/tmp}/ss106/check.json')); b=json.load(open('${TMPDIR:-/tmp}/ss106/compile.json')); assert a['schemaVersion'], a; assert a['schemaVersion']==b['schemaVersion'], (a['schemaVersion'], b['schemaVersion']); assert a['verb'] != b['verb'], (a['verb'], b['verb']); assert 'diagnostics' in a['report'] and 'diagnostics' not in b['report'], (sorted(a['report']), sorted(b['report']))"
# 3.4: the member spec 044 added to `index check`'s payload is present. This is
# the antecedent 044 3.6's rule is about, and 044's block never asserted it.
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss106/check.json'))['report']; assert sorted(r['diagnostics'])==['byCode','errors','warnings'], r"
# --- spec 084's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
target/release/spec-spine registry show 084 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["amendsVerification"] == ["044-index-diagnostics-reach-a-gate"], d; assert d["amends"] == ["044-index-diagnostics-reach-a-gate"], d'
# Spec 044's file is not edited (spec 037 3.1): its own block still carries all
# three superseded assertion forms. These go red the moment someone resolves
# this by editing 050 instead.
grep -qF 'index check)" = "index is fresh"' specs/044-index-diagnostics-reach-a-gate/spec.md
grep -qF 'print(len(json.load(sys.stdin)))' specs/044-index-diagnostics-reach-a-gate/spec.md
grep -qF '= "0.2.0"' specs/044-index-diagnostics-reach-a-gate/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 050` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count:
# a name filter that matches nothing exits 0 (measured: `0 passed; 31 filtered
# out`), so the bare line would stay green while asserting nothing (D-7).
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact spec103_a_cycle_does_not_hang_verify spec103_a_spec_with_no_amender_runs_its_own_block spec103_a_withdrawn_amender_does_not_hold_the_acceptance spec103_resolution_follows_the_chain spec103_verify_runs_the_replacement_block 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
rm -rf "${TMPDIR:-/tmp}/ss106"
```
