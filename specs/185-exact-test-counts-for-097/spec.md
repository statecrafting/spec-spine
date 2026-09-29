---
id: "185-exact-test-counts-for-097"
title: "Exact test counts for 097"
status: draft
kind: "test"
created: "2026-09-29"
summary: >
  Spec 097's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 097's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
amends:
  - "097-a-path-leaves-the-corpus-the-way-a-spec-does"
---

# 185: Exact test counts for 097

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 097's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 097:6 | core `retire` | whole | 62 |

"Block line" counts lines inside 097's `verify:cli` fence. The names each
line ran:

- `retire` (whole target): `a_backticked_and_a_bare_occurrence_on_one_line_are_each_rewritten_once`, `a_backticked_citation_ending_a_sentence_is_rewritten`, `a_backticked_citation_is_replaced`, `a_backticked_glob_is_handled_by_the_glob_form`, `a_bare_citation_is_replaced`, `a_blank_line_between_a_key_and_its_items_does_not_empty_it`, `a_block_scalar_key_does_not_open_an_edge_list`, `a_comment_inside_a_fence_is_not_a_heading`, `a_directory_entry_without_a_trailing_slash_is_refused`, `a_dot_slash_prefixed_path_is_refused`, `a_form_with_no_rule_is_refused_rather_than_skipped`, `a_glob_deletion_does_not_shift_a_later_skip_out_of_alignment`, `a_glob_may_be_removed_but_a_citation_may_not`, `a_glob_pattern_that_matches_only_the_retired_path_is_removed`, `a_glob_replacement_does_not_strand_a_citation_on_the_same_line`, `a_glob_replacement_naming_the_path_is_not_rewritten_again`, `a_glob_replacement_that_is_not_a_prefix_is_refused`, `a_glob_rule_does_not_fire_on_a_longer_paths_glob`, `a_historical_file_is_left_alone_and_reported`, `a_historical_file_outside_the_corpus_is_refused`, `a_historical_section_covers_its_sub_headings`, `a_historical_section_is_left_alone_and_reported`, `a_line_one_entry_spares_is_not_rewritten_by_another`, `a_longer_path_is_not_a_retired_one`, `a_markdown_link_label_is_a_citation_and_is_rewritten`, `a_must_not_clause_is_not_a_citation`, `a_negation_is_not_a_citation`, `a_negation_on_a_longer_path_is_not_a_skip_of_the_retired_one`, `a_path_form_does_not_preempt_a_unit_action_on_the_same_line`, `a_path_form_replacement_containing_the_path_fires_once`, `a_path_only_plan_does_not_reach_inside_a_backtick_citation`, `a_period_terminates_a_citation_only_at_the_end_of_a_sentence`, `a_retarget_does_not_rewrite_a_sibling_sharing_the_prefix`, `a_retarget_rewrites_the_unit_and_keeps_the_edge`, `a_retarget_with_an_empty_target_is_refused`, `a_retarget_with_no_target_is_refused`, `a_retire_plan_parses_from_yaml`, `a_retired_path_in_a_windows_form_is_refused`, `a_retired_path_outside_the_corpus_is_refused`, `a_retirement_counts_its_rewrites`, `a_scalar_whose_value_ends_in_a_colon_is_not_an_emptied_key`, `a_second_occurrence_abutting_the_first_still_reads_its_left_boundary`, `a_second_path_on_a_spared_line_is_reported_too`, `a_unit_claiming_a_longer_path_is_not_the_retired_one`, `a_unit_on_an_approved_spec_needs_the_human_acknowledgement`, `a_withdrawal_beats_a_retarget_on_the_same_line_and_is_recorded_once`, `a_withdrawal_that_also_names_a_target_is_refused`, `a_withdrawal_that_empties_an_edge_removes_the_key`, `an_empty_historical_entry_is_refused`, `an_empty_retired_path_is_refused`, `an_entry_does_not_act_on_an_occurrence_another_entry_created`, `an_entry_with_no_clause_of_its_own_takes_the_lines`, `an_implicit_multiline_scalar_does_not_open_an_edge_list`, `an_occurrence_another_entrys_replacement_created_is_not_spared`, `an_occurrence_no_rule_covers_is_reported_as_leftover`, `an_underscore_prefixed_path_is_not_the_retired_one`, `each_occurrence_on_a_spared_line_carries_its_own_clause`, `every_glob_occurrence_on_a_line_is_replaced`, `every_occurrence_of_a_duplicated_unit_line_is_withdrawn`, `every_skip_names_the_clause_that_produced_it`, `every_unit_action_matching_a_line_fires`, `retiring_a_path_the_tree_does_not_have_is_refused`.

## 2. Territory

This spec establishes nothing. It edits 097 only to add spec 082 §3.4's
superseded-acceptance note above 097's block.

## 3. Behavior

### 3.1 097's block is carried

`spec-spine verify 097` MUST run this
spec's block, which carries 097's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

