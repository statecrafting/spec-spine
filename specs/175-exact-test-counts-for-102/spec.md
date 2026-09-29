---
id: "175-exact-test-counts-for-102"
title: "Exact test counts for 102"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 102's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 102's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "102-a-ready-spec-carries-its-status"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "102-a-ready-spec-carries-its-status"
amends:
  - "102-a-ready-spec-carries-its-status"
---

# 175: Exact test counts for 102

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 102's. 102 holds 053 and 087, so their plans resolve here too; the superseded notes of 053 and 087 name this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 102:62 | core `verify` | `spec103_` | 5 |

"Block line" counts lines inside 102's `verify:cli` fence. The names each
line ran:

- `verify` `spec103_`: `spec103_a_cycle_does_not_hang_verify`, `spec103_a_spec_with_no_amender_runs_its_own_block`, `spec103_a_withdrawn_amender_does_not_hold_the_acceptance`, `spec103_resolution_follows_the_chain`, `spec103_verify_runs_the_replacement_block`.

## 2. Territory

This spec establishes nothing. It edits 102 only to add spec 082 §3.4's
superseded-acceptance note above 102's block, and the notes of 053 and 087 to name this spec.

## 3. Behavior

### 3.1 102's block is carried

`spec-spine verify 102` and `verify` of 053 and 087 MUST run this
spec's block, which carries 102's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

