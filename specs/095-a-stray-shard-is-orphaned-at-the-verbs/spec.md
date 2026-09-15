---
id: "095-a-stray-shard-is-orphaned-at-the-verbs"
title: "A stray shard is orphaned at the verbs"
status: approved
kind: "tooling"
created: "2026-09-14"
summary: >
  Spec 086 §3.1 says `orphaned` covers "a stray file in either shard
  directory", and `compare_shard_dir` classifies by name without parsing, so
  the library holds to it. The verbs do not. `check_report` and
  `check_freshness_json` compute the freshness verdict and then call
  `diagnostics::committed_counts(config, repo_root)?`, which parses every file
  in the shard directories; an unparseable stray makes that `?` throw the
  computed verdict away, and `index check` and `check` exit 3 with a parse
  error where `couple` and `index coverage` exit 2 and name the orphan.
  Measured at 0.19.0, and the same at 0.18.0, so it is not a regression and
  not a bypass. 086's acceptance asserted the classification on the library
  function only, which is exactly how it passed. This spec makes the tally
  best-effort and the verdict authoritative, and asserts it at the verbs.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "050-index-diagnostics-reach-a-gate"
  - "075-one-name-one-freshness-verb"
  - "086-the-committed-index-is-compared-not-trusted"
extends:
  # 3.2, 3.3: the tally becomes best-effort and reports what it skipped.
  - { spec: "050-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-core/src/diagnostics.rs", nature: corrective }
  # 3.4: the two facade halves that discard the verdict today.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: corrective }
  # 3.4: the CLI arms of `index check` and `check`.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  - { spec: "075-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: corrective }
  # 3.6: the acceptance, at the verbs and in 086's own suite.
  - { spec: "086-the-committed-index-is-compared-not-trusted", unit: "crates/spec-spine-core/tests/index_body.rs", nature: additive }
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 095: A stray shard is orphaned at the verbs

## 1. Purpose

### 1.1 The verdict is computed and then discarded

`check_report` (the composed verb spec 075 named) does this, in order:

1. compile, compare the committed registry, build the registry half;
2. `check_index_freshness`, which after spec 086 compares committed shard bytes
   against the recompute and classifies a file in the directory that the
   recompute does not expect as `orphaned`;
3. `diagnostics::committed_counts(config, repo_root)?`, which calls
   `read_committed_index_shards` and parses **every** file in both shard
   directories to read the diagnostics each one records.

Step 3's `?` is the defect. The verdict from step 2 is already in hand, and a
file that cannot be deserialized into an index shard makes step 3 return
`Error::Parse`, which propagates and becomes exit 3. The verb never reports what
it had already decided.

### 1.2 Measured, at the verbs

On 2026-09-14, with a copy of a valid shard and with `{"nope": 1}` placed in
`.derived/codebase-index/by-spec/`:

| Stray file | `couple` | `index coverage` | `index check` | `check` |
|---|---|---|---|---|
| a copy of a valid shard | 2, `orphaned` | 2, `orphaned` | 2, `orphaned` | 2, `orphaned` |
| `{"nope": 1}` | 2 | 2 | **3, parse error** | **3, parse error** |

A non-`.json` stray is ignored by both. The 0.18.0 binary behaves the same on
every row, so this is not a regression, and exit 3 still refuses in CI, so it is
not a bypass. It is a verb that answers the wrong question, and the maintainer's
ruling on 2026-09-13 was to ship v0.19.0 and file this.

### 1.3 Why the library test did not catch it

`core/tests/index_body.rs::a_stray_file_reads_orphaned` asserts exactly the
right thing about `check_index_freshness`. The defect is in what the composed
verb does **after** that function returns, so no assertion on that function
could see it. This is the vacuous-coverage shape this repository has now met
several times: an acceptance that exercises the layer below the one the claim is
about. §3.6 is written to refuse it here.

## 2. Territory

This spec changes how the diagnostics tally handles a file it cannot parse
(`diagnostics.rs`), the two facade halves and two CLI arms that call it
(`lib.rs`, `cmd_index.rs`, `cmd_check.rs`), and adds acceptance at the verbs
(`tests/cli.rs`) beside the library case in 086's suite
(`tests/index_body.rs`). It changes no emitted artifact and moves no schema
version: the one new report member is additive with a `default`, which spec 050
§3.6 settled for exactly this payload and spec 057 followed.

## 3. Behavior

### 3.1 The verdict outranks the tally

The freshness verdict is the answer `index check` and `check` exist to give.
The diagnostics tally adorns it. Once a verdict has been computed, a failure to
build the tally MUST NOT replace it, change its exit code, or suppress its
stale-shard list.

### 3.2 An unparseable file in the shard directory is drift, not a parse error

