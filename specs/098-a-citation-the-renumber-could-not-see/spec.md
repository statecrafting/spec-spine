---
id: "098-a-citation-the-renumber-could-not-see"
title: "A citation the renumber could not see"
status: draft
kind: "tooling"
created: "2026-09-21"
summary: >
  Spec 096 made compaction a verb and recorded seven defects the hand renumber
  produced. Two of them, the wrapped citation and the multi-ordinal citation,
  were measured after 096 was filed and were never repaired: `compact` refuses a
  plan naming a spec the corpus no longer has, so it prevents the next renumber
  and not the last one. Re-measuring on 2026-09-21 against the collapse commit
  found the debt is not 41 occurrences but 637, in two further classes nobody
  had looked for: 90 spec documents are still titled with another spec's
  ordinal, and 474 citations are written as a bare ordinal (`084 §3.1`, `050's`)
  which none of 096's three forms matches. This spec repairs all 637, adds the
  two forms and the file-selection fix that would have prevented them, and makes
  a title heading self-checking with a new lint code so this class cannot come
  back silently.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "003-conformance-lint"
  - "095-the-corpus-describes-what-exists"
  - "096-compaction-is-a-verb-not-a-session"
extends:
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: { kind: file, path: "crates/spec-spine-core/src/compact.rs" }, nature: additive }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: { kind: file, path: "crates/spec-spine-core/tests/compact.rs" }, nature: additive }
  - { spec: "003-conformance-lint", unit: { kind: file, path: "crates/spec-spine-core/src/lint.rs" }, nature: additive }
  - { spec: "046-depends-on-ordinal-monotonicity", unit: { kind: file, path: "crates/spec-spine-core/tests/lint.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/corpus-map.md" }, role: context }
---

# 098: A citation the renumber could not see

## 1. Purpose

### 1.1 What the map could not repair

Spec 095 collapsed 27 specs into three and renumbered 93 survivors by hand.
Spec 096 §1.1 records seven defects from that rewrite; defects 6 and 7, the
wrapped citation and the multi-ordinal citation, were measured on 2026-09-20
*after* 096 was filed and were left standing on the default branch. The note
recording them said the repair "is a spec of its own and is unfiled". This is
that spec.

The debt was re-measured on 2026-09-21, and the earlier figure of 41 was low
because it was counted with a weaker method. The method that settles it is a
comparison against **the tree before the collapse**: a citation whose
surrounding sentence, with line breaks and comment markers normalized away, is
still the pre-collapse sentence was never rewritten. A grep of the working tree
cannot answer this, because after a contiguous renumber every old ordinal is
also a *current* ordinal: `spec 050` in `index.rs` is correct where the sentence
is new and wrong where it is old, and only the history distinguishes them. D-8
and D-9 record the two weaker tests this one replaced and what each missed.

Measured that way, 637 occurrences in 148 files still name a document that moved:

| Class | What it is | Count | Files |
|---|---|---|---|
| A | A spec document's own **title heading** names another spec's ordinal | 90 | 90 |
| B | A **prose citation** (`spec NNN`), 096 §3.3 form 2, wrapped or multi-ordinal | 59 | 38 |
| C/D | A **bare ordinal** used as a citation: `084 §3.1`, `050 3.6`, `092 D-3`, `056's`, `086-...` | 488 | 80 |

Class A is the loudest: 90 of the 98 documents in this corpus open with a line
naming a spec that is not them. `specs/006-distribution/spec.md` is titled
`# 007: Distribution, uvx/PyPI wheel shim`, and 007 today is a different
document. Nothing looked, because a heading is prose to the compiler in exactly
the way §1.1 of spec 096 says a citation is.

### 1.2 Three mechanism gaps, not one mistake

Each class exists because the rewrite could not see it, and each gap is still
open in `compact`, so the next compaction reproduces it:

1. **A form the rewrite does not know.** 096 §3.3 closes the rewritable set at
   three forms, all of which either spell the full `NNN-slug` or put the word
   `spec` in front of the digits. A document's own title heading is neither, and
   neither is `084 §3.1`. Class A and class C/D are each a form, not a bug in a
   form.
2. **A file the rewrite never opens.** `scannable_files` selects by extension,
   with a three-name allowlist beside it. `.gitignore` and `.gitattributes` have
   no extension: `rsplit_once('.')` reads `.gitignore` as an empty stem with the
   extension `gitignore`, which is on no list. Eight of class B sit in those two
   files, and they are *plain* citations the line-oriented rule of 095 would
   have caught had it been given the line.
