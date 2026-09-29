---
id: "181-exact-test-counts-for-149"
title: "Exact test counts for 149"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 149's acceptance block carries one line that accepts
  `test result: ok. [1-9][0-9]* passed`, which passes while any one test in the
  target runs. This
  spec holds 149's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "149-the-installer-is-tested"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "149-the-installer-is-tested"
amends:
  - "149-the-installer-is-tested"
---

# 181: Exact test counts for 149

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 149's. 149 holds 006, 007, 021, 032, 034, 038, 039, 040, 041, 042 and 139, so their plans resolve here too; the superseded note of 139 names this spec, and the others have no block of their own.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 149:22 | core `attest` | whole | 30 |

"Block line" counts lines inside 149's `verify:cli` fence. The names each
line ran:

- `attest` (whole target): `a_location_that_resolves_but_cannot_be_read_is_an_error`, `a_reference_is_not_attested_territory`, `ac1_attest_is_byte_identical_on_an_unchanged_corpus`, `ac2_recompute_matches_unchanged_then_flags_an_edited_spec`, `ac3_with_coupling_verdict_is_independently_checkable`, `ac5_a_different_tool_version_is_a_named_version_mismatch`, `an_unknown_spec_id_is_not_found`, `an_unresolved_unit_is_recorded_not_refused`, `editing_a_unit_moves_exactly_that_units_hash`, `editing_the_spec_moves_its_source_hash`, `lifecycle_distinguishes_an_absent_key_from_n_a`, `lint_and_compile_verdicts_go_false_for_the_attested_spec`, `recompute_matches_then_names_what_moved`, `recompute_reports_unit_changes_by_identity_not_position`, `spec083_a_crate_unit_attests_over_its_package_root`, `spec083_a_non_utf8_file_is_hashed_rather_than_refused`, `spec083_a_pruned_to_empty_directory_hashes_like_an_empty_one`, `spec083_a_regular_file_unit_does_not_take_the_directory_path`, `spec083_a_symlink_is_recorded_by_target_text_and_never_followed`, `spec083_a_symlink_to_a_directory_is_not_descended_into`, `spec083_a_trailing_slash_file_unit_attests_rather_than_erroring`, `spec083_an_empty_directory_hashes_its_own_path_not_the_empty_input`, `spec083_an_explicit_directory_unit_attests`, `spec083_the_declared_state_root_is_pruned`, `the_byte_check_leaves_a_more_specific_outcome_alone`, `the_facade_accepts_the_text_form_and_decides_on_those_bytes`, `the_facade_refuses_an_unknown_member_and_an_unknown_major`, `the_facade_refuses_both_forms_and_neither`, `the_major_gate_reuses_the_loaders_wording`, `the_payload_is_reproducible`.

## 2. Territory

This spec establishes nothing. It edits 149 only to add spec 082 §3.4's
superseded-acceptance note above 149's block, and the note of 139 to name this spec.

## 3. Behavior

### 3.1 149's block is carried

`spec-spine verify 149` and `verify` of 006, 007, 021, 032, 034, 038, 039, 040, 041, 042 and 139 MUST run this
spec's block, which carries 149's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

149 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 139 keeps its note and gains one line naming this spec. 006, 007, 021, 032, 034, 038, 039, 040, 041 and 042 have no block of their own, so no note.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 149's plan and that of 006, 007, 021, 032, 034, 038, 039, 040, 041, 042 and 139 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

