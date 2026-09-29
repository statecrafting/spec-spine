---
id: "180-exact-test-counts-for-138"
title: "Exact test counts for 138"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 138's acceptance block carries two lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 138's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
amends:
  - "138-the-legacy-ledger-is-paid-coupling-and-freshness"
---

# 180: Exact test counts for 138

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 138's. 138 holds 005, 008, 027, 028, 029, 030, 031, 033, 035 and 036, so their plans resolve here too, and none has a block of its own to carry a note.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 138:5 | core `couple` | whole | 53 |
| 138:18 | core `coverage` | whole | 30 |

"Block line" counts lines inside 138's `verify:cli` fence. The names each
line ran:

- `couple` (whole target): `a_declared_state_root_is_bypassed_and_a_claim_cannot_override_it`, `a_regenerated_shard_at_the_configured_derived_root_is_not_drift`, `amends_expands_owners_when_base_set_nonempty`, `amends_strict_guard_never_enrols_unowned_spec_md`, `bypass_floor_and_additive_config`, `bypassed_paths_never_raise_c002`, `c001_carries_its_owners_as_data`, `c002_carries_no_owners`, `claim_overrides_adopter_bypass_for_exactly_the_claimed_file`, `custom_specs_dir_clears_drift_via_the_owning_spec`, `custom_specs_dir_keeps_amends_awareness`, `default_specs_dir_does_not_clear_under_a_custom_root`, `deleted_path_absent_from_a_read_snapshot_has_no_owner`, `deleted_path_explicit_claim_overrides_bypass_at_the_prior_snapshot`, `deleted_path_floor_only_still_refuses`, `deleted_path_resolves_owners_at_the_merge_base`, `deleted_path_supersedes_transfer_applies_at_the_prior_snapshot`, `deleted_path_with_surviving_co_owner_still_refuses`, `deleted_paths_are_never_c002`, `deletion_reports_which_snapshot_answered`, `directory_form_claim_overrides_for_the_subtree`, `empty_owners_is_omitted_from_json`, `explicit_claim_overrides_the_floor`, `explicit_claim_under_bypass_is_c001_not_c002`, `file_drift_then_clearance`, `floor_only_path_is_c002_naming_the_floor`, `floor_only_path_without_the_ratchet_is_plain_c001`, `governed_scope::a_bypassed_path_stays_bypassed_unless_a_unit_claims_it`, `governed_scope::an_unclaimed_governed_scope_file_is_c002_and_a_claimed_one_is_not`, `governed_scope::without_a_scope_the_script_is_not_asked_about`, `header_claimed_path_is_specific`, `implicit_ownership_does_not_override_bypass`, `is_bypassed_path_is_claim_aware`, `legacy_couple_with_is_byte_identical_and_reports_head_tree`, `modifications_and_additions_are_unaffected_by_a_prior_snapshot`, `one_diff_can_carry_both_codes_but_one_path_carries_one`, `partial_supersedes_scopes_transfer_to_the_named_unit`, `partial_supersedes_without_unit_transfers_nothing`, `paths_outside_the_universe_never_raise_c002`, `ratchet_off_skips_an_unowned_path`, `ratchet_on_refuses_an_unowned_path`, `references_never_satisfy_the_ratchet`, `section_claim_under_floor_is_evaluated_with_span_semantics`, `section_granularity_distinguishes_owners`, `supersedes_transfers_authority_additively`, `symbol_granularity_drift_detection`, `the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym`, `the_effective_bypass_list_reports_the_configured_derived_root`, `the_ownership_ratchet_does_not_reach_into_the_state_root`, `unit_claimed_path_is_c001_not_c002`, `waiver_clears_c002_exactly_as_c001`, `waiver_suppresses_exit_but_retains_violations`, `whole_file_floor_via_implementing_path`.
- `coverage` (whole target): `a_claim_inside_the_state_root_is_an_l006_error`, `a_declared_state_root_leaves_both_sides_of_the_coverage_ratio`, `a_populated_universe_is_not_empty`, `a_resolver_exclusion_and_a_bypass_stay_out`, `a_scope_adds_files_outside_packages_and_outside_source_exts`, `a_set_scope_matching_nothing_keeps_both_members`, `absent_and_supplied_empty_inventories_are_different_answers`, `an_empty_scope_is_the_report_spec_032_emits`, `an_empty_universe_with_no_packages_is_reported_with_its_cause`, `classifier_tiers_directly`, `coverage_is_freshness_guarded`, `coverage_lists_only_planned_territory_that_is_not_yet_written`, `coverage_reports_each_near_miss_reason`, `coverage_reports_planned_territory_separately_from_the_counts`, `crate_unit_claims_the_whole_package`, `exclusions_remove_only_what_the_scope_added`, `facade_round_trips_the_report`, `l006_covers_every_ownership_bearing_edge`, `lint_diagnostic_codes_are_unique`, `near_misses_change_no_classification`, `packages_without_source_files_point_at_resolver_exclusions`, `report_classifies_every_tier`, `report_is_deterministic_and_a_function_of_the_path_set`, `require_ownership_defaults_off_and_parses`, `root_package_attributes_nested_files_to_the_nested_package`, `the_report_window_ends_at_line_64`, `the_resolver_does_not_reach_into_the_state_root`, `the_walk_skips_git_and_the_state_root`, `universe_excludes_noise_and_is_the_gates_predicate`, `writing_state_does_not_stale_the_committed_index`.

