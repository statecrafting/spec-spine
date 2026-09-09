---
id: "079-a-dead-glob-is-dead-in-both-tables"
title: "A dead glob is dead in both tables"
status: draft
kind: "tooling"
created: "2026-09-09"
summary: >
  `L-010` refuses a pattern ending in `/**` in `[index] extra_hashed_inputs`,
  where it enumerates directories and can contribute no bytes. `[index.slices]`
  carries pattern lists with the same documented semantics and the same
  file-only walk, and the lint is silent about it. An adopter audited on
  2026-09-09 had nine dead patterns across the two tables; the slice half
  survived a fix pass and a review because nothing named it, and the survivors
  pointed at directories that do not exist yet, so they would have stayed
  silently empty on the day those directories arrived. This spec extends the
  rule to the second table, and requires the slice message to speak about the
  slice's own hash rather than repeating a sentence about `contentHash` that is
  false for slices.
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "003-conformance-lint"
  - "012-index-hash-slices"
  - "053-depends-on-ordinal-monotonicity"
  - "069-the-shipped-default-hashes-what-it-names"
extends:
  # 3.1 to 3.3: the rule, and the message that has to say which table it is
  # talking about. Claimed the way spec 074 claimed the same file when it
  # added `L-010` in the first place, through 003 rather than through 074:
  # see the decision in section 5.
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  # 3.4: the rule's own acceptance, on a fixture rather than this corpus.
  - { spec: "053-depends-on-ordinal-monotonicity", unit: "crates/spec-spine-core/tests/lint.rs", nature: additive }
---

# 079: A dead glob is dead in both tables

## 1. Purpose

`L-010` (spec 074 §3.1) refuses an `[index] extra_hashed_inputs` pattern ending
in `/**`, because in the `glob` crate `dir/**` enumerates directories and the
hasher keeps only files, so such a pattern can never contribute a byte. The
lint reads that one table and is silent about the other.

`[index.slices]` (spec 012) holds pattern lists too. `IndexConfig::slices` is
documented in its own doc comment as carrying "`extra_hashed_inputs` pattern
semantics", and the slice walk in `index.rs` keeps only files exactly as the
global one does. So `dir/**` is equally inert there, and nothing says so.

This is not hypothetical. An adopter audited on 2026-09-09 carried nine dead
patterns across the two tables. The six in `extra_hashed_inputs` were found and
fixed; the two in `[index.slices]` survived the same fix pass and reached a pull
request that had already been reviewed once, precisely because no lint named
them and the fix was driven off the lint's output. The remaining dead slice
patterns pointed at directories that do not exist yet, which is the worse half
of the defect: they cost nothing on the day they are written and stay silently
empty on the day the directory arrives.

An adopter who upgrades is currently told about one table and not the other.
That is a worse position than being told about neither, because the silence
reads as clearance.

## 2. Territory

This spec claims no new file. It extends two units it shares with the specs
that own them:

- `crates/spec-spine-core/src/lint.rs`, through `003-conformance-lint`, for the
  rule and its message. Spec 074 carries the same unit through the same spec.
- `crates/spec-spine-core/tests/lint.rs`, through
  `053-depends-on-ordinal-monotonicity`, for the acceptance.

`spec-spine.toml` in this repository declares no `[index.slices]` table and this
spec does not add one. Section 3.4 says what that costs and how acceptance is
arranged around it.

## 3. Behavior

### 3.1 `L-010` reads both pattern tables

`lint` MUST apply the `L-010` rule to every pattern in every list under
`[index.slices]`, under the same test it applies to
`[index] extra_hashed_inputs`: a pattern ending in `/**` is refused.

The check stays on the **pattern**, not on whether it currently matches, for
the reason 074 §3.1 already gave: a pattern matching nothing today is a
legitimate forward-looking entry in a specify-first corpus, while a pattern
ending `/**` is inert under every tree and so is decidable from the config
alone. That reasoning is if anything stronger here, since the adopter evidence
above is exactly a forward-looking slice entry that would never have fired.

It stays at **warning** tier, so `lint --fail-on-warn` refuses it and a bare
`lint` reports it.

### 3.2 The message names which table, and which slice

One code now covers two tables, so the message MUST say which one it is
reporting, and for a slice it MUST name the slice. A reader who sees `L-010`
against `workflows` must be able to find the offending line without guessing
which table it came from.

### 3.3 The slice message MUST NOT claim a content hash

