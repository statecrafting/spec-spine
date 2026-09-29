---
id: "178-exact-test-counts-for-098"
title: "Exact test counts for 098"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 098's acceptance block carries two lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 098's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "098-a-citation-the-renumber-could-not-see"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "098-a-citation-the-renumber-could-not-see"
amends:
  - "098-a-citation-the-renumber-could-not-see"
---

# 178: Exact test counts for 098

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 098's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 098:34 | core `compact` | whole | 38 |
| 098:37 | core `lint` | whole | 39 |

"Block line" counts lines inside 098's `verify:cli` fence. The names each
line ran:

- `compact` (whole target): `a_bare_ordinal_followed_by_a_reference_is_rewritten`, `a_bare_ordinal_in_any_other_position_is_left_alone`, `a_bare_short_id_in_a_command_is_rewritten`, `a_block_that_names_its_own_fence_is_located_whole`, `a_citation_broken_across_a_line_is_rewritten`, `a_citation_broken_across_two_blank_lines_is_not_a_citation`, `a_citation_in_an_extension_less_file_is_rewritten`, `a_citation_naming_more_than_one_ordinal_rewrites_every_one`, `a_citation_of_another_projects_corpus_is_left_alone`, `a_citation_rule_does_not_fire_inside_a_full_id`, `a_documents_title_heading_follows_its_new_ordinal`, `a_fence_inside_a_block_does_not_open_a_second_one`, `a_frontmatter_comment_opening_with_an_ordinal_is_not_the_title`, `a_full_id_is_rewritten_to_its_new_id`, `a_heading_naming_another_spec_is_left_alone_by_form_4`, `a_hyphen_that_is_not_an_ellipsis_is_not_an_elided_id`, `a_list_rewrites_its_mapped_ordinals_and_leaves_the_unmapped_one`, `a_plan_naming_a_spec_the_corpus_does_not_have_is_refused`, `a_plan_parses_from_yaml`, `a_plan_whose_answering_spec_is_itself_removed_is_refused`, `a_plan_with_an_unknown_key_is_refused`, `a_prose_citation_is_rewritten`, `a_removed_specs_full_id_becomes_the_spec_that_answers_for_it`, `a_renamed_spec_directory_holding_an_uncarryable_file_is_refused`, `a_short_id_after_a_flag_is_still_the_commands_argument`, `a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone`, `a_spec_directory_follows_its_new_id`, `an_elided_full_id_is_rewritten`, `an_extension_less_binary_file_is_not_scanned`, `an_only_argument_behind_repo_addresses_a_fixture_corpus`, `an_ordinal_collision_is_refused_and_names_both`, `an_unmapped_ordinal_is_left_alone`, `applying_the_output_to_the_output_is_idempotent`, `the_map_document_carries_every_id_in_both_directions`, `the_new_forms_are_idempotent`, `the_report_names_the_form_behind_every_rewrite_and_counts_per_form`, `the_report_of_a_plan_that_changes_nothing_is_empty_not_absent`, `the_sweeps_only_argument_is_a_short_id`.
- `lint` (whole target): `a_claim_inside_the_state_root_defers_to_l006`, `a_claim_on_a_file_that_does_not_exist_is_silent`, `a_covering_glob_witnesses_the_claim_and_the_directory_form_does_not`, `a_forward_dependency_does_not_fail_compile`, `a_forward_dependency_is_an_error_and_a_backward_one_is_silent`, `a_frontmatter_comment_is_not_the_title_heading`, `a_heading_that_continues_past_the_anchor_is_not_the_section`, `a_self_dependency_is_caught_by_the_equality_arm`, `a_title_heading_naming_another_spec_is_reported`, `a_title_heading_naming_its_own_spec_is_silent`, `an_id_or_a_heading_without_an_ordinal_is_silent`, `an_id_without_an_ordinal_is_silence_in_either_position`, `an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies`, `l009_is_info_tier_and_names_the_anchor`, `l010_does_not_fire_on_a_pattern_that_merely_matches_nothing_yet`, `l010_is_silent_on_the_working_glob_form`, `l010_is_silent_on_the_working_slice_form`, `l010_refuses_a_hashed_input_pattern_that_can_match_no_file`, `l010_refuses_a_slice_pattern_that_can_match_no_file`, `l010_reports_both_tables_and_each_form_names_its_own_table`, `l011_is_silent_while_the_work_is_genuinely_in_flight`, `l011_refuses_a_completed_spec_that_still_plans_territory`, `l012_is_silent_while_the_planned_unit_is_still_unwritten`, `l012_reports_a_planned_unit_that_has_resolved`, `retroactive_origin_alone_produces_no_diagnostic`, `the_allowlist_suppresses_the_warning_but_not_the_count`, `the_comparison_is_numeric_not_lexical`, `the_defects_heading_is_matched_by_anchor_at_any_level_or_numbering`, `the_knob_defaults_off_and_off_emits_nothing`, `the_l010_message_names_the_working_form`, `the_near_miss_rule_catches_attempts_and_not_prose`, `the_planned_refusals_are_silent_on_a_corpus_that_plans_nothing`, `the_slice_l010_message_does_not_claim_a_content_hash`, `the_slice_l010_message_names_the_table_the_slice_and_the_working_form`, `v015_allows_a_shared_plan_declared_as_co_authority`, `v015_refuses_two_specs_planning_the_same_unit`, `v016_refuses_planning_territory_another_spec_already_owns`, `v017_refuses_extending_a_unit_that_is_only_planned`, `witnessed_paths_covers_every_hash_contributor`.