## 2. Territory

This spec establishes nothing. It edits 138 only to add spec 082 §3.4's
superseded-acceptance note above 138's block.

## 3. Behavior

### 3.1 138's block is carried

`spec-spine verify 138` and `verify` of 005, 008, 027, 028, 029, 030, 031, 033, 035 and 036 MUST run this
spec's block, which carries 138's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

138 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 005, 008, 027, 028, 029, 030, 031, 033, 035 and 036 have no block of their own, so no note.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 138's plan and that of 005, 008, 027, 028, 029, 030, 031, 033, 035 and 036 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 138-the-legacy-ledger-is-paid-coupling-and-freshness (amends_verification), and through it 005, 008, 027, 028, 029, 030, 031, 033, 035, 036; each loose test count names its tests (153 3.2) ----
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(005-coupling-gate|008-coupling-floor-claim-precedence|027-cargo-workflow-dependency-waiver|028-registry-freshness-check|029-ownership-coverage|030-dependency-cycle-refusal|031-references-non-owning-paths|033-configured-corpus-root|035-registry-plan-ready-set|036-declared-state-dir)" scripts/verify-sweep.sh'
# ---- carried for 005-coupling-gate (amends_verification) ----
# amended by 008, 027, 033, 071 and 100 among others: the whole coupling suite at HEAD is the amended behavior.
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact a_declared_state_root_is_bypassed_and_a_claim_cannot_override_it a_regenerated_shard_at_the_configured_derived_root_is_not_drift amends_expands_owners_when_base_set_nonempty amends_strict_guard_never_enrols_unowned_spec_md bypass_floor_and_additive_config bypassed_paths_never_raise_c002 c001_carries_its_owners_as_data c002_carries_no_owners claim_overrides_adopter_bypass_for_exactly_the_claimed_file custom_specs_dir_clears_drift_via_the_owning_spec custom_specs_dir_keeps_amends_awareness default_specs_dir_does_not_clear_under_a_custom_root deleted_path_absent_from_a_read_snapshot_has_no_owner deleted_path_explicit_claim_overrides_bypass_at_the_prior_snapshot deleted_path_floor_only_still_refuses deleted_path_resolves_owners_at_the_merge_base deleted_path_supersedes_transfer_applies_at_the_prior_snapshot deleted_path_with_surviving_co_owner_still_refuses deleted_paths_are_never_c002 deletion_reports_which_snapshot_answered directory_form_claim_overrides_for_the_subtree empty_owners_is_omitted_from_json explicit_claim_overrides_the_floor explicit_claim_under_bypass_is_c001_not_c002 file_drift_then_clearance floor_only_path_is_c002_naming_the_floor floor_only_path_without_the_ratchet_is_plain_c001 governed_scope::a_bypassed_path_stays_bypassed_unless_a_unit_claims_it governed_scope::an_unclaimed_governed_scope_file_is_c002_and_a_claimed_one_is_not governed_scope::without_a_scope_the_script_is_not_asked_about header_claimed_path_is_specific implicit_ownership_does_not_override_bypass is_bypassed_path_is_claim_aware legacy_couple_with_is_byte_identical_and_reports_head_tree modifications_and_additions_are_unaffected_by_a_prior_snapshot one_diff_can_carry_both_codes_but_one_path_carries_one partial_supersedes_scopes_transfer_to_the_named_unit partial_supersedes_without_unit_transfers_nothing paths_outside_the_universe_never_raise_c002 ratchet_off_skips_an_unowned_path ratchet_on_refuses_an_unowned_path references_never_satisfy_the_ratchet section_claim_under_floor_is_evaluated_with_span_semantics section_granularity_distinguishes_owners supersedes_transfers_authority_additively symbol_granularity_drift_detection the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym the_effective_bypass_list_reports_the_configured_derived_root the_ownership_ratchet_does_not_reach_into_the_state_root unit_claimed_path_is_c001_not_c002 waiver_clears_c002_exactly_as_c001 waiver_suppresses_exit_but_retains_violations whole_file_floor_via_implementing_path 2>&1 | grep -q "test result: ok. 53 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test couple -- --exact real_git_diff_detects_drift 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 008-coupling-floor-claim-precedence (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact claim_overrides_adopter_bypass_for_exactly_the_claimed_file explicit_claim_overrides_the_floor implicit_ownership_does_not_override_bypass is_bypassed_path_is_claim_aware 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test couple -- --exact claimed_floor_path_refuses_the_auto_waiver 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 027-cargo-workflow-dependency-waiver (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test couple -- --exact cargo_dependency_bump_stays_fresh_and_auto_waives cargo_feature_edit_without_bump_still_drifts claimed_floor_path_refuses_the_auto_waiver dependency_bump_stays_fresh_and_auto_waives 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact changing_which_action_runs_changes_the_projection 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 028-registry-freshness-check (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact check_never_writes_even_when_the_tree_is_stale compile_check_exit_contract 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact edited_spec_without_recompiling_reads_as_modified freshness_check_passes_on_a_just_compiled_tree_and_writes_nothing unbuilt_registry_is_stale_not_an_error 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 029-ownership-coverage (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact index_coverage_reports_and_gates 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# exact on a Unix sweep host with default features: coverage has a test behind the default `symbol-resolution` feature (spec 153 3.3).
sh -c 'cargo test -p spec-spine-core --locked --test coverage -- --exact a_claim_inside_the_state_root_is_an_l006_error a_declared_state_root_leaves_both_sides_of_the_coverage_ratio a_populated_universe_is_not_empty a_resolver_exclusion_and_a_bypass_stay_out a_scope_adds_files_outside_packages_and_outside_source_exts a_set_scope_matching_nothing_keeps_both_members absent_and_supplied_empty_inventories_are_different_answers an_empty_scope_is_the_report_spec_032_emits an_empty_universe_with_no_packages_is_reported_with_its_cause classifier_tiers_directly coverage_is_freshness_guarded coverage_lists_only_planned_territory_that_is_not_yet_written coverage_reports_each_near_miss_reason coverage_reports_planned_territory_separately_from_the_counts crate_unit_claims_the_whole_package exclusions_remove_only_what_the_scope_added facade_round_trips_the_report l006_covers_every_ownership_bearing_edge lint_diagnostic_codes_are_unique near_misses_change_no_classification packages_without_source_files_point_at_resolver_exclusions report_classifies_every_tier report_is_deterministic_and_a_function_of_the_path_set require_ownership_defaults_off_and_parses root_package_attributes_nested_files_to_the_nested_package the_report_window_ends_at_line_64 the_resolver_does_not_reach_into_the_state_root the_walk_skips_git_and_the_state_root universe_excludes_noise_and_is_the_gates_predicate writing_state_does_not_stale_the_committed_index 2>&1 | grep -q "test result: ok. 30 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact owner_separates_unit_floor_and_header_linkage 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 030-dependency-cycle-refusal (amends_verification) ----
# amended by 035: plan and compile agree on a cycle.
sh -c 'cargo test -p spec-spine-core --locked --test compile -- --exact a_cycle_written_with_short_ids_is_still_a_cycle a_dangling_dependency_is_v010_and_never_a_cycle a_diamond_is_not_a_cycle cycle_reached_through_a_longer_chain_names_only_the_loop self_dependency_is_a_cycle the_cycle_is_not_stored_in_a_shard_but_is_recomputed_on_read the_reported_cycle_is_deterministic_across_runs two_spec_dependency_cycle_is_an_error_naming_the_path 2>&1 | grep -q "test result: ok. 8 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test query -- --exact plan_and_compile_agree_on_a_cycle_among_retired_specs 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 031-references-non-owning-paths (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact references_does_not_confer_c001_ownership references_unit_does_not_seed_implementing_paths 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test query -- --exact plan_does_not_report_an_overlap_reached_only_through_references 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test attest -- --exact a_reference_is_not_attested_territory 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 033-configured-corpus-root (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact custom_specs_dir_clears_drift_via_the_owning_spec default_specs_dir_does_not_clear_under_a_custom_root 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 035-registry-plan-ready-set (amends_verification) ----
# amended by 042: the order is unchanged by status.
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact registry_plan_partitions_the_corpus 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test query -- --exact plan_blocked_entries_are_ordered_by_id plan_walks_a_linear_chain_one_step_at_a_time ready_order_is_unchanged_by_status 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 036-declared-state-dir (amends_verification) ----
sh -c 'cargo test -p spec-spine-types --locked --test config -- --exact state_dir_is_unset_by_default_and_inert 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test coverage -- --exact a_claim_inside_the_state_root_is_an_l006_error a_declared_state_root_leaves_both_sides_of_the_coverage_ratio the_resolver_does_not_reach_into_the_state_root the_walk_skips_git_and_the_state_root 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact a_declared_state_root_is_bypassed_and_a_claim_cannot_override_it 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test config_facade -- --exact a_state_dir_the_loader_refuses_is_refused_at_every_entry 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
```