102 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 053 and 087 keep their notes and gain one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 102's plan and that of 053 and 087 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 102-a-ready-spec-carries-its-status (amends_verification), and through it 053, 087; each loose test count names its tests (153 3.2) ----
# --- spec 053's acceptance, held by 087 and carried here (D-7) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test query --locked
# A scratch corpus with two ready specs and one blocked by the first, so the
# shape assertions below hold whatever this repository's own backlog is doing,
# and so "the first element of ready" has a failing case (3.4, D-3).
rm -rf "${TMPDIR:-/tmp}/ss053" && mkdir -p "${TMPDIR:-/tmp}/ss053/specs/001-alpha" "${TMPDIR:-/tmp}/ss053/specs/002-beta" "${TMPDIR:-/tmp}/ss053/specs/003-gamma" && : > "${TMPDIR:-/tmp}/ss053/spec-spine.toml" && printf -- '---\nid: "001-alpha"\ntitle: "First thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/001-alpha/spec.md"\n---\n\n# 001-alpha\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/001-alpha/spec.md" && printf -- '---\nid: "002-beta"\ntitle: "Second thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\ndepends_on:\n  - "001-alpha"\nestablishes:\n  - "specs/002-beta/spec.md"\n---\n\n# 002-beta\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/002-beta/spec.md" && printf -- '---\nid: "003-gamma"\ntitle: "Third thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/003-gamma/spec.md"\n---\n\n# 003-gamma\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/003-gamma/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" compile >/dev/null
# The two documents are captured to files, so each verb's own exit status is its
# line's status, which a pipeline into `python3` would not be (3.2, D-5).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan --json > "${TMPDIR:-/tmp}/ss053/plan.json"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan --next --json > "${TMPDIR:-/tmp}/ss053/next.json"
# 053 3.3: the ready array carries titles, so no consumer needs a second call.
# With two ready specs this is also a claim about order (3.4).
python3 -c "import json; p=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json')); assert p['ready'][0]=={'id':'001-alpha','title':'First thing','status':'approved'}, p"
# 053 3.1: and each blocked entry carries its title and the state of every
# blocker, rather than a count of them.
python3 -c "import json; b=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json'))['blocked'][0]; assert b['title']=='Second thing', b; assert b['blockedBy'][0]['id']=='001-alpha', b; assert b['blockedBy'][0]['state'], b"
# 053 3.1: the prose form renders what the structure holds, remainder included.
# Captured once rather than piped twice, so the verb's own exit status is a
# line's status and the two assertions read the same rendering (3.2).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan > "${TMPDIR:-/tmp}/ss053/plan.txt"
grep -q 'not schedulable' "${TMPDIR:-/tmp}/ss053/plan.txt"
grep -q 'blocked by 001-alpha' "${TMPDIR:-/tmp}/ss053/plan.txt"
# 087 3.3: 053 3.2's pick, read from the member spec 074 moved it into. Sorted keys
# and the version member are 074 3.2's rule for every governed read, and the
# pick is compared by value so a `--next` that dropped the title fails (D-1).
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss053/next.json')); k=list(d); assert k==sorted(k), k; assert d['schemaVersion'], d; assert d['next']=={'id':'001-alpha','title':'First thing','status':'approved'}, d"
# 3.4: 053 3.2's projection requirement, which a one-element ready set cannot
# show. `--next` names the first element of `ready` rather than reimplementing
# selection; with two ready specs, answering `003-gamma` fails this line.
python3 -c "import json; n=json.load(open('${TMPDIR:-/tmp}/ss053/next.json'))['next']; p=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json')); assert p['ready'][0]==n, (p['ready'], n)"
# 053 3.2: and an empty ready set is a true answer at exit 0, not a failure.
# This repository is that case now. Nothing is asserted about the contents:
# that document's `next: null` path is guarded by spec 074 3.8 in
# crates/spec-spine-cli/tests/cli.rs, and asserting it here would pin corpus
# state, which 053's own decision of 2026-09-08 rejects (3.6, D-4).
target/release/spec-spine registry plan --next
rm -rf "${TMPDIR:-/tmp}/ss053"
# 053 3.3: the ledger is untouched by a read verb.
target/release/spec-spine compile --check
# --- spec 087's own mechanism (087 3.5), carried unchanged ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-5 gives: at the parent commit this
# verb exits 1 and prints nothing. The file is named for this spec, whose
# mechanism it is, not for 053, whose acceptance the half above is (087 3.2).
target/release/spec-spine registry show 087 --json > "${TMPDIR:-/tmp}/ss087-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss087-show.json')); assert d['amendsVerification'] == ['053-plan-answers-the-whole-question'], d; assert d['amends'] == ['053-plan-answers-the-whole-question'], d"
rm -f "${TMPDIR:-/tmp}/ss087-show.json"
# Spec 053's file is not edited (spec 037 3.1): its own block still carries the
# superseded whole-document equality. This goes red the moment someone resolves
# this by editing 053 instead.
grep -qF 'assert json.load(sys.stdin)=={"id":"001-alpha","title":"First thing"}' specs/053-plan-answers-the-whole-question/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 053`, `verify 087` and
# `verify 102` run, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact spec103_a_cycle_does_not_hang_verify spec103_a_spec_with_no_amender_runs_its_own_block spec103_a_withdrawn_amender_does_not_hold_the_acceptance spec103_resolution_follows_the_chain spec103_verify_runs_the_replacement_block 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss087-verify-tests.txt"
# --- this spec's amendment of 087 (D-7) ---
target/release/spec-spine registry show 102 --json > "${TMPDIR:-/tmp}/ss102-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss102-show.json')); assert d['amendsVerification'] == ['087-the-answer-is-a-member-not-the-document'], d; assert d['amends'] == ['087-the-answer-is-a-member-not-the-document'], d"
rm -f "${TMPDIR:-/tmp}/ss102-show.json"
# 087's commands are not edited either (only its superseded note, spec 082
# 3.4): it still carries the two-member form this block replaced. Red if
# someone repairs 087's commands in place instead.
grep -qF "assert p['ready'][0]=={'id':'001-alpha','title':'First thing'}, p" specs/087-the-answer-is-a-member-not-the-document/spec.md
# --- this spec's own acceptance ---
# 3.1: the field exists, and is the verbatim status string.
grep -q 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs
grep -A6 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs | grep -q 'pub status: String'
# 3.1: and is not a derived boolean.
test -z "$(grep -A6 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs | grep 'approved: bool')"
# 3.4: blocked entries did not gain it.
test -z "$(grep -A8 'pub struct BlockedSpec' crates/spec-spine-core/src/query.rs | grep 'pub status')"
# 3.2: membership and order are unchanged, asserted by name.
grep -q 'ready_order_is_unchanged_by_status' crates/spec-spine-core/tests/query.rs
cargo test -p spec-spine-core --test query --locked
# 3.3: the read schema version moved and the registry schema did not.
grep -q 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
cargo test --workspace emitted_registry_conforms --locked
# The document actually carries it.
./target/release/spec-spine registry plan --json
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