3. **Nothing re-asks the question.** Every class was silent under `compile`,
   `index`, `lint`, `couple` and `check`, for the same reason all seven of 096's
   defects were. A wrong citation is only detectable where the document makes a
   statement the tool can check, and exactly one of these classes does: a spec's
   title heading names an ordinal the spec's own directory already fixes.

### 1.3 What is decidable, and what is left

The repair covers what a rule can decide without reading the sentence. It stops
there deliberately: 096 §3.8 refuses "a reference it cannot classify", and
guessing is the failure mode that produced the seven defects in the first place.

After the 637 are repaired, roughly 930 three-digit tokens on pre-collapse lines
still name a moved ordinal, and most of them are not citations at all:
`V-001/005/006` in a code list, `exit 101`, `feat(017): ...` as an example
commit message, a fixture named `"016-short"`. The genuine citations left among
them are written in forms a pattern cannot separate from those: `before 084`,
`pre-096`, a bare `(101,` opening a list. Telling one from the other is reading,
not matching. §4 names it as the debt it is, with the measurement, rather than
letting a wide rule repair nine citations and corrupt one exit code.

## 2. Territory

| Path | What |
|---|---|
| `crates/spec-spine-core/src/compact.rs` | two further reference forms (§3.1, §3.2) and the scan's file selection (§3.3) |
| `crates/spec-spine-core/src/lint.rs` | `L-013`, the title heading a document can check about itself (§3.4) |
| `crates/spec-spine-core/tests/compact.rs` | the forms, and the exclusions that keep them narrow |
| `crates/spec-spine-core/tests/lint.rs` | `L-013` fires, and is silent where it must be |

The repair of §3.5 touches 148 further files and claims none of them: it changes
no requirement and no behavior, only which document a sentence points at.

## 3. Behavior

### 3.1 A fourth form: the document's own title heading

A spec document whose id is remapped MUST have its **own title heading**
rewritten to its new ordinal, reported under the form `title-heading`.

The heading is located as: the first line **after the frontmatter** matching
`#{1,6} <NNN>: `, where `<NNN>` equals the ordinal of the id the document is
being renumbered *from*. Both halves of that are load-bearing:

- *After the frontmatter*, because a YAML comment inside the frontmatter begins
  with `#` too, and this corpus has several that open with an ordinal
  (`# 033, 038 and 041 give them mechanical consequences`). Those are prose
  citations, and form 2 or §3.2 answers for them, not this rule.
- *Equal to the document's own old ordinal*, because a spec may quote another
  spec's heading, and a heading that names a different spec is a citation of
  that spec rather than a title of this one.

A heading whose ordinal already equals the document's new ordinal is not
rewritten, which is what makes the rule idempotent under 096 §3.4.

### 3.2 A fifth form: a bare ordinal in a position only a citation occupies

A bare `NNN` MUST be rewritten, reported under the form `bare-ordinal`, when the
digits are immediately followed by one of:

- a **section reference**: `§`, or a decimal section number `N.N`, with at most
  one space between (`084 §3.1`, `050 3.6`, `093 3.2's`);
- a **decision reference**: `D-<digit>` (`092 D-3`);
- a **possessive**: `'s` or `’s` (`056's`, `050's`);
- an **elided slug**: `-...`, the shape the corpus writes when a full id is too
  long for the sentence (`specs/086-.../spec.md`). Form 1 matches the map's keys
  and `086-...` is not one, so without this clause the one reference shape that
  spells the id out is the one nothing reaches.

The exclusions of 096 §3.3 form 2 apply unchanged and are what keep this narrow:
the digits MUST NOT be preceded by `-`, `.` or a word character, and MUST NOT be
followed by `-` or a word character. That is what separates `084 §3.1` from
`V-014`, `L-011` and `0.18.0`, all of which this corpus is full of.

Nothing else about a bare ordinal is rewritten. `before 084`, `(101,` and
`pre-096` are citations to a reader and indistinguishable from a count to a
rule; §1.3 and §4 say why they are left.

A reference carried to a **removed** spec keeps its section number, which the
answering spec does not share. That is 096's rule for form 2 unchanged, and D-6
records what it means.

### 3.3 The scan selects a file by content when it has no extension

`scannable_files` MUST select a file that carries **no extension** when its
bytes are valid UTF-8 and contain no NUL, and MUST treat a leading-dot name as
carrying no extension: in `.gitignore` the dot opens the name, it does not
separate one.

