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
  2026-09-09 carried seventeen dead patterns, nine in the table the lint reads
  and eight in the table it does not. A fix pass corrected all nine and six of
  the eight; the two survivors reached an already-reviewed pull request and were
  caught by a second human reading, since no tool could report them. Both
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

Spec 069 fixed the shipped default to the `**/*` form and, by its own §4,
deferred the lint: a pattern matching nothing today can be a legitimate
forward-looking entry, and 069 would not refuse a form on that evidence. 074
§3.1 took the lint up once the form was shown to be inert under every tree
rather than merely empty under this one. Neither pass looked at the second
table, so the value 069 could not reach in an adopter's own file is still
unreachable there.

This is not hypothetical. An adopter audited on 2026-09-09 carried **seventeen**
dead patterns: nine in `[index] extra_hashed_inputs` and eight in
`[index.slices]`. A fix pass corrected all nine in the table the lint reads, and
six of the eight in the table it does not. The two survivors, `deploy/**` and
`eval/**`, reached a pull request that had already been reviewed once, and were
caught by a second human reading rather than by any tool, because no tool can
report them. A second adopter in the same audit carried six more in
`extra_hashed_inputs` and declares no slices table at all.

That split is the argument. The table `L-010` reads came out of the pass clean,
and the table it does not read came out of the same pass still broken, with
nothing able to say so. Both survivors pointed at directories that do not exist
yet, which is the worse half of the defect: they cost nothing on the day they
are written and stay silently empty on the day the directory arrives.

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

Each violation MUST occupy **one line**, as every existing `L-` code's message
already does. This is not a style preference, it is what makes 3.3's prohibition
checkable. A reader, and the acceptance in 3.4, isolates a slice's violation by
selecting the line naming the table; if a message were split across a summary
line and a continuation, a filter that selects the naming line would not see the
rest of the claim, and a negative assertion over it would pass while the
forbidden text sat one line below. The alternative, searching the whole output
for the forbidden phrase, cannot work here either, because the
`extra_hashed_inputs` form of `L-010` says `content hash` legitimately and would
trip it. One line per violation is what keeps the two forms separable.

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

The prohibition is on the **claim**, in whichever spelling: none of
`content hash`, `content-hash` or `contentHash` may appear in a slice's `L-010`
message, in any capitalisation. Naming every form is not pedantry. The acceptance in 3.4 asserts the absence of that
phrase, and an assertion that matches only one spelling passes trivially against
an implementation that emits the other, which would leave the rule this section
exists to enforce untested.

### 3.4 The refusal is proven on a fixture, not on this corpus

This repository declares no `[index.slices]` table, so its own `lint` output
cannot demonstrate the rule and could not regress if the rule were deleted.
Acceptance MUST construct a config that trips the rule and assert the refusal
against it, and MUST also assert the corrected form passes, so the test pins
the boundary rather than the mere presence of a warning.

The refusal assertion MUST be attributable to this rule. A bare `test $? -eq 1`
on `lint --fail-on-warn` is satisfied by **any** warning, so it would pass on a
fixture that tripped something unrelated, and would go on passing if this rule
were later deleted while another warning took its place. The assertion MUST
therefore check the same invocation's output for the slice violation as well as
its exit code.

The negative assertion of 3.3 MUST be scoped to the slice's own line, which
3.2's one-line rule makes sound, rather than run over the whole output: the
`extra_hashed_inputs` form of the same code says `content hash` correctly, so an
unscoped search would refuse a true message.

It MUST additionally assert that a bare `lint` exits 0 on the tripping fixture.
Asserting only the `--fail-on-warn` refusal leaves the tier half-proven: an
implementation that emitted `L-010` at error tier would satisfy every other
assertion here while contradicting 3.1. The tier is the claim most easily lost
in implementation, because promoting a warning is the natural reflex when a
rule turns out to have no legitimate counter-example.

## 4. Out of scope

**Correcting spec 012's own example.** `specs/012-index-hash-slices/spec.md`
teaches `workflows = [".github/workflows/**"]`, the dead form, and adopter
slice tables were copied from it. 012 is approved, and a new spec does not get
to rewrite an approved spec's text through its own decision entry. That
correction is a human's direct one-token edit to line 62, taken outside this
spec.

As of filing that edit is **not** on `main`: line 62 still reads
`.github/workflows/**`. Nothing in this spec's acceptance depends on it, and 079
can be built with 012 unchanged.

What the gap is, precisely. `lint` reads `spec-spine.toml`, not spec markdown,
so 012's fenced example changes no verdict for anyone and nothing regresses in
the tool. The cost is entirely on the reading side: an adopter who copies the
example writes a pattern the tool then refuses, which is how at least one
adopter's dead slice table got there. That makes it a documentation defect with
a measured victim, not a behavior defect.

