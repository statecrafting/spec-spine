---
id: "158-the-affected-selection-is-a-governed-read"
title: "The affected selection is a governed read"
status: approved
kind: "tooling"
created: "2026-09-27"
summary: >
  Spec 150's selector, which names every spec whose acceptance a change can
  break, lives in an embedded Python block of `scripts/verify-sweep.sh`, a
  script only this repository carries. Spec 157 moves the pre-merge sweep into
  the merge queue across every Statecraft-governed repository, and the
  Statecraft owner decided (2026-09-27) that spec-spine owns selection: the
  profile's sweep job calls a spec-spine read and shards the plan it returns.
  This spec adds that read, `verify --affected-by <base> --plan --json`: the
  CLI turns the Git range into changed paths, the core selects as a pure
  function of the configuration, the corpus and those paths, and the answer
  names each selected spec, the rule that selected it and its effective plan.
  The whole-corpus trigger becomes configuration rather than this repository's
  layout, and `verify-sweep.sh` calls the read instead of keeping a second
  selector.
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "043-verify-declared-acceptance"
  - "150-a-change-runs-the-acceptance-it-can-break"
  - "157-the-affected-sweep-runs-in-the-merge-queue"
amends:
  - "150-a-change-runs-the-acceptance-it-can-break"
extends:
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-cli/src/cmd_verify.rs", nature: additive }
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-core/src/verify.rs", nature: additive }
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "012-declared-extra-frontmatter-passthrough", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "spec-spine.toml", nature: additive }
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/test-verify-sweep.py", nature: additive }
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/schema.rs", nature: additive }
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/tests/conformance.rs", nature: additive }
  - { spec: "047-effective-config-is-a-governed-read", unit: "crates/spec-spine-cli/src/cmd_config.rs", nature: additive }
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-cli/src/cmd_delta.rs", nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: "crates/spec-spine-core/tests/read.rs", nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: "docs/schema-versioning.md", nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-cli/tests/scope.rs", nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-core/tests/scope.rs", nature: additive }
  - { spec: "155-selected-content-accessor", unit: "crates/spec-spine-cli/tests/content.rs", nature: additive }
  - { spec: "155-selected-content-accessor", unit: "crates/spec-spine-core/tests/content.rs", nature: additive }
  - { spec: "067-a-short-id-names-the-same-spec-at-every-verb", unit: "crates/spec-spine-core/src/spec_id.rs", nature: additive }
establishes:
  - { kind: file, path: "crates/spec-spine-core/src/affected.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/affected.rs" }
  - { kind: file, path: "crates/spec-spine-types/schemas/affected.schema.json" }
obligations:
  - { id: "R-1", kind: requirement, text: "verify --affected-by <base> --plan --json names every spec whose acceptance the change can break, the rule that selected it and its effective plan, and runs nothing.", anchor: "3-1-the-read" }
  - { id: "R-2", kind: requirement, text: "Selection is a pure function of the configuration, the corpus at the head and the changed paths; the core runs no git.", anchor: "3-2-selection-is-pure" }
  - { id: "R-3", kind: requirement, text: "The paths that select the whole corpus are configured, not built in.", anchor: "3-3-the-whole-corpus-trigger-is-configured" }
  - { id: "R-4", kind: requirement, text: "verify-sweep.sh --affected-by selects through this read and keeps no selector of its own.", anchor: "3-5-one-selector" }
intent:
  goal: "Give every Statecraft-governed repository the affected-acceptance selection as a spec-spine read, so a merge-queue job can shard and run it."
  non_goals:
    - "Running acceptance: the read plans, `verify <id>` and the sweep execute."
    - "Sharding: the caller partitions the returned plan."
---

# 158: The affected selection is a governed read

## 1. Purpose

Spec 157 makes the affected-acceptance sweep a required merge-queue check and
assigns the job to the Statecraft profile, so every governed repository runs
it. It left one question open (157 D-2): the only selector is spec 150's,
embedded in `scripts/verify-sweep.sh`, which other repositories do not carry.

The Statecraft owner answered on 2026-09-27: spec-spine owns selection.
Statecraft's `accept` evidence does not select. The profile's sweep job
(Statecraft Profile 13) calls a spec-spine read and shards the plan it
returns, and that profile work starts once the read exists.

Selection is a question about the corpus: which specs' declared acceptance a
change can reach. It belongs beside `verify --plan`, which already prints one
spec's effective plan, carried blocks included.

## 2. Territory

