---
id: "136-the-legacy-ledger-is-paid-registry-and-grammar"
title: "The legacy ledger is paid: registry and grammar"
status: approved
kind: "tooling"
created: "2026-09-25"
summary: >
  10 specs filed before `verify` existed (001, 002, 009, 012, 013, 014, 015, 017, 018, 026) sit
  on the legacy ledger in `scripts/verify-sweep.sh` with no executable
  acceptance, although the tests that hold their behavior exist and pass. This
  spec carries that acceptance for them under `amends_verification`, the route
  spec 132 used, and deletes their ledger lines in the same change. Every
  command names its tests exactly and asserts the exact pass count, so a
  renamed or deleted test turns the block red instead of passing on an empty
  filter. The area is the registry compiler, its query surface, and the frontmatter and edge grammar.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "089-nothing-reruns-a-merged-acceptance"
amends_verification:
  - "001-compile-registry"
  - "002-registry-query"
  - "009-registry-query-projection-flags"
  - "012-declared-extra-frontmatter-passthrough"
  - "013-edge-paths-grammar-sugar"
  - "014-establishes-wrapper-na-alias"
  - "015-short-id-resolution"
  - "017-constrains-discriminator-optional-unit"
  - "018-structured-partial-supersedes"
  - "026-references-provenance-derived-at"
amends:
  - "001-compile-registry"
  - "002-registry-query"
  - "009-registry-query-projection-flags"
  - "012-declared-extra-frontmatter-passthrough"
  - "013-edge-paths-grammar-sugar"
  - "014-establishes-wrapper-na-alias"
  - "015-short-id-resolution"
  - "017-constrains-discriminator-optional-unit"
  - "018-structured-partial-supersedes"
  - "026-references-provenance-derived-at"
extends:
  # 3.2 the ledger lines this spec retires (089 3.4: deleted in the same change).
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 136: The legacy ledger is paid: registry and grammar

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
sh -c '! grep -Eqx "(001-compile-registry|002-registry-query|009-registry-query-projection-flags|012-declared-extra-frontmatter-passthrough|013-edge-paths-grammar-sugar|014-establishes-wrapper-na-alias|015-short-id-resolution|017-constrains-discriminator-optional-unit|018-structured-partial-supersedes|026-references-provenance-derived-at)" scripts/verify-sweep.sh'
# ---- carried for 001-compile-registry (amends_verification) ----
sh -c 'cargo test -p spec-spine-core --locked --test compile 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
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
# ---- carried for 026-references-provenance-derived-at (amends_verification) ----
sh -c 'cargo test -p spec-spine-types --locked --test grammar -- --exact references_provenance_derived_at_round_trips references_provenance_unknown_field_is_rejected references_provenance_without_derived_at_omits_the_field 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
```
