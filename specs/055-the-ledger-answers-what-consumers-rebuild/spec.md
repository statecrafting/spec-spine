---
id: "055-the-ledger-answers-what-consumers-rebuild"
title: "The ledger answers what consumers rebuild by hand"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "002-registry-query"
  - "004-codebase-index"
  - "024-index-sharding"
  - "032-ownership-coverage"
extends:
  # `index owner <path>`: the resolution the gate does privately, as a verb.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # `registry show --json` gains the shard's content hash as an output field.
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  # Six new names join the crate root's re-export list, and two functions the
  # gate kept private become public there.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # The shard-hash read's acceptance sits beside the other `query` tests.
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Two facts the ledger computes and then keeps to itself, each of which an
  adopter has reimplemented. Ownership of a path is resolved inside
  `couple.rs::owners_for_path`, a private function, so a consumer that wants to
  know which spec governs a file rebuilds a path-to-spec map from raw edges and
  gets the subtree, supersession and comment-header rules wrong. A spec's
  content hash is written into every registry shard as `shardHash` and is
  absent from every read verb, so claude-observatory reimplemented `hash.rs`
  normalization in TypeScript in order to pin against it. This spec adds
  `spec-spine index owner <path>` and puts `contentHash` on `registry show
  --json`'s output object. Neither touches a committed artifact, a DTO, or a
  schema version: both facts already exist and are merely unreachable.
---

# 055: The ledger answers what consumers rebuild by hand

## 1. Purpose

The authority ledger's whole promise is that "who owns this" and "has this
changed" are answerable mechanically. It keeps that promise to itself and not
to its consumers. Two questions, both already computed, neither reachable.

**Which spec owns this path?** The coupling gate answers it on every run.
`couple.rs::owners_for_path` walks the index's resolved units and implementing
paths, applies `claim_matches` (an exact match or a directory-prefix match for a
subtree unit), transfers authority transitively across `supersedes`, filters by
hunk overlap for span-bearing units, and adds amends-awareness for a `spec.md`
path. That is five separate rules, four of which a naive consumer will not
guess, and the function is private. So consumers rebuild it: the audit found
them reconstructing path-to-spec maps from raw edges, which gets bare-string
file units right and everything else wrong.

**What is this spec's content hash?** Every registry shard carries `shardHash`,
SHA-256 over that spec's `spec.md` under the corpus's normalization rules (BOM
stripped, CRLF and CR to LF, hashed as `<path>\0<bytes>`). No read verb reports
it. `registry show --json` returns the record and not the shard. So
claude-observatory, which wants to pin a spec's text so it can detect that the
contract moved under it, reimplemented `hash.rs`'s normalization in TypeScript.
That is a second implementation of the determinism claim, maintained by someone
who does not own it, which will be wrong the first time the normalization gains
a case.

Both are the same failure. A fact is computed, used internally, and discarded at
the boundary, so the consumer computes it again, worse. These are items 6 and 7
of the adopter audit's ranked backlog for the tool, and they belong in one
change because they are one mistake.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/couple.rs` | 005 | `owners_for_path` becomes reachable |
| `crates/spec-spine-core/src/index.rs` | 004 | the public owner query |
| `crates/spec-spine-cli/src/cmd_index.rs` | 004 | the `owner` action |
| `crates/spec-spine-core/tests/index.rs` | 004 | its acceptance |
| `crates/spec-spine-core/src/query.rs` | 002 | `show` can carry the shard hash |
| `crates/spec-spine-cli/src/cmd_registry.rs` | 002 | reporting it |

Specs 002, 004 and 005 each keep everything they require. Nothing here changes a
resolution rule, a hash input, or a gate decision: two existing computations
become readable.

## 3. Behavior

### 3.1 `spec-spine index owner <path>`

A new read action beside `orphans`, `diagnostics` and `coverage`, taking
`--json`.

It reports, for one repo-relative path, the specs that own it and how:

```
crates/spec-spine-core/src/couple.rs
  005-coupling-gate      unit        crates/spec-spine-core/src/couple.rs
  052-couple-names-the-crossing  extends     crates/spec-spine-core/src/couple.rs
  001-compile-registry   floor       crates/spec-spine-cli (package manifest)
