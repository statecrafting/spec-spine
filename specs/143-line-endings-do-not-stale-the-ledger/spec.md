---
id: "143-line-endings-do-not-stale-the-ledger"
title: "Line endings do not stale the ledger"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Every emitted derived file is canonical JSON with LF endings, and the
  freshness reads compare the committed bytes with the recompute exactly (spec
  086). Git on Windows with `core.autocrlf=true` and no `.gitattributes` rule
  for the derived tree checks those files out with CRLF, so every shard read as
  `modified`: a false staleness that regenerating cannot cure, because the next
  checkout converts again. Spec 134 quarantined three tests for it as WF-7. This
  spec makes the comparison read a committed file whose only difference is CRLF
  for LF as the same text, in the registry, the index and the inputs sidecar,
  keeps every other byte significant, and lifts the quarantine so the Windows
  test job proves it.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "069-the-committed-index-is-compared-not-trusted"
  - "134-the-suite-runs-on-windows"
  - "141-a-governance-edit-rewrites-one-file"
amends:
  # 3.1: 028's and 069's comparisons were byte-exact; they fold CRLF now.
  - "028-registry-freshness-check"
  - "069-the-committed-index-is-compared-not-trusted"
  # 3.2: 134 listed WF-7 as quarantined; it is fixed.
  - "134-the-suite-runs-on-windows"
extends:
  # 3.1 the fold, and the three comparisons that use it.
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  # 3.2 the quarantine 134 placed, lifted.
  - { spec: "134-the-suite-runs-on-windows", unit: "crates/spec-spine-cli/tests/cli.rs", nature: corrective }
  - { spec: "134-the-suite-runs-on-windows", unit: "crates/spec-spine-cli/tests/couple.rs", nature: corrective }
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/line_endings.rs" }
---

# 143: Line endings do not stale the ledger

## 1. Purpose

Spec 134 put the workspace suite on `windows-latest` and found WF-7: three
tests that commit shards to a scratch repository failed there, because git
(`core.autocrlf=true` on the runner) checked the shards out with CRLF endings
and the freshness comparison of specs 028 and 069 compared bytes. Its cause
was inferred rather than measured. `tests/line_endings.rs` measures it on every
platform by converting a committed tree to CRLF: before this spec, the registry
and the index both read `modified`.

An adopter on Windows without a `.gitattributes` rule for the derived tree gets
the same false staleness from every `check`, and the remedy the tool names,
regenerating, is undone by the next checkout.

## 2. Territory

- `crates/spec-spine-core/src/shard.rs`: `same_committed_text`.
- `crates/spec-spine-core/src/compile.rs`, `index.rs`: the three comparisons
  use it.
- The three quarantined tests, and `docs/windows-findings.md`.

## 3. Behavior

### 3.1 CRLF is the same text (amends 028 §3.1, 069 §3.1)

The freshness comparison of a committed registry shard, index shard or inputs
sidecar (spec 141) with the recompute MUST treat the committed file as
unchanged when its bytes equal the emitted bytes, or equal them once every
`\r\n` pair is read as `\n`. Nothing else folds: a lone `\r`, trailing
whitespace, a reordered key or any other byte is still `modified`.

What the tool writes is unchanged: canonical JSON, LF endings.

### 3.2 WF-7 is fixed (amends 134)

The three tests quarantined for WF-7 run on Windows again, each with a comment
naming the fix, and `docs/windows-findings.md` records WF-7 as fixed. The
`test (windows)` job is the proof on a real CRLF checkout; `line_endings.rs`
is the proof on every platform.

## 4. Out of scope

**Attestations.** A signed attestation and its seal are verified over the exact
bytes that were signed (spec 068), so a CRLF checkout of them still fails
verification. That is the right answer for a signature. A repository that
commits attestations should pin the derived tree to LF in `.gitattributes`
(`<derived_dir>/** text eol=lf`), as this one does for every text file.

**The scaffold.** It produces no `.gitattributes`, and this spec adds none:
the comparison no longer needs one, and the file belongs to the adopter.

## 5. Resolved decisions

**D-1 (2026-09-25): fold on read, not a `.gitattributes` fragment.** A
fragment would fix a repository only once someone adds it, and would add a file
to the scaffold's contract with Statecraft. Reading CRLF as the same text fixes
every checkout, including one made before the fragment existed.

**D-2 (2026-09-25): only `\r\n`.** Git's conversion produces `\r\n` from `\n`
and nothing else. Folding a lone `\r` too would accept a file git did not
produce.

## Verification

```verify:cli
# 3.1: a CRLF checkout of a fresh tree is fresh; a real change under CRLF is
# still seen; only CRLF folds. Two of the three fail with the strict byte
# comparison restored (recorded in the PR).
sh -c 'cargo test -p spec-spine-core --locked --test line_endings 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.2: the quarantine is gone, and the findings list records the fix.
sh -c '! grep -rq "ignore = \"WF-7" crates/'
grep -q '^| WF-7 | .* | \*\*fixed\*\* in spec 143' docs/windows-findings.md
# 3.2: the formerly quarantined tests pass on this platform.
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact delta_classifies_under_the_merge_base_not_the_checked_out_head delta_prose_says_what_not_required_does_not_mean 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test couple -- --exact no_deletion_builds_no_prior_snapshot 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
```
