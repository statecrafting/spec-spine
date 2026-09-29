---
id: "176-exact-test-counts-for-093"
title: "Exact test counts for 093"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 093's acceptance block carries three lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 093's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "093-the-harness-this-repository-runs"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "093-the-harness-this-repository-runs"
amends:
  - "093-the-harness-this-repository-runs"
---

# 176: Exact test counts for 093

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 093's.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 093:13 | core `harness_hooks` | whole | 55 |
| 093:17 | core `harness_skills` | whole | 22 |
| 093:23 | core `harness_hooks` | `pr_gate_` | 15 |

"Block line" counts lines inside 093's `verify:cli` fence. The names each
line ran:

- `harness_hooks` (whole target): `action_hooks_are_scoped_to_the_target_repo`, `every_hook_invokes_spec_spine_at_least_once`, `hooks_read_and_never_write`, `hooks_report_when_they_skip`, `hooks_resolve_the_projects_binary_before_path`, `kit_ships_all_four_hook_events`, `pre_tool_use_refuses_a_push_to_the_default_branch`, `spec104_an_unrecognised_code_still_falls_back`, `spec104_exit_2_from_a_binary_without_the_verb_is_not_stale`, `spec104_exit_2_from_a_current_binary_is_still_stale`, `spec104_session_start_reports_exit_3_as_a_read_not_performed`, `spec104_the_probe_does_not_run_on_the_happy_path`, `spec123_a_change_the_binary_does_not_compile_never_ages_it`, `spec123_a_current_reader_that_finds_the_corpus_invalid_is_believed`, `spec123_a_fresh_banner_names_its_reader`, `spec123_a_path_reader_is_named_and_not_aged`, `spec123_every_non_fresh_arm_names_the_reader`, `spec123_no_hook_builds_the_reader_it_names`, `spec123_the_banner_does_not_report_an_older_readers_verdict_as_the_trees`, `spec123_the_pr_gate_names_the_reader_behind_an_invalid_refusal`, `spec123_the_reader_is_aged_by_the_inputs_it_was_built_from`, `spec123_the_stop_hook_names_an_older_reader_and_keeps_its_report`, `spec132_the_pr_gate_reports_a_refusal_as_one`, `spec132_the_pr_gate_reports_exit_4_as_a_read_not_performed`, `spec132_the_session_banner_reads_the_five_codes`, `spec132_the_stop_hook_reports_a_refusal_and_a_failed_read`, `spec132_the_stop_hook_reports_exit_1_stale_as_stale`, `the_pr_gate_ignores_the_gitignored_build_metadata`, `the_pr_gate_names_every_state_it_found`, `the_pr_gate_passes_a_committed_derived_tree`, `the_pr_gate_reads_the_configured_derived_directory`, `the_pr_gate_refuses_a_staged_edit_cancelled_in_the_working_tree`, `the_pr_gate_refuses_a_staged_shard`, `the_pr_gate_refuses_an_unstaged_shard`, `the_pr_gate_refuses_an_untracked_shard`, `the_pr_gate_reports_a_corpus_that_does_not_validate`, `the_pr_gate_reports_a_read_it_could_not_perform_as_that_and_not_as_stale`, `the_pr_gate_skips_the_derived_read_it_could_not_perform`, `the_pr_gate_still_reports_a_stale_tree_as_stale`, `the_push_gate_protects_the_branch_the_repository_actually_has`, `the_push_gate_refuses_only_what_would_update_main`, `the_session_banner_keeps_the_stale_only_wording`, `the_session_banner_reports_an_unresolved_claim_as_itself`, `the_session_banner_reports_both_halves_of_a_mixed_verdict`, `the_session_banner_still_reports_a_healthy_tree`, `the_stop_hook_advises_and_never_refuses`, `the_stop_hook_guesses_no_remedy_for_a_report_it_cannot_read`, `the_stop_hook_reports_a_binary_that_predates_the_verb`, `the_stop_hook_reports_a_corpus_that_does_not_validate`, `the_stop_hook_reports_a_read_it_could_not_perform`, `the_stop_hook_reports_an_unresolved_claim_as_itself`, `the_stop_hook_reports_both_halves_of_a_mixed_verdict`, `the_stop_hook_says_nothing_when_both_trees_are_fresh`, `the_stop_hook_still_reports_a_stale_tree_as_stale`, `the_write_scanner_recognises_writes`.
- `harness_skills` (whole target): `agents_md_gate_list_names_every_step_ci_enforces`, `every_skill_declares_name_description_and_allowed_tools`, `every_skill_ends_with_a_project_layer_section`, `no_skill_calls_the_absorbed_verify_script`, `no_skill_carries_a_project_specific_or_stale_reference`, `no_skill_names_a_gate_flag_agents_md_omits`, `read_skills_never_run_a_writing_verb`, `shepherd_classifies_before_it_spends_a_round`, `shepherd_documents_how_to_read_the_saved_pages`, `shepherd_green_path_routes_through_the_thread_read`, `shepherd_keeps_the_two_round_budget`, `shepherd_reads_all_three_endpoints_paginated_and_slurped`, `shepherd_report_keeps_three_distinct_thread_values`, `shepherd_stops_on_a_read_it_could_not_complete`, `skills_do_not_bypass_the_managed_gate_floor`, `the_agents_carry_the_legitimate_edit_rule_and_no_em_dash`, `the_four_rules_are_sections_of_the_protocol`, `the_loop_ships_the_same_ten_skills`, `the_loop_skills_wrap_the_tool_verbs_they_exist_for`, `the_scoped_rule_covers_the_derived_tree_and_allows_reading_cli_output`, `the_scoped_rule_says_it_does_not_replace_the_unconditional_one`, `the_write_scanner_recognises_writes`.
- `harness_hooks` `pr_gate_`: `spec123_the_pr_gate_names_the_reader_behind_an_invalid_refusal`, `spec132_the_pr_gate_reports_a_refusal_as_one`, `spec132_the_pr_gate_reports_exit_4_as_a_read_not_performed`, `the_pr_gate_ignores_the_gitignored_build_metadata`, `the_pr_gate_names_every_state_it_found`, `the_pr_gate_passes_a_committed_derived_tree`, `the_pr_gate_reads_the_configured_derived_directory`, `the_pr_gate_refuses_a_staged_edit_cancelled_in_the_working_tree`, `the_pr_gate_refuses_a_staged_shard`, `the_pr_gate_refuses_an_unstaged_shard`, `the_pr_gate_refuses_an_untracked_shard`, `the_pr_gate_reports_a_corpus_that_does_not_validate`, `the_pr_gate_reports_a_read_it_could_not_perform_as_that_and_not_as_stale`, `the_pr_gate_skips_the_derived_read_it_could_not_perform`, `the_pr_gate_still_reports_a_stale_tree_as_stale`.