- `crates/spec-spine-core/src/affected.rs` (planned): the selector.
- `crates/spec-spine-core/tests/affected.rs` (planned): one case per rule.
- `cmd_verify.rs` and `verify.rs`: `--affected-by`, `--head`.
- `config.rs`: `[acceptance] select_all_on`.
- `version.rs`: the read schema MINOR bump.
- `spec-spine.toml`: this repository's `select_all_on`.
- `scripts/verify-sweep.sh` and its tests: the embedded selector is replaced
  by a call to the read.

## 3. Behavior

### 3.1 The read

`spec-spine verify --affected-by <base> [--head <rev>] --plan --json` MUST
print one read document and execute nothing. `<id>` is not given with
`--affected-by`; giving both, or `--affected-by` without `--plan`, is a usage
refusal (exit 3). `--head` defaults to `HEAD`.

The document MUST carry:

- `base`, `head` and `mergeBase`, each a full commit SHA;
- `changedPaths`: the repository-relative POSIX paths the change touches,
  sorted, deletions and renames (both sides) included;
- `corpusSize` and `selectAll` (true when §3.3 fired, with the paths that
  fired it);
- `selected`: one entry per selected spec in corpus order, with `id`, `rule`
  (`select-all`, `changed-spec`, `names-spec`, `names-path` or `names-test`,
  the first that matched in that order) and `plan`, the command lines
  `verify <id> --plan` prints for that spec at `head`.

An empty selection is a true answer and exits 0. The document is
deterministic: the same repository state gives the same bytes.

Without `--json` the verb prints one line per selected spec, `<id>\t<rule>`,
and the same exit code.

### 3.2 Selection is pure

The CLI resolves `base`, `head` and their merge base and computes the changed
paths from Git, as `couple` does. The core receives the configuration, the
corpus read at `head` and the changed paths, and selects with no Git, clock,
environment or filesystem read of its own.

The rules are 150 §3.1 and its D-3, unchanged except for §3.3: a spec is
selected when its `spec.md` changed, when its effective plan names a changed
spec by full id or by three-digit ordinal as a token, names a changed path as
written from the repository root, or runs `cargo test` over a changed test
file's crate and target.

A plan that cannot be read is a spec the selector cannot rule out, so the read
refuses (exit 1, naming the spec) rather than leave it out (150 D-4). A base
that does not resolve, or shares no history with the head, is a refusal
before anything is selected.

### 3.3 The whole-corpus trigger is configured

150's engine-source rule names this repository's layout (`crates/*/src/**`,
`crates/*/schemas/**`, `Cargo.toml`, `Cargo.lock`, `scripts/verify-sweep.sh`).
That is a fact about what this corpus's plans run, not about spec-spine.

`[acceptance] select_all_on`, a list of globs in `spec-spine.toml`, MUST
replace it: a change touching any path one of them matches selects every spec
with rule `select-all`. The default is empty. This repository MUST set it to
150's list, so its selection is unchanged. `config show` MUST print it.

### 3.4 The read is versioned

The document is a read document on the read schema, whose version takes a
MINOR bump for the new shape. `docs/schema-versioning.md` records it, and the
conformance test validates an emitted document against the embedded schema.

### 3.5 One selector

`verify-sweep.sh --affected-by` MUST obtain its selection from this read and
MUST NOT keep a selector of its own. Its report keeps 150 D-5's shape, with
`selectedBy` taken from the read's `rule` (`select-all` replaces
`engine-source`).

150's six named selector cases in `scripts/test-verify-sweep.py` MUST still
pass, with `test_affected_by_engine_source_selects_every_spec` now meaning the
configured trigger; `crates/spec-spine-core/tests/affected.rs` MUST carry one
case per rule and a case proving the read is narrower than the corpus when no
trigger fires.

## 4. Out of scope

