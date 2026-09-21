---
id: "085-a-version-pin-is-not-a-contract"
title: "A version pin is not a contract"
status: approved
kind: "core"
created: "2026-09-17"
summary: >
  Spec 049's `## Verification` block pins the verdict envelope at the literal
  `0.3.0`, which spec 071 legitimately moved to `0.4.0`, so
  `spec-spine verify 056` has been red since 088 merged. The pin was never what
  056 required: 056 3.4 states that `--spec` emits the spec 034 envelope with a
  `verb` token distinguishing it from `compile --check`, and records the bump it
  performed as a fact about that bump, not as a standing ceiling. A literal pin
  cannot assert either half, because it goes red for every move the rule permits
  and stays green for a verb token that collides. This spec declares 056's
  acceptance replaced and carries the corrected block, without editing 056's
  file. It is the first of the four follow-ons spec 084 4 named.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "049-compile-one-spec"
  - "071-a-change-is-classified-under-the-bases-rules"
  - "082-an-amended-acceptance-is-the-one-that-runs"
amends: ["049-compile-one-spec"]
# 3.1: this spec's `## Verification` block IS 056's acceptance from now on.
# 056's own file is not edited (spec 037 3.1), and 088's is not either: 088
# states a rule that is true and complete, and this spec changes none of it.
# What changes is what 056 accepts.
amends_verification: ["049-compile-one-spec"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 107: A version pin is not a contract

## 1. Purpose

### 1.1 One red assertion, no defect behind it

Measured on 2026-09-17 against `e4d7d28` with the 0.20.0 binary:

```
$ spec-spine verify 056
verify: 049-compile-one-spec: FAILED at command 5 (exit 1)
```

Command 5 is

```
compile --spec 022 --json | python3 -c '... assert v["schemaVersion"]=="0.3.0"'
```

and the envelope answers `0.4.0`. Spec 071 moved `VERDICT_SCHEMA_VERSION` from
`0.3.0` to `0.4.0` when `delta` was added, which is the additive case that
constant's own documentation names, and every other assertion in 056's block is
green. Run one line at a time to the end of the block, this is the only red one:

| Cmd | Assertion | Status |
|-----|-----------|--------|
| 5 | `schemaVersion == "0.3.0"` | **red**, the envelope is `0.4.0` |
| 6 | `compile --check` exit 0 | green |
| 7 | `compile --spec 999` exit 1 | green |
| 8 | `compile --spec 022 --check` exit 3 | green |

`verify` stops at the first failure, so a reader who runs the verb sees command
5 and learns nothing about the other three.

### 1.2 The pin asserted neither half of the rule it stood for

056 3.4 is two requirements and one record. The requirements are that `--spec`
accepts `--json` and emits the spec 034 verdict envelope, and that the envelope
carries a `verb` token **distinguishing it from `compile --check`**. The record
is that `VERDICT_SCHEMA_VERSION` moved from `0.2.0` to `0.3.0` for that
addition, with the reasoning for a MINOR bump spelled out.

A literal pin at `0.3.0` asserts neither requirement and misreads the record:

- It is **too strong** for the version. 056 3.4 states what that one bump was,
  in the past tense, and gives the rule that governs future ones: additive is
  MINOR. An assertion that refuses every subsequent MINOR bump forbids what the
  rule it is standing for explicitly permits, and 088 then performed exactly the
  permitted move. Spec 083 D-2 named the test this fails: an assertion that must
  eventually fail for a legitimate reason is not an assertion.
- It is **too weak** for the verb. The line does assert `verb == "compile.spec"`,
  which is half of the distinguishing requirement. The other half is the
  distinction itself, and nothing in 056's block ever ran `compile --check
  --json` to see what it answers. Give both verbs the token `compile.spec` and
  056's block stays green while the requirement it exists for is broken.

This is the third site in this corpus with the same defect and the second
consecutive spec to correct one: spec 074's calendar assertion (corrected by
103), spec 044's `VERDICT_SCHEMA_VERSION` pin at `0.2.0` (corrected by 106), and
this. 106 1.2 states the general form; this spec is one instance of it, and the
instance where the pin sits beside the requirement it should have been asserting.

### 1.3 The crossing was silent

Spec 071 changed a constant that another approved spec's acceptance pinned, and
nothing told it so. `verify` is the one verb that executes what the corpus
declares, so it sits outside the gate chain deliberately (`AGENTS.md`); CI never
runs it. 088's own gate was green, its own block was green, and 056's went red
in the same merge without a line of output anywhere.

Spec 084 1.3 records the identical mechanism for 050 and names this spec's
target among the four remaining. Nothing here adds to that diagnosis; the repair
is what this spec adds.

## 2. Territory

This spec establishes no code. It owns its own `spec.md` and one claim about
another spec's file: that 056's `## Verification` block is no longer the one
that runs. Nothing under `crates/` changes, no schema constant moves, and no
committed shard changes except the two this spec's own frontmatter produces.

## 3. Behavior

### 3.1 What spec 049's acceptance now is

This spec's `## Verification` block MUST replace spec 049's in full, through
`amends_verification` (spec 082 3.1), and 056's file MUST NOT be edited. The
block is 056's, with command 5 replaced per 3.3, with the payload assertions of
3.4 added, and with this spec's own assertions (3.5) after them under a heading
that says whose is whose.

Replacing the block in full rather than the one line follows spec 082 3.5 and
spec 083 D-1: a reader asking what 056 accepts today should find one block that
answers, not a base document plus a patch to apply in their head.

### 3.2 The verb's exit status is the line's status

Command 5 pipes the verb into `python3`, so the line reports `python3`'s status
and the verb's is discarded. The replacement MUST redirect each document to a
file under `${TMPDIR:-/tmp}/ss056` and assert against the files, so a verb that
fails takes the line down at the redirect.

The scratch directory is named for **spec 049**, whose acceptance this block is,
not for this spec, which merely holds it. Spec 083 carries 098's block under
`ss098` for the same reason, and 3.1 makes the block 056's from the moment this
spec merges.

### 3.3 The version is compared across the pair the rule names

The literal pin MUST be replaced by a comparison across `compile --spec --json`
and `compile --check --json`, asserting three things:

- each verb's token, `compile.spec` and `compile.check`, which is the
  distinguishing requirement of 056 3.4 asserted on both sides rather than one;
- that the envelope carries a non-empty `schemaVersion` at all, which is the
  spec 034 envelope requirement;
- that the two are **equal**, which is the property a single
  `VERDICT_SCHEMA_VERSION` constant has and per-verb versioning would not.

The counter-verb is not chosen for convenience. 056 3.4 names `compile --check`
as the thing `--spec` must be distinguishable from, so the comparison runs
against the verb the rule itself names, and there is no anchor to justify
(contrast spec 084 D-8, which had to pick one).

What this comparison does **not** prove is that 088's bump was legitimate, or
that any future bump will be. No command run against one tree can observe
causation in the past. It proves the standing structural property and says so
here, rather than leaving a reader to infer that the line proves more than it
does.

### 3.4 The payload is asserted, because short-id resolution lives in it

056 3.1 requires `--spec` to resolve a short id, and 056's block asserts that
only as an exit code: `compile --spec 022` exits 0. An exit code cannot say
**which** spec was judged, so a `--spec` that resolved `024` to the wrong spec,
or that validated the whole corpus and returned 0, passes 056's block unchanged.

The replacement MUST assert that the `--spec` payload names the resolved full
id, its path, and an empty `violations` list, and that the `compile --check`
payload carries no `specId`. The second is what makes the shared version say
something about payload independence: the two verbs answer different questions
under one envelope constant.

The `compile --check` arm is contingent, and loudly so. If `compile --check`'s
report ever gains a `specId` the line goes red and a reader is sent to D-3,
rather than passing while asserting less than it appears to. That is the
distinction from the pin 1.2 rejects: a pin goes red on a routine and expected
event, which 088 performed, while this goes red only if two verbs' payloads
converge, which no rule here forbids but nothing plans either.

### 3.5 The acceptance

`registry show 107 --json` MUST carry `amendsVerification` and `amends`, both
naming 056 and nothing else: that is the declaration, and it is read through the
CLI rather than off the shard.

Spec 049's file MUST still carry the superseded literal pin. That is the spec
037 3.1 half, and it is the assertion that goes red if someone ever resolves
this by editing 056 instead, which is the move
`AGENTS.md` "Adversarial prompt refusal" refuses.

The resolution itself MUST keep passing through spec 082's own cases (`spec103_`
in `crates/spec-spine-core/tests/verify.rs`). No code changes here, so the
mechanism is what is asserted, not a new one. The run MUST be captured and its
summary asserted to name a non-zero pass count: a name filter that matches
nothing exits 0, so the bare line would stay green while asserting nothing (spec
084 D-7).