For the verbs that **judge** the committed ledger (`index check`, `check`, and
the facade functions behind them), a file in either shard directory that cannot
be deserialized into a shard MUST be reported as drift and MUST NOT raise
`Error::Parse`. It is already drift by name whenever the recompute does not
expect it, and after spec 086 a file the recompute does expect is compared byte
for byte, so an expected file that fails to parse differs from its recompute and
is stale by that comparison too. In both cases the remedy is the same
(`spec-spine index`), and exit 2 is the code that names it.

Verbs that **consume** the ledger as input rather than judging it keep exit 3 on
an unparseable shard: `index owner`, `index render`, `index orphans`,
`index coverage`'s classifier, `couple`'s ownership resolution and the
projections. A consumer that cannot read the ledger MUST NOT proceed, and those
verbs already refuse a stale ledger before reading it, so the exit-3 path is
reachable only for a ledger that is fresh and corrupt, which is a genuine
schema failure.

### 3.3 The tally is best-effort and says what it skipped

`committed_counts` MUST count the shards it can parse and skip the ones it
cannot, returning the number skipped alongside the counts. `IndexCheckReport`
MUST carry that number as an additive member with **both**
`#[serde(default)]` and `skip_serializing_if = "is_zero"`, so:

- a pre-095 consumer deserializing a 095 payload is unaffected;
- a 095 consumer deserializing a pre-095 payload reads zero;
- a payload with nothing skipped **omits the member entirely**, which is what
  makes §3.6 case 4's byte-identity promise true. Emitting `0` would add a line
  to every `check --json` payload in every corpus in exchange for a value that
  says nothing happened, and it would make this spec a breaking output change
  for the common case rather than an invisible one;
- `VERDICT_SCHEMA_VERSION` does not move, following spec 050 §3.6.

The omission is the same treatment spec 076 §3.6 gives `plannedTerritory` and
for the same reason: an additive member that is absent when it has nothing to
report costs a corpus with nothing to report exactly nothing.

Skipping MUST NOT be silent: the count is in the payload, and the prose form
MUST name the files it could not read, on the same lines that already list the
stale shards.

### 3.4 One path, four entry points

`check_report`, `check_freshness_json`, `index check`'s CLI arm and `check`'s
CLI arm MUST all reach this behavior through the same function. Spec 057 §3.3
pins the facade and the CLI payloads against each other and `tests/cli.rs`
carries that assertion; this spec keeps that pairing rather than fixing two call
sites and leaving two.

### 3.5 What does not change

