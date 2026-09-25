---
id: "151-carried-acceptance-tests-what-it-names"
title: "Carried acceptance tests what it names"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  The 0.27.0 cross-reference of 136 to 146 found three acceptance plans that
  could pass without testing what their spec now says. 018's plan (carried by
  136) never runs 142's amendment of it: its coupling tests build their index by
  hand, so they pass whether or not the index hands a partially superseded unit
  over. 144's link-rule line accepts any positive test count, so a link test
  that stops running still passes. 145's binary line drives only `index
  coverage`, while six other verbs read the same guard. This spec carries all
  three plans under `amends_verification`: 018's section gains the hand-off,
  run through `index` and the shipped binary; 144's line names its three tests;
  145's block gains one fixture driven through all seven verbs. Each new line
  fails under a mutation the carried line it tightens passes.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "136-the-legacy-ledger-is-paid-registry-and-grammar"
  - "142-a-relocation-is-proven"
  - "144-a-repository-path-is-one-type"
  - "145-an-unresolved-claim-is-not-called-stale"
amends_verification:
  - "136-the-legacy-ledger-is-paid-registry-and-grammar"
  - "144-a-repository-path-is-one-type"
  - "145-an-unresolved-claim-is-not-called-stale"
amends:
  - "136-the-legacy-ledger-is-paid-registry-and-grammar"
  - "144-a-repository-path-is-one-type"
  - "145-an-unresolved-claim-is-not-called-stale"
---

# 151: Carried acceptance tests what it names

## 1. Purpose

Measured on `baae17c9` (0.27.0 plus drafts 147 to 150), 2026-09-25.

### 1.1 018 does not re-test 142's amendment

142 §3.1 amended 018 §4.3: a live partial `supersedes` now hands its unit over
exclusively, and the index stops treating the predecessor's claim as ownership.
018's plan is 136's block (136 holds 018 under `amends_verification`). Its three
coupling tests (`partial_supersedes_scopes_transfer_to_the_named_unit` and two
others) build the index as JSON by hand, with both specs owning the unit, so
they exercise `couple` given an index and never the index that 142 changed.
With the hand-off removed from `index.rs`, 018's plan still passes (1.4).

018's plan can only change through its holder: `verify` follows the
`amends_verification` chain (`018 -> 136`) and `compile` refuses a second
holder of one target. So this spec holds 136, carries 136's block unchanged for
the ten specs it holds, and adds 018's lines inside 018's section.

### 1.2 144's test count is loose

144's second line runs all of `tests/repo_path.rs` and accepts
`test result: ok. [1-9][0-9]* passed`. The file holds three tests, two of them
the link rule's. A build that drops the refusal and loses its test (deleted,
renamed, `#[ignore]`d or gated to another platform) still reports `2 passed`,
and the line passes (1.4).

### 1.3 145 drives one verb

145's binary line runs `index coverage` only. Seven verbs read the committed
index through `guard_committed_index` or the same partition, and each prints
the claim on 0.27.0 (exit 1, `I-004`, "not staleness"): `check`, `index check`,
`index coverage`, `index owner`, `couple`, `delta` and `scope evaluate`. 145's
library tests cover `coverage` and `owner`. A verb that reverted to the pre-145
read (the folded freshness verdict, "index is stale") at `couple`, `delta` or
`scope` passes 145's block (1.4).

### 1.4 Mutation evidence

Run 2026-09-25 on `baae17c9` in a scratch checkout, one mutation at a time,
each built with `cargo build --release --locked`; "old" is the plan `verify`
runs on `baae17c9`, "new" is this spec's block. The mutation diffs are in the
pull request.

| Mutation | Old plan | New plan |
|---|---|---|
| M1: `index.rs`, the hand-off's `*ownership = false` removed | 018: passed (22 commands) | 018: FAILED at command 23, the `relocation` hand-off tests |
| M2: `pathutil.rs`, the outside-link refusal disabled, and its test `a_link_leaving_the_repository_refuses_the_read` gated to `cfg(windows)` | 144: passed (5 commands) | 144: FAILED at command 27, the exact `repo_path` line |
| M3: `couple.rs`, the 145 guard replaced by the pre-145 folded read | 145: passed (3 commands) | 145: FAILED at command 34, the seven-verb line (`couple`: exit 1, `index is stale ... blocking-diagnostics by-spec/001-a.json`) |

