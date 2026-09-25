---
id: "137-the-legacy-ledger-is-paid-index-and-lint"
title: "The legacy ledger is paid: index, lint and resolution"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  10 specs filed before `verify` existed (003, 004, 010, 011, 016, 020, 022, 023, 024, 025) sit
  on the legacy ledger in `scripts/verify-sweep.sh` with no executable
  acceptance, although the tests that hold their behavior exist and pass. This
  spec carries that acceptance for them under `amends_verification`, the route
  spec 132 used, and deletes their ledger lines in the same change. Every
  command names its tests exactly and asserts the exact pass count, so a
  renamed or deleted test turns the block red instead of passing on an empty
  filter. The area is the codebase index, lint, section and symbol resolution, and sharding.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "089-nothing-reruns-a-merged-acceptance"
amends_verification:
  - "003-conformance-lint"
  - "004-codebase-index"
  - "010-index-render-orphans"
  - "011-index-hash-slices"
  - "016-directory-crate-module-units"
  - "020-keypath-section-anchors"
  - "022-index-sharding"
  - "023-unresolved-unit-severity"
  - "024-resolution-discovery-fixes"
  - "025-symbol-resolution-feature-gate"
amends:
  - "003-conformance-lint"
  - "004-codebase-index"
  - "010-index-render-orphans"
  - "011-index-hash-slices"
  - "016-directory-crate-module-units"
  - "020-keypath-section-anchors"
  - "022-index-sharding"
  - "023-unresolved-unit-severity"
  - "024-resolution-discovery-fixes"
  - "025-symbol-resolution-feature-gate"
extends:
  # 3.2 the ledger lines this spec retires (089 3.4: deleted in the same change).
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 137: The legacy ledger is paid: index, lint and resolution

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
sh -c '! grep -Eqx "(003-conformance-lint|004-codebase-index|010-index-render-orphans|011-index-hash-slices|016-directory-crate-module-units|020-keypath-section-anchors|022-index-sharding|023-unresolved-unit-severity|024-resolution-discovery-fixes|025-symbol-resolution-feature-gate)" scripts/verify-sweep.sh'
# ---- carried for 003-conformance-lint (amends_verification) ----
# amended by 116: the deferred-contract exemption from L-001 is part of what lint does now.
sh -c 'cargo test -p spec-spine-core --locked --test lint 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test coverage -- --exact lint_diagnostic_codes_are_unique 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test deferred_contract 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
# ---- carried for 004-codebase-index (amends_verification) ----
# amended by 023, 031, 038, 041, 069 and 079 among others: the whole index suite at HEAD is the amended behavior.
sh -c 'cargo test -p spec-spine-core --locked --test index 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
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