The three-name allowlist beside the extension filter is removed. Two of its
three entries (`AGENTS.md`, `CLAUDE.md`) were already matched by the `md`
extension, and the third (`Makefile`) is exactly the extension-less case this
rule now decides by content. An allowlist that must be extended by hand for
every new name is the shape defect 8 has: it was written to cover the files
somebody remembered.

### 3.4 `L-013`: a title heading names its own spec

`lint` MUST report `L-013`, **warning** tier, for a spec whose title heading
names an ordinal other than its own.

It is asked of the first heading after the frontmatter matching
`#{1,6} <NNN>: `, and it is **silent** when the spec's id carries no ordinal, or
when the document's first heading carries none. A corpus is not required to
number its ids (`V-001` requires only that the directory equal the id), and
telling a corpus that its headings are wrong when it never claimed the
convention is spec 046 §3.3's mistake to avoid.

Warning tier, not error: the convention is this repository's, held by
`lint --fail-on-warn` where it is held at all. The scaffolded template writes a
literal `# NNN: Title` (`scaffold.rs`), whose ordinal is not digits, so a corpus
that has only ever been scaffolded is silent until it numbers a heading and gets
it wrong. An adopter upgrading into a `lint --fail-on-warn` gate should be told
in the release notes all the same: a new code at warning tier is a new refusal
wherever that flag is set.

This is the only one of the three classes a lint can decide, and it is the
reason class A is worth separating from the rest: the document states its own
ordinal twice, in its directory and in its title, and two statements of one fact
can be compared.

### 3.5 The repair

Every occurrence in §1.1's three classes MUST be rewritten to the ordinal
`docs/corpus-map.md` gives, in this change.

The repair is a rewrite of prose, not of behavior. It changes no requirement,
no acceptance command and no test expectation; a rewritten citation in an
approved spec is a correction of where a sentence points, which is why it is not
an amendment under spec 034 and needs none.

That is measured, not assumed. 35 of the 637 fall inside a `verify:cli` block,
and all 35 are comment lines: no command, argument or grep pattern in any
acceptance block is touched. Had one been, rewriting it would have moved an
assertion's expectation without moving what it asserts about, which is the
family of defect spec 083 exists for.

### 3.6 The repair is not a verb

The one-shot repair MUST NOT be added to `compact` as a mode.

`compact` is a pure function of `(cfg, corpus, plan)` and the engine forbids it
`git` (`docs/design/00-architecture.md`). The repair is decidable **only**
against history: after a contiguous renumber a stale ordinal and a correct one
are the same three digits, and the comparison of §1.1 against the pre-collapse
tree is what tells them apart.
A `--repair` mode without that evidence would have to guess, and 096 §3.8
already refuses to guess between a typo and a citation. The forms of §3.1 and
§3.2 are what `compact` gains; the repair itself is a measured, reviewed edit
made once.

## 4. Out of scope

- **The ~930 remaining tokens.** §1.3. Measured, and left, because separating a
  citation from a count in `before 084`, `pre-096` or `(101,` is reading. The
  measurement is reproducible: for every tracked file, take each three-digit
  token whose value `docs/corpus-map.md` moved, keep it when the sentence around
  it, normalized as D-9 describes, is still that file's pre-collapse sentence,
  and subtract the classes of §1.1.
- **The 155 citations carried by hand into rewritten sentences.** D-9. The only
  test that finds them also returns correct citations in bulk, so the answer is
  a read of each and not a rule.
- **Requiring citations to carry the full id.** `spec 096-compaction-is-a-verb`
  is self-verifying where `spec 096` is not, and a corpus that cited that way
  would need no history at all. It would also rewrite about three thousand
  sentences and make every one of them longer. It is a corpus-style decision,
  not a defect, and it belongs in its own spec if anywhere.
- **Anything about the ordinals themselves.** No spec is renumbered here. The
  map is read, never written.

## 5. Resolved decisions

**D-1 (2026-09-21): the debt is measured against history, not with a grep.**
The figure this spec inherited was 41; the figure it repairs is 637. The
difference is not that more debt accrued, it is that the earlier count asked
whether the same citation *text* survived anywhere in the same file, which
counts a correct new citation as stale and misses a stale one in a file that was
edited. The comparison against the pre-collapse tree asks the question that
decides it: is this still the sentence that was there before the collapse. A
measurement that cannot distinguish the two answers is not a measurement, which
is the same family as an assertion that cannot fail.

