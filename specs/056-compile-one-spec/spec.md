---
id: "056-compile-one-spec"
title: "Validate one draft without a temporary repository"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "001-compile-registry"
  - "031-registry-freshness-check"
  - "037-machine-readable-verdicts"
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/tests/compile.rs", nature: additive }
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-types/src/verdict.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Writing a spec means validating it before it lands, and the tool offers no
  way to do that. `compile` writes the whole shard tree, which an author with an
  unfinished draft must not do; `compile --check` refuses the whole corpus
  because the draft has no committed shard, reporting the author's work in
  progress as staleness. So two adopters wrote down a `mktemp -d` ritual:
  copy the corpus, copy the draft in, compile there, read the violations, delete
  the copy. It has a documented gotcha, because a `references` edge into `docs/`
  does not resolve in a corpus copy that has no `docs/`. This spec adds
  `compile --spec <id>`: validate one spec against the committed registry,
  report only that spec's violations, and write nothing.
---

# 056: Validate one draft without a temporary repository

## 1. Purpose

The corpus is authored one spec at a time, and every spec is wrong at least
once before it is right: a `depends_on` naming an id that does not exist, a
`domain` outside the allowlist, a malformed edge, a directory that does not
equal the id. Those are `V-` codes, and `compile` reports every one of them.

An author with an unfinished draft cannot run it.

`compile` **writes**. It emits the shard tree, so running it to see whether a
draft parses commits that draft to the ledger as a side effect of asking a
question. In a repository whose shards are committed, that dirties the tree with
an artifact for a spec that may not survive the next edit. It is the same
mistake spec 031 was written about and spec 046 found in three of the kit's
hooks: writing while reading hides what the committed copy said.

`compile --check` **refuses**. It compares the corpus to the committed shards,
and a new draft has no committed shard, so the whole run is stale. The output is
correct and useless: it reports the author's own unfinished work as drift, mixed
in with nothing else, and gives no line about whether the draft is well-formed.

What is left is the ritual two adopters wrote down: `mktemp -d`, copy
`spec-spine.toml` and the corpus, copy the draft in, `compile` there, read the
violations, delete the directory. Both documented the same gotcha, and it is
instructive: a `references` edge pointing into `docs/` does not resolve in a
copy that has no `docs/`, so the ritual produces a violation the real corpus
would not, and the adopter has to know to ignore it. A workaround that teaches
you which of its own outputs to disbelieve is a missing feature with extra
steps.