Without a mutation, the new block passes for 151, 018, 144, 145 and 001 (34
commands each).

## 2. Territory

This spec establishes nothing. It edits 136, 144 and 145 only to add spec 082
§3.4's superseded-acceptance note above each block.

## 3. Behavior

### 3.1 018's section runs the hand-off

`spec-spine verify` for 136 and every spec 136 holds (018 among them) MUST run
this spec's block. It carries 136's block unchanged and adds, in 018's section,
142's two index-level hand-off tests by exact name, and a binary fixture in
which a partial `supersedes` of `src/y.rs` leaves `index owner src/y.rs` naming
the successor alone and `index owner src/x.rs` naming the predecessor alone.

### 3.2 144's line names its tests

`spec-spine verify 144` MUST run this spec's block. 144's commands are carried
unchanged except the `repo_path` line, which runs the file's three tests with
`--exact` and requires `3 passed`.

### 3.3 145's block drives every reading verb

`spec-spine verify 145` MUST run this spec's block. 145's commands are carried
unchanged, followed by one fixture (a complete spec claiming an absent file, in
a git repository of two commits) driven through the seven verbs of 1.3. Each
MUST exit 1, print `I-004`, the claimed path and "not staleness", and print
none of `index is stale`, `STALE` or `stale shard`. The `--json` forms are out
of scope here: `check --json` and `index check --json` still say
`fresh: false` and `stale shard`, which spec 152 changes.

### 3.4 The amended specs say so

136, 144 and 145 each carry spec 082 §3.4's superseded-acceptance note naming
this spec above their own block.

## 4. Out of scope