Who does it and when. The edit is a human's, because 012 is approved and a draft
may not rewrite it. It **should land before 079 is ratified**, so that no window
exists in which the corpus has ratified a rule its own specification
contradicts. This spec does not block on it: building 079, and merging the
build, are fine with 012 unchanged. Ratification is the gate, and it is the
ratifier's check rather than an acceptance criterion, since 079 cannot assert
anything about a file it does not claim.

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
even though 074 is where `L-010` was established, and the test file is claimed
through `053-depends-on-ordinal-monotonicity`. Each unit is routed through the
spec that **establishes** it: 003 establishes
`crates/spec-spine-core/src/lint.rs`, and 053 establishes
`crates/spec-spine-core/tests/lint.rs`, which it created because `L-007` was the
first rule to need a dedicated lint test file. 074
carries the same two units the same two ways, so this follows the path already
taken for these exact files. The routing is worth stating because the corpus is
not consistent about it: specs 057 and 058 carry `tests/lint.rs` through 003,
which does not own it, and an `extends` unit is a first-class claim either way,
so nothing refuses them.

074 is left out of both edge lists for a second reason. It is still
`status: draft`, held only by its §3.6, which is about deleting
`kit/scripts/verify-spec.sh` and has nothing to do with this rule. Making 079
depend on 074 would report 079 as blocked behind a decision it does not wait on.
074 is named throughout the prose as the rule's origin, which is where that
relationship belongs.

053 also appears in `depends_on`, where the relationship is ownership routing
rather than behavioral precedence: 079 needs nothing 053 decided about ordinal
monotonicity, it needs the file 053 created. The field name implies more than is
meant, so it is written down here rather than left for a reader auditing the
dependency graph to find surprising.

D-2 (2026-09-09, one code rather than `L-011`). The defect, the reasoning and
the remedy are identical in both tables; only the sentence about what is lost
differs, which 3.3 handles. A second code would make an adopter learn two names
for one mistake, and would let a corpus pass `L-010` while carrying the same
dead form one table over.

## Verification

The two assertions that carry the rule fail against pre-079 code, because
`lint` does not read `[index.slices]` at all today: the co-occurrence chain and
the `--fail-on-warn` refusal both exit non-zero. The bare-`lint` tier line and
the corrected-form line are boundary controls and pass both before and after,
which is what makes them controls.

Each line is one command. Spec 049 §3.2 makes a fence's body line a command, so
no line may depend on a variable another line set, and the fixture setup is one
long line rather than a continuation.

Each line is run as its own `sh -c`, a full POSIX shell, with no `set -e`
(`crates/spec-spine-cli/src/cmd_verify.rs`, the `Command::new("sh").arg("-c")`
call that executes each planned command and compares its status). That is what
makes `;`, `!` and `&&` mean what they read as, and in particular what makes
`<cmd> ; test $? -eq 1` a real assertion rather than a no-op: `$?` is the
preceding command's status in the same shell. Spec 059's block, approved and
`implementation: complete`, relies on the same guarantee. Stated here because a
reader who assumes a restricted line executor would read the tier assertion as
vacuous, and the tier is the claim this spec most needs to hold.

```verify:cli
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.1: a fixture whose only defect is a dead pattern in a slice.
rm -rf "${TMPDIR:-/tmp}/ss079" && mkdir -p "${TMPDIR:-/tmp}/ss079/specs/001-x" && printf '[index.slices]\nworkflows = [".github/workflows/**"]\n' > "${TMPDIR:-/tmp}/ss079/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-09"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n' > "${TMPDIR:-/tmp}/ss079/specs/001-x/spec.md"
# 3.1: the refusal is attributable to this rule. Exit 1 alone would be
# satisfied by any warning at all, so the same invocation's output must also
# carry the slice violation. One line, so no state crosses a line boundary.
o=$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint --fail-on-warn 2>&1) ; test $? -eq 1 && printf '%s\n' "$o" | grep -F 'L-010' | grep -qF 'index.slices'
# 3.2: the message names the table and the slice.
# Chained, not three independent greps: separate greps prove only that each
# string appears somewhere, which an implementation splitting the message across
# lines would satisfy while leaving 3.2's one-line rule unverified and 3.3's
# scoped negative toothless. The chain proves all three share one line.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -F 'L-010' | grep -F 'index.slices' | grep -qF 'workflows'
# 3.1: the tier. A bare `lint` reports the warning and still exits 0; an
# implementation that promoted it to an error would pass every other line here.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint
# 3.3: no line about a slice claims a content hash is affected. The line above
# already asserts a slice line exists, so this one only has to be negative, and
# it runs the binary once.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint 2>&1 | grep -F 'index.slices' | grep -qiE 'content[ -]?hash'
# 3.4: the corrected form passes, so the test pins the boundary.
printf '[index.slices]\nworkflows = [".github/workflows/**/*"]\n' > "${TMPDIR:-/tmp}/ss079/spec-spine.toml" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss079" lint --fail-on-warn
rm -rf "${TMPDIR:-/tmp}/ss079"
# 3.4: this repository declares no slices table, so its own lint is unmoved.
target/release/spec-spine lint --fail-on-warn
target/release/spec-spine compile --check
```
