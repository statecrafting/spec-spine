---
id: "207-the-registry-lists-what-verify-would-run"
title: "The registry lists specs by implementation state and by whether verify would run anything"
status: draft
kind: "tooling"
created: "2026-10-27"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "009-registry-query-projection-flags"
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "153-a-test-count-is-exact"
establishes:
  # The filters' end-to-end tests are a new file, planned (spec 063), so the
  # draft blocks no other change. The `extends` edges onto the shared query,
  # record and schema files are added by the build, after ratification, and
  # are named in section 2.
  - { kind: file, path: "crates/spec-spine-cli/tests/registry_list_filters.rs", planned: true }
summary: >
  `registry list` filters by lifecycle status only. Adopters that run every
  implemented spec's acceptance keep that set by hand, as a Makefile variable
  and as CI partitions validated against it, and resolve conflicts in it
  whenever two closures race. One adopter's hand list, compared on
  2026-10-27 with what its corpus says, includes one spec still in progress
  and omits one complete spec that declares acceptance. This spec adds two
  filters that compose with `--status` and with `--ids-only`:
  `--implementation <state>`, over the frontmatter value, and
  `--has-acceptance`, true when `spec-spine verify <id>` would run at least
  one command, resolved the way `verify` resolves an amended block. The
  record gains `declaresAcceptance`, computed at compile, so the answer comes
  from the committed ledger like every other registry read.
intent:
  goal: "An adopter derives the set of specs whose acceptance it runs from the ledger, in one typed read, instead of maintaining it by hand."
  non_goals:
    - "Partitioning that set into CI groups"
    - "Running any acceptance, or reporting whether one passed"
---

# 207: The registry lists specs by implementation state and by whether verify would run anything

## 1. Purpose

`spec-spine verify <id>` runs one spec's declared acceptance (spec 043). Which
specs to run it for is left to the adopter, and adopters answer it by hand:

- a list of ids in a Makefile (`VERIFIED_SPECS := 001 002 ...`), which a
  `verify` target loops over;
- CI partitions of that list, with a script that refuses a partition that does
  not cover the list exactly once;
- a merge resolver for the list itself, because two pull requests that each
  close a spec both append to it, and they conflict on the same line.

The list is meant to be "the specs whose acceptance holds now", which the
corpus already knows: a spec's `implementation` value, and whether its
Verification section declares a command. Compared on 2026-10-27, one adopter's
hand list of 30 specs and the 30 specs its corpus derives
(`implementation: complete` with acceptance declared) differ in two places:
the hand list includes a spec that is `in-progress`, and leaves out a
`complete` spec that declares acceptance. Either may be deliberate. The point
is that nobody can tell from the list, and the derivation is one read away.

Reading it today means parsing `registry list --json` and every spec body,
the ad-hoc read the governed-read rule forbids.

## 2. Territory

New file, claimed as planned by this draft:

- `crates/spec-spine-cli/tests/registry_list_filters.rs`: the filters'
  end-to-end tests over a fixture corpus.

The build adds, after ratification, additive `extends` edges onto the files
other specs own: `crates/spec-spine-cli/src/cmd_registry.rs` and
`crates/spec-spine-core/src/query.rs` (spec 002, the flags and `ListFilter`),
`crates/spec-spine-types/src/registry.rs` (the record field), the shard schema
`crates/spec-spine-types/schemas/registry-spec-shard.schema.json`, and the
compile step that fills the field. They are left out of the draft on purpose:
a merged draft that claims a shared file refuses every change touching that
file until it is ratified.

## 3. Behavior

### 3.1 `--implementation <state>`

`registry list --implementation <state>` keeps the specs whose frontmatter
`implementation` equals `<state>`. `<state>` is one of the frontmatter tokens:
`pending`, `in-progress`, `complete`, `n-a`, `deferred`. Any other value is a
usage error (exit 3) naming the five. A spec with no `implementation` value
matches no state.

### 3.2 `--has-acceptance`

`registry list --has-acceptance` keeps the specs for which
`spec-spine verify <id>` would run at least one command. "Would run" is
resolved exactly as `verify` resolves the plan's source:

