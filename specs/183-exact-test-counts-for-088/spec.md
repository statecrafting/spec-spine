---
id: "183-exact-test-counts-for-088"
title: "Exact test counts for 088"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 088's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 088's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "088-the-template-teaches-the-whole-grammar"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "088-the-template-teaches-the-whole-grammar"
amends:
  - "088-the-template-teaches-the-whole-grammar"
---

# 183: Exact test counts for 088

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 088's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 088:7 | types `dogfood` | `authoring_template` | 1 |

"Block line" counts lines inside 088's `verify:cli` fence. The names each
line ran:

- `dogfood` `authoring_template`: `authoring_template_documents_every_frontmatter_key`.

## 2. Territory

This spec establishes nothing. It edits 088 only to add spec 082 §3.4's
superseded-acceptance note above 088's block.

## 3. Behavior

### 3.1 088's block is carried

`spec-spine verify 088` MUST run this
spec's block, which carries 088's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

088 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 088's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 088-the-template-teaches-the-whole-grammar (amends_verification); each loose test count names its tests (153 3.2) ----
cargo build --release --locked
# 3.5: the key list the template is checked against is the parser's own, and
# the test that reads it is this spec's acceptance in the gate chain. The run
# is captured and its summary asserted to name a non-zero pass count: a filter
# matching nothing exits 0 and would leave this line green while asserting
# nothing.
sh -c 'cargo test -p spec-spine-types --locked --test dogfood -- --exact authoring_template_documents_every_frontmatter_key 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss111-dogfood.txt"
# 3.5: the guard that keeps the iteration above from passing vacuously. Its
# subject is named, so it fires when a key of §1.2 leaves the grammar rather
# than when the list merely changes size (D-6).
grep -qF 'left KNOWN_KEYS; this test' crates/spec-spine-types/tests/dogfood.rs
# 3.1: the syntax, shown with the `amends` entry it requires.
grep -qF '# amends_verification: ["NNN-predecessor"]' standards/spec/templates/spec-template.md
# 3.1: the guidance, clause by clause. A key name alone teaches nothing.
grep -qF 'Every entry MUST also appear in `amends` (`V-018`)' standards/spec/templates/spec-template.md
grep -qF 'V-019' standards/spec/templates/spec-template.md
grep -qF 'V-019' standards/spec/templates/spec-template.md
grep -qF 'skips a `superseded` or `retired`' standards/spec/templates/spec-template.md
grep -qF "prints which spec's block it ran" standards/spec/templates/spec-template.md
grep -qF 'amendsVerification' standards/spec/templates/spec-template.md
grep -qF 'in FULL with the defect corrected, not a patch' standards/spec/templates/spec-template.md
grep -qF 'the amended file still carries the' standards/spec/templates/spec-template.md
# 3.2: the four other keys the parser accepts and the template had lost, and
# the sentence that marks the two inert ones as inert.
grep -qF '# code_aliases:' standards/spec/templates/spec-template.md
grep -qF '# feature_branch:' standards/spec/templates/spec-template.md
grep -qF '# amendment_record:' standards/spec/templates/spec-template.md
grep -qF '# origin:' standards/spec/templates/spec-template.md
grep -qF 'read by nothing today' standards/spec/templates/spec-template.md
# 3.3: the nested forms 96 and 12 specs write.
grep -qF 'nature: additive' standards/spec/templates/spec-template.md
grep -qF 'planned: true' standards/spec/templates/spec-template.md
grep -qF 'raises no `W-001`' standards/spec/templates/spec-template.md
# 3.3: all six unit granularities.
grep -qF 'kind: directory' standards/spec/templates/spec-template.md
grep -qF 'kind: crate' standards/spec/templates/spec-template.md
grep -qF 'kind: module' standards/spec/templates/spec-template.md
# 3.4: the two sections the template stopped short of.
grep -qF '## 5. Resolved decisions' standards/spec/templates/spec-template.md
grep -qF '```verify:cli' standards/spec/templates/spec-template.md
grep -qF 'green before the work asserts' standards/spec/templates/spec-template.md
# The claim is declared, read through the CLI rather than off the shard.
# Redirected, not piped: a failing verb prints nothing and a pipeline would
# report that as a JSON decode error naming the wrong defect (spec 085 D-4).
target/release/spec-spine registry show 088 --json > "${TMPDIR:-/tmp}/ss111-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss111-show.json')); assert 'standards/spec/templates/spec-template.md' in json.dumps(d['establishes']), d['establishes']"
rm -f "${TMPDIR:-/tmp}/ss111-show.json"
# The ownership answer the five prior specs could not get. Exit 0 with the id
# named is the difference this spec makes to `index owner`.
target/release/spec-spine index owner standards/spec/templates/spec-template.md > "${TMPDIR:-/tmp}/ss111-owner.txt"
grep -qF '088-the-template-teaches-the-whole-grammar' "${TMPDIR:-/tmp}/ss111-owner.txt"
rm -f "${TMPDIR:-/tmp}/ss111-owner.txt"
```
