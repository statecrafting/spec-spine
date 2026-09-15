---
id: "094-a-claim-below-the-header-window-is-not-silent"
title: "A claim below the header window is not silent"
status: draft
kind: "tooling"
created: "2026-09-14"
summary: >
  A `// Spec: specs/<id>/spec.md` comment header claims the file it sits in,
  and `index.rs::scan_comment_headers` looks for it in
  `content.lines().take(16)`. That bound is claim eligibility, and no spec,
  standard or adopter-facing document records it: a header on line 17 claims
  nothing, the file then reads as unclaimed, and under
  `[coupling] require_ownership` a change to it is a `C-002` refusal for a
  reason nothing states. Two neighbouring failures are equally silent: a
  header whose reference names no spec in the corpus, and the `//!` doc-comment
  form, which the scanner deliberately does not accept. This spec declares the
  window (16, unchanged) and the recognizer, and makes `index coverage` report
  the near misses, so a file that tried to claim itself and failed is
  distinguishable from a file that never tried.
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "004-codebase-index"
  - "032-ownership-coverage"
  - "059-read-verbs-on-a-code-free-corpus"
  - "076-planned-territory-is-declared-not-inferred"
extends:
  # 3.1 to 3.3: the window constant, the recognizer, and the near-miss scan.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.4: the report gains the list; the classifier is untouched.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-types/src/coverage.rs", nature: additive }
  # 3.5: the prose form of `index coverage`.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  # 3.7: the tests, in the two suites that already cover the scanner and the report.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 094: A claim below the header window is not silent

## 1. Purpose

### 1.1 An eligibility rule with no record

Spec 032 gave a source file a third way to be claimed: a
`// Spec: specs/<id>/spec.md` comment header, which needs no frontmatter edit
anywhere and claims exactly the file it sits in. It is how this repository
retired its own coverage debt without touching the tier-1 bootstrap spec, and it
is the mechanism an adopter reaches for first, because it is the only claim a
single file can make about itself.

The scanner reads `content.lines().take(16)`. Nothing else in the tree bounds
the search, and nothing anywhere records that it is bounded: verified by
grepping `specs/`, `standards/spec/` and `docs/` for the window in any spelling.
The constant has been in `index.rs` since 2026-06-09 and reached a document for
the first time when a review pass wrote it into `CLAUDE.md`, which is how its
absence from the corpus became visible.

### 1.2 What the silence costs

Claim eligibility is the ownership ratchet's input. A header on line 17 of a
file with a license block or a long module doc comment claims nothing, so:

1. `index coverage` counts the file as unclaimed debt;
2. with `[coupling] require_ownership` on, the next change to it is a `C-002`
   refusal;
3. the refusal names no reason that would lead the author to line 17.

The author's next move is to add the file to some spec's `establishes`, which
is a different claim in a different place, or to argue with a gate whose rule is
undocumented. Both are worse than being told.

### 1.3 Two neighbours, equally silent

The same function has two other paths that end in nothing happening:

- **An unresolvable reference.** `spec_id_from_path` requires the trailing
  segment to be a spec id in the corpus. A header naming a spec that was
  renamed, retired or mistyped matches the recognizer, stops the scan (the loop
  breaks at the first `Spec:`-shaped line), and claims nothing.
- **The `//!` form.** The scanner strips one `//` or one `#`; a Rust inner doc
  comment leaves `!` in the way, so `//! Spec: ...` does not match. This is
  deliberate (a module doc comment is prose, not a claim), and it is a trap
  precisely because the file looks claimed to a reader.

### 1.4 What the corpus can be told

This spec does not change what claims. It records the rule and reports the near
misses, which is the smallest change that converts three silent failures into
three sentences.

## 2. Territory

This spec names the claim window as a constant in `index.rs` and adds the
near-miss scan beside `scan_comment_headers`; it adds one additive member to
`CoverageReport` and the prose line that prints it; it adds cases to the index
and coverage test suites. It changes no committed artifact: the near-miss list
is computed on read, like every other member of the coverage report, and no
index shard records it.

## 3. Behavior

### 3.1 The claim window is declared

