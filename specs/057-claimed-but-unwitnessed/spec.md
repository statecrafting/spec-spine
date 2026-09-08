---
id: "057-claimed-but-unwitnessed"
title: "A claim no hash witnesses"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "003-conformance-lint"
  - "004-codebase-index"
  - "023-ledger-seal"
  - "032-ownership-coverage"
extends:
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # The `index check` count line (3.3) and the payload field carrying it.
  - { spec: "050-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-core/src/diagnostics.rs", nature: additive }
  - { spec: "050-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  # The allowlist knob (3.5), beside spec 053's in the same table.
  - { spec: "053-depends-on-ordinal-monotonicity", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/tests/lint.rs", nature: additive }
  # The facade half of the payload, and the two acceptance tests that pin the
  # CLI and the facade against each other (3.3).
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
summary: >
  A `file` unit carries no span, and only span-backing source files are folded
  into a per-spec index shard hash. So a spec can claim a file, `index check`
  can report fresh, `index coverage` can report it claimed, and the file's
  contents can be rewritten from end to end with no gate noticing. This is not
  hypothetical in this repository: `AGENTS.md` is claimed by three specs, and
  appending a line to it leaves both `index check` and `compile --check`
  reporting fresh. Twenty-four claimed paths here are in that state, and spec
  023 signs an attestation over a ledger that never hashed any of them. This
  spec adds `L-008`, a warning naming each claimed path that contributes to no
  content hash, and one line of `index check` output reporting the count. It
  does not fold claimed files into the hash: that would change what staleness
  means, and the honest first move is to say how large the gap is.
---

# 057: A claim no hash witnesses

## 1. Purpose

The determinism claim is the centre of this system, and the sentence people read
it as is stronger than the sentence it makes.

What is actually hashed, per spec shard, is three things: that spec's `spec.md`,
the source files backing its resolved `section` / `symbol` / `module` spans, and
a global scalar over `spec-spine.toml` plus `index.extra_hashed_inputs`. A
per-package shard hashes its manifest's governance projection and the same
scalar. Nothing else.

`file`, `directory` and `crate` units carry no span. `span_files_for_mapping`
says so in as many words, and it is correct to: those units have no line range
to shift. The consequence is what nobody wrote down. **A file claimed by a bare
`file` unit contributes nothing to any hash**, unless it happens to be a
manifest or to fall inside an `extra_hashed_inputs` glob.

This is measurable here, not inferred. `AGENTS.md` is claimed by specs 029, 047
and 051. Appending a line to it and re-running both freshness gates:

```
$ printf '\n<!-- probe -->\n' >> AGENTS.md
$ spec-spine index check
index is fresh
$ spec-spine compile --check
spec-registry is fresh: 53 shard(s) match the corpus
```

Twenty-eight paths outside `crates/` and `npm/` are claimed in this corpus.
Four of them fall under `standards/**` or `.github/workflows/**` and are
therefore hashed. The other twenty-four are not: every `.claude/` rule, agent
and skill, both `.githooks/` scripts, `install.sh`, `docs/releasing.md`,
`scripts/verify-spec.sh`, `py/scripts/generate_wheels.py`, and the entire `kit/`
tree that specs 029, 046, 048 and 051 exist to govern.

Three separate things read as safe and are not:

- **The Stop and PostToolUse hooks** run `index check` after an edit to
  `AGENTS.md`, `CLAUDE.md`, `.claude/rules/*.md` and `.claude/skills/*/*.md`.
  Every one of those paths is unhashed, so the check is a formality on exactly
  the paths the hook lists.
- **`index coverage`** reports a claimed file as claimed, which is true and is
  about ownership, not about witnessing. A reader takes the two together and
  concludes the file is governed and pinned. Only the first half holds.
- **Spec 023's attestation** signs a reproducible corpus seal built from the
  ledger. For these twenty-four paths, the seal attests to a claim and to
  nothing about the content claimed.

hqgit found the same thing from the other end: its spec 001 claims four scripts
no glob covers, and they can be rewritten with `index check` still reporting
fresh. That is item 9 of the adopter audit's ranked backlog.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/lint.rs` | 003 | the `L-008` check |
| `crates/spec-spine-core/src/index.rs` | 004 | the witnessed-input predicate |

The implementing change creates `crates/spec-spine-core/tests/lint.rs` if spec
053 has not already, and claims it in whichever spec creates it.

No hash input changes. No committed artifact changes. `INDEX_SCHEMA_VERSION`
does not move.

## 3. Behavior

### 3.1 One predicate, in the indexer

`index.rs` MUST expose a predicate answering, for a repo-relative path, whether
that path contributes to any content hash the index computes. It is true when
the path is a spec's `spec.md`, a discovered package's manifest,
`spec-spine.toml`, matched by an `index.extra_hashed_inputs` glob, or a
span-backing file of some resolved unit.

It MUST be derived from the same code that builds the hashes, not restated
beside it. A second list of what is hashed would be wrong the first time the
first list changes, and this whole spec exists because a reader's model of the
hash inputs was wrong.

A path under `layout.state_dir` is never witnessed, by construction (spec 039
excludes state from every content hash). It is also never claimable: `L-006`
already refuses a unit claimed inside the state root at error tier. So the two
checks cannot both fire on one path, and `L-008` MUST defer: if `L-006` fires,
`L-008` MUST NOT, because "you claimed the ungoverned directory" is the whole
diagnosis and adding "and it is not hashed" is noise on a path that must move.

### 3.2 `L-008`

`lint` MUST emit one `L-008` diagnostic, at **warning** severity, for each
ownership-bearing claimed path that the predicate reports as unwitnessed:

```
L-008  spec '051-harness-runs-the-verbs-it-ships' claims 'AGENTS.md', which is
       in no content hash: its contents can change without staling any shard.
       Add a covering glob to [index] extra_hashed_inputs, or claim a section
       or symbol unit, whose span is hashed.
```

**Warning, not error.** An unwitnessed claim is a real gap and a legitimate
state. A spec may deliberately claim a file whose content it does not want
staling the ledger, and on a specify-first corpus a spec routinely claims files
that do not exist yet, which are unwitnessed for a reason that will resolve
itself. Error tier would refuse a corpus for a condition many corpora hold on
purpose. Warning tier means `lint --fail-on-warn` refuses it, which is what this
repository runs in CI, and which is the decision this corpus should make
deliberately rather than inherit.

The message MUST name both remedies, because they are genuinely different
choices and the right one depends on the file. A glob in `extra_hashed_inputs`
folds the file into the **global** scalar, which restamps every shard when it
changes: correct for a small set of governance files, wrong for a large tree of
source. A `section` or `symbol` unit is hashed through its span and stales only
the claiming spec's shard: correct for source, and unavailable for a file with
no parseable sections.

A path that does not exist on disk MUST NOT produce `L-008`. It is already
diagnosed: an unresolved unit is `W-001` or `W-002` (spec 025), and telling a
specify-first corpus that a file it has not written yet is also not hashed would
put a second warning on every pending claim in the corpus. `L-008` is about
files that exist and are claimed and are invisible to the ledger.

### 3.3 `index check` reports the count

`index check` MUST report the number of claimed-but-unwitnessed paths on one
line, beside the unresolved-unit counts spec 050 added:

```
index is fresh
  unresolved: 0 (W-001: 0, W-002: 0)
  unwitnessed claims: 24
```

Reporting only. `index check` MUST NOT change its exit code for this, with or
without `--fail-on-unresolved`. The gate half is the lint's, at warning tier,
and putting a second refusal on `check` would make one flag mean two conditions.

The line exists because `index check` is where a person reads the word "fresh",
and "fresh" is the word this spec is qualifying. A count there is the smallest
honest correction to what the reader is being told.

Under `--json`, the count joins the existing `IndexCheckReport` payload as an
additive field, on **both** sides: the CLI and the `check_freshness_json`
facade emit one shape, and `cli.rs` pins them against each other. That test
caught the first cut of this change, where the field existed on the CLI side
only, which is exactly the drift it was written to refuse. `VERDICT_SCHEMA_VERSION` does not move: spec 050 §3.6 settled
that adding a member to one verb's report payload is additive and must not move
the constant that versions the envelope, and this spec follows that decision
rather than reopening it.

### 3.4 Nothing is folded into a hash

The alternative the audit offered, folding claimed `file` units into the shard
hash, is **not** taken, and the reason is worth recording rather than leaving to
inference.

It would change what staleness means. Today a shard is stale when the spec's
text, its spans, or the global inputs changed: an edit to a claimed source file
does not stale the index, because drift between code and spec is the coupling
gate's question, asked at PR time against a diff. Folding claimed files in would
make every source edit stale the index, so every code change would require
regenerating and committing shards, and `index check` would start refusing for a
condition `couple` already refuses better.

It would also cost the property spec 024 was written for. A claimed file shared
between two specs would stale both shards, so two PRs touching different specs
would write the same files again, which is the conflict sharding removed.

Naming the gap is the change that is clearly right. Closing it by folding is a
change to the staleness contract that deserves its own spec, its own argument,
and a corpus that already knows how big the gap is. This spec produces that
number.

### 3.5 A corpus may declare a gap deliberate

`[lint] unwitnessed_allowed` MUST accept glob patterns naming claimed paths a
corpus has decided to leave out of every content hash, suppressing `L-008` for
them.

It suppresses the **warning**, never the **count**: `index check` reports the
total and how many of it the list covers, so a declared exception is explicit
rather than invisible. That is the difference between writing a decision down
and turning the check off, and it is why this is not simply an opt-in knob on
the lint.

**Decision, 2026-09-07.** The knob is added because §4's instruction ("resolve
the corpus to green") turned out to require it. §1 estimated twenty-four
unwitnessed paths; the predicate found **ninety-four**, of which seventy-four
are Rust sources under `crates/` claimed as bare `file` units. That is not an
oversight in those specs: a `file` unit carries no span by design, and folding
claimed files into the hash is what §4 puts out of scope. Without a way to
declare the remainder deliberate, the only routes to green were to weaken
`--fail-on-warn` for every `L-` code or to make `L-008` itself opt-in, and both
turn the check off rather than record a decision.

## 4. Out of scope

**Deciding this repository's ninety-four.** Decided, 2026-09-07, and recorded in
`spec-spine.toml` rather than here, because the decision is a configuration:

- **Twenty covered by a hashed-input glob.** The governance and harness files
  specs claim (`AGENTS.md`, `CLAUDE.md`, `.claude/rules/`, `.githooks/`,
  `scripts/`, `install.sh`, three `docs/` files, the `kit/` copies) plus the
  five embedded JSON Schemas, which are hashed rather than allowlisted because a
  schema edit is exactly what the ledger should notice.
- **Sixty-nine declared deliberate**, as `crates/**/*.rs`. They are not
  undefended: the coupling gate refuses a changed source file whose owning spec
  did not change, which is the check that actually protects them. What the gap
  costs is narrower and is now written down: `index check` will not call the
  index stale for an edit to one, and spec 023's attestation covers a ledger
  that never hashed their bytes.

**Decision, 2026-09-07: two glob entries in this repository matched nothing.**
`extra_hashed_inputs` was `["standards/**", ".github/workflows/**"]`, and in the
`glob` crate `dir/**` matches directories only; files need `dir/**/*`. So the
constitution, the contract, the spec templates and all five CI workflows had
never contributed to any content hash, and spec 023 has been signing an
attestation over a ledger that had not read them. Fixed here, and pinned by a
test that asserts both the trap and the working form, because a silent
zero-match is the same class of defect as a silent unwitnessed claim.

**Decision, 2026-09-07: the patterns are narrow on purpose.** `extra_hashed_inputs`
hashes whatever is on disk, tracked or not. A bare `.claude/**/*` folds in
`.DS_Store`, `agent-memory/` and `settings.local.json`, which would make the
shard hashes machine-dependent and surface as a platform difference in the
four-triple determinism gate. Narrow patterns also keep the blast radius
honest: an entry here restales all sixty-nine shards when it changes, so it
earns its place by being a governance file some spec claims. A verb that
reported what a configured glob currently matches would have caught the
zero-match entries years earlier; spec 054 §4 already defers that idea, and this
is a second reason to want it.

**Folding claimed files into the hash.** §3.4.

**Extending the seal.** Spec 023's attestation covers what the ledger hashes.
Making it cover more is a change to the seal's input set and belongs with the
folding decision, not ahead of it.

**Unresolved units.** `W-001` and `W-002` (specs 025, 050) diagnose a claim that
resolves to nothing. `L-008` diagnoses a claim that resolves to something no
hash covers. Different conditions, and §3.2 keeps them from firing on the same
path.

## 5. Verification

`L-008` and the `index check` line both fail against pre-057 code: neither the
code nor the line existed.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.3: `index check` reports the count, and says how much of it is declared.
target/release/spec-spine index check | grep -q 'unwitnessed claims'
# 3.3: reporting only. The exit code is unchanged, with and without the flag.
target/release/spec-spine index check --fail-on-unresolved
# 3.2: a synthetic corpus claiming an existing file no hash covers produces
# L-008, isolated from this repository's own remedies.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs/001-x" && : > "$tmp/spec-spine.toml" && printf 'claimed\n' > "$tmp/thing.sh" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "thing.sh"\n---\n\n# x\n## body\n' > "$tmp/specs/001-x/spec.md" && target/release/spec-spine --repo "$tmp" index >/dev/null && target/release/spec-spine --repo "$tmp" lint | grep -q 'L-008'
# 4: the two entries that matched nothing now match. `standards/` and the
# workflows contribute to the ledger, which is what this spec found they never
# had. Asserted through `config show` (spec 054), a governed read.
target/release/spec-spine config show --json | python3 -c 'import json,sys; g=json.load(sys.stdin)["index"]["extra_hashed_inputs"]; assert "standards/**/*.md" in g, g; assert "standards/**" not in g, g'
# 4: this corpus is green at the tier CI gates on, which means every one of the
# ninety-four was either covered or declared.
target/release/spec-spine lint --fail-on-warn
target/release/spec-spine compile --check
```
