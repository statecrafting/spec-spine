---
id: "174-exact-test-counts-for-086"
title: "Exact test counts for 086"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 086's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 086's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
amends:
  - "086-an-exact-key-set-refuses-what-the-rule-allows"
---

# 174: Exact test counts for 086

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 086's. 086 holds 052, so its plan resolves here too; the superseded note of 052 names this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 086:68 | core `verify` | `spec103_` | 5 |

"Block line" counts lines inside 086's `verify:cli` fence. The names each
line ran:

- `verify` `spec103_`: `spec103_a_cycle_does_not_hang_verify`, `spec103_a_spec_with_no_amender_runs_its_own_block`, `spec103_a_withdrawn_amender_does_not_hold_the_acceptance`, `spec103_resolution_follows_the_chain`, `spec103_verify_runs_the_replacement_block`.

## 2. Territory

This spec establishes nothing. It edits 086 only to add spec 082 §3.4's
superseded-acceptance note above 086's block, and the note of 052 to name this spec.

## 3. Behavior

### 3.1 086's block is carried

`spec-spine verify 086` and `verify` of 052 MUST run this
spec's block, which carries 086's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

086 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 052 keeps its note and gains one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 086's plan and that of 052 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 086-an-exact-key-set-refuses-what-the-rule-allows (amends_verification), and through it 052; each loose test count names its tests (153 3.2) ----
# --- spec 052's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test render --locked
cargo test -p spec-spine-core --test coverage --locked
# 3.1 of spec 052, as 3.3 here reads it: `orphans --json` is an object carrying
# both named arrays. The document is captured to a file so the verb's own exit
# status is this line's status, which a pipeline into `python3` would not be
# (3.2, D-4). It is captured beside the fixture root rather than inside it,
# because the next line begins by deleting that root (D-6).
target/release/spec-spine index orphans --json > "${TMPDIR:-/tmp}/ss059-live.json"
# Presence, type, sorted keys and the version member. Not a key-set equality:
# 059 requires the two arrays to be there, not to be the only members, and
# spec 074 added `schemaVersion` to every governed read (3.3).
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss059-live.json')); k=list(d); assert k==sorted(k), k; assert d['schemaVersion'], d; assert isinstance(d['orphaned'], list), d; assert isinstance(d['inFlight'], list), d"
rm -f "${TMPDIR:-/tmp}/ss059-live.json"
# 3.2 + 3.3 of spec 052: the empty-universe fixture. Built once at a fixed path,
# because each line here is its own shell and a `$(mktemp -d)` would not survive
# to the next assertion. Inherited from 052's block unchanged.
rm -rf "${TMPDIR:-/tmp}/ss059" && mkdir -p "${TMPDIR:-/tmp}/ss059/specs/001-x" && : > "${TMPDIR:-/tmp}/ss059/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n' > "${TMPDIR:-/tmp}/ss059/specs/001-x/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index >/dev/null
# 3.4: 052 3.1's first recorded decision, on the corpus where it can be seen.
# Both arrays are emitted even though both are empty, so a consumer never has to
# tell absent from empty. Omitting an empty group fails this line (D-2).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index orphans --json > "${TMPDIR:-/tmp}/ss059-empty.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss059-empty.json')); assert d['orphaned']==[], d; assert d['inFlight']==[], d"
rm -f "${TMPDIR:-/tmp}/ss059-empty.json"
# 3.4: 052 3.1's second recorded decision. With nothing to report the prose form
# prints nothing, rather than two headers and two `(none)` lines. The redirect
# keeps the verb's status on its own line: `test -z "$(...)"` would pass for a
# verb that failed and printed nothing (D-4). This line fails against this
# repository's own corpus, which has orphans and prints them.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index orphans > "${TMPDIR:-/tmp}/ss059-prose.txt"
test ! -s "${TMPDIR:-/tmp}/ss059-prose.txt"
rm -f "${TMPDIR:-/tmp}/ss059-prose.txt"
# 3.2 of spec 052: without the flag it still exits 0 and reports the fact. The
# report is a read verb, and "no source files" is a true and useful thing to say.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage
# 3.2 of spec 052: with the flag it refuses. An assertion over an empty set is
# vacuously true, and a CI step that did not run its check should not be green.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced ; test $? -eq 1
# 3.3 of spec 052: and the message names which of the two empty cases this is.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss059" index coverage --fail-on-untraced 2>&1 | grep -q 'no package was discovered'
rm -rf "${TMPDIR:-/tmp}/ss059"
# 3.4 of spec 052: this repository's own coverage assertion is unaffected, so
# the refusal is unreachable here.
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine compile --check
# --- spec 086's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-4 gives: at the parent commit this
# verb exits 1 and prints nothing, and a pipeline would report that as a JSON
# decode error naming the wrong defect. The file is named for this spec, whose
# mechanism it is, not for 059, whose acceptance the half above is (D-6).
target/release/spec-spine registry show 086 --json > "${TMPDIR:-/tmp}/ss108-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss108-show.json')); assert d['amendsVerification'] == ['052-read-verbs-on-a-code-free-corpus'], d; assert d['amends'] == ['052-read-verbs-on-a-code-free-corpus'], d"
rm -f "${TMPDIR:-/tmp}/ss108-show.json"
# Spec 052's file is not edited (spec 037 3.1): its own block still carries the
# superseded key-set equality. This goes red the moment someone resolves this by
# editing 059 instead.
grep -qF 'set(o) == {"orphaned", "inFlight"}' specs/052-read-verbs-on-a-code-free-corpus/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 059` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact spec103_a_cycle_does_not_hang_verify spec103_a_spec_with_no_amender_runs_its_own_block spec103_a_withdrawn_amender_does_not_hold_the_acceptance spec103_resolution_follows_the_chain spec103_verify_runs_the_replacement_block 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss108-spec103.txt"
```