A comment-header claim MUST be recognized only in the **first 16 lines** of the
file. The bound MUST be a named constant in `index.rs` with the spec section
that declares it cited in a comment, so the value and its authority sit
together.

The value does not change. Raising or lowering it would reclassify files in
every adopter corpus at once: raising it turns files that read as unclaimed
today into claimed ones, which silently moves ownership, and lowering it turns
claims into debt. Sixteen is what shipped, and a claim is a file's opening
statement, so the bound that exists is the bound this spec declares.

### 3.2 The recognizer is declared with it

For each line in the window, in order:

1. leading whitespace is trimmed;
2. **one** leading marker is stripped, either `//` or `#`, and nothing else;
3. the remainder is trimmed and MUST begin with `Spec:`;
4. the reference after `Spec:` is trimmed, one trailing `/spec.md` is removed,
   and the final path segment MUST be the id of a spec in the corpus.

Steps 1 to 3 decide whether a line is a claim attempt; step 4 decides whether
the attempt resolves. The scan MUST stop at the first line satisfying steps 1
to 3, whether or not step 4 resolves, which is the shipped behavior: a file gets
one header, and a second `Spec:` line further down is not a second claim.

`//!` does not satisfy step 2, so an inner doc comment MUST NOT claim. `.py`
and `.sh` files claim with `#`, which is why the marker list has two entries and
not one.

### 3.3 The near-miss scan

Alongside the claim scan, the indexer MUST collect near misses over the same
file universe (`SOURCE_EXTS` inside a discovered package, `resolver_exclusions`
pruned). A near miss is one of:

- **`outside-window`**: a line satisfying §3.2 steps 1 to 4 that lies **after**
  the claim window and within the report window (§3.6), in a file that made no
  claim inside the window;
- **`unknown-spec`**: a line inside the claim window satisfying steps 1 to 3
  whose reference does not resolve (step 4 fails);
- **`doc-comment-marker`**: a line inside the claim window whose trimmed form
  begins with `//!` and whose remainder, after the `!`, satisfies step 3.

Each near miss MUST record the path, the 1-based line number, the reason, and
the spec id when the reference resolves. Near misses MUST be sorted by path then
line, so the report is a pure function of the tree.

A file that claims successfully inside the window MUST NOT produce an
`outside-window` near miss, because nothing was missed.

### 3.4 Where it is reported

`CoverageReport` MUST gain one additive member, `nearMissHeaders`, carrying the
near misses sorted by path then line, omitted when empty, following
`plannedTerritory` (spec 076 §3.6) in shape and for the same reason: a corpus
with none emits exactly what it did before.

`index coverage` is the right home because a near miss is a fact about a file's
ownership, which is the question that verb answers. The coverage classifier MUST
NOT change: a near-miss file is classified exactly as it is today (unclaimed, or
floor-only, or claimed by some other unit). The list explains a classification;
it does not alter one.

### 3.5 What refuses, and what does not

`index coverage --fail-on-untraced` MUST NOT consider near misses: it refuses on
unclaimed files, and in the common case the near-miss file is already one of
them, so the refusal already exists and only its explanation was missing. A near
miss on a file that is claimed by another unit refuses nothing at all.

No new diagnostic code is emitted, so no diagnostics tier, no
`--fail-on-unresolved` set and no committed shard changes (§5 D-2 records why).

### 3.6 The report window, and why it is bounded

The near-miss scan for `outside-window` MUST cover lines 17 through **64** and
stop there.

The bound is measured, not chosen for elegance. Across this repository's
tracked source files:

| Detector | Files reported |
|---|---|
| substring `Spec:` past line 16 | 11 |
| §3.2's recognizer, resolving id, past line 16, whole file | 1 |
| §3.2's recognizer, resolving id, lines 17 to 64 | 0 |

The eleven are prose about the mechanism (doc comments describing
`// Spec:`) and test fixture strings; the recognizer rejects all of them, which
is the argument for using the scanner's own recognizer rather than a search.
The one survivor is `crates/spec-spine-core/src/kit_embedded.rs` line 1948, an
embedded copy of a hook that legitimately carries its own claim header. That
file is generated, 2500 lines long, and would be reported forever. A misplaced
header is near the top, after a license block or a banner; a claim-shaped line
two thousand lines in is file content, and the bound is what separates the two.