**No `verify` command may appear in this block.** It is the block
`spec-spine verify 056` runs, and `cmd_verify::run` refuses a nested call on its
own spec (`SPEC_SPINE_VERIFY_STACK`) **before** it honours `--plan`, so even
reading the plan from inside is a validation failure. The instance-level fact,
that `spec-spine verify 056` and `spec-spine verify 107` both exit 0 on the
merged tree, is what a reviewer runs and what the release sweep runs.

### 3.6 No code changes

No file under `crates/` is edited, no `*_SCHEMA_VERSION` constant moves, no
embedded schema changes, and no CLI surface is added or removed. The mechanism
this spec uses was built and shipped by spec 082 in v0.20.0; this spec is a
corpus change that uses it.

## 4. Out of scope

- **Editing spec 049, or spec 071.** 1.2 and the frontmatter comment. 056 keeps
  the block it was ratified with, which is the record spec 037 3.2 protects, and
  088's text is true as written: it moved a constant the rule permits it to move.
- **The other three red blocks.** Specs 052, 060 and 071 are red for the same
  family of reason and each needs its own amendment, because
  `amends_verification` replaces a target's whole block with the amender's
  single one. Spec 084 4 names all four; this spec is the first.
- **The remaining green literal pins.** Spec 047's block pins `config_version`
  at `0.1.0` and spec 083's carries a `schemaVersion=='0.4.0'` inherited from
  098. Both are the shape 1.2 names and both are green today. A green line in
  another spec's block is not this spec's to change; 106 4 already names them.
