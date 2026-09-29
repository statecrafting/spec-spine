---
id: "179-exact-test-counts-for-137"
title: "Exact test counts for 137"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 137's acceptance block carries three lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 137's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "137-the-legacy-ledger-is-paid-index-and-lint"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "137-the-legacy-ledger-is-paid-index-and-lint"
amends:
  - "137-the-legacy-ledger-is-paid-index-and-lint"
---

# 179: Exact test counts for 137

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 137's. 137 holds 003, 004, 010, 011, 016, 020, 022, 023, 024 and 025, so their plans resolve here too, and none has a block of its own to carry a note.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 137:5 | core `lint` | whole | 39 |
| 137:7 | core `deferred_contract` | whole | 5 |
| 137:10 | core `index` | whole | 57 |

"Block line" counts lines inside 137's `verify:cli` fence. The names each
line ran:

- `lint` (whole target): `a_claim_inside_the_state_root_defers_to_l006`, `a_claim_on_a_file_that_does_not_exist_is_silent`, `a_covering_glob_witnesses_the_claim_and_the_directory_form_does_not`, `a_forward_dependency_does_not_fail_compile`, `a_forward_dependency_is_an_error_and_a_backward_one_is_silent`, `a_frontmatter_comment_is_not_the_title_heading`, `a_heading_that_continues_past_the_anchor_is_not_the_section`, `a_self_dependency_is_caught_by_the_equality_arm`, `a_title_heading_naming_another_spec_is_reported`, `a_title_heading_naming_its_own_spec_is_silent`, `an_id_or_a_heading_without_an_ordinal_is_silent`, `an_id_without_an_ordinal_is_silence_in_either_position`, `an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies`, `l009_is_info_tier_and_names_the_anchor`, `l010_does_not_fire_on_a_pattern_that_merely_matches_nothing_yet`, `l010_is_silent_on_the_working_glob_form`, `l010_is_silent_on_the_working_slice_form`, `l010_refuses_a_hashed_input_pattern_that_can_match_no_file`, `l010_refuses_a_slice_pattern_that_can_match_no_file`, `l010_reports_both_tables_and_each_form_names_its_own_table`, `l011_is_silent_while_the_work_is_genuinely_in_flight`, `l011_refuses_a_completed_spec_that_still_plans_territory`, `l012_is_silent_while_the_planned_unit_is_still_unwritten`, `l012_reports_a_planned_unit_that_has_resolved`, `retroactive_origin_alone_produces_no_diagnostic`, `the_allowlist_suppresses_the_warning_but_not_the_count`, `the_comparison_is_numeric_not_lexical`, `the_defects_heading_is_matched_by_anchor_at_any_level_or_numbering`, `the_knob_defaults_off_and_off_emits_nothing`, `the_l010_message_names_the_working_form`, `the_near_miss_rule_catches_attempts_and_not_prose`, `the_planned_refusals_are_silent_on_a_corpus_that_plans_nothing`, `the_slice_l010_message_does_not_claim_a_content_hash`, `the_slice_l010_message_names_the_table_the_slice_and_the_working_form`, `v015_allows_a_shared_plan_declared_as_co_authority`, `v015_refuses_two_specs_planning_the_same_unit`, `v016_refuses_planning_territory_another_spec_already_owns`, `v017_refuses_extending_a_unit_that_is_only_planned`, `witnessed_paths_covers_every_hash_contributor`.
- `deferred_contract` (whole target): `a_deferred_spec_that_names_a_unit_is_still_diagnosed`, `a_deferred_spec_with_no_territory_is_exempt_from_l001`, `an_n_a_spec_with_no_territory_still_raises_l001`, `scheduling_a_deferred_spec_re_arms_l001`, `the_exemption_moves_no_other_diagnostic`.
- `index` (whole target): `a_comment_or_reformat_only_workflow_edit_leaves_the_projection_unchanged`, `a_complete_draft_that_told_the_truth_is_silent`, `a_hash_header_claims_in_a_shell_script`, `a_header_on_line_16_claims_and_on_line_17_does_not`, `a_non_owning_reference_stays_w002_in_every_combination`, `a_planned_unit_before_completion_still_produces_nothing`, `a_planned_unit_on_a_complete_spec_is_an_unresolved_claim`, `a_planned_unit_produces_no_diagnostic_and_an_unmarked_one_still_does`, `a_planned_unit_that_resolves_is_owned_like_any_other`, `a_superseding_spec_is_reported_as_inherited`, `a_uses_ref_bump_leaves_the_workflow_projection_unchanged`, `a_workflow_bump_leaves_every_shard_hash_alone_and_a_run_edit_does_not`, `ac1_unresolved_reference_is_w002_warning_not_error`, `ac2_draft_owning_unit_is_w001_warning_not_error`, `ac3_pending_owning_unit_is_w001_warning_not_error`, `ac4_settled_owning_unit_still_errors_i004`, `ac5_reference_on_draft_spec_is_w002_edge_type_precedence`, `an_inner_doc_comment_header_does_not_claim`, `an_unmarked_claim_keeps_its_classification`, `an_unparseable_workflow_falls_back_to_raw_bytes`, `an_unresolvable_header_stops_the_scan_and_shadows_a_valid_one`, `authorities_resolves_owners`, `changing_which_action_runs_changes_the_projection`, `completion_defeats_draft_leniency_across_both_axes`, `conforms_to_embedded_schema`, `crate_unit_resolves_to_package_subtree`, `directory_unit_resolves_to_subtree`, `discovers_rust_and_npm_packages`, `emitted_index_shards_conform_to_embedded_schema`, `every_loose_form_in_the_recognizer_table_still_claims`, `every_other_workflow_edit_changes_the_projection`, `foreign_yaml_section_unit_resolves_no_i006`, `in_progress_leniency_reports_rather_than_ignores`, `indexes_deterministically`, `missing_directory_unit_is_blocking_diagnostic_i007`, `missing_file_unit_is_blocking_diagnostic_i004`, `module_unit_resolves_inline_and_file_modules`, `nested_pnpm_workspace_discovers_members`, `owner_of_an_unowned_or_absent_path_is_empty`, `owner_reports_exactly_the_gates_id_set`, `owner_separates_unit_floor_and_header_linkage`, `planned_does_not_soften_any_other_classification`, `references_does_not_confer_c001_ownership`, `references_unit_does_not_seed_implementing_paths`, `references_unit_survives_in_resolved_units`, `resolves_rust_symbols_with_exact_spans`, `resolves_section_unit`, `resolves_typescript_symbols_with_exact_spans`, `spec_scoped_constrains_produces_no_resolved_unit`, `staleness_detects_input_change`, `staleness_detects_symbol_source_line_shift`, `statecraft_derived_and_state_are_pruned_and_the_rest_is_governed`, `statecraft_derived_matching_is_separator_aware`, `the_shipped_default_hashes_the_files_it_names`, `unknown_crate_unit_is_blocking_diagnostic_i003`, `unpinning_an_action_changes_the_projection`, `unresolved_module_unit_is_blocking_diagnostic_i008`.