```

Three linkage kinds, because a consumer's next decision depends on which one it
is. A **unit** owner claimed the path in an ownership-bearing edge. A **floor**
owner owns it only through a package's `[package.metadata.<ns>].spec`, which
counts for `C-001` and counts as debt for coverage (spec 032). A **header**
owner is a `// Spec:` comment in the file itself. Collapsing these into a flat
list of spec ids would reproduce exactly the ambiguity spec 032 was written to
remove.

It MUST live under `index`, not under `registry`, and the reason is not
cosmetic. The registry is the spec-as-source view: it holds units as authored,
including symbol ids and section anchors that name no path at all. Resolving a
unit to a set of files is the indexer's job and its output is the index. A
`registry owner <path>` verb would either have to resolve (making the registry
query depend on the index, which inverts the two views) or answer only for bare
file units, which is the wrong answer stated confidently. The audit offered
`registry owner`; the corpus's own layering says `index owner`.

**Freshness.** Like `coverage`, it MUST refuse a stale committed index with
`Error::Stale` (exit 2) rather than answer from a ledger that no longer
describes the tree. An owner answer read off a stale index is the one kind of
wrong answer this verb must never give, because its caller is deciding what to
edit.

**No owner is not an error.** A path nothing owns exits `0` with an empty result
(`[]` under `--json`, a single explanatory line otherwise). It is a true and
common answer, especially on a specify-first corpus, and `NotFound` is reserved
for a thing that was asked for by name and does not exist. The path need not
exist on disk: asking who *would* own a file before creating it is a legitimate
question and the honest answer is computed the same way.

### 3.2 The owner query is the gate's own function

`owners_for_path`'s rules MUST NOT be reimplemented. The public query MUST call
the same code the gate calls, so an answer from `index owner` and a `C-001`
decision cannot disagree.

That is the entire value of the verb. A second implementation that agreed today
and drifted next quarter would be worse than no verb, because a consumer would
have stopped rebuilding the map by hand and started trusting an answer nobody
tests against the gate.

Two of the gate's inputs are absent outside a diff and MUST be handled
explicitly rather than faked:

- **Hunk overlap.** The gate filters span-bearing owners by whether the change
  touched their lines. With no diff there are no hunks, and the query MUST use
  the whole-file interpretation the gate already defines for empty hunks
  (`--paths-from` mode uses it today). Every owner whose span is in the file is
  reported, which is the right answer to "who owns this file".
- **Amends-awareness.** The gate widens the owner set for a `<specs_dir>/<id>/spec.md`
  path, and only when the base set is non-empty (the strict-expansion guard).
  The query MUST apply the same rule under the same guard, so asking about a
  `spec.md` path returns what the gate would use.

Supersession transfer applies unchanged: a successor inherits its predecessor's
authority, additively, and both are reported.

**Decision, 2026-09-07: a fourth linkage kind, `inherited`.** §3.1 names three,
which are the three ways a spec makes a claim. Supersession transfer and the
amends widening make a spec an owner *without* a claim, and this section
requires both to be reported. Reporting them as `unit` would attribute a claim
that was never written, and dropping them would make the reported id set
disagree with the gate's, which is the one thing §3.2 forbids. So they carry
`inherited`, and the claim column names the relation that conferred the
authority (`supersedes 019-…`, `amends 026-…`) rather than a path. The three
claim kinds keep their meanings exactly.

### 3.3 `contentHash` on `registry show --json`

`registry show --json` MUST gain a `contentHash` field carrying the value the
spec's committed registry shard records as `shardHash`. The prose form MUST
print it as one additional line.

This is additive at the output boundary and nowhere else:

- **`SpecRecord` does not change.** The field is added to what `show` prints,
  not to the record type. Adding it to `SpecRecord` would put it in every
  committed shard, whose schema is `additionalProperties: false`, which means a
  schema-file edit and a `REGISTRY_SCHEMA_VERSION` bump for a value the shard
  already carries one line above the record.
- **No committed artifact changes.** Every shard is byte-identical and
  `compile --check` stays fresh.
