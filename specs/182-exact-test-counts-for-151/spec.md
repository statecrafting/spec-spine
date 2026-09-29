---
id: "182-exact-test-counts-for-151"
title: "Exact test counts for 151"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 151's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 151's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "151-carried-acceptance-tests-what-it-names"
amends:
  - "151-carried-acceptance-tests-what-it-names"
---

# 182: Exact test counts for 151

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 151's. 151 holds 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144 and 145, so their plans resolve here too, and their superseded notes name this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 151:7 | core `compile` | whole | 51 |

"Block line" counts lines inside 151's `verify:cli` fence. The names each
line ran:

- `compile` (whole target): `a_cycle_written_with_short_ids_is_still_a_cycle`, `a_dangling_dependency_is_reported_once`, `a_dangling_dependency_is_v010_and_never_a_cycle`, `a_diamond_is_not_a_cycle`, `added_spec_without_recompiling_reads_as_missing`, `compile_spec_reports_a_duplicate_ordinal_and_names_the_holder`, `compile_spec_reports_only_the_named_specs_violations`, `compile_spec_resolves_the_short_id_and_refuses_an_unknown_one`, `compile_spec_validates_an_uncommitted_draft_and_writes_nothing`, `compile_spec_works_before_the_registry_exists`, `compiles_clean_corpus_deterministically`, `constrains_scoped_forms_compile_clean`, `content_hash_changes_with_content_but_is_stable_otherwise`, `cycle_reached_through_a_longer_chain_names_only_the_loop`, `dangling_depends_on_stays_warning_tier`, `dangling_short_id_is_left_unchanged_and_still_warns`, `declared_map_key_order_is_canonicalized`, `declared_nested_extra_roundtrips_deterministically`, `edited_spec_without_recompiling_reads_as_modified`, `establishes_wrapper_and_na_alias_are_byte_equivalent`, `extra_frontmatter_is_copied_into_the_registry`, `fail_on_warn_writes_identical_shards`, `freshness_check_passes_on_a_just_compiled_tree_and_writes_nothing`, `freshness_report_caps_the_named_shards`, `oap_dialect_refines_fixture_compiles_clean`, `paths_sugar_grammar_violations_are_v002`, `paths_sugar_is_byte_equivalent_to_single_unit_items`, `registry_freshness_facade_reports_both_verdicts`, `removed_spec_leaves_an_orphaned_shard`, `self_dependency_is_a_cycle`, `short_id_depends_on_resolves_to_full_id`, `short_id_superseded_by_resolves`, `supersedes_full_emits_bare_string_partial_emits_object`, `the_cycle_is_not_stored_in_a_shard_but_is_recomputed_on_read`, `the_reported_cycle_is_deterministic_across_runs`, `two_spec_dependency_cycle_is_an_error_naming_the_path`, `unbuilt_registry_is_stale_not_an_error`, `undeclared_nested_extra_keeps_pre013_guard`, `v001_directory_must_equal_id`, `v002_malformed_frontmatter_is_recorded_not_fatal`, `v003_duplicate_id`, `v004_duplicate_prefix`, `v004_ignores_a_multibyte_prefix_that_lands_on_a_boundary`, `v004_ignores_ids_with_no_numeric_prefix`, `v004_reads_a_non_ascii_id_without_panicking`, `v005_domain_allowlist_when_enabled`, `v007_cap_unchanged_in_presence_of_declared_keys`, `v007_extra_frontmatter_count_cap_with_exemption`, `v008_superseded_requires_resolvable_superseded_by`, `v011_constrains_item_must_scope_unit_or_target_specs`, `v013_unrepresentable_declared_value`.

## 2. Territory

This spec establishes nothing. It edits 151 only to add spec 082 §3.4's
superseded-acceptance note above 151's block, and the notes of 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144 and 145 to name this spec.

## 3. Behavior

### 3.1 151's block is carried

`spec-spine verify 151` and `verify` of 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144 and 145 MUST run this
spec's block, which carries 151's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

151 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144 and 145 keep their notes and gain one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 151's plan and that of 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144 and 145 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 151-carried-acceptance-tests-what-it-names (amends_verification), and through it 001, 002, 009, 012, 013, 014, 015, 017, 018, 026, 136, 144, 145; each loose test count names its tests (153 3.2) ----
# Self-contained: the binary lines below use the release build.
cargo build --release --locked
# ---- carried for 136-the-legacy-ledger-is-paid-registry-and-grammar (amends_verification), and through it the ten specs it holds; 018's section gains 3.1 ----
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(001-compile-registry|002-registry-query|009-registry-query-projection-flags|012-declared-extra-frontmatter-passthrough|013-edge-paths-grammar-sugar|014-establishes-wrapper-na-alias|015-short-id-resolution|017-constrains-discriminator-optional-unit|018-structured-partial-supersedes|026-references-provenance-derived-at)" scripts/verify-sweep.sh'
# ---- carried for 001-compile-registry (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact a_cycle_written_with_short_ids_is_still_a_cycle a_dangling_dependency_is_reported_once a_dangling_dependency_is_v010_and_never_a_cycle a_diamond_is_not_a_cycle added_spec_without_recompiling_reads_as_missing compile_spec_reports_a_duplicate_ordinal_and_names_the_holder compile_spec_reports_only_the_named_specs_violations compile_spec_resolves_the_short_id_and_refuses_an_unknown_one compile_spec_validates_an_uncommitted_draft_and_writes_nothing compile_spec_works_before_the_registry_exists compiles_clean_corpus_deterministically constrains_scoped_forms_compile_clean content_hash_changes_with_content_but_is_stable_otherwise cycle_reached_through_a_longer_chain_names_only_the_loop dangling_depends_on_stays_warning_tier dangling_short_id_is_left_unchanged_and_still_warns declared_map_key_order_is_canonicalized declared_nested_extra_roundtrips_deterministically edited_spec_without_recompiling_reads_as_modified establishes_wrapper_and_na_alias_are_byte_equivalent extra_frontmatter_is_copied_into_the_registry fail_on_warn_writes_identical_shards freshness_check_passes_on_a_just_compiled_tree_and_writes_nothing freshness_report_caps_the_named_shards oap_dialect_refines_fixture_compiles_clean paths_sugar_grammar_violations_are_v002 paths_sugar_is_byte_equivalent_to_single_unit_items registry_freshness_facade_reports_both_verdicts removed_spec_leaves_an_orphaned_shard self_dependency_is_a_cycle short_id_depends_on_resolves_to_full_id short_id_superseded_by_resolves supersedes_full_emits_bare_string_partial_emits_object the_cycle_is_not_stored_in_a_shard_but_is_recomputed_on_read the_reported_cycle_is_deterministic_across_runs two_spec_dependency_cycle_is_an_error_naming_the_path unbuilt_registry_is_stale_not_an_error undeclared_nested_extra_keeps_pre013_guard v001_directory_must_equal_id v002_malformed_frontmatter_is_recorded_not_fatal v003_duplicate_id v004_duplicate_prefix v004_ignores_a_multibyte_prefix_that_lands_on_a_boundary v004_ignores_ids_with_no_numeric_prefix v004_reads_a_non_ascii_id_without_panicking v005_domain_allowlist_when_enabled v007_cap_unchanged_in_presence_of_declared_keys v007_extra_frontmatter_count_cap_with_exemption v008_superseded_requires_resolvable_superseded_by v011_constrains_item_must_scope_unit_or_target_specs v013_unrepresentable_declared_value 2>&1 | grep -q "test result: ok. 51 passed; 0 failed"'
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