1. a spec whose block is replaced by another spec's (spec 082 3.2, followed
   transitively as spec 153 3.2 does) takes the answer of the spec whose block
   runs;
2. otherwise, the spec's own Verification section answers.

A block answers true when it holds at least one command: a non-empty line of a
`verify:cli` fence that is not a comment, by the same parse `verify` uses
(spec 043 3.2). A fence holding only comments, a section with no fence, and a
fence of another tag all answer false.

### 3.3 The record field

The compiled record gains `declaresAcceptance`: true when the spec's **own**
Verification section holds at least one command (3.2's parse), and omitted
when false. The field describes the spec's own text only; 3.2's resolution
through `amends_verification` happens at query time, from edges the registry
already records. `REGISTRY_SCHEMA_VERSION` takes a MINOR, as every additive
record field has.

Every shard already carries `specVersion`, so a MINOR repaints every shard on
the engine bump that brings it; this field adds no repaint of its own.

### 3.4 Composition and order

The filters intersect with each other and with `--status`. `--ids-only`
projects the result as spec 009 3.1 defines, so

```
spec-spine registry list --implementation complete --has-acceptance --ids-only
```

prints the ids one per line, and a JSON string array under `--json`. Output
order is registry (id) order, as `list` already is. An empty result prints
nothing under `--ids-only` and `(no specs)` otherwise, and exits 0.

Under `--json` without `--ids-only`, each record carries `declaresAcceptance`
as compiled, plus `acceptanceFrom`: the id of the spec whose block 3.2
resolved to, present only when `--has-acceptance` is passed and only when it
differs from the record's own id. That is what lets a consumer see why a spec
with no block of its own was kept.

### 3.5 What it does not change

Without either new flag, `registry list` output is byte-identical to today's
for the same ledger, apart from the record field under `--json`.

## 4. Acceptance criteria

1. Each of the five `--implementation` tokens selects exactly the fixture
   specs carrying it; an unknown token exits 3 and names the five.
2. `--has-acceptance` keeps a spec with one command and drops a spec whose
   fence holds only comments, a spec with no fence, and a spec whose only fence
   is not `verify:cli`.
3. A spec whose block another spec replaces is kept when the replacing block
   has a command, with `acceptanceFrom` naming the replacing spec, and dropped
   when it has none; a two-step chain resolves to its end.
4. The filters intersect with each other and with `--status`, in id order.
5. `--ids-only` and `--ids-only --json` project the filtered set as spec 009
   defines.
6. Without the new flags, the text and `--ids-only` outputs are unchanged.
7. `declaresAcceptance` appears in a compiled shard exactly when the spec's own
   section holds a command, and the shard schema accepts it.

## 5. Out of scope

- Partitioning the derived set into CI groups. Partitions stay authored; a
  consumer can validate them against this read.
- Running acceptance, or recording whether it passed (spec 043 and the
  sweep, spec 157).
- Any change to `verify`'s own resolution, which this spec reads and does not
  restate differently.

## 6. Resolved decisions

**D-1 (2026-10-27): the field is compiled, not computed at query time.**
Reading spec bodies inside `registry list` would make one answer mix the
committed ledger with the working tree, and a stale ledger would then disagree
with itself. A compiled field keeps the read on the ledger, and the freshness
gate already covers it.

**D-2 (2026-10-27): the field records the spec's own text; amendment is
resolved at query time.** The `amends_verification` edges are already in the
registry. Compiling the resolved answer into each record would make a shard
depend on another spec's text and repaint it whenever that spec changes.

**D-3 (2026-10-27): no `--exclude` list.** An adopter that deliberately
leaves a spec out can subtract it in the consumer, where the reason can be
written down; a flag would move that reason into a command line nobody reads.

## Verification

Each line below is one command (spec 043 3.2). Until the build, these fail:
the flags and the test file do not exist yet.

```verify:cli
cargo build --release -p spec-spine-cli --locked
# 3.1 to 3.5: every acceptance criterion over a fixture corpus.
cargo test -p spec-spine-cli --locked --test registry_list_filters
# 3.3: the shard schema accepts the field.
grep -qF '"declaresAcceptance"' crates/spec-spine-types/schemas/registry-spec-shard.schema.json
```
