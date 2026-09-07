---
id: "067-the-docs-name-what-adopters-derived"
title: "The docs name what adopters derived by experiment"
status: draft
kind: "documentation"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "009-coupling-floor-claim-precedence"
  - "017-directory-crate-module-units"
  - "037-machine-readable-verdicts"
  - "044-in-progress-is-in-flight"
establishes:
  # Both files exist and no spec claimed them. This spec takes authority over
  # the two documents it edits; `docs/specify-first.md` is claimed by the
  # implementing change that creates it.
  - "docs/adoption-guide.md"
  - "docs/schema-versioning.md"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "README.md" }, role: context }
summary: >
  Three documentation gaps the audit measured, each with a named victim. There
  is no page for specify-first adoption, which is how three of the four governed
  repositories work, so each of them independently derived the guarded Makefile,
  the CI manifest probe, `implementation: n-a` for record specs, and the fact
  that two hundred and forty-eight W-001 warnings is the defined state rather
  than a backlog. Two interactions are documented nowhere and were found by
  experiment: a path can be on the coupling bypass floor and simultaneously a
  hashed input, and a trailing-slash directory unit in `establishes` satisfies
  ownership recursively. And spec 037 changed `attest`'s stdout, which
  claude-observatory regexes in two places, with no migration note. This spec
  writes the page and the three paragraphs.
---

# 067: The docs name what adopters derived by experiment

## 1. Purpose

The audit's question was what adopters had to work out for themselves. Three
answers are documentation, and each has a name attached.

**Specify-first has no page.** Three of the four governed repositories ratify
the entire corpus before a line of code exists, and live at `approved` plus
`pending` for months. Specs 041 and 044 were built for that mode. No document
mentions it. So each adopter independently derived the same four things: a
Makefile whose language targets are guarded on a manifest probe so the composite
gate is green on a code-free tree; the CI job-output probe, because GitHub
rejects `hashFiles` in a job-level `if`; `implementation: n-a` as the convention
for a record spec that owns nothing; and the fact that aicortex's two hundred
and forty-eight `W-001` warnings are the defined state of a corpus that has not
been built, not a backlog to burn down. Four rediscoveries, three times each.

**Two interactions were found by experiment.** Both are consequences of designs
that are individually documented and jointly surprising:

- *A path can be on the coupling bypass floor and simultaneously a hashed
  input.* `docs/` is on `DEFAULT_BYPASS_PREFIXES`, so editing a doc raises no
  `C-001`. `standards/**` is in the default `extra_hashed_inputs`, so editing a
  standards file restamps the global scalar and stales every index shard. An
  adopter reasonably assumes bypassed means invisible, and then watches a
  documentation edit stale sixty shards.
- *A trailing-slash directory unit in `establishes` satisfies ownership
  recursively.* `- "src/thing/"` claims the subtree, so every file under it is
  specifically claimed for coverage and for `C-002`. This is the cheapest way to
  retire coverage debt and it is documented only in spec 017's own text.

**Spec 037 has no migration note.** It made every verdict verb emit a canonical
JSON envelope, and in doing so changed what `attest` writes to stdout.
claude-observatory regexes `attestationHash:` out of that stdout in two places.
Nothing told it to stop.

These are items 4, 8 and 13 of the adopter audit's ranked backlog for the kit
and the scaffold.

## 2. Territory

This spec claims no code and no committed artifact. It owns prose in `docs/`,
which is on the coupling gate's built-in bypass floor, so the claims here are
ledger facts rather than gate-enforced ones.

| Unit | Owner | What changes |
|---|---|---|
| `docs/adoption-guide.md` | this spec | the two interactions |
| `docs/schema-versioning.md` | this spec | the 037 migration note |
| `docs/specify-first.md` | this spec, from the build | new page |

Neither of the two existing documents was claimed by any spec, so this one takes
authority over them rather than extending somebody. The new page is created by
the implementing change and claimed there, which is one of the two
always-legitimate mid-build edits. `docs/` is not a hashed input in this repository's
configuration (only `standards/**` and `.github/workflows/**` are), so these
edits stale nothing.

## 3. Behavior

### 3.1 `docs/specify-first.md`

A new page MUST describe the mode three of four adopters work in, covering:

