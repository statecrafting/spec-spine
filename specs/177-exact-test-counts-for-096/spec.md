---
id: "177-exact-test-counts-for-096"
title: "Exact test counts for 096"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 096's acceptance block carries eight lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 096's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "096-compaction-is-a-verb-not-a-session"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "096-compaction-is-a-verb-not-a-session"
amends:
  - "096-compaction-is-a-verb-not-a-session"
---

# 177: Exact test counts for 096

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 096's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 096:13 | core `compact` | whole | 38 |
| 096:17 | core `compact` | `collision` | 1 |
| 096:21 | core `compact` | `short_id` | 4 |
| 096:24 | core `compact` | `idempotent` | 2 |
| 096:27 | core `compact` | `broken_across` | 2 |
| 096:30 | core `compact` | `more_than_one_ordinal` | 1 |
| 096:34 | core `compact` | `fence` | 2 |
| 096:41 | core `compact` | `report` | 2 |

"Block line" counts lines inside 096's `verify:cli` fence. The names each
line ran:

- `compact` (whole target): `a_bare_ordinal_followed_by_a_reference_is_rewritten`, `a_bare_ordinal_in_any_other_position_is_left_alone`, `a_bare_short_id_in_a_command_is_rewritten`, `a_block_that_names_its_own_fence_is_located_whole`, `a_citation_broken_across_a_line_is_rewritten`, `a_citation_broken_across_two_blank_lines_is_not_a_citation`, `a_citation_in_an_extension_less_file_is_rewritten`, `a_citation_naming_more_than_one_ordinal_rewrites_every_one`, `a_citation_of_another_projects_corpus_is_left_alone`, `a_citation_rule_does_not_fire_inside_a_full_id`, `a_documents_title_heading_follows_its_new_ordinal`, `a_fence_inside_a_block_does_not_open_a_second_one`, `a_frontmatter_comment_opening_with_an_ordinal_is_not_the_title`, `a_full_id_is_rewritten_to_its_new_id`, `a_heading_naming_another_spec_is_left_alone_by_form_4`, `a_hyphen_that_is_not_an_ellipsis_is_not_an_elided_id`, `a_list_rewrites_its_mapped_ordinals_and_leaves_the_unmapped_one`, `a_plan_naming_a_spec_the_corpus_does_not_have_is_refused`, `a_plan_parses_from_yaml`, `a_plan_whose_answering_spec_is_itself_removed_is_refused`, `a_plan_with_an_unknown_key_is_refused`, `a_prose_citation_is_rewritten`, `a_removed_specs_full_id_becomes_the_spec_that_answers_for_it`, `a_renamed_spec_directory_holding_an_uncarryable_file_is_refused`, `a_short_id_after_a_flag_is_still_the_commands_argument`, `a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone`, `a_spec_directory_follows_its_new_id`, `an_elided_full_id_is_rewritten`, `an_extension_less_binary_file_is_not_scanned`, `an_only_argument_behind_repo_addresses_a_fixture_corpus`, `an_ordinal_collision_is_refused_and_names_both`, `an_unmapped_ordinal_is_left_alone`, `applying_the_output_to_the_output_is_idempotent`, `the_map_document_carries_every_id_in_both_directions`, `the_new_forms_are_idempotent`, `the_report_names_the_form_behind_every_rewrite_and_counts_per_form`, `the_report_of_a_plan_that_changes_nothing_is_empty_not_absent`, `the_sweeps_only_argument_is_a_short_id`.
- `compact` `collision`: `an_ordinal_collision_is_refused_and_names_both`.
- `compact` `short_id`: `a_bare_short_id_in_a_command_is_rewritten`, `a_short_id_after_a_flag_is_still_the_commands_argument`, `a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone`, `the_sweeps_only_argument_is_a_short_id`.
- `compact` `idempotent`: `applying_the_output_to_the_output_is_idempotent`, `the_new_forms_are_idempotent`.
- `compact` `broken_across`: `a_citation_broken_across_a_line_is_rewritten`, `a_citation_broken_across_two_blank_lines_is_not_a_citation`.
- `compact` `more_than_one_ordinal`: `a_citation_naming_more_than_one_ordinal_rewrites_every_one`.
- `compact` `fence`: `a_block_that_names_its_own_fence_is_located_whole`, `a_fence_inside_a_block_does_not_open_a_second_one`.
- `compact` `report`: `the_report_names_the_form_behind_every_rewrite_and_counts_per_form`, `the_report_of_a_plan_that_changes_nothing_is_empty_not_absent`.

## 2. Territory

This spec establishes nothing. It edits 096 only to add spec 082 §3.4's
superseded-acceptance note above 096's block.

## 3. Behavior

### 3.1 096's block is carried

`spec-spine verify 096` MUST run this
spec's block, which carries 096's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