- A copy of a **valid** shard still reads `orphaned` at exit 2 everywhere.
- A non-`.json` file in the shard directories is still ignored.
- `compile --check` and the registry half are untouched: the registry's
  comparison never parses a stray, and `compile --check` renders the bare
  freshness object by design (spec 050's note on this payload).
- `--fail-on-unresolved` and `--fail-on-warn` mean what they meant. Neither flag
  gains a condition.

### 3.6 The acceptance runs the verbs

Every case MUST be asserted by invoking `index check`, `check` and
`check --json` on a repository with the stray file present, and MUST assert the
exit code and the named orphan. A library-level assertion on
`check_index_freshness` MUST NOT be the only evidence for any case in this spec,
because that is how the defect shipped (§1.3). 086's library case stays where it
is: it is still true, and it is not sufficient.

The cases:

1. **An unexpected stray that does not parse.** `{"nope": 1}` written to
   `by-spec/999-stray.json`: `index check` and `check` exit 2 and name the
   orphan; `check --json` reports `fresh: false` and the skipped count;
2. **An unexpected stray that does parse.** A valid shard copied to a name the
   recompute does not expect: exit 2, unchanged from today;
3. **An unexpected stray in the other tree.** An unparseable file in
   `by-package/`: same as case 1;
4. **An expected shard corrupted in place.** A shard the recompute *does*
   expect, overwritten with bytes that do not deserialize: `index check` and
   `check` exit 2, and the shard is named as **stale** rather than as an
   orphan, because the recompute expects that path and §3.2's byte comparison
   is what it fails. This case is separate from cases 1 and 3 on purpose: it is
   the one §3.2's second sentence reasons about, and the expected and
   unexpected paths reach the drift verdict through different comparisons, so a
   fix that only handled the unexpected path would leave exit 3 reachable here;
5. **A fresh tree.** Exit 0, nothing skipped, and the `--json` payload
   **byte-identical** to the pre-095 payload, which §3.3's omission rule is what
   secures;
6. **The consumer half is unchanged.** `index owner` on the tree from case 1
   still refuses at exit 3, so §3.2's split is not loosened by accident.

## 4. Out of scope

**Repairing the tree.** These verbs never write. The remedy stays
`spec-spine index`, run by a person or by the one sanctioned hook write.

**The registry half.** A stray file under `by-spec/` in the registry tree is
already reported as `orphaned` by `compile --check` and its comparison does not
parse strays.

**Making `Freshness` carry a structured drift list.** The verdict's shape is
spec 031 §3.3's contract and a consumer reads the shard names from it. This spec
needs no structured list: the tally reports what it skipped, and the drift list
already names the orphan.

**A new exit code.** The taxonomy is fixed at `0/1/2/3` and mapped in one place.

## 5. Resolved decisions

**D-1 (2026-09-14). An unparseable committed shard is staleness, not a parse
error, at the judging verbs.** The alternative was to keep exit 3 and merely
stop discarding the verdict, which would mean a verb that had decided "stale,
here are the orphans" still exiting with a code that says "I could not read the
inputs". The remedy for both is regeneration, and the consumer half of §3.2
keeps exit 3 where a caller must not proceed.

**D-2 (2026-09-14). Best-effort with a count, not "unavailable when stale".**
Reporting the tally as absent whenever the tree is stale was considered and
rejected: the counts over the parseable shards are still what the committed
ledger records, a consumer may use them, and dropping them would lose
information to solve a problem one skipped file caused.

**D-3 (2026-09-15). The skipped count is omitted when it is zero.** §3.3. As
first drafted this spec required only `#[serde(default)]`, which makes the
member *readable* by an old consumer but still writes `"skippedShards": 0` into
every payload, contradicting §3.6's own promise of a byte-identical fresh
payload. The two sections were checked against each other in review and §3.3 was
the one that had to move: the promise is the point, since a corpus that never
has a stray should not be able to tell this spec shipped.

**D-4 (2026-09-15). A corrupted expected shard is its own acceptance case.**
§3.6 case 4. The unexpected and the expected paths reach the drift verdict by
different routes: an unexpected file is `orphaned` by name, while an expected
file that will not deserialize is *stale* by spec 086's byte comparison. The
first draft's cases covered only the unexpected route, so an implementation that
caught `Error::Parse` where strays are enumerated and nowhere else would have
passed every case while leaving exit 3 reachable on the more likely tree: a
shard truncated by a bad merge or a killed write, which is a file the recompute
expects.

**D-5 (2026-09-15). Case 6 asserts the consumer half as it is, not at exit 3
for `index owner`.** §3.6 case 6 says `index owner` on case 1's tree "still
refuses at exit 3". Measured while building, at 0.19.0 and before any change:
`index owner` exits **2** on that tree, because it calls the freshness guard
first and the guard names the orphan, which is exactly what §3.2's own list says
of it ("those verbs already refuse a stale ledger before reading it"). The two
sentences disagree, and the case's stated purpose, that §3.2's split "is not
loosened by accident", is the part the build can honor. Case 6 therefore
asserts both consumer shapes unchanged on that tree: `index owner` refuses at
exit 2 through its guard, and `index render`, which reads the shards with no
guard in front of it, still refuses at exit 3 naming the stray. An
implementation that loosened the shared reader would fail the second assertion.
The alternative rejected was to make `index owner` exit 3, which would move an
exit code §3.2 says does not move.

**D-6 (2026-09-15). The names ride beside the count, unserialized, and one
function carries the tally.** §3.3, §3.4. The payload member is
`skippedShards`, a number, as §3.3 requires; the prose must name the files, so
`DiagnosticCounts` carries them as a `#[serde(skip)]` list that no payload
emits, and `annotate_unreadable` appends `(unreadable: not counted in the
diagnostics tally)` to the drift line naming each one, adding its own line only
for a name the capped list does not show. `committed_counts` keeps its
signature (spec 050's tests call it) and becomes best-effort, erroring only on
an index never built; `verdict_tally` in `lib.rs` is the one function the two
facades and the two CLI arms call after the verdict, which is §3.4's single
path. A shard with a foreign schema MAJOR is skipped by the tally rather than
refused there: spec 086 D-5's refusal still happens in the freshness
comparison, which runs first, so the tally never gets the chance to discard it.

## Verification

Each line is one command. The first `grep` fails against pre-095 code, because
the tally has no skipped-file member; it is the fail-first structural evidence.
The `cargo test` lines are **not** fail-first: the cases in §3.6 do not exist at
the parent commit, so the suites pass vacuously. They carry the behavioral
assertions, which need a repository with a stray file and therefore cannot run
against this tree, whose shard directories are exactly the recompute.

```verify:cli
# 3.3: the report names what the tally could not read.
grep -qE 'unreadable|skipped' crates/spec-spine-core/src/diagnostics.rs
# 3.2, 3.4: the verdict is not thrown away by the tally call.
! grep -qF 'committed_counts(config, repo_root)?' crates/spec-spine-core/src/lib.rs
! grep -qF 'committed_counts(&config, root)?' crates/spec-spine-core/src/lib.rs
# 3.6: the cases, asserted at the verbs and in 086's suite.
cargo test -p spec-spine-cli --test cli --locked
cargo test -p spec-spine-core --test index_body --locked
# 3.5: this tree is fresh and stays fresh, at both verbs.
target/release/spec-spine index check
target/release/spec-spine check
```