```verify:cli
# ---- carried for 149-the-installer-is-tested (amends_verification), and through it 006, 007, 021, 032, 034, 038, 039, 040, 041, 042, 139; each loose test count names its tests (153 3.2) ----
# 3.1: all seven cases, counted exactly.
sh -c 'sh scripts/test-install.sh > "${TMPDIR:-/tmp}/ss149.out" 2>&1; rc=$?; grep -qx "install.sh: 7 of 7 cases passed" "${TMPDIR:-/tmp}/ss149.out"; g=$?; rm -f "${TMPDIR:-/tmp}/ss149.out"; test $rc -eq 0 && test $g -eq 0'
# 3.2: the npm shim's offline unit test and pack-install-launch smoke test.
sh -c 'cd npm && npm test'
sh -c 'cd npm && npm run smoke'
# 3.3: the ledger no longer lists 006.
sh -c '! grep -qx "006-distribution" scripts/verify-sweep.sh'
# ---- carried for 139-the-legacy-ledger-is-paid-verdicts-and-lifecycle (amends_verification) ----
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(007-python-distribution|021-ledger-seal|032-stdout-closed-reader|034-machine-readable-verdicts|038-completion-held-to-claims|039-per-spec-attestation|040-governance-document-gaps|041-in-progress-is-in-flight|042-absent-implementation-defers-to-status)" scripts/verify-sweep.sh'
# 3.3: exactly the three documented exemptions remain, each with its reason on
# the line above it. Amended by 149 3.3: 006 left the ledger (was four).
sh -c 'n=$(sed -n "/^legacy_ledger()/,/^}/p" scripts/verify-sweep.sh | grep -cE "^[0-9]{3}-"); test "$n" -eq 3'
sh -c 'grep -B1 -x "000-spec-spine-bootstrap" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
# (006's exemption line is gone: 149 3.3, asserted by 149's own line above.)
sh -c 'grep -B1 -x "019-release-supply-chain-artifacts" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
sh -c 'grep -B1 -x "037-amendment-authoring" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
# ---- carried for 007-python-distribution (amends_verification) ----
sh -c 'cd py && out=$(PYTHONPATH=src python3 -m unittest discover -s test -v 2>&1); printf "%s\n" "$out" | grep -q "^test_triples_match_release_yml_matrix .* ok$" && printf "%s\n" "$out" | grep -qx OK'
# ---- carried for 021-ledger-seal (amends_verification) ----
# amended by 039: the per-spec seal lives beside the corpus seal.
# exact on a Unix sweep host with default features: attest has two `#[cfg(unix)]` tests (spec 153 3.3).
sh -c 'cargo test -p spec-spine-core --locked --test attest -- --exact a_location_that_resolves_but_cannot_be_read_is_an_error a_reference_is_not_attested_territory ac1_attest_is_byte_identical_on_an_unchanged_corpus ac2_recompute_matches_unchanged_then_flags_an_edited_spec ac3_with_coupling_verdict_is_independently_checkable ac5_a_different_tool_version_is_a_named_version_mismatch an_unknown_spec_id_is_not_found an_unresolved_unit_is_recorded_not_refused editing_a_unit_moves_exactly_that_units_hash editing_the_spec_moves_its_source_hash lifecycle_distinguishes_an_absent_key_from_n_a lint_and_compile_verdicts_go_false_for_the_attested_spec recompute_matches_then_names_what_moved recompute_reports_unit_changes_by_identity_not_position spec083_a_crate_unit_attests_over_its_package_root spec083_a_non_utf8_file_is_hashed_rather_than_refused spec083_a_pruned_to_empty_directory_hashes_like_an_empty_one spec083_a_regular_file_unit_does_not_take_the_directory_path spec083_a_symlink_is_recorded_by_target_text_and_never_followed spec083_a_symlink_to_a_directory_is_not_descended_into spec083_a_trailing_slash_file_unit_attests_rather_than_erroring spec083_an_empty_directory_hashes_its_own_path_not_the_empty_input spec083_an_explicit_directory_unit_attests spec083_the_declared_state_root_is_pruned the_byte_check_leaves_a_more_specific_outcome_alone the_facade_accepts_the_text_form_and_decides_on_those_bytes the_facade_refuses_an_unknown_member_and_an_unknown_major the_facade_refuses_both_forms_and_neither the_major_gate_reuses_the_loaders_wording the_payload_is_reproducible 2>&1 | grep -q "test result: ok. 30 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact json_verify_attestation_reports_the_signature_mode the_seal_path_follows_the_attestation_it_signs 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
# ---- carried for 032-stdout-closed-reader (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact closed_reader_exits_cleanly_rather_than_panicking no_panicking_stdout_macro_remains_in_the_cli scanner_does_not_let_a_stderr_call_mask_a_stdout_one 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 034-machine-readable-verdicts (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact json_envelope_on_every_adjudicating_verb json_error_path_is_an_envelope_on_stdout json_exit_codes_match_the_prose_form json_report_equals_the_facade_payload 2>&1 | grep -q "test result: ok. 4 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-types --locked --test dtos -- --exact verdict_envelope_round_trips_with_the_documented_members 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 038-completion-held-to-claims (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact a_complete_draft_that_told_the_truth_is_silent completion_defeats_draft_leniency_across_both_axes 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test index_body -- --exact the_completion_claim_is_named_only_when_it_was_made 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 039-per-spec-attestation (amends_verification) ----
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact attest_exits_zero_on_a_false_verdict_in_both_scopes attest_refuses_with_coupling_scoped_to_one_spec attest_spec_writes_signs_and_verifies_one_spec 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test attest -- --exact recompute_reports_unit_changes_by_identity_not_position 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 040-governance-document-gaps (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test scaffold -- --exact scaffolded_constitution_states_an_executable_amendment_mechanism the_constitution_template_is_the_real_one_and_cannot_drift the_scaffolded_contract_carries_the_lifecycle_table_and_extra_keys 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
# ---- carried for 041-in-progress-is-in-flight (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact in_progress_leniency_reports_rather_than_ignores 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test render -- --exact every_arm_of_the_in_flight_predicate_lands_where_044_puts_it 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# ---- carried for 042-absent-implementation-defers-to-status (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test query -- --exact an_absent_implementation_is_scheduled_by_status_and_reports_it plan_reads_an_absent_implementation_key_on_a_draft_as_pending plan_reads_an_absent_implementation_key_on_a_ratified_spec_as_settled 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test scaffold -- --exact scaffolded_corpus_has_nothing_ready_to_schedule 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
```