## 2. Territory

This spec establishes nothing. It edits 137 only to add spec 082 §3.4's
superseded-acceptance note above 137's block.

## 3. Behavior

### 3.1 137's block is carried

`spec-spine verify 137` and `verify` of 003, 004, 010, 011, 016, 020, 022, 023, 024 and 025 MUST run this
spec's block, which carries 137's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

137 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 003, 004, 010, 011, 016, 020, 022, 023, 024 and 025 have no block of their own, so no note.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 137's plan and that of 003, 004, 010, 011, 016, 020, 022, 023, 024 and 025 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 137-the-legacy-ledger-is-paid-index-and-lint (amends_verification), and through it 003, 004, 010, 011, 016, 020, 022, 023, 024, 025; each loose test count names its tests (153 3.2) ----
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(003-conformance-lint|004-codebase-index|010-index-render-orphans|011-index-hash-slices|016-directory-crate-module-units|020-keypath-section-anchors|022-index-sharding|023-unresolved-unit-severity|024-resolution-discovery-fixes|025-symbol-resolution-feature-gate)" scripts/verify-sweep.sh'
# ---- carried for 003-conformance-lint (amends_verification) ----
# amended by 116: the deferred-contract exemption from L-001 is part of what lint does now.
sh -c 'cargo test -p spec-spine-core --locked --test lint -- --exact a_claim_inside_the_state_root_defers_to_l006 a_claim_on_a_file_that_does_not_exist_is_silent a_covering_glob_witnesses_the_claim_and_the_directory_form_does_not a_forward_dependency_does_not_fail_compile a_forward_dependency_is_an_error_and_a_backward_one_is_silent a_frontmatter_comment_is_not_the_title_heading a_heading_that_continues_past_the_anchor_is_not_the_section a_self_dependency_is_caught_by_the_equality_arm a_title_heading_naming_another_spec_is_reported a_title_heading_naming_its_own_spec_is_silent an_id_or_a_heading_without_an_ordinal_is_silent an_id_without_an_ordinal_is_silence_in_either_position an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies l009_is_info_tier_and_names_the_anchor l010_does_not_fire_on_a_pattern_that_merely_matches_nothing_yet l010_is_silent_on_the_working_glob_form l010_is_silent_on_the_working_slice_form l010_refuses_a_hashed_input_pattern_that_can_match_no_file l010_refuses_a_slice_pattern_that_can_match_no_file l010_reports_both_tables_and_each_form_names_its_own_table l011_is_silent_while_the_work_is_genuinely_in_flight l011_refuses_a_completed_spec_that_still_plans_territory l012_is_silent_while_the_planned_unit_is_still_unwritten l012_reports_a_planned_unit_that_has_resolved retroactive_origin_alone_produces_no_diagnostic the_allowlist_suppresses_the_warning_but_not_the_count the_comparison_is_numeric_not_lexical the_defects_heading_is_matched_by_anchor_at_any_level_or_numbering the_knob_defaults_off_and_off_emits_nothing the_l010_message_names_the_working_form the_near_miss_rule_catches_attempts_and_not_prose the_planned_refusals_are_silent_on_a_corpus_that_plans_nothing the_slice_l010_message_does_not_claim_a_content_hash the_slice_l010_message_names_the_table_the_slice_and_the_working_form v015_allows_a_shared_plan_declared_as_co_authority v015_refuses_two_specs_planning_the_same_unit v016_refuses_planning_territory_another_spec_already_owns v017_refuses_extending_a_unit_that_is_only_planned witnessed_paths_covers_every_hash_contributor 2>&1 | grep -q "test result: ok. 39 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test coverage -- --exact lint_diagnostic_codes_are_unique 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test deferred_contract -- --exact a_deferred_spec_that_names_a_unit_is_still_diagnosed a_deferred_spec_with_no_territory_is_exempt_from_l001 an_n_a_spec_with_no_territory_still_raises_l001 scheduling_a_deferred_spec_re_arms_l001 the_exemption_moves_no_other_diagnostic 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
# ---- carried for 004-codebase-index (amends_verification) ----
# amended by 023, 031, 038, 041, 069 and 079 among others: the whole index suite at HEAD is the amended behavior.
# exact on a Unix sweep host with default features: index has tests behind the default `symbol-resolution` feature (spec 153 3.3).
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact a_comment_or_reformat_only_workflow_edit_leaves_the_projection_unchanged a_complete_draft_that_told_the_truth_is_silent a_hash_header_claims_in_a_shell_script a_header_on_line_16_claims_and_on_line_17_does_not a_non_owning_reference_stays_w002_in_every_combination a_planned_unit_before_completion_still_produces_nothing a_planned_unit_on_a_complete_spec_is_an_unresolved_claim a_planned_unit_produces_no_diagnostic_and_an_unmarked_one_still_does a_planned_unit_that_resolves_is_owned_like_any_other a_superseding_spec_is_reported_as_inherited a_uses_ref_bump_leaves_the_workflow_projection_unchanged a_workflow_bump_leaves_every_shard_hash_alone_and_a_run_edit_does_not ac1_unresolved_reference_is_w002_warning_not_error ac2_draft_owning_unit_is_w001_warning_not_error ac3_pending_owning_unit_is_w001_warning_not_error ac4_settled_owning_unit_still_errors_i004 ac5_reference_on_draft_spec_is_w002_edge_type_precedence an_inner_doc_comment_header_does_not_claim an_unmarked_claim_keeps_its_classification an_unparseable_workflow_falls_back_to_raw_bytes an_unresolvable_header_stops_the_scan_and_shadows_a_valid_one authorities_resolves_owners changing_which_action_runs_changes_the_projection completion_defeats_draft_leniency_across_both_axes conforms_to_embedded_schema crate_unit_resolves_to_package_subtree directory_unit_resolves_to_subtree discovers_rust_and_npm_packages emitted_index_shards_conform_to_embedded_schema every_loose_form_in_the_recognizer_table_still_claims every_other_workflow_edit_changes_the_projection foreign_yaml_section_unit_resolves_no_i006 in_progress_leniency_reports_rather_than_ignores indexes_deterministically missing_directory_unit_is_blocking_diagnostic_i007 missing_file_unit_is_blocking_diagnostic_i004 module_unit_resolves_inline_and_file_modules nested_pnpm_workspace_discovers_members owner_of_an_unowned_or_absent_path_is_empty owner_reports_exactly_the_gates_id_set owner_separates_unit_floor_and_header_linkage planned_does_not_soften_any_other_classification references_does_not_confer_c001_ownership references_unit_does_not_seed_implementing_paths references_unit_survives_in_resolved_units resolves_rust_symbols_with_exact_spans resolves_section_unit resolves_typescript_symbols_with_exact_spans spec_scoped_constrains_produces_no_resolved_unit staleness_detects_input_change staleness_detects_symbol_source_line_shift statecraft_derived_and_state_are_pruned_and_the_rest_is_governed statecraft_derived_matching_is_separator_aware the_shipped_default_hashes_the_files_it_names unknown_crate_unit_is_blocking_diagnostic_i003 unpinning_an_action_changes_the_projection unresolved_module_unit_is_blocking_diagnostic_i008 2>&1 | grep -q "test result: ok. 57 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact index_then_check_fresh_then_stale 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 010-index-render-orphans (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact index_render_and_orphans_projections 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test render -- --exact orphans_partitions_by_the_in_flight_predicate render_omits_empty_sections 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 011-index-hash-slices (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact index_slice_hashes_and_check 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test lint -- --exact l010_refuses_a_slice_pattern_that_can_match_no_file 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test config_facade -- --exact a_malformed_slice_is_refused_at_every_entry 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 016-directory-crate-module-units (amends_verification) ----
# amended by 025: module resolution is feature-gated; the default-features run carries it.
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact directory_crate_module_units_parse directory_subtree_detection 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact unresolved_module_unit_is_blocking_diagnostic_i008 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 020-keypath-section-anchors (amends_verification) ----
# amended by 024: a foreign-YAML section unit resolves.
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact foreign_yaml_section_unit_resolves_no_i006 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --lib -- --exact sections::tests::cargo_toml_table_keypaths sections::tests::eligibility_boundary_is_hard sections::tests::package_json_member_keypaths sections::tests::workflow_bounds_reject_index_and_overdepth sections::tests::workflow_keypaths_and_back_compat 2>&1 | grep -q "test result: ok. 5 passed; 0 failed"'
# ---- carried for 022-index-sharding (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact emitted_index_shards_conform_to_embedded_schema 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test conformance -- --exact emitted_registry_shards_conform_to_embedded_schema 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-types --locked --test dtos -- --exact schema_versions_are_pinned 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact index_then_check_fresh_then_stale 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 023-unresolved-unit-severity (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact ac1_unresolved_reference_is_w002_warning_not_error ac2_draft_owning_unit_is_w001_warning_not_error ac3_pending_owning_unit_is_w001_warning_not_error ac4_settled_owning_unit_still_errors_i004 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
# ---- carried for 024-resolution-discovery-fixes (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact foreign_yaml_section_unit_resolves_no_i006 nested_pnpm_workspace_discovers_members 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --lib -- --exact sections::tests::makefile_multiple_pending_tags_are_not_lost sections::tests::makefile_targets_and_tags 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 025-symbol-resolution-feature-gate (amends_verification) ----
sh -c 'cargo tree -p spec-spine-core -e normal --locked | grep -q tree-sitter'
sh -c '! cargo tree -p spec-spine-core --no-default-features -e normal --locked | grep -q tree-sitter'
cargo check -p spec-spine-core --no-default-features --locked --quiet
```
