---
id: "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
title: "The legacy ledger is paid: verdicts, attestation and lifecycle"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  9 specs filed before `verify` existed (007, 021, 032, 034, 038, 039, 040, 041, 042) sit
  on the legacy ledger in `scripts/verify-sweep.sh` with no executable
  acceptance, although the tests that hold their behavior exist and pass. This
  spec carries that acceptance for them under `amends_verification`, the route
  spec 132 used, and deletes their ledger lines in the same change. Every
  command names its tests exactly and asserts the exact pass count, so a
  renamed or deleted test turns the block red instead of passing on an empty
  filter. The area is verdicts, attestation, lifecycle leniency, governance documents and the PyPI shim.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "089-nothing-reruns-a-merged-acceptance"
amends_verification:
  - "007-python-distribution"
  - "021-ledger-seal"
  - "032-stdout-closed-reader"
  - "034-machine-readable-verdicts"
  - "038-completion-held-to-claims"
  - "039-per-spec-attestation"
  - "040-governance-document-gaps"
  - "041-in-progress-is-in-flight"
  - "042-absent-implementation-defers-to-status"
amends:
  - "007-python-distribution"
  - "021-ledger-seal"
  - "032-stdout-closed-reader"
  - "034-machine-readable-verdicts"
  - "038-completion-held-to-claims"
  - "039-per-spec-attestation"
  - "040-governance-document-gaps"
  - "041-in-progress-is-in-flight"
  - "042-absent-implementation-defers-to-status"
extends:
  # 3.2 the ledger lines this spec retires (089 3.4: deleted in the same change).
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---
# 139: The legacy ledger is paid: verdicts, attestation and lifecycle

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

### 3.3 The four that stay, and why

Four ledger entries remain, and each carries its reason as a `# exempt:`
comment on the line above it. They are not debt this series could pay with
existing tests:

- **000-spec-spine-bootstrap**: tier-1 constitution: its claims (determinism, the authoring/derived boundary, typed authority) are what the whole gate chain and determinism.yml test on every change, so no one block can own them.
- **006-distribution**: the npm half is spec 007's neighbour and runs in npm test, but install.sh, the tag-gated release pipeline and npm publication have no test that can run on a merged revision; its acceptance is the release run (docs/releasing.md).
- **019-release-supply-chain-artifacts**: the SBOM and SLSA provenance are produced only by a tag-triggered release run with OIDC; verify runs on a merged revision, so the acceptance is the release run itself (docs/releasing.md).
- **037-amendment-authoring**: a process rule about authors (declare amends, do not edit the amended file): what it forbids is an edit in a pull request, which no check on a merged tree can see; its enforceable descendants 082, 083, 095 and 118 carry acceptance.

The ledger stays closed at 043 and still only shrinks (089 3.4). A future spec
that gives one of the four an executable acceptance deletes its line as this
one did.

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

**D-4 (2026-09-25): 043's example is spec 140's.** Spec 043's acceptance and
one of its tests use 041 as the corpus's standing example of a spec with no
executable acceptance. Carrying 041 here makes that false, so spec 140, filed in
the same change, moves the example to 037.

## Verification

```verify:cli
# 3.2: every target's line is gone from the legacy ledger (the sweep also refuses a stale entry).
sh -c '! grep -Eqx "(007-python-distribution|021-ledger-seal|032-stdout-closed-reader|034-machine-readable-verdicts|038-completion-held-to-claims|039-per-spec-attestation|040-governance-document-gaps|041-in-progress-is-in-flight|042-absent-implementation-defers-to-status)" scripts/verify-sweep.sh'
# 3.3: exactly the four documented exemptions remain, each with its reason on the line above it.
sh -c 'n=$(sed -n "/^legacy_ledger()/,/^}/p" scripts/verify-sweep.sh | grep -cE "^[0-9]{3}-"); test "$n" -eq 4'
sh -c 'grep -B1 -x "000-spec-spine-bootstrap" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
sh -c 'grep -B1 -x "006-distribution" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
sh -c 'grep -B1 -x "019-release-supply-chain-artifacts" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
sh -c 'grep -B1 -x "037-amendment-authoring" scripts/verify-sweep.sh | head -1 | grep -q "^# exempt: "'
# ---- carried for 007-python-distribution (amends_verification) ----
sh -c 'cd py && out=$(PYTHONPATH=src python3 -m unittest discover -s test -v 2>&1); printf "%s\n" "$out" | grep -q "^test_triples_match_release_yml_matrix .* ok$" && printf "%s\n" "$out" | grep -qx OK'
# ---- carried for 021-ledger-seal (amends_verification) ----
# amended by 039: the per-spec seal lives beside the corpus seal.
sh -c 'cargo test -p spec-spine-core --locked --test attest 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed; 0 failed"'
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
