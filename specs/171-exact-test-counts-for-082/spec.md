---
id: "171-exact-test-counts-for-082"
title: "Exact test counts for 082"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 082's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 082's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "082-an-amended-acceptance-is-the-one-that-runs"
amends:
  - "082-an-amended-acceptance-is-the-one-that-runs"
---

# 171: Exact test counts for 082

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 082's. 082 holds 074, so its plan resolves here too; the superseded note of 074 names this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 082:39 | core `verify` | `superseded` | 1 |

"Block line" counts lines inside 082's `verify:cli` fence. The names each
line ran:

- `verify` `superseded`: `every_superseded_verification_block_says_so_in_its_own_document`.

## 2. Territory

This spec establishes nothing. It edits 082 only to add spec 082 §3.4's
superseded-acceptance note above 082's block, and the note of 074 to name this spec.

## 3. Behavior

### 3.1 082's block is carried

`spec-spine verify 082` and `verify` of 074 MUST run this
spec's block, which carries 082's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

082 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 074 keeps its note and gains one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 082's plan and that of 074 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 082-an-amended-acceptance-is-the-one-that-runs (amends_verification), and through it 074; each loose test count names its tests (153 3.2) ----
# --- spec 082's own mechanism ---
sh -c 'n=$(cargo test -p spec-spine-core --test verify --locked spec103_ 2>&1 | grep -c "^test spec103_"); test "$n" -ge 4 || { echo "expected at least 4 spec103_ tests, ran $n"; exit 1; }'
cargo test -p spec-spine-core --test verify --locked spec103_
cargo test -p spec-spine-cli --test cli --locked spec103_
# 3.1: the compiled field is a governed read.
target/release/spec-spine registry show 082 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["amendsVerification"] == ["074-a-governed-read-names-its-version"], d'
# --- spec 074's acceptance, which this block now holds (3.5) ---
# 3.4: the axis exists and starts where the note says.
grep -qF 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
# 3.2, 3.5: every object read is sorted and versioned.
target/release/spec-spine registry plan --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry show 074 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --nonzero-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry relationships 074 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index owner Cargo.toml --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index orphans --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
# 3.3, 3.6: the three array reads carry their items under a versioned object,
# and the ids-only projection still projects ids.
target/release/spec-spine registry list --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
target/release/spec-spine registry list --ids-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert all(isinstance(x, str) for x in d["items"]); assert d["schemaVersion"]'
target/release/spec-spine index diagnostics --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
# 3.3, 3.6: the pick is a named member, present whether or not it is
# populated. Spec 082 3.5 corrected this line: it asserted the corpus had
# something ready, which 074 3.3 never required and a finished corpus denies.
target/release/spec-spine registry plan --next --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]; assert "next" in d; assert d["next"] is None or d["next"]["id"]'
# 3.7: `config show` is sorted and keeps 047's version member, with no second one.
target/release/spec-spine config show --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert "config_version" in d; assert "schemaVersion" not in d'
# 3.8: the emitter's own properties, and the per-document assertions.
cargo test -p spec-spine-core --test read --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.4: the axis is documented where the others are.
grep -qF 'READ_SCHEMA_VERSION' docs/schema-versioning.md
# 3.4, the document half: every spec whose acceptance another spec holds says
# so above its own fence, and no spec that still holds its own says it. Read
# over the real corpus, because the corpus is the set the rule quantifies over,
# and resolved through `verify_plan` rather than by trusting the paragraph.
sh -c 'cargo test -p spec-spine-core --locked --test verify -- --exact every_superseded_verification_block_says_so_in_its_own_document 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss082-sup.txt"
# 3.4: and one concrete instance beside the quantified one, the spec whose
# acceptance this block IS. Asserted as an ORDERING, because a note after the
# fence is read by nobody who stopped at the commands.
python3 -c 'p="specs/074-a-governed-read-names-its-version/spec.md"; t=open(p).read(); assert t.index("> **Superseded acceptance") < t.index("```verify:cli"), p'
```
