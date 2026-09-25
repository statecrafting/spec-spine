---
id: "138-the-legacy-ledger-is-paid-coupling-and-freshness"
title: "The legacy ledger is paid: coupling, coverage and freshness"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  10 specs filed before `verify` existed (005, 008, 027, 028, 029, 030, 031, 033, 035, 036) sit
  on the legacy ledger in `scripts/verify-sweep.sh` with no executable
  acceptance, although the tests that hold their behavior exist and pass. This
  spec carries that acceptance for them under `amends_verification`, the route
  spec 132 used, and deletes their ledger lines in the same change. Every
  command names its tests exactly and asserts the exact pass count, so a
  renamed or deleted test turns the block red instead of passing on an empty
  filter. The area is the coupling gate, ownership coverage, freshness and scheduling.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "089-nothing-reruns-a-merged-acceptance"
amends_verification:
  - "005-coupling-gate"
  - "008-coupling-floor-claim-precedence"
  - "027-cargo-workflow-dependency-waiver"
  - "028-registry-freshness-check"
  - "029-ownership-coverage"
  - "030-dependency-cycle-refusal"
  - "031-references-non-owning-paths"
  - "033-configured-corpus-root"
  - "035-registry-plan-ready-set"
  - "036-declared-state-dir"
amends:
  - "005-coupling-gate"
  - "008-coupling-floor-claim-precedence"
  - "027-cargo-workflow-dependency-waiver"
  - "028-registry-freshness-check"
  - "029-ownership-coverage"
  - "030-dependency-cycle-refusal"
  - "031-references-non-owning-paths"
  - "033-configured-corpus-root"
  - "035-registry-plan-ready-set"
  - "036-declared-state-dir"
extends:
  # 3.2 the ledger lines this spec retires (089 3.4: deleted in the same change).
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 138: The legacy ledger is paid: coupling, coverage and freshness

## 1. Purpose

Spec 089 closed the legacy ledger at ordinal 043: the specs below it were
filed before spec 043 built `verify`, and each was recorded as tracked debt, to
be retired when it gained a `## Verification` block. The release sweep reports
them `exempt`, which counts as success, so their behavior has been asserted
only by the workspace test suite and never by the acceptance a merged revision
is asked for.

For the specs this one carries, the tests exist. Each was found by name in
the owning test file at `8f2a8f75` and passes there. What was missing is a
declared acceptance that runs them and fails when they stop running.

## 2. Territory

This spec establishes nothing. It amends the acceptance of each target through
`amends_verification` and edits no target's file (spec 037). It extends 089's
`scripts/verify-sweep.sh` only to delete the retired ledger lines.

## 3. Behavior

### 3.1 The carried acceptance

`spec-spine verify <target>` MUST run this spec's `## Verification` block for
each target listed in `amends_verification` (spec 082). The block carries one
section per target, headed `carried for <id>`.

Each command runs one test binary and either:

- names its tests with `--exact` and asserts the exact number that passed, so a
  renamed or deleted test fails the command; or
- runs a whole test file that the target owns and asserts a non-zero pass
  count with no failures, because `cargo test` exits 0 when a filter matches
  nothing.

Where a later spec amended a target, the carried commands check the behavior
as amended: the tests are the ones that pass at HEAD, and a comment names the
amender.

### 3.2 The ledger shrinks in the same change

Each target's line MUST be deleted from `legacy_ledger()` in
`scripts/verify-sweep.sh` in the change that adds its block (089 3.4). The sweep
already refuses a listed spec that declares acceptance, so the two cannot drift
apart.

## 4. Out of scope

**New tests.** Every command runs a test that already exists. Where a target's
behavior has no test at all, it is not in this series.

**The sweep's cost.** The sweep runs a holder's whole block once per target it
carries. The block runs in about 8 seconds on a warm tree, which is why the
targets are split across four holders of about ten each rather than one holder
of 39 (D-1).

## 5. Resolved decisions

**D-1 (2026-09-25): four holders, not one and not 39.** The sweep runs this
block for each target and for this spec, 11 times. One holder of all 39 targets
would run a block of about 80 commands 40 times; 39 holders would add 39 specs
to the corpus for one line each. Four themed holders keep the added sweep time
near six minutes. The themes follow the ledger's own areas.

**D-2 (2026-09-25): exact names over filters.** A substring filter from the
drafting proposal (for example `short_id_`) was expanded to the exact names it
matched at `8f2a8f75`, so the pass count is exact. Whole-file runs are kept only
where the file is the target's own.

**D-3 (2026-09-25): fail-first by mutation.** The behavior is already built, so
a block cannot be shown red against an unbuilt tree. The pull request records
one mutation per holder that turns a carried command red, plus the two controls
every command relies on: a missing test name, and a missing test file, each
exit 1.

## Verification

```verify:cli
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(005-coupling-gate|008-coupling-floor-claim-precedence|027-cargo-workflow-dependency-waiver|028-registry-freshness-check|029-ownership-coverage|030-dependency-cycle-refusal|031-references-non-owning-paths|033-configured-corpus-root|035-registry-plan-ready-set|036-declared-state-dir)" scripts/verify-sweep.sh'
# ---- carried for 005-coupling-gate (amends_verification) ----
# amended by 008, 027, 033, 071 and 100 among others: the whole coupling suite at HEAD is the amended behavior.
sh -c 'cargo test -p spec-spine-core --locked --test couple 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
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
sh -c 'cargo test -p spec-spine-core --locked --test coverage 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
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
