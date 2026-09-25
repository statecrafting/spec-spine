---
id: "140-a-prose-only-example-stays-prose-only"
title: "A prose-only example stays prose-only"
status: approved
kind: "tooling"
created: "2026-09-25"
summary: >
  Spec 043's acceptance, and one of its tests, use spec 041 as the corpus's
  standing example of a spec whose acceptance is prose only: `verify 041` must
  report `not-declared`. Spec 139 carries 041's acceptance, which makes that
  example false while the rule it illustrates still holds. This spec carries
  043's block unchanged except that the example is 037, which stays prose-only
  by design (139 3.3), and moves the test's example the same way.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
amends_verification:
  - "043-verify-declared-acceptance"
amends:
  - "043-verify-declared-acceptance"
extends:
  # 3.2 the test's live example.
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-core/tests/verify.rs", nature: additive }
---

# 140: A prose-only example stays prose-only

## 1. Purpose

Spec 043 §1.2 requires that a spec whose acceptance is written only in prose
stay distinguishable from a pass: `verify` reports it `not-declared` and exits
0. Its acceptance demonstrates that on a live spec, 041, and
`tests/verify.rs::spec_041_is_not_declared` asserts the same.

Spec 139, filed in the same change, gives 041 an executable acceptance under
`amends_verification`. Against that tree, `verify 041` runs 139's block and
reports `passed`, so 043's block fails at its sixth command and the test fails.
Nothing about the rule changed; its example did.

## 2. Territory

This spec establishes nothing. It carries 043's acceptance under
`amends_verification` without editing 043's file (spec 037), and extends 043's
`crates/spec-spine-core/tests/verify.rs` to change which spec the test reads.

## 3. Behavior

### 3.1 043's acceptance names a spec that stays prose-only

`spec-spine verify 043` MUST run this spec's block. The block is 043's, command
for command, with `041-in-progress-is-in-flight` and `041` replaced by
`037-amendment-authoring` and `037`, and 043's self-re-entry line naming the
running spec rather than 043 (D-3). Spec 037 is one of the four legacy
entries 139 3.3 keeps exempt by design, because what it forbids cannot be
checked on a merged tree, so it is expected to stay prose-only.

### 3.2 The test reads the same spec

`spec_041_is_not_declared` becomes `spec_037_is_not_declared`, reading 037. The
assertion is unchanged.

### 3.3 043 says its block no longer runs

043's own `## Verification` section carries spec 082 3.4's "Superseded
acceptance" note naming this spec, as 067 to 070 do for 132. The note is the
one edit 082 requires in an amended spec's file; nothing 043 requires changes.

## 4. Out of scope

**A fixture instead of a live spec.** The live example is kept deliberately:
043 wants the corpus's own shape demonstrated, and a fixture already covers the
grammar in `tests/verify.rs`. If 037 ever gains an executable acceptance, this
example moves again.

## 5. Resolved decisions

**D-1 (2026-09-25): a holder of its own.** 043's block runs the whole CLI test
suite. Carried by 139, it would run once for each of 139's ten targets on every
release sweep; carried here, it runs twice (for 043 and for this spec).

**D-2 (2026-09-25): the superseded-note test finds the fence by line.**
`every_superseded_verification_block_says_so_in_its_own_document` located a
spec's block with the first occurrence of the fence text anywhere in the file.
043 names the fence in its §3.2 grammar table, above its block, so the test
looked for the note above the table and failed with the note correctly placed.
No held spec had named the fence before its block until now. The test now
matches the fence at the start of a line, which is what a fence is; the rule
it asserts is unchanged.

**D-3 (2026-09-25): the re-entry line names the running spec.** 043's block
ends with `verify 043; test $? -eq 1`, which demonstrates the re-entry refusal
because, run as 043, it re-enters itself. The guard keys on the id that was
asked for, so carried here the line re-enters under `verify 043` but not under
`verify 140`, where it runs 043's plan once more and passes. The carried line
re-enters the last id on `SPEC_SPINE_VERIFY_STACK`, the stack `verify` gives
every command it runs, which is the running spec under either name.

## Verification

```verify:cli
# ---- carried for 043-verify-declared-acceptance (amends_verification), the example moved from 041 to 037 (3.1) ----
# The block is self-contained: the commands below invoke the release binary,
# and `cargo test` builds only debug artifacts, so it is built first. An
# orchestrator runs this in a clean checkout of the merged sha.
cargo build --release --locked
cargo test -p spec-spine-core --test verify --locked
cargo test -p spec-spine-types --test dtos --locked
cargo test -p spec-spine-cli --locked
# not-declared is an honest zero: 037 declares acceptance in prose only.
target/release/spec-spine verify 037-amendment-authoring
test "$(target/release/spec-spine verify 037 --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["report"]["outcome"])')" = "not-declared"
# The short id resolves, and the envelope names this verb (spec 034).
test "$(target/release/spec-spine verify 037 --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["verb"])')" = "verify"
# A missing spec is 1 (not found), never 2 (stale).
target/release/spec-spine verify 999-no-such-spec; test $? -eq 1
# 3.7, demonstrated on this very block: re-entry is refused, so this line
# terminates instead of forking without bound. Carried, the block runs as 043
# and as 140, so it re-enters whichever is running: the last id on the stack
# `verify` hands its commands (D-3).
sh -c 'target/release/spec-spine verify "${SPEC_SPINE_VERIFY_STACK##*,}"; test $? -eq 1'
# ---- 140's own ----
# 3.2: the test reads 037, and the old name is gone.
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact spec_037_is_not_declared 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c '! grep -q "fn spec_041_is_not_declared" crates/spec-spine-core/tests/verify.rs'
# 3.3 and D-2: 043 carries the note, and the corpus-wide note test holds with it.
grep -q '^> `140-a-prose-only-example-stays-prose-only` declares this spec in$' specs/043-verify-declared-acceptance/spec.md
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact every_superseded_verification_block_says_so_in_its_own_document 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
```