This is item 8 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/compile.rs` | 001 | single-spec validation |
| `crates/spec-spine-cli/src/cmd_compile.rs` | 001 | the `--spec` flag |
| `crates/spec-spine-core/tests/compile.rs` | 001 | its acceptance |
| `crates/spec-spine-types/src/verdict.rs` | 037 | the verdict's report payload |

Spec 001 owns `compile` and defines the `V-` codes and the emission contract. It
requires that compiling the corpus writes the registry; it says nothing about a
mode that validates without writing, which spec 031 already established is a
legitimate second mode of the same verb. This spec adds a third.

## 3. Behavior

### 3.1 `spec-spine compile --spec <id>`

`compile` MUST accept `--spec <id>`, which validates exactly one spec and writes
nothing.

`<id>` accepts the short form (spec 016): `056` resolves to `056-compile-one-spec`
when exactly one directory carries that ordinal. An id matching none or several
is `Error::NotFound`, exit `1`, as everywhere else the short form is accepted.

The spec need not have a committed shard. That is the entire point: the verb
exists for a draft that has never compiled.

Exit codes follow the existing contract exactly. `0` when the spec produces no
error-tier violation, `1` when it does, `3` for I/O, parse, schema or config
failure. It MUST NOT exit `2`: nothing here is a staleness question, and
reporting a draft as stale is precisely the wrong answer `compile --check` gives
today.

`--spec` is incompatible with `--check` and MUST be refused as a usage error if
both are given. They are different questions (is this well-formed / do the
committed shards match) and a combined form would have to invent an answer for a
spec with no shard.

### 3.2 It reports one spec's violations, and says so

The report MUST contain the violations attributable to the named spec and no
others.

Spec 001's structure already draws this line and this verb reuses it rather than
inventing a filter. `CROSS_SPEC_CODES` (`V-003`, `V-004`, `V-008`, `V-010`,
`V-014`) are the codes whose verdict depends on the whole corpus, which is why
they are recomputed on read instead of stored per shard. Everything else is
local to one spec and is exactly what `--spec` reports.

The cross-spec codes are **not** silently dropped. They are evaluated with the
named spec assembled against the committed registry, and reported, because they
are the codes an author is most likely to trip: a `depends_on` naming a spec
that does not exist (`V-010`) and a duplicate ordinal (`V-004`) are the two
mistakes a new draft actually makes. Assembling against the committed registry
rather than against a recompile of the corpus is what makes this fast and what
keeps it honest: the author is asking whether their draft fits the corpus as it
stands, and the corpus as it stands is what was committed.

When a cross-spec code fires, the message MUST name the other spec involved, as
it does today. An author told "duplicate numeric prefix '056'" without being
told which spec already holds it has been given a puzzle.

### 3.3 It resolves against the real repository

Validation MUST read the real `spec-spine.toml` and the real corpus root. No
copy, no temporary directory, no synthesized configuration.

This is what kills the `references`-into-`docs/` gotcha rather than documenting
it. The draft is validated where it lives, so every path in it resolves the way
it will resolve after it lands, and a violation reported here is a violation
that would be reported there.

### 3.4 The verdict envelope

`--spec` MUST accept `--json` and emit the spec 037 verdict envelope, with a
`verb` token distinguishing it from `compile --check`.

`compile --check` already carries one, and the flag's contract is that it
changes what is written and never what is decided: every exit code is identical
with and without it.

`VERDICT_SCHEMA_VERSION` moves from `0.2.0` to `0.3.0`: a MINOR bump, because
adding a `verb` token is the additive case that constant's documentation names.
Spec 049 set the precedent when `verify` was added, and the reasoning has not
changed: a consumer's existing branches still match, and a consumer that
switches exhaustively on `verb` learns about the new one from the version.

No registry shard changes and `REGISTRY_SCHEMA_VERSION` does not move. This verb
writes nothing.

### 3.5 Writing is still `compile`

`--spec` MUST NOT write a shard, not even the named spec's.

A per-spec write would be a genuinely useful verb and a different one, and
offering it here would put an author one flag away from committing a shard for a
draft they were only checking. The gate chain stays `compile` (write all) then
`index` then `lint` then `couple`, and `--spec` sits outside it as a read, next
to `--check`.

## 4. Out of scope

**Linting one spec.** `lint --spec <id>` is the obvious sibling and is not in
this change. The `L-` codes are conventions, most are corpus-wide by nature, and
the audit's evidence is about `V-` codes on an unlanded draft. A single-spec
lint can be added later without redoing anything here.

**Validating a file outside the corpus.** `compile --spec` names a spec by id,
which means a directory under `specs_dir`. Validating a `spec.md` sitting
somewhere else would need a second resolution story and answers no question an
adopter has asked.

**Making `compile --check` tolerant of new drafts.** Its job is to report whether
the committed shards match the corpus, and an uncommitted draft genuinely means
they do not. Teaching it to ignore that would break the freshness contract spec
031 exists to hold, in order to work around the absence of the verb this spec
adds.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test compile --locked
# The flag exists, resolves the short id, and validates without writing.
# Until this ships, `--spec` is an unknown argument and clap refuses it.
target/release/spec-spine compile --spec 024
target/release/spec-spine compile --spec 024-index-sharding --json
# It wrote nothing: the committed shards are untouched.
target/release/spec-spine compile --check
# An unknown id is exit 1 (not found), never exit 2.
target/release/spec-spine compile --spec 999 ; test $? -eq 1
```