- **Putting `verify` in the gate chain.** Spec 083 4 declined it and recommended
  a maintainer sweep instead. 1.3 restates the consequence and does not relitigate
  the decision.
- **Teaching `amends_verification` in the template.**
  `standards/spec/templates/spec-template.md` does not carry the key spec 082
  added. 106 4 names it as a real gap in territory neither spec owns.
- **The piped `registry show` line in specs 083's and 106's blocks.** Both carry
  the form D-4 rejects, and both are green, because on a merged tree the verb
  they read succeeds. Correcting either is an amendment of that spec rather than
  a repair of this block.

## 5. Resolved decisions

D-1 (2026-09-17, why the counter-verb needed no anchor argument). Spec 084 D-8
had to justify picking spec 000 as the id its comparison ran against, because
`compile --spec` needs an id that exists and any id can be deleted. This spec's
comparison is between two **verbs**, and the second verb is the one 056 3.4
names in the sentence the line is asserting. There is no arbitrary choice left
to defend. The `--spec 022` lines inherited from 056's block do depend on 024
existing, and they already did before this spec; adding the `specId` assertion
of 3.4 to the same lines adds no dependency that was not already there.

D-2 (2026-09-17, why the payload assertions are added rather than left out).
Adding to a block being repaired is scope this spec had to justify, as spec 084
D-3 did for its one added line. The reason is that 056 3.1's short-id resolution
was asserted only as an exit code, and an exit code cannot distinguish "resolved
024 and found it valid" from "resolved something else and found that valid".
Measured: with `report.specId` rewritten to `000-spec-spine-bootstrap` the
assertion fails, and the exit-code line it sits beside does not. These are
assertions about spec 049's own territory, not new territory.

D-3 (2026-09-17, what the `compile --check` arm rests on). `'specId' not in r`
is a claim about what `compile --check`'s payload happens to contain, not about
what the verb is, and that contingency is recorded rather than hidden. The
structural weight is carried by the two verb tokens, which spec 034 makes part
of the envelope. The member arm adds that the two payloads differ in a named
way, which is what makes a shared version say anything at all about payload
independence. Spec 084 D-8 records the same distinction for the same reason, and
the reasoning is not restated here beyond naming where it lives.

D-4 (2026-09-17, why the pipeline becomes a redirect). `cmd --json | python3 -c
...` reports `python3`'s status, so a verb that failed and printed nothing feeds
`json.load` an empty document and the line fails for the wrong reason with a
message naming the wrong defect. Worse in a two-document comparison, where two
failures could compare equal. 3.2 requires files for that reason, following spec
084 D-4, which recorded the same hazard when it hit it.

The rule reaches **every** line in the block, including 3.5's `registry show`.
The first draft left that one as a pipeline, inherited from spec 084's block,
which put this spec in the position of stating a rule in D-4 and breaking it
eleven lines later. Review caught it. The case is not hypothetical there and is
the one a reviewer meets first: at the parent commit `registry show 107` exits 1
and prints nothing, so the pipeline form reports a JSON decode error for what is
a not-found, which is precisely the misattribution this entry exists to prevent.
Spec 084's and spec 083's copies are green and are another spec's acceptance, so
they are named in 4 rather than edited here.

D-5 (2026-09-17, what the fail-first evidence is and is not). This spec changes
no code, so the corrected lines pass at the parent commit `e4d7d28` against the
same binary whose output made 056's original red. That is correct rather than a
gap, and it is spec 084 D-5's finding restated at a second site: what was red at
the parent was 056's block, and this spec makes nothing go green that was
legitimately red.

What an assertion owes instead is **failability against the condition it exists
to catch**, and each corrected line was run against that condition. Measured on
2026-09-17 by rewriting the captured documents:

| Condition | Line's status |
|---|---|
| the two envelope versions diverge | fails (`('0.4.0', '9.9.9')`) |
| `--spec` answers the token `compile.check` | fails |
| `--spec` resolves `024` to another spec's id | fails |
| `compile --check`'s report gains a `specId` | fails |
| the tree as it stands | passes |