**D-2 (2026-09-21): class A is repaired *and* linted; B and C/D are repaired
only.** A lint can decide a title heading because the directory already states
the answer. Nothing states the answer for `spec 050` in a comment, so a lint for
B or C/D would either need history (which `lint` does not have and should not)
or would flag every correct citation in the corpus. One class is checkable and
gets a check; two are not and get a form in `compact` instead, which is where
the next renumber can still go wrong.

**D-3 (2026-09-21): `L-013` is a warning, and the corpus is repaired in the same
change.** A new error-tier code on 90 existing violations would make `lint` red
on the parent commit, which is the shape of a gate that arrives before the work
it gates. Warning tier plus a repair in the same change leaves
`lint --fail-on-warn` green on both sides and the rule live from the first
commit that has it.

**D-4 (2026-09-21): the scan decides by content, not by a longer allowlist.**
Adding `.gitignore` and `.gitattributes` to `NAMES` would have closed the two
files this measurement found and nothing else. The gap is that the selector was
an allowlist; the fix is a rule. An extension-less file that is valid UTF-8 with
no NUL is prose, and there is no third thing it can be that a rewrite would
damage: the walk is already pruned of the derived root, the state root, `.git`
and `resolver_exclusions`.

**D-5 (2026-09-21): the repair leaves a sentence half-repaired rather than
guessing at the rest.** `docs/design/05-remaining-waves-2026-09.md` contains
`101, amending 086 §3.1 and 098 §3.1`; this change rewrites the two ordinals
that carry a section reference and leaves `101,` alone. A reader of that line
now sees one stale ordinal beside two correct ones, which is worse to look at
and better to trust: every ordinal this change touched was decided by a rule,
and `101,` is indistinguishable from a count without reading the sentence around
it. §4 records the whole residue so the next reader measures instead of
guessing.

**D-6 (2026-09-21): a section number does not survive a removal, and the
repair carries it anyway.** 81 of the 637 name a spec spec 095 removed, so
`(116 D-13)` becomes `(093 D-13)` and 093's D-13 is a different decision. Both
available answers are wrong in some way: left alone, `082 D-2` resolves today to
a spec that has nothing to do with it, because after a contiguous renumber every
removed ordinal is somebody else's. Carried, the document is right and the
section number is not. The repair carries it, because that is what
`docs/corpus-map.md` §1 says a removed spec's citation means ("requirements now
in ...") and it is what 096's form 2 already did to 2,910 citations during the
collapse itself. A repair that adopted a second rule for 81 occurrences would
leave the corpus disagreeing with itself about what a citation of a removed spec
points at.

**D-7 (2026-09-21): the repair declares no ownership edge, and the gate
clears it.** A one-line comment fix in `coverage.rs` trips `C-001`, which is
what the note behind this spec measured and why it expected the repair to need
`extends` edges to roughly two dozen specs. It needs none: `couple` clears a
path when **any** of its owning specs' documents is in the diff, and a repair
that rewrites 90 title headings puts almost every owning spec's `spec.md` there.
138 paths checked, no drift, with no edge added.

That clearance is the rule as written and not a bypass, and it is recorded
because it is incidental: it holds for a corpus-wide repair and would not hold
for the same fix made to one file. Claiming those units on this spec to make the
gate happier would have been worse than the alternative it replaced: an
`extends` edge is a permanent claim, and 098 would become an owner of two dozen
units it has nothing to say about, able to clear any future change to them.

**D-8 (2026-09-21): `git blame` cannot see inside a squash merge, and the first
measurement was short by five.** This repository squash-merges, so the collapse
commit is not an ancestor of `main`: every line the collapse's own pull request
wrote is attributed to the squash commit and reads as post-collapse. A blame
test therefore misses exactly the lines that pull request touched without
renumbering, and it missed five, including `spec\n# 120 3.6` in `ci.yml` and two
in `.gitignore`. Comparing the line's **content** against the pre-collapse tree
finds them: a line still byte-identical to its pre-collapse self was never
rewritten, whatever commit claims it. Four of the five are renumbered survivors
and are repaired. D-9 is where that test in turn ran out.

The fifth is not, and it is the one place a human overrode the rule.
`tests/verify.rs` narrates a history: "It was 048 from spec 043 until spec 092:
048 was the first approved spec carrying `verify:cli` fences ... so a plan built
for `048` is no longer 048's block". The possessive matches §3.2, and rewriting
it to `093's` would make the sentence false rather than stale, because 093 never
held that block. Spec 097 §3.5 already draws this line: a sentence recording
what a document used to be is not a citation of the document that answers for it
now. That distinction is a read, not a rule, which is §3.6's whole argument for
why the repair is a reviewed edit and not a verb.

