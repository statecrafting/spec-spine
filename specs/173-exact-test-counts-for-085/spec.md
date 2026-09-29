---
id: "173-exact-test-counts-for-085"
title: "Exact test counts for 085"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 085's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 085's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "085-a-version-pin-is-not-a-contract"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "085-a-version-pin-is-not-a-contract"
amends:
  - "085-a-version-pin-is-not-a-contract"
---

# 173: Exact test counts for 085

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 085's. 085 holds 049, so its plan resolves here too; the superseded note of 049 names this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 085:55 | core `verify` | `spec103_` | 5 |

"Block line" counts lines inside 085's `verify:cli` fence. The names each
line ran:

- `verify` `spec103_`: `spec103_a_cycle_does_not_hang_verify`, `spec103_a_spec_with_no_amender_runs_its_own_block`, `spec103_a_withdrawn_amender_does_not_hold_the_acceptance`, `spec103_resolution_follows_the_chain`, `spec103_verify_runs_the_replacement_block`.

## 2. Territory

This spec establishes nothing. It edits 085 only to add spec 082 §3.4's
superseded-acceptance note above 085's block, and the note of 049 to name this spec.

## 3. Behavior

### 3.1 085's block is carried

`spec-spine verify 085` and `verify` of 049 MUST run this
spec's block, which carries 085's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

085 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 049 keeps its note and gains one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 085's plan and that of 049 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 085-a-version-pin-is-not-a-contract (amends_verification), and through it 049; each loose test count names its tests (153 3.2) ----
# --- spec 049's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test compile --locked
# 3.1: the flag exists, resolves the short id, and validates without writing.
target/release/spec-spine compile --spec 022
target/release/spec-spine compile --spec 022-index-sharding --json
# Scratch: each line is its own shell, so a verb's stdout is carried in a file
# rather than a variable. The redirect leaves the verb's own exit status as the
# line's status, which a pipeline into `python3` would not (3.2, D-4).
rm -rf "${TMPDIR:-/tmp}/ss056" && mkdir -p "${TMPDIR:-/tmp}/ss056"
target/release/spec-spine compile --spec 022 --json > "${TMPDIR:-/tmp}/ss056/spec.json"
target/release/spec-spine compile --check --json > "${TMPDIR:-/tmp}/ss056/check.json"
# 3.3: 049 3.4 requires a verb token that distinguishes `--spec` from
# `compile --check`, so both tokens are read and the envelope version is
# compared across the pair rather than pinned to a literal. The equality
# witnesses one VERDICT_SCHEMA_VERSION constant rather than per-verb
# versioning; it does NOT prove that any past bump was legitimate, which no
# command run against one tree can observe.
python3 -c "import json; a=json.load(open('${TMPDIR:-/tmp}/ss056/spec.json')); b=json.load(open('${TMPDIR:-/tmp}/ss056/check.json')); assert a['verb']=='compile.spec', a; assert b['verb']=='compile.check', b; assert a['schemaVersion'], a; assert a['schemaVersion']==b['schemaVersion'], (a['schemaVersion'], b['schemaVersion'])"
# 3.4: the payload names which spec was judged, which is where 049 3.1's
# short-id resolution actually shows. The exit-code line above passes whatever
# `024` resolved to; this one does not.
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss056/spec.json'))['report']; assert r['specId']=='022-index-sharding', r; assert r['specPath']=='specs/022-index-sharding/spec.md', r; assert r['violations']==[], r"
# 3.4: and `compile --check` answers a different question under the same
# envelope. Contingent on that verb's payload, deliberately and loudly (D-3).
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss056/check.json'))['report']; assert 'specId' not in r, r"
rm -rf "${TMPDIR:-/tmp}/ss056"
# 3.5 of spec 049: it wrote nothing. The committed shards are exactly as they were.
target/release/spec-spine compile --check
# 049 3.1: an unknown id is exit 1 (not found), never exit 2.
target/release/spec-spine compile --spec 999 ; test $? -eq 1
# 049 3.1: `--spec` and `--check` are different questions, and the pair is refused.
target/release/spec-spine compile --spec 022 --check ; test $? -eq 3
# --- spec 085's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-4 gives: at the parent commit this
# verb exits 1 and prints nothing, and a pipeline would report that as a JSON
# decode error naming the wrong defect. The file is named for this spec, whose
# mechanism it is, not for 056, whose acceptance the half above is (D-6).
target/release/spec-spine registry show 085 --json > "${TMPDIR:-/tmp}/ss107-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss107-show.json')); assert d['amendsVerification'] == ['049-compile-one-spec'], d; assert d['amends'] == ['049-compile-one-spec'], d"
rm -f "${TMPDIR:-/tmp}/ss107-show.json"
# Spec 049's file is not edited (spec 037 3.1): its own block still carries the
# superseded literal pin. This goes red the moment someone resolves this by
# editing 056 instead.
grep -qF 'v["schemaVersion"]=="0.3.0"' specs/049-compile-one-spec/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 056` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact spec103_a_cycle_does_not_hang_verify spec103_a_spec_with_no_amender_runs_its_own_block spec103_a_withdrawn_amender_does_not_hold_the_acceptance spec103_resolution_follows_the_chain spec103_verify_runs_the_replacement_block 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss107-spec103.txt"
```