- The same loose count elsewhere: 21 spec files on `baae17c9` carry a
  `[1-9][0-9]* passed` assertion (144's, superseded here, 147's draft, and
  136's `--test compile` line, carried here unchanged). Tightening them is a
  corpus-wide pass, left to a later spec.
- Windows: 144's two link tests are `#[cfg(unix)]`; spec 148 owns the Windows
  cases.

## 5. Resolved decisions

**D-1 (2026-09-25): filed and built together.** A draft that declares
`amends_verification` changes the plans of the specs it holds as soon as it
merges, so its block must pass at the merge. The block needs no product code,
so this spec is filed with `implementation: complete`, as 146 was.

**D-2 (2026-09-25): hold 136, not 018.** 018 has no block of its own and 136
holds it; a second holder of 018 is a fork `compile` refuses. Holding 136
carries the other nine specs' lines byte-identical.

## Verification

```verify:cli
# Self-contained: the binary lines below use the release build.
cargo build --release --locked
# ---- carried for 136-the-legacy-ledger-is-paid-registry-and-grammar (amends_verification), and through it the ten specs it holds; 018's section gains 3.1 ----
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(001-compile-registry|002-registry-query|009-registry-query-projection-flags|012-declared-extra-frontmatter-passthrough|013-edge-paths-grammar-sugar|014-establishes-wrapper-na-alias|015-short-id-resolution|017-constrains-discriminator-optional-unit|018-structured-partial-supersedes|026-references-provenance-derived-at)" scripts/verify-sweep.sh'
# ---- carried for 001-compile-registry (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test conformance -- --exact emitted_registry_shards_conform_to_embedded_schema 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact compile_ok_then_queries 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 002-registry-query (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test query -- --exact list_filters_by_status load_registry_accepts_our_major load_registry_rejects_unknown_major relationships_show_incoming_and_outgoing show_finds_or_not_found status_report_counts 2>&1 | grep -q "test result: ok. 6 passed; 0 failed"'
# ---- carried for 009-registry-query-projection-flags (amends_verification) ----
# amended by 074: the projections name their read version, which the same tests assert.
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact registry_list_ids_only_projection registry_status_report_nonzero_only_projection 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 012-declared-extra-frontmatter-passthrough (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact declared_map_key_order_is_canonicalized declared_nested_extra_roundtrips_deterministically undeclared_nested_extra_keeps_pre013_guard v013_unrepresentable_declared_value 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-types --locked --test frontmatter -- --exact declared_key_carries_nested_yaml_verbatim 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 013-edge-paths-grammar-sugar (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact paths_sugar_grammar_violations_are_v002 paths_sugar_is_byte_equivalent_to_single_unit_items 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact extends_paths_sugar_expands_to_file_units refines_paths_sugar_expands_symmetrically 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 014-establishes-wrapper-na-alias (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact establishes_wrapper_and_na_alias_are_byte_equivalent 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact establishes_accepts_unit_wrapper implementation_na_slash_is_alias_for_na unit_wrapper_form_unwraps 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 015-short-id-resolution (amends_verification) ----
# amended by 067: an ambiguous ordinal is one refusal at every argument.
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact compile_spec_resolves_the_short_id_and_refuses_an_unknown_one dangling_short_id_is_left_unchanged_and_still_warns short_id_depends_on_resolves_to_full_id short_id_superseded_by_resolves 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test spec_id -- --exact the_whole_leading_segment_must_match 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test spec_id -- --exact an_ambiguous_ordinal_is_one_refusal_at_all_six_arguments 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 017-constrains-discriminator-optional-unit (amends_verification) ----
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact constrains_discriminator_and_optional_unit 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact constrains_scoped_forms_compile_clean v011_constrains_item_must_scope_unit_or_target_specs 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact spec_scoped_constrains_produces_no_resolved_unit 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 018-structured-partial-supersedes (amends_verification) ----
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact supersedes_full_and_partial_forms_parse 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact supersedes_full_emits_bare_string_partial_emits_object 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact partial_supersedes_scopes_transfer_to_the_named_unit partial_supersedes_without_unit_transfers_nothing supersedes_transfers_authority_additively 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.1 (151): 142 §3.1 amended 018 §4.3. The couple tests above build their index by hand,
# so they pass whatever the index does; these run the hand-off through `index`.
sh -c 'cargo test -p spec-spine-core --locked --test relocation -- --exact a_partial_supersedes_hands_the_unit_over a_retired_successor_hands_nothing_over 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 3.1 (151): through the shipped binary, the successor alone owns the handed-over unit.
sh -c 'T="${TMPDIR:-/tmp}/ss151-018"; B="$PWD/target/release/spec-spine"; rm -rf "$T" && mkdir -p "$T/specs/002-env" "$T/specs/008-y" "$T/src" && printf "pub fn x() {}\n" > "$T/src/x.rs" && printf "pub fn y() {}\n" > "$T/src/y.rs" && printf -- "---\nid: \"002-env\"\ntitle: \"Env\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/x.rs\"\n  - \"src/y.rs\"\n---\n# 002\n" > "$T/specs/002-env/spec.md" && printf -- "---\nid: \"008-y\"\ntitle: \"Y\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nsupersedes:\n  - { spec: \"002-env\", scope: partial, unit: \"src/y.rs\" }\n---\n# 008\n" > "$T/specs/008-y/spec.md" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 && "$B" --repo "$T" index owner src/y.rs > "$T.y" 2>&1 && "$B" --repo "$T" index owner src/x.rs > "$T.x" 2>&1; rc=$?; grep -q "008-y" "$T.y" && ! grep -q "002-env" "$T.y" && grep -q "002-env" "$T.x" && ! grep -q "008-y" "$T.x"; g=$?; test $g -eq 0 || cat "$T.y" "$T.x"; rm -rf "$T" "$T.y" "$T.x"; test $rc -eq 0 && test $g -eq 0'
# ---- carried for 026-references-provenance-derived-at (amends_verification) ----
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact references_provenance_derived_at_round_trips references_provenance_unknown_field_is_rejected references_provenance_without_derived_at_omits_the_field 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 144-a-repository-path-is-one-type (amends_verification); the repo_path line is exact (3.2) ----
# 3.1: the rule and the type.
sh -c 'cargo test -p spec-spine-types --locked --lib -- --exact repo_path::tests::plain_relative_paths_pass repo_path::tests::every_escape_and_windows_hazard_is_refused repo_path::tests::the_type_refuses_what_the_rule_refuses 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.2, 3.4: the layout roots and the link rule, each test named (151 3.2). Each
# fails with its rule removed (recorded in 144's PR). The two link tests are
# `#[cfg(unix)]`, so this line is for a Unix sweep host; spec 148 owns Windows.
sh -c 'cargo test -p spec-spine-core --locked --test repo_path -- --exact every_layout_root_follows_the_one_rule a_link_leaving_the_repository_refuses_the_read a_linked_root_and_the_derived_tree_are_not_this_rules_business 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# 3.3: plan paths in Windows forms.
sh -c 'cargo test -p spec-spine-core --locked --test retire -- --exact a_retired_path_in_a_windows_form_is_refused an_empty_historical_entry_is_refused 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 3.4: this repository holds no link that leaves it.
cargo build --release --locked
target/release/spec-spine check
# ---- carried for 145-an-unresolved-claim-is-not-called-stale (amends_verification); the last line is new (3.3) ----
# 3.1: the readers name the claim; drift stays staleness and comes first; a
# blocked shard that moved is stale; a resolved corpus reads. With the old
# folded guard restored, 3 of 4 fail (recorded in the PR).
sh -c 'cargo test -p spec-spine-core --locked --test unresolved_guard 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
# 3.1 and 3.2 at the binary: coverage names the claim and does not say stale.
cargo build --release --locked
sh -c 'T="${TMPDIR:-/tmp}/ss145"; rm -rf "$T" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/missing.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && B="$PWD/target/release/spec-spine" && "$B" --repo "$T" compile >/dev/null 2>&1; "$B" --repo "$T" index >/dev/null 2>&1; "$B" --repo "$T" index coverage > "$T.out" 2>&1; rc=$?; rm -rf "$T"; test $rc -eq 1 && grep -q "I-004 \[src/missing.rs\] unresolved claim, not staleness" "$T.out" && ! grep -q "index is stale" "$T.out"; r=$?; rm -f "$T.out"; exit $r'
# 3.1, 3.2 at every verb that reads the committed index (151 3.3): check, index
# check, index coverage, index owner, couple, delta and scope evaluate each exit
# 1 naming the claim, and none calls it stale. Their --json forms are spec 152's.
sh -c 'T="${TMPDIR:-/tmp}/ss151-145"; B="$PWD/target/release/spec-spine"; g() { git -C "$T" -c user.name=v -c user.email=v@v -c commit.gpgsign=false "$@"; }; rm -rf "$T" "$T.scope" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/missing.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && printf "{\"id\": \"W-1\", \"ownSpec\": \"001\", \"mutable\": [\"src/missing.rs\"]}\n" > "$T.scope" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 && g init -q && g add -A && g commit -qm base && printf "x\n" > "$T/notes.txt" && g add -A && g commit -qm head || exit 1; r=0; v() { "$B" --repo "$T" "$@" > "$T.out" 2>&1; rc=$?; { test $rc -eq 1 && grep -q "I-004" "$T.out" && grep -q "src/missing.rs" "$T.out" && grep -q "not staleness" "$T.out" && ! grep -qE "index is stale|STALE|stale shard" "$T.out"; } || { echo "$*: exit $rc"; cat "$T.out"; r=1; }; }; v check; v index check; v index coverage; v index owner src/missing.rs; v couple --base HEAD~1 --head HEAD; v delta --base HEAD~1 --head HEAD; v scope evaluate --scope "$T.scope"; rm -rf "$T" "$T.scope" "$T.out"; exit $r'
```