**D-9 (2026-09-21): three tests, and the mode none of them decides.** The
comparison of §1.1 got stronger twice while this spec was being built, and each
step found more:

1. **the line's commit** (`git blame`) — misses everything the collapse's own
   squashed pull request wrote (D-8);
2. **the line's content** — misses a line that was *reflowed* since, which is
   how `ci.yml` and five lines of `docs/design/05-...` kept a stale ordinal
   through a prose rewrap;
3. **the sentence's normalized context**, comment markers and line breaks
   removed — what the final figure is measured with.

The mode none of them decides is a citation an author **carried by hand into a
sentence they rewrote**. `standards/spec/contract.md` and
`standards/spec/constitution.md` each say "spec 040" in a paragraph that spec
092's pull request rewrote, and 040 is now a different document. Both are
repaired here because both were read and both are unambiguous: the sentence is
about declaring an amendment, which is spec 037.

The other 155 candidates for that mode are **not** repaired, and the reason is
the measurement rather than the effort. The test that finds them is "this file
also said `spec NNN` before the collapse", which is the weak test D-1 rejects:
`index.rs` says `// ===== spec 050: which paths any content hash witnesses
=====` and that is *correct*, because the current 050 is exactly that spec. The
candidate set is mostly right citations. Separating them is a read of each, and
§4 records it as the debt it is rather than letting a rule guess 157 times.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured on 2026-09-21 on this branch before the build:
`grep -c '"L-013"' crates/spec-spine-core/src/lint.rs` is `0`;
`grep -c 'title-heading' crates/spec-spine-core/src/compact.rs` is `0`;
`grep -c 'bare-ordinal' crates/spec-spine-core/src/compact.rs` is `0`;
`grep -n '^# 007:' specs/006-distribution/spec.md` matches, which is class A
itself; and `grep -c 'spec 023' .gitignore` is `1`, which is class B in a file
`compact` could not open.

```verify:cli
cargo build --release --locked
# 3.4: the code exists, at warning tier, and reads the heading after the frontmatter.
grep -qF '"L-013"' crates/spec-spine-core/src/lint.rs
# 3.1 / 3.2: both forms are in the report's vocabulary.
grep -qF 'title-heading' crates/spec-spine-core/src/compact.rs
grep -qF 'bare-ordinal' crates/spec-spine-core/src/compact.rs
# 3.3: selection by content, and the hand-written name allowlist is gone.
grep -qF 'fn is_scannable_text' crates/spec-spine-core/src/compact.rs
! grep -qF 'const NAMES' crates/spec-spine-core/src/compact.rs
# 3.5 class A: every spec document is titled with its own ordinal. The loop is
# the assertion; it was red for 90 of 98 documents before this change.
sh -c 'bad=0; for d in specs/*/; do o=$(basename "$d" | cut -c1-3); h=$(grep -m1 -E "^#{1,6} [0-9]{3}: " "$d/spec.md" | sed -E "s/^#{1,6} ([0-9]{3}):.*/\1/"); if [ -n "$h" ] && [ "$h" != "$o" ]; then bad=$((bad+1)); fi; done; test "$bad" -eq 0'
# The same question asked through the verb, which is what makes it a gate.
target/release/spec-spine lint --fail-on-warn
# 3.5 class B, in the two files the scan could not open before 3.3.
grep -qF 'spec 021' .gitignore
! grep -qF 'spec 023' .gitignore
grep -qF 'spec 094' .gitattributes
! grep -qF 'spec 020' .gitattributes
# 3.5 class C/D: three of the 483, one of each shape the form admits.
grep -qF '078 §3.6' crates/spec-spine-core/src/coverage.rs
grep -qF "047's" crates/spec-spine-cli/src/cmd_config.rs
grep -qF 'specs/069-.../spec.md' specs/077-one-hash-one-construction-one-name/spec.md
# D-8: the five a blame test could not see, four repaired and one deliberately not.
grep -qF 'spec 092 3.7, 3.9' .gitignore
grep -qF "048's block" crates/spec-spine-core/tests/verify.rs
# The tests that pin the forms and their exclusions.
cargo test -p spec-spine-core --test compact --locked > "${TMPDIR:-/tmp}/ss098-c.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss098-c.txt"
rm -f "${TMPDIR:-/tmp}/ss098-c.txt"
cargo test -p spec-spine-core --test lint --locked > "${TMPDIR:-/tmp}/ss098-l.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss098-l.txt"
rm -f "${TMPDIR:-/tmp}/ss098-l.txt"
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