- **No schema version moves.** Not `REGISTRY_SCHEMA_VERSION`, which versions the
  artifact, and not `VERDICT_SCHEMA_VERSION`, which versions gate envelopes that
  this verb does not emit.

The hash is read from the committed shard, never recomputed. `registry` is the
read-side view of what was committed, and a `show` that recomputed from
`spec.md` would report a hash the ledger does not hold, silently repairing a
staleness that `compile --check` exists to reveal.

### 3.4 What the hash means, said once

`contentHash` is SHA-256 over that spec's `spec.md` alone: the registry's only
hashed input, unchanged since before sharding. It is deliberately **not** the
index's per-spec shard hash, which additionally folds span-backing source files
and the global-inputs scalar.

A consumer pinning "has this spec's text changed" wants the registry value. A
consumer asking "has anything this spec depends on changed" wants the index
value and a different question. The prose form MUST say which one it is
reporting in the same breath as reporting it, because the two are the same shape
and a consumer that confuses them gets a pin that fires on unrelated edits.

## 4. Out of scope

**`index coverage --by-path --json`.** The audit's alternative shape. Coverage
answers a whole-tree question with a classification per file; `owner` answers a
single path with its linkage detail. Bolting a per-path projection onto coverage
would give the second answer in the first answer's vocabulary, which is
`Specific` / `FloorOnly` / `Unowned` and cannot name a spec.

**A reverse query (paths owned by a spec).** `index render` already projects it
per spec, and a dedicated verb is a different design with a different output
shape. Not needed by any evidence the audit collected.

**Changing any ownership rule.** Subtree matching, supersession transfer,
comment headers, the floor, the strict-expansion guard: all exactly as specs
004, 005, 009, 019 and 032 left them. This spec makes a decision readable; it
does not revisit it.

**The index's per-spec shard hash as a read verb.** A real gap and a different
one, tangled with what is and is not a hashed input, which spec 057 is about.

## 5. Verification

`index owner` and `contentHash` both fail against pre-055 code: `owner` is an
unknown action clap refuses, and `show`'s object has no such field.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test index --locked
cargo test -p spec-spine-core --test query --locked
# 3.1: the verb reports the owners, and separates how each one owns.
target/release/spec-spine index owner crates/spec-spine-core/src/couple.rs --json | python3 -c 'import json,sys; o=json.load(sys.stdin)["owners"]; k={x["specId"]:x["kind"] for x in o}; assert k["005-coupling-gate"]=="unit", k; assert k["001-compile-registry"]=="floor", k'
# 3.2: the id set is the gate's own. `couple` refuses this path naming exactly
# the specs `owner` reports, so the two cannot disagree.
test "$(target/release/spec-spine index owner crates/spec-spine-core/src/couple.rs --json | python3 -c 'import json,sys; print(",".join(sorted({o["specId"] for o in json.load(sys.stdin)["owners"]})))')" = "$(printf 'crates/spec-spine-core/src/couple.rs\n' | target/release/spec-spine couple --paths-from /dev/stdin --json | python3 -c 'import json,sys; print(",".join(sorted(json.load(sys.stdin)["report"]["violations"][0]["owners"])))')"
# 3.1: a path nothing owns is an empty answer, not an error, and the path need
# not exist on disk.
target/release/spec-spine index owner no/such/path.rs --json | python3 -c 'import json,sys; assert json.load(sys.stdin)["owners"]==[]'
# 3.3: `show` reports the hash the committed shard holds.
target/release/spec-spine registry show 024-index-sharding --json | python3 -c 'import json,sys; h=json.load(sys.stdin)["contentHash"]; assert len(h)==64, h'
# 3.3: that it is the shard's value and not a recomputation is asserted by
# `shard_content_hash_is_read_from_the_committed_shard` in the query tests
# above, which edits a spec.md without recompiling and watches the reported
# hash stay put. It is asserted there rather than here because reading the
# shard to compare against it is exactly the ad-hoc parse
# `.claude/rules/governed-artifact-reads.md` forbids.
# 3.3: nothing committed moves.
target/release/spec-spine compile --check
```