## 2. Territory

This spec establishes nothing. It edits 098 only to add spec 082 §3.4's
superseded-acceptance note above 098's block.

## 3. Behavior

### 3.1 098's block is carried

`spec-spine verify 098` MUST run this
spec's block, which carries 098's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

098 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 098's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 098-a-citation-the-renumber-could-not-see (amends_verification); each loose test count names its tests (153 3.2) ----
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
# 3.7 (class E): the sweep's argument and the assertion beside it name the same
# spec again, and the two fixture-corpus lines behind `--repo` still say 001.
grep -qF -- '--only 011 --out' specs/089-nothing-reruns-a-merged-acceptance/spec.md
! grep -qF -- '--only 012' specs/089-nothing-reruns-a-merged-acceptance/spec.md
grep -qF -- '--only 001,001,001' specs/089-nothing-reruns-a-merged-acceptance/spec.md
grep -qF -- '"--only"' crates/spec-spine-core/src/compact.rs
# The tests that pin the forms and their exclusions.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_bare_ordinal_followed_by_a_reference_is_rewritten a_bare_ordinal_in_any_other_position_is_left_alone a_bare_short_id_in_a_command_is_rewritten a_block_that_names_its_own_fence_is_located_whole a_citation_broken_across_a_line_is_rewritten a_citation_broken_across_two_blank_lines_is_not_a_citation a_citation_in_an_extension_less_file_is_rewritten a_citation_naming_more_than_one_ordinal_rewrites_every_one a_citation_of_another_projects_corpus_is_left_alone a_citation_rule_does_not_fire_inside_a_full_id a_documents_title_heading_follows_its_new_ordinal a_fence_inside_a_block_does_not_open_a_second_one a_frontmatter_comment_opening_with_an_ordinal_is_not_the_title a_full_id_is_rewritten_to_its_new_id a_heading_naming_another_spec_is_left_alone_by_form_4 a_hyphen_that_is_not_an_ellipsis_is_not_an_elided_id a_list_rewrites_its_mapped_ordinals_and_leaves_the_unmapped_one a_plan_naming_a_spec_the_corpus_does_not_have_is_refused a_plan_parses_from_yaml a_plan_whose_answering_spec_is_itself_removed_is_refused a_plan_with_an_unknown_key_is_refused a_prose_citation_is_rewritten a_removed_specs_full_id_becomes_the_spec_that_answers_for_it a_renamed_spec_directory_holding_an_uncarryable_file_is_refused a_short_id_after_a_flag_is_still_the_commands_argument a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone a_spec_directory_follows_its_new_id an_elided_full_id_is_rewritten an_extension_less_binary_file_is_not_scanned an_only_argument_behind_repo_addresses_a_fixture_corpus an_ordinal_collision_is_refused_and_names_both an_unmapped_ordinal_is_left_alone applying_the_output_to_the_output_is_idempotent the_map_document_carries_every_id_in_both_directions the_new_forms_are_idempotent the_report_names_the_form_behind_every_rewrite_and_counts_per_form the_report_of_a_plan_that_changes_nothing_is_empty_not_absent the_sweeps_only_argument_is_a_short_id 2>&1 | grep -q "test result: ok. 38 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss098-c.txt"
sh -c 'cargo test -p spec-spine-core --locked --test lint -- --exact a_claim_inside_the_state_root_defers_to_l006 a_claim_on_a_file_that_does_not_exist_is_silent a_covering_glob_witnesses_the_claim_and_the_directory_form_does_not a_forward_dependency_does_not_fail_compile a_forward_dependency_is_an_error_and_a_backward_one_is_silent a_frontmatter_comment_is_not_the_title_heading a_heading_that_continues_past_the_anchor_is_not_the_section a_self_dependency_is_caught_by_the_equality_arm a_title_heading_naming_another_spec_is_reported a_title_heading_naming_its_own_spec_is_silent an_id_or_a_heading_without_an_ordinal_is_silent an_id_without_an_ordinal_is_silence_in_either_position an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies l009_is_info_tier_and_names_the_anchor l010_does_not_fire_on_a_pattern_that_merely_matches_nothing_yet l010_is_silent_on_the_working_glob_form l010_is_silent_on_the_working_slice_form l010_refuses_a_hashed_input_pattern_that_can_match_no_file l010_refuses_a_slice_pattern_that_can_match_no_file l010_reports_both_tables_and_each_form_names_its_own_table l011_is_silent_while_the_work_is_genuinely_in_flight l011_refuses_a_completed_spec_that_still_plans_territory l012_is_silent_while_the_planned_unit_is_still_unwritten l012_reports_a_planned_unit_that_has_resolved retroactive_origin_alone_produces_no_diagnostic the_allowlist_suppresses_the_warning_but_not_the_count the_comparison_is_numeric_not_lexical the_defects_heading_is_matched_by_anchor_at_any_level_or_numbering the_knob_defaults_off_and_off_emits_nothing the_l010_message_names_the_working_form the_near_miss_rule_catches_attempts_and_not_prose the_planned_refusals_are_silent_on_a_corpus_that_plans_nothing the_slice_l010_message_does_not_claim_a_content_hash the_slice_l010_message_names_the_table_the_slice_and_the_working_form v015_allows_a_shared_plan_declared_as_co_authority v015_refuses_two_specs_planning_the_same_unit v016_refuses_planning_territory_another_spec_already_owns v017_refuses_extending_a_unit_that_is_only_planned witnessed_paths_covers_every_hash_contributor 2>&1 | grep -q "test result: ok. 39 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss098-l.txt"
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