The existing `L-010` text ends "so it can contribute no bytes to any content
hash". That is true of `extra_hashed_inputs` and **false of a slice**: spec 012
makes slices independent of the global hash, and `IndexConfig::slices` says so
in terms ("listing a file here does NOT fold it into `contentHash`"). A correct
slice pattern contributes no bytes to `contentHash` either.

The slice message MUST therefore speak about the slice's own hash, the one
`index check --slice <name>` gates, and MUST NOT tell the reader their content
hash is affected. Reusing the existing sentence verbatim would ship a false
statement under a true code.

### 3.4 The refusal is proven on a fixture, not on this corpus

This repository declares no `[index.slices]` table, so its own `lint` output
cannot demonstrate the rule and could not regress if the rule were deleted.
Acceptance MUST construct a config that trips the rule and assert the refusal
against it, and MUST also assert the corrected form passes, so the test pins
the boundary rather than the mere presence of a warning.

## 4. Out of scope

**Correcting spec 012's own example.** `specs/012-index-hash-slices/spec.md`
teaches `workflows = [".github/workflows/**"]`, the dead form, and adopter
slice tables were copied from it. 012 is approved, and a new spec does not get
to rewrite an approved spec's text through its own decision entry. That
correction is a human's direct one-token edit to line 62, taken outside this
spec and before it.

**An `L-008` analogue for slices.** `L-008` flags a claimed path that no content
hash witnesses. A slice is an opt-in named group, not an ownership claim, so
"claimed but in no slice" is not a defect and must not become a warning.

**Folding slices into `contentHash`.** Their independence is spec 012's design,
and 3.3 depends on it rather than changing it.

**Refusing the form at config load.** `deny_unknown_fields` style refusal would
make this exit 3 at parse time in every verb, including read verbs. 074 chose
the lint tier for the same rule and this spec does not reopen that.

## 5. Resolved decisions

D-1 (2026-09-09, the edge target). The lint unit is claimed through
`003-conformance-lint`, not through `074-shipped-is-not-the-same-as-working`,
even though 074 is where `L-010` was established. Two reasons. 074 itself
carries `lint.rs` through 003, so this follows the path already taken for this
exact file. And 074 is still `status: draft`, held only by its §3.6, which is
about deleting `kit/scripts/verify-spec.sh` and has nothing to do with this
rule; making 079 depend on 074 would report 079 as blocked behind a decision it
does not wait on. 074 is named throughout the prose as the rule's origin, which
is where that relationship belongs.

D-2 (2026-09-09, one code rather than `L-011`). The defect, the reasoning and
the remedy are identical in both tables; only the sentence about what is lost
differs, which 3.3 handles. A second code would make an adopter learn two names
for one mistake, and would let a corpus pass `L-010` while carrying the same
dead form one table over.

## Verification

Every assertion below fails against pre-079 code: `lint` does not read
`[index.slices]` at all today, so the fixture that trips the rule exits 0.

Each line is one command. Spec 049 §3.2 makes a fence's body line a command, so
no line may depend on a variable another line set, and the fixture setup is one
long line rather than a continuation.

```verify:cli
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.1: a fixture whose only defect is a dead pattern in a slice.
rm -rf "${TMPDIR:-/tmp}/ss079" && mkdir -p "${TMPDIR:-/tmp}/ss079/specs/001-x" && printf '[index.slices]\nworkflows = [".github/workflows/**"]\n' > "${TMPDIR:-/tmp}/ss079/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-09"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n' > "${TMPDIR:-/tmp}/ss079/specs/001-x/spec.md"
# 3.1: the warning tier refuses it under --fail-on-warn.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint --fail-on-warn ; test $? -eq 1
# 3.2: the message names the table and the slice.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -q 'L-010'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -q 'index.slices'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -q 'workflows'
# 3.3: the slice message does not claim a content hash is affected.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -q 'index.slices' && ! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep 'index.slices' | grep -q 'content hash'
# 3.4: the corrected form passes, so the test pins the boundary.
printf '[index.slices]\nworkflows = [".github/workflows/**/*"]\n' > "${TMPDIR:-/tmp}/ss079/spec-spine.toml" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint --fail-on-warn
rm -rf "${TMPDIR:-/tmp}/ss079"
# 3.4: this repository declares no slices table, so its own lint is unmoved.
target/release/spec-spine lint --fail-on-warn
target/release/spec-spine compile --check
```
