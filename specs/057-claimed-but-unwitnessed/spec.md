---
id: "057-claimed-but-unwitnessed"
title: "A claim no hash witnesses"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
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
additive field. `VERDICT_SCHEMA_VERSION` does not move: spec 050 §3.6 settled
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

## 4. Out of scope

**Deciding this repository's twenty-four.** The lint reports them; which get a
glob, which get a narrower unit, and which are deliberately unwitnessed is a
sequence of judgements about specific files, and none of them is a mechanical
consequence of this spec. `lint --fail-on-warn` runs in CI here, so the
implementing change must resolve the corpus to green, and the resolution it
picks is a decision recorded in this spec at that time, not predetermined now.

**Folding claimed files into the hash.** §3.4.

**Extending the seal.** Spec 023's attestation covers what the ledger hashes.
Making it cover more is a change to the seal's input set and belongs with the
folding decision, not ahead of it.

**Unresolved units.** `W-001` and `W-002` (specs 025, 050) diagnose a claim that
resolves to nothing. `L-008` diagnoses a claim that resolves to something no
hash covers. Different conditions, and §3.2 keeps them from firing on the same
path.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# `index check` reports the count. Until this ships, the line is absent.
target/release/spec-spine index check | grep -q 'unwitnessed claims'
# A synthetic corpus claiming an existing file that no hash covers produces
# L-008. This is the condition 3.2 defines, isolated from this repository's
# own remedies.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs/001-x" && : > "$tmp/spec-spine.toml" \
  && printf 'claimed\n' > "$tmp/thing.sh" \
  && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "thing.sh"\n---\n\n# x\n' > "$tmp/specs/001-x/spec.md" \
  && target/release/spec-spine --repo "$tmp" lint | grep -q 'L-008'
# This corpus is green at the tier CI gates on, which means the twenty-four
# unwitnessed claims named in 1 were each resolved or deliberately covered by
# the implementing change.
target/release/spec-spine lint --fail-on-warn
```