The one line that is fail-first at the parent in the ordinary sense is
`registry show 107`, a not-found exit 1 there, which is 3.5's half.

D-6 (2026-09-17, why the scratch directory is named `ss056`). The block is spec
049's acceptance from the moment this spec merges (3.1), and the state it
carries between lines is that block's state. Spec 083 carries 098's block under
`ss098` on the same reading. The alternative reading, that the directory names
whichever spec holds the block, would rename 056's scratch state every time the
acceptance moves again, which is the outcome the first reading avoids. Recorded
because the question has been asked of this family three times.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the scratch directory under `${TMPDIR:-/tmp}/ss056` carries the
state instead. This block is spec 049's acceptance (3.1) as well as this spec's
own, so it is read in two halves and labelled as such.

**Fail-first evidence.** Only `registry show 107` is red at the parent commit, a
not-found exit 1. The corrected lines are not, and measurably so: this spec
changes no code, so they exit 0 against the parent's binary. D-5 records what
each corrected line was measured against instead, and that it failed there.

```verify:cli
# --- spec 049's acceptance, which this block now holds (3.1) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test compile --locked
# 3.1: the flag exists, resolves the short id, and validates without writing.
target/release/spec-spine compile --spec 022
target/release/spec-spine compile --spec 022-index-sharding --json
# Scratch: each line is its own shell, so a verb's stdout is carried in a file
# rather than a variable. The redirect leaves the verb's own exit status as the
# line's status, which a pipeline into `python3` would not (3.2, D-4).
rm -rf "${TMPDIR:-/tmp}/ss056" && mkdir -p "${TMPDIR:-/tmp}/ss056"
target/release/spec-spine compile --spec 022 --json > "${TMPDIR:-/tmp}/ss056/spec.json"
target/release/spec-spine compile --check --json > "${TMPDIR:-/tmp}/ss056/check.json"
# 3.3: 056 3.4 requires a verb token that distinguishes `--spec` from
# `compile --check`, so both tokens are read and the envelope version is
# compared across the pair rather than pinned to a literal. The equality
# witnesses one VERDICT_SCHEMA_VERSION constant rather than per-verb
# versioning; it does NOT prove that any past bump was legitimate, which no
# command run against one tree can observe.
python3 -c "import json; a=json.load(open('${TMPDIR:-/tmp}/ss056/spec.json')); b=json.load(open('${TMPDIR:-/tmp}/ss056/check.json')); assert a['verb']=='compile.spec', a; assert b['verb']=='compile.check', b; assert a['schemaVersion'], a; assert a['schemaVersion']==b['schemaVersion'], (a['schemaVersion'], b['schemaVersion'])"
# 3.4: the payload names which spec was judged, which is where 056 3.1's
# short-id resolution actually shows. The exit-code line above passes whatever
# `024` resolved to; this one does not.
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss056/spec.json'))['report']; assert r['specId']=='022-index-sharding', r; assert r['specPath']=='specs/022-index-sharding/spec.md', r; assert r['violations']==[], r"
# 3.4: and `compile --check` answers a different question under the same
# envelope. Contingent on that verb's payload, deliberately and loudly (D-3).
python3 -c "import json; r=json.load(open('${TMPDIR:-/tmp}/ss056/check.json'))['report']; assert 'specId' not in r, r"
rm -rf "${TMPDIR:-/tmp}/ss056"
# 3.5 of spec 049: it wrote nothing. The committed shards are exactly as they were.
target/release/spec-spine compile --check
# 056 3.1: an unknown id is exit 1 (not found), never exit 2.
target/release/spec-spine compile --spec 999 ; test $? -eq 1
# 056 3.1: `--spec` and `--check` are different questions, and the pair is refused.
target/release/spec-spine compile --spec 022 --check ; test $? -eq 3
# --- spec 085's own mechanism (3.5) ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-4 gives: at the parent commit this
# verb exits 1 and prints nothing, and a pipeline would report that as a JSON
# decode error naming the wrong defect. The file is named for this spec, whose
# mechanism it is, not for 056, whose acceptance the half above is (D-6).
target/release/spec-spine registry show 085 --json > "${TMPDIR:-/tmp}/ss107-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss107-show.json')); assert d['amendsVerification'] == ['049-compile-one-spec'], d; assert d['amends'] == ['049-compile-one-spec'], d"
rm -f "${TMPDIR:-/tmp}/ss107-show.json"
# Spec 049's file is not edited (spec 037 3.1): its own block still carries the
# superseded literal pin. This goes red the moment someone resolves this by
# editing 056 instead.
grep -qF 'v["schemaVersion"]=="0.3.0"' specs/049-compile-one-spec/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 056` runs, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss107-spec103.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss107-spec103.txt"
rm -f "${TMPDIR:-/tmp}/ss107-spec103.txt"
```