- Sharding, which the caller does over `selected` (157 §3.2 for this
  repository's script; Statecraft Profile 13 for the profile job).
- Selecting by diagnostic code or message text (150 §4).
- Executing anything. `verify <id>` executes; this read never does.

## 5. Resolved decisions

**D-1 (2026-09-27, owner): spec-spine owns selection.** Decided by the
Statecraft owner in answer to 157 D-2. Statecraft Profile 13 calls this read.

**D-2 (2026-09-27, draft): the read lives on `verify`, not `registry`.** The
answer is a set of acceptance plans, which is what `verify --plan` already
prints for one spec, and the name is the one Statecraft asked for.

**D-3 (2026-09-27, draft): the trigger becomes configuration.** A built-in
engine-source rule would select every spec on an adopter's `crates/` change
for a reason that is only true here. Moving it to `spec-spine.toml` keeps this
repository's selection byte-for-byte and gives every other corpus the default,
which selects by reference alone.

**D-4 (2026-10-04, build): the read has an embedded schema of its own.** 3.4
says the conformance test validates an emitted document against the embedded
schema, but the read axis had no schema file: spec 074 and its successors
(155 included) version the shape by constant and pin it in tests. The document
is therefore given `crates/spec-spine-types/schemas/affected.schema.json`,
embedded as `AFFECTED_SCHEMA` and validated in `tests/conformance.rs`; other
read documents are unchanged. The read axis moves to `0.10.0`.

**D-5 (2026-10-04, build): `selectAll` is a boolean and its paths a sibling.**
3.1 says `selectAll` is true when 3.3 fired "with the paths that fired it".
The document carries `selectAll: bool` and `selectAllPaths`, the changed paths
a `select_all_on` glob matched, sorted and empty when it did not fire.

**D-6 (2026-10-04, build): the head's configuration decides the trigger.** The
CLI exports the head tree and reads `spec-spine.toml` and the corpus from it,
so `select_all_on` is the head's. The core takes the `Config` it is given. The
globs match with `*` and `?` stopping at `/` and `**` spanning directories, so
this repository's list reproduces 150's engine-source predicate exactly
(`crates/*/Cargo.toml` is a crate's own manifest, not a nested one).

**D-7 (2026-10-04, build): what refuses.** A base or head that does not resolve,
or that share no merge base, is `Refused` (exit 2: nothing was selected). A
`spec.md` the CLI cannot read at the head makes the core refuse with
`NotFound` naming the spec (exit 1, 150 D-4). A `select_all_on` entry that is
not a glob is a config error (exit 2). `--head` without `--affected-by`, no
`<id>` and no `--affected-by`, `<id>` with `--affected-by`, and `--affected-by`
without `--plan` are usage (exit 3). The text form prints `<id>\t<rule>`.

**D-8 (2026-10-04, build): the sweep reads the document, not stderr.**
`verify-sweep.sh --affected-by` runs the read against its isolated worktree,
keeps the document as `<out>/affected.json`, and takes `selectedBy`, the
selection table and `affectedBy.changedPaths` from it. The `config_version`
does not move for the new optional `[acceptance]` table (precedent: 078's
`[coverage]`).

**D-9 (2026-10-04, build): this repository's trigger lands after a release.**
The governance gate (Statecraft Profile 13, spec 191) runs the spec-spine
release `[meta] required_version` pins, not the candidate, and that release
refuses an `[acceptance]` table it does not know. Setting `select_all_on` in
`spec-spine.toml` in the same change as the code that reads it therefore
cannot pass the gate. The change is split: this one ships the read, the
configuration key and the script; a follow-up, once a release carrying them is
published and pinned, sets this repository's `select_all_on` to 150's list
(recorded as a comment in `spec-spine.toml` until then) and flips
`implementation` to `complete`. Until it lands, an engine-source change selects
only the specs whose plans name it, not the whole corpus.

**D-10 (2026-10-04, build): the names-spec rule takes the ordinal from
`spec_id.rs`.** The rule matches a changed spec's ordinal as a token, so it
needs the id's leading dash-segment. Spec 067 3.4 keeps that expression in
`spec_id.rs` alone, and its single-copy acceptance line refused a private
copy in `affected.rs` in the merge-queue sweep. `spec_id.rs` gains
`leading_segment(id)`, which `match_spec_id` and the selector both call, so the
segment is still taken in one place.

## Verification

```verify:cli
cargo build --release --locked
# 3.1: the read exists and plans without running.
target/release/spec-spine verify --help 2>&1 | grep -q -- '--affected-by'
# 3.1: over an empty range it is an empty, valid selection.
target/release/spec-spine verify --affected-by HEAD --plan --json | python3 -c "import json,sys;d=json.load(sys.stdin);assert d['selected']==[] and d['changedPaths']==[], d"
# 3.1: an id and --affected-by together are a usage refusal.
target/release/spec-spine verify 158 --affected-by HEAD --plan >/dev/null 2>&1; test $? -eq 3
# 3.3: this repository keeps 150's trigger through configuration.
target/release/spec-spine config show | grep -qF 'select_all_on'
# 3.2, 3.5: one case per rule, and the script's cases still pass.
cargo test -p spec-spine-core --test affected --locked
python3 scripts/test-verify-sweep.py
# 3.5: the script keeps no selector of its own.
! grep -qF 'SELECTOR' scripts/verify-sweep.sh
grep -qF 'verify --affected-by' scripts/verify-sweep.sh
```