- **What specify-first means**: the corpus is ratified before the code exists,
  and `approved` plus `pending` is the steady state, not a transitional one.
- **The lifecycle table**, by reference to the contract section spec 066 adds
  rather than as a second copy. Two tables that must agree by hand is the defect
  this page would otherwise create.
- **`implementation: n-a` for record specs**: a spec that owns no code is not
  pending forever, and the scaffolded bootstrap spec already carries `n-a` for
  exactly this reason (spec 045).
- **Why the warnings are fine**: `W-001` counts on an unbuilt corpus are the
  defined state, `index check` is right to call the ledger fresh, and
  `--fail-on-unresolved` is the flag for a corpus that has moved past this stage.
  aicortex's 248 is the worked example.
- **The guarded composite gate**: pointing at `kit/Makefile` and `kit/govern.yml`
  (spec 064) rather than restating them, and naming the `hashFiles` finding once
  so a reader who is not using the kit still gets it.
- **What to do first**: the honest sequence for a repository with no code, which
  is `compile`, `index`, `lint`, and specifically not `couple` until there is a
  base and a head to compare.

It MUST be linked from `README.md`'s documentation table and from
`docs/adoption-guide.md`, because a page nobody links is a page nobody finds,
and the adopters who needed it were reading both.

### 3.2 The two interactions, in the adoption guide

`docs/adoption-guide.md` MUST gain a short section, anchored
`bypass-and-hashing-are-independent`, stating that the coupling bypass floor and
the content-hash input set are **independent axes**, that a path can be on both,
neither, or either, and giving `docs/` (bypassed, not hashed here) and
`standards/**` (bypassed, hashed) as the two worked examples.

The sentence that closes it: bypassed means the coupling gate will not refuse a
change to it; hashed means a change to it stales shards. Neither implies the
other, and the two mechanisms answer different questions.

It MUST also gain a section, anchored `directory-units-claim-recursively`,
stating that a trailing slash makes a `file` unit a subtree claim, that every
file under it counts as specifically claimed for `index coverage` and for
`C-002`, and that this is the intended instrument for retiring coverage debt
across a directory rather than enumerating files.

Both are short. They are things a reader needs to have been told once.

### 3.3 The 037 migration note

`docs/schema-versioning.md` MUST gain a migration note for spec 037 saying, in
its own words: if you regex a field out of a verdict verb's stdout, stop, and
parse the `--json` envelope instead.

It MUST name `attest` and `attestationHash` specifically, because that is the
case that broke and a general note would not have told claude-observatory that
its two call sites were the ones affected.

It MUST state the guarantee that makes migration safe: `--json` changes what is
written and never what is decided, so every exit code is identical with and
without it. A consumer switching to the envelope is not changing its control
flow, only its parsing.

### 3.4 Nothing mechanical changes

No code, no committed artifact, no schema version. This spec's whole output is
prose, and its `## Verification` block can only assert that the prose exists and
that the corpus is unchanged by it. That is a weaker acceptance than a code
spec's and it is stated plainly rather than dressed up: a documentation spec is
verified by a reader, and the mechanical check is that the files are there and
the ledger did not move.

## 4. Out of scope

**Writing the Makefile or the workflow.** Spec 064. This page points at them.

**A second lifecycle table.** §3.1. The contract holds it (spec 066).

**Restructuring the documentation set.** One new page, two sections and one
note, each in an existing document's register.

**Fixing claude-observatory's two call sites.** An adopter-side follow-up, in
the audit's §5. This spec gives it the note to act on.

**A general migration-notes convention.** One note for one change that broke a
known consumer. A standing "Migration" section per schema bump is a good idea
and a separate one.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
# The page exists and is linked from the places its readers were already in.
test -f docs/specify-first.md
grep -q 'specify-first' README.md
grep -q 'specify-first' docs/adoption-guide.md
# The two interactions are written down.
grep -q 'bypass' docs/adoption-guide.md
grep -q 'trailing slash' docs/adoption-guide.md
# The 037 migration note names the field that broke a real consumer.
grep -q 'attestationHash' docs/schema-versioning.md
# Prose only: the ledger did not move.
target/release/spec-spine compile --check
target/release/spec-spine index check
```
