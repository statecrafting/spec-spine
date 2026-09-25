---
id: "145-an-unresolved-claim-is-not-called-stale"
title: "An unresolved claim is not called stale"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Spec 079 taught `check` and `index check` to tell a stale shard from an
  unresolved claim, because the two have different remedies and regenerating
  cures only the first. The other verbs that read the committed index (`index
  coverage`, `index owner`, `couple`, `scope`, and `delta` at the merge base)
  kept the folded verdict and still said "index is stale" about a claim that
  resolves to nothing, naming a `blocking-diagnostics` shard and implying the
  wrong fix. This spec gives them one guard that keeps the two apart: a
  committed shard whose bytes moved is staleness, as before; an unresolved claim
  with byte-exact shards is a validation finding that names the claim and says
  regenerating does not clear it. The exit code stays 1; under `--json` the
  error kind becomes `validation`, with the claim in `violations`.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "079-a-blocking-claim-is-not-a-stale-shard"
  - "132-one-exit-contract-for-the-family"
amends:
  # 3.1: 079 §3.1 left the guarded readers on the folded verdict; they move
  # onto the partition.
  - "079-a-blocking-claim-is-not-a-stale-shard"
extends:
  # 3.1 the guard and the readers that call it.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: corrective }
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: corrective }
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/src/delta.rs", nature: corrective }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-core/src/scope.rs", nature: corrective }
  # 3.2 a validation failure prints its violations.
  - { spec: "132-one-exit-contract-for-the-family", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/unresolved_guard.rs" }
---

# 145: An unresolved claim is not called stale

## 1. Purpose

Measured with 0.26.0 on a corpus whose complete spec `001-a` claims
`src/missing.rs`, which does not exist, with every shard freshly written:

```
$ spec-spine index check      # exit 1
index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1 spec(s), which is not staleness
$ spec-spine index coverage   # exit 1
spec-spine: index is stale: expected content-hash 1 shard(s) matching the corpus, got 1 stale shard(s):
  blocking-diagnostics by-spec/001-a.json
```

`couple` and `index owner` print the same "index is stale" line. The fact is
one; three verbs name it wrongly, and the remedy their wording implies,
regenerating, changes nothing.

## 2. Territory

- `crates/spec-spine-core/src/index.rs`: `IndexFreshnessReport::guard`,
  `guard_committed_index`, and the report's `blocked_drift`.
- The guarded readers: `coverage.rs`, `couple.rs`, `scope.rs`, `delta.rs` and
  `index::owner`.
- `crates/spec-spine-cli/src/main.rs`: a validation failure prints its
  violations.

## 3. Behavior

### 3.1 One guard, two refusals (amends 079)

Every verb that refuses to read a committed index that does not match the
corpus MUST call one guard, which:

- refuses with `Error::Stale` when any committed shard's bytes differ from the
  recompute, a shard that also blocks included, naming the moved shards;
- otherwise refuses with `Error::Validation` when the corpus holds an unresolved
  claim (the codes spec 044 blocks on), one violation per claim carrying its
  code, its unit as `path`, and a message that says it is an unresolved claim,
  not staleness, and that regenerating does not clear it;
- otherwise reads.

Both refusals exit 1 (spec 132). `delta` keeps "at the merge base" in its
staleness wording.

### 3.2 A validation failure names what failed

When a verb's error is a validation failure and `--json` is not set, the CLI
MUST print each violation on its own line under the summary (code, path,
message), rather than only the count.

## 4. Out of scope

`check` and `index check` already report the partition (079) and are
unchanged.

## 5. Resolved decisions

**D-1 (2026-09-25): staleness first.** When shards moved and a claim is also
unresolved, the guard reports staleness: regenerating is the first step, and
the claim is reported on the next run if it remains. A blocked shard whose
bytes moved counts as moved, because 079's one-line-per-shard report hides that
from `stale`; the report carries those lines as `blocked_drift`.

**D-2 (2026-09-25): a validation finding, not a new error kind.** An unresolved
claim is a finding about the corpus, which is what `validation` means in the
closed kind set of spec 132. A new kind would change the family contract for a
case that already has a name.

## Verification

```verify:cli
# 3.1: the readers name the claim; drift stays staleness and comes first; a
# blocked shard that moved is stale; a resolved corpus reads. With the old
# folded guard restored, 3 of 4 fail (recorded in the PR).
sh -c 'cargo test -p spec-spine-core --locked --test unresolved_guard 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
# 3.1 and 3.2 at the binary: coverage names the claim and does not say stale.
cargo build --release --locked
sh -c 'T="${TMPDIR:-/tmp}/ss145"; rm -rf "$T" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/missing.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && B="$PWD/target/release/spec-spine" && "$B" --repo "$T" compile >/dev/null 2>&1; "$B" --repo "$T" index >/dev/null 2>&1; "$B" --repo "$T" index coverage > "$T.out" 2>&1; rc=$?; rm -rf "$T"; test $rc -eq 1 && grep -q "I-004 \[src/missing.rs\] unresolved claim, not staleness" "$T.out" && ! grep -q "index is stale" "$T.out"; r=$?; rm -f "$T.out"; exit $r'
```