097 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 097's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 097-a-path-leaves-the-corpus-the-way-a-spec-does (amends_verification); each loose test count names its tests (153 3.2) ----
cargo build --release --locked
test -f crates/spec-spine-core/tests/retire.rs
target/release/spec-spine compact --help > "${TMPDIR:-/tmp}/ss097-help.txt" 2>&1
grep -qF 'retire' "${TMPDIR:-/tmp}/ss097-help.txt"
rm -f "${TMPDIR:-/tmp}/ss097-help.txt"
sh -c 'cargo test -p spec-spine-core --locked --test retire -- --exact a_backticked_and_a_bare_occurrence_on_one_line_are_each_rewritten_once a_backticked_citation_ending_a_sentence_is_rewritten a_backticked_citation_is_replaced a_backticked_glob_is_handled_by_the_glob_form a_bare_citation_is_replaced a_blank_line_between_a_key_and_its_items_does_not_empty_it a_block_scalar_key_does_not_open_an_edge_list a_comment_inside_a_fence_is_not_a_heading a_directory_entry_without_a_trailing_slash_is_refused a_dot_slash_prefixed_path_is_refused a_form_with_no_rule_is_refused_rather_than_skipped a_glob_deletion_does_not_shift_a_later_skip_out_of_alignment a_glob_may_be_removed_but_a_citation_may_not a_glob_pattern_that_matches_only_the_retired_path_is_removed a_glob_replacement_does_not_strand_a_citation_on_the_same_line a_glob_replacement_naming_the_path_is_not_rewritten_again a_glob_replacement_that_is_not_a_prefix_is_refused a_glob_rule_does_not_fire_on_a_longer_paths_glob a_historical_file_is_left_alone_and_reported a_historical_file_outside_the_corpus_is_refused a_historical_section_covers_its_sub_headings a_historical_section_is_left_alone_and_reported a_line_one_entry_spares_is_not_rewritten_by_another a_longer_path_is_not_a_retired_one a_markdown_link_label_is_a_citation_and_is_rewritten a_must_not_clause_is_not_a_citation a_negation_is_not_a_citation a_negation_on_a_longer_path_is_not_a_skip_of_the_retired_one a_path_form_does_not_preempt_a_unit_action_on_the_same_line a_path_form_replacement_containing_the_path_fires_once a_path_only_plan_does_not_reach_inside_a_backtick_citation a_period_terminates_a_citation_only_at_the_end_of_a_sentence a_retarget_does_not_rewrite_a_sibling_sharing_the_prefix a_retarget_rewrites_the_unit_and_keeps_the_edge a_retarget_with_an_empty_target_is_refused a_retarget_with_no_target_is_refused a_retire_plan_parses_from_yaml a_retired_path_in_a_windows_form_is_refused a_retired_path_outside_the_corpus_is_refused a_retirement_counts_its_rewrites a_scalar_whose_value_ends_in_a_colon_is_not_an_emptied_key a_second_occurrence_abutting_the_first_still_reads_its_left_boundary a_second_path_on_a_spared_line_is_reported_too a_unit_claiming_a_longer_path_is_not_the_retired_one a_unit_on_an_approved_spec_needs_the_human_acknowledgement a_withdrawal_beats_a_retarget_on_the_same_line_and_is_recorded_once a_withdrawal_that_also_names_a_target_is_refused a_withdrawal_that_empties_an_edge_removes_the_key an_empty_historical_entry_is_refused an_empty_retired_path_is_refused an_entry_does_not_act_on_an_occurrence_another_entry_created an_entry_with_no_clause_of_its_own_takes_the_lines an_implicit_multiline_scalar_does_not_open_an_edge_list an_occurrence_another_entrys_replacement_created_is_not_spared an_occurrence_no_rule_covers_is_reported_as_leftover an_underscore_prefixed_path_is_not_the_retired_one each_occurrence_on_a_spared_line_carries_its_own_clause every_glob_occurrence_on_a_line_is_replaced every_occurrence_of_a_duplicated_unit_line_is_withdrawn every_skip_names_the_clause_that_produced_it every_unit_action_matching_a_line_fires retiring_a_path_the_tree_does_not_have_is_refused 2>&1 | grep -q "test result: ok. 62 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss097-t.txt"
# 3.3: a unit on an approved spec is refused without the human acknowledgement.
grep -qF 'acknowledge_approved' crates/spec-spine-core/src/compact.rs
# 3.5: the exclusions exist as clauses, not as prose.
grep -qF 'historical_sections' crates/spec-spine-core/src/compact.rs
grep -qF 'historical_files' crates/spec-spine-core/src/compact.rs
# 1.3, still true of the retirement this spec was measured on: the eleven
# survivors are still there, and the path is still gone.
! test -e .claude/rules
grep -qF 'MUST NOT exist' specs/093-the-harness-this-repository-runs/spec.md
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