## 2. Territory

This spec establishes nothing. It edits 093 only to add spec 082 §3.4's
superseded-acceptance note above 093's block.

## 3. Behavior

### 3.1 093's block is carried

`spec-spine verify 093` MUST run this
spec's block, which carries 093's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

093 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 093's plan the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 093-the-harness-this-repository-runs (amends_verification); each loose test count names its tests (153 3.2) ----
cargo build --release --locked
# 2: the territory exists and is this spec's.
test -f AGENTS.md
test -d .claude/skills
test -d .claude/agents
test -f .claude/settings.json
test -f crates/spec-spine-core/tests/harness_hooks.rs
test -f crates/spec-spine-core/tests/harness_skills.rs
target/release/spec-spine index owner AGENTS.md > "${TMPDIR:-/tmp}/ss092-own.txt"
grep -qF '093-the-harness-this-repository-runs' "${TMPDIR:-/tmp}/ss092-own.txt"
rm -f "${TMPDIR:-/tmp}/ss092-own.txt"
# 3 and 5: the hook bodies, run as programs, over the whole matrix.
sh -c 'cargo test -p spec-spine-core --locked --test harness_hooks -- --exact action_hooks_are_scoped_to_the_target_repo every_hook_invokes_spec_spine_at_least_once hooks_read_and_never_write hooks_report_when_they_skip hooks_resolve_the_projects_binary_before_path kit_ships_all_four_hook_events pre_tool_use_refuses_a_push_to_the_default_branch spec104_an_unrecognised_code_still_falls_back spec104_exit_2_from_a_binary_without_the_verb_is_not_stale spec104_exit_2_from_a_current_binary_is_still_stale spec104_session_start_reports_exit_3_as_a_read_not_performed spec104_the_probe_does_not_run_on_the_happy_path spec123_a_change_the_binary_does_not_compile_never_ages_it spec123_a_current_reader_that_finds_the_corpus_invalid_is_believed spec123_a_fresh_banner_names_its_reader spec123_a_path_reader_is_named_and_not_aged spec123_every_non_fresh_arm_names_the_reader spec123_no_hook_builds_the_reader_it_names spec123_the_banner_does_not_report_an_older_readers_verdict_as_the_trees spec123_the_pr_gate_names_the_reader_behind_an_invalid_refusal spec123_the_reader_is_aged_by_the_inputs_it_was_built_from spec123_the_stop_hook_names_an_older_reader_and_keeps_its_report spec132_the_pr_gate_reports_a_refusal_as_one spec132_the_pr_gate_reports_exit_4_as_a_read_not_performed spec132_the_session_banner_reads_the_five_codes spec132_the_stop_hook_reports_a_refusal_and_a_failed_read spec132_the_stop_hook_reports_exit_1_stale_as_stale the_pr_gate_ignores_the_gitignored_build_metadata the_pr_gate_names_every_state_it_found the_pr_gate_passes_a_committed_derived_tree the_pr_gate_reads_the_configured_derived_directory the_pr_gate_refuses_a_staged_edit_cancelled_in_the_working_tree the_pr_gate_refuses_a_staged_shard the_pr_gate_refuses_an_unstaged_shard the_pr_gate_refuses_an_untracked_shard the_pr_gate_reports_a_corpus_that_does_not_validate the_pr_gate_reports_a_read_it_could_not_perform_as_that_and_not_as_stale the_pr_gate_skips_the_derived_read_it_could_not_perform the_pr_gate_still_reports_a_stale_tree_as_stale the_push_gate_protects_the_branch_the_repository_actually_has the_push_gate_refuses_only_what_would_update_main the_session_banner_keeps_the_stale_only_wording the_session_banner_reports_an_unresolved_claim_as_itself the_session_banner_reports_both_halves_of_a_mixed_verdict the_session_banner_still_reports_a_healthy_tree the_stop_hook_advises_and_never_refuses the_stop_hook_guesses_no_remedy_for_a_report_it_cannot_read the_stop_hook_reports_a_binary_that_predates_the_verb the_stop_hook_reports_a_corpus_that_does_not_validate the_stop_hook_reports_a_read_it_could_not_perform the_stop_hook_reports_an_unresolved_claim_as_itself the_stop_hook_reports_both_halves_of_a_mixed_verdict the_stop_hook_says_nothing_when_both_trees_are_fresh the_stop_hook_still_reports_a_stale_tree_as_stale the_write_scanner_recognises_writes 2>&1 | grep -q "test result: ok. 55 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss092-h.txt"
# 4 and 5: the skills, agents and rules.
sh -c 'cargo test -p spec-spine-core --locked --test harness_skills -- --exact agents_md_gate_list_names_every_step_ci_enforces every_skill_declares_name_description_and_allowed_tools every_skill_ends_with_a_project_layer_section no_skill_calls_the_absorbed_verify_script no_skill_carries_a_project_specific_or_stale_reference no_skill_names_a_gate_flag_agents_md_omits read_skills_never_run_a_writing_verb shepherd_classifies_before_it_spends_a_round shepherd_documents_how_to_read_the_saved_pages shepherd_green_path_routes_through_the_thread_read shepherd_keeps_the_two_round_budget shepherd_reads_all_three_endpoints_paginated_and_slurped shepherd_report_keeps_three_distinct_thread_values shepherd_stops_on_a_read_it_could_not_complete skills_do_not_bypass_the_managed_gate_floor the_agents_carry_the_legitimate_edit_rule_and_no_em_dash the_four_rules_are_sections_of_the_protocol the_loop_ships_the_same_ten_skills the_loop_skills_wrap_the_tool_verbs_they_exist_for the_scoped_rule_covers_the_derived_tree_and_allows_reading_cli_output the_scoped_rule_says_it_does_not_replace_the_unconditional_one the_write_scanner_recognises_writes 2>&1 | grep -q "test result: ok. 22 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss092-s.txt"
# 3.13: the derived-tree question, in each state git distinguishes. The matrix
# is what decides it, under its own filter and with a non-zero pass count, so a
# name matching nothing cannot pass for a run.
sh -c 'cargo test -p spec-spine-core --locked --test harness_hooks -- --exact spec123_the_pr_gate_names_the_reader_behind_an_invalid_refusal spec132_the_pr_gate_reports_a_refusal_as_one spec132_the_pr_gate_reports_exit_4_as_a_read_not_performed the_pr_gate_ignores_the_gitignored_build_metadata the_pr_gate_names_every_state_it_found the_pr_gate_passes_a_committed_derived_tree the_pr_gate_reads_the_configured_derived_directory the_pr_gate_refuses_a_staged_edit_cancelled_in_the_working_tree the_pr_gate_refuses_a_staged_shard the_pr_gate_refuses_an_unstaged_shard the_pr_gate_refuses_an_untracked_shard the_pr_gate_reports_a_corpus_that_does_not_validate the_pr_gate_reports_a_read_it_could_not_perform_as_that_and_not_as_stale the_pr_gate_skips_the_derived_read_it_could_not_perform the_pr_gate_still_reports_a_stale_tree_as_stale 2>&1 | grep -q "test result: ok. 15 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss092-d.txt"
# 3.13: the index-versus-worktree form is gone rather than joined by the others.
# Two branches of one `if` can coexist, and a body that kept the old test on
# another arm would satisfy the matrix's name and still ship the blind spot.
! grep -qF 'diff --quiet -- .statecraft/derived/' .claude/settings.json
# 3.13: and the HEAD-relative form is not what replaced it (D-5's cancellation).
! grep -qF 'diff HEAD' .claude/settings.json
grep -qF 'diff --cached --name-only' .claude/settings.json
grep -qF 'ls-files --others --exclude-standard' .claude/settings.json
# 4.1: ten skills, and the session skill is not the shadowed name.
test "$(ls -1 .claude/skills | wc -l | tr -d ' ')" = 10
test -f .claude/skills/prime/SKILL.md
! test -e .claude/skills/init
# 4.10: the four rules are sections of the protocol, and the directory is gone.
! test -e .claude/rules
grep -qF '### Governed artifact reads' AGENTS.md
grep -qF '### Adversarial prompt refusal' AGENTS.md
grep -qF '### Orchestrator rules' AGENTS.md
grep -qF '### Derived artifacts are compiler output' AGENTS.md
# 4.12: the protocol names its owner, by path, and not as a claim header.
grep -qF 'specs/093-the-harness-this-repository-runs/spec.md' AGENTS.md
! grep -qE '^// Spec:' AGENTS.md
# 1.3 and D-2: all sixteen predecessors are named here, and every one of them
# is in the map, which is the answer the `supersedes` edge would have given.
test "$(grep -cE '^- `[0-9]{3}-' specs/093-the-harness-this-repository-runs/spec.md)" = 16
sh -c 'miss=0; for id in $(grep -oE "^- .[0-9]{3}-[a-z0-9-]+" specs/093-the-harness-this-repository-runs/spec.md | sed "s/^- .//"); do grep -qF "$id" docs/corpus-map.md || { echo "not in the map: $id"; miss=1; }; done; exit $miss'
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