Because the measurement is zero, this spec adds no near miss to this
repository's own coverage output, and a reviewer can read the report as empty
rather than as a list to triage.

### 3.7 Documentation

The window and the recognizer MUST be stated in
`website/docs/cli/index.md`, where the comment-header claim is already
described for adopters. `CLAUDE.md` already states the window and needs no
edit.

### 3.8 The tests

`crates/spec-spine-core/tests/index.rs` MUST cover the recognizer: a claim on
line 16 claims, the same line at 17 does not, `//! Spec:` does not claim, a `#`
header claims in a `.sh` file, and an unresolvable reference stops the scan
without claiming.

`crates/spec-spine-core/tests/coverage.rs` MUST cover the report: each of the
three reasons appears with its path, line and reason; a successful claim
produces no near miss; a claim-shaped line past the report window produces none;
and `--fail-on-untraced`'s verdict is unchanged by the presence of near misses.

## 4. Out of scope

**Changing the window's value** (§3.1).

**Making the window configurable.** A knob would mean two corpora disagreeing
about what a claim is, and a claim that resolves differently per configuration
is worse than one bound nobody wrote down. If a corpus needs a header lower in
the file, the answer is a unit in frontmatter, which has no positional rule.

**Accepting `//!` as a claim.** The scanner's refusal is deliberate. Reporting
it (§3.3) is the change; accepting it would make every module doc comment
mentioning a spec a claim.

**A diagnostic code.** See §5 D-2.

**Editing `docs/adoption-guide.md`.** It is in `[index] extra_hashed_inputs`,
so a line added there restales every shard in the repository. The statement
belongs in the adopter documentation that is not a hashed input, and the
adoption guide can gain it in a change that is already paying that cost.

## 5. Resolved decisions

**D-1 (2026-09-14). The bound on the report window is 64, from a measurement.**
See §3.6. The alternative, scanning the whole file, produces one permanent false
report in this repository and would produce more in any corpus that embeds file
contents as string literals, which is a normal thing for a test suite to do.

**D-2 (2026-09-14). A coverage member, not a diagnostic code.** A `W-003` was
considered and rejected. Committed index diagnostics are attributed per spec
shard (`committed_diagnostics` reads `shard.diagnostics` and stamps the shard's
spec id), and two of the three near-miss reasons name no spec that could own the
entry: `unknown-spec` resolves to nothing by definition. Attributing a file's
near miss to the spec it names would also put an entry in that spec's shard,
changing committed bytes for a fact about a file the spec does not own. The
coverage report is computed on read, already enumerates exactly this universe,
and already answers ownership questions per file.

**D-3 (2026-09-14). The recognizer is stated as behavior, not left to the
code.** Spec 032 described the claim without bounding it, so the bound lived
only in an expression. Stating steps 1 to 4 makes the next change to the
scanner a change to a declared rule.

## Verification

Each line is one command. The three `grep` lines fail against pre-094 code:
there is no named window constant, no near-miss member on the report DTO, and
no statement of the bound in the adopter documentation. Those are the fail-first
evidence. The `cargo test` lines are **not** fail-first: at the parent commit
the cases in §3.8 do not exist, so the suites pass vacuously. They carry the
behavioral assertions, which cannot be made against this corpus's own output
because §3.6 measured it to have no near miss to report.

```verify:cli
# 3.1: the window is a named constant, not a literal in an expression.
grep -qE 'COMMENT_HEADER_(CLAIM_)?WINDOW' crates/spec-spine-core/src/index.rs
# 3.4: the report DTO carries the member.
grep -qE 'near_miss' crates/spec-spine-types/src/coverage.rs
# 3.7: the adopter documentation states the bound.
grep -qE '16 lines|first 16' website/docs/cli/index.md
# 3.3, 3.4: the report parses and this corpus shows no near miss (3.6).
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("nearMissHeaders", []) == [], d["nearMissHeaders"]'
# 3.5: the untraced verdict is unchanged on this corpus.
target/release/spec-spine index coverage --fail-on-untraced
# 3.8: the recognizer and the report.
cargo test -p spec-spine-core --test index --locked
cargo test -p spec-spine-core --test coverage --locked
```