096 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 096's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 096-compaction-is-a-verb-not-a-session (amends_verification); each loose test count names its tests (153 3.2) ----
cargo build --release --locked
# 2: the territory exists.
test -f crates/spec-spine-core/src/compact.rs
test -f crates/spec-spine-core/tests/compact.rs
test -f crates/spec-spine-cli/src/cmd_compact.rs
# 3: the verb exists, and reads before it writes.
target/release/spec-spine compact --help > "${TMPDIR:-/tmp}/ss096-help.txt" 2>&1
grep -qF -- '--plan' "${TMPDIR:-/tmp}/ss096-help.txt"
rm -f "${TMPDIR:-/tmp}/ss096-help.txt"
# 3.1 - 3.8: every rule, and 5's five defects each with a case that fails
# against the rule that produced it. A non-zero pass count, so a filter that
# matched nothing cannot pass for a run (spec 084 D-7).
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_bare_ordinal_followed_by_a_reference_is_rewritten a_bare_ordinal_in_any_other_position_is_left_alone a_bare_short_id_in_a_command_is_rewritten a_block_that_names_its_own_fence_is_located_whole a_citation_broken_across_a_line_is_rewritten a_citation_broken_across_two_blank_lines_is_not_a_citation a_citation_in_an_extension_less_file_is_rewritten a_citation_naming_more_than_one_ordinal_rewrites_every_one a_citation_of_another_projects_corpus_is_left_alone a_citation_rule_does_not_fire_inside_a_full_id a_documents_title_heading_follows_its_new_ordinal a_fence_inside_a_block_does_not_open_a_second_one a_frontmatter_comment_opening_with_an_ordinal_is_not_the_title a_full_id_is_rewritten_to_its_new_id a_heading_naming_another_spec_is_left_alone_by_form_4 a_hyphen_that_is_not_an_ellipsis_is_not_an_elided_id a_list_rewrites_its_mapped_ordinals_and_leaves_the_unmapped_one a_plan_naming_a_spec_the_corpus_does_not_have_is_refused a_plan_parses_from_yaml a_plan_whose_answering_spec_is_itself_removed_is_refused a_plan_with_an_unknown_key_is_refused a_prose_citation_is_rewritten a_removed_specs_full_id_becomes_the_spec_that_answers_for_it a_renamed_spec_directory_holding_an_uncarryable_file_is_refused a_short_id_after_a_flag_is_still_the_commands_argument a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone a_spec_directory_follows_its_new_id an_elided_full_id_is_rewritten an_extension_less_binary_file_is_not_scanned an_only_argument_behind_repo_addresses_a_fixture_corpus an_ordinal_collision_is_refused_and_names_both an_unmapped_ordinal_is_left_alone applying_the_output_to_the_output_is_idempotent the_map_document_carries_every_id_in_both_directions the_new_forms_are_idempotent the_report_names_the_form_behind_every_rewrite_and_counts_per_form the_report_of_a_plan_that_changes_nothing_is_empty_not_absent the_sweeps_only_argument_is_a_short_id 2>&1 | grep -q "test result: ok. 38 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss096-t.txt"
# 1.1 defect 1: an ordinal collision is refused, and both entries are named.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact an_ordinal_collision_is_refused_and_names_both 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 1.1 defect 2: a bare short id in a command is rewritten, and one behind
# `--repo` is not.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_bare_short_id_in_a_command_is_rewritten a_short_id_after_a_flag_is_still_the_commands_argument a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone the_sweeps_only_argument_is_a_short_id 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
# 1.1 defect 3, and 3.4: applying the output to the output changes nothing.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact applying_the_output_to_the_output_is_idempotent the_new_forms_are_idempotent 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 1.1 defect 6: a citation broken across a line is rewritten.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_citation_broken_across_a_line_is_rewritten a_citation_broken_across_two_blank_lines_is_not_a_citation 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# 1.1 defect 7: a citation naming more than one ordinal rewrites every one.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_citation_naming_more_than_one_ordinal_rewrites_every_one 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss096-6.txt" "${TMPDIR:-/tmp}/ss096-7.txt"
# 1.1 defect 4: a block that greps for its own fence is rewritten whole.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact a_block_that_names_its_own_fence_is_located_whole a_fence_inside_a_block_does_not_open_a_second_one 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss096-1.txt" "${TMPDIR:-/tmp}/ss096-2.txt" "${TMPDIR:-/tmp}/ss096-3.txt" "${TMPDIR:-/tmp}/ss096-4.txt"
# 3.1, D-2: the library writes nothing. A source read, because the contract is
# what a caller relies on and a doc comment does not fail.
! grep -qE 'fs::(write|create|remove)' crates/spec-spine-core/src/compact.rs
# 3.6: the report names the form that produced each rewrite, per file.
sh -c 'cargo test -p spec-spine-core --locked --test compact -- --exact the_report_names_the_form_behind_every_rewrite_and_counts_per_form the_report_of_a_plan_that_changes_nothing_is_empty_not_absent 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss096-5.txt"
# Declared and read through the CLI.
target/release/spec-spine registry show 096 --json > "${TMPDIR:-/tmp}/ss096-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss096-show.json')); assert d['id'] == '096-compaction-is-a-verb-not-a-session', d"
rm -f "${TMPDIR:-/tmp}/ss096-show.json"
# The governed loop and the stack's own gate.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```
