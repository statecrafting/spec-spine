---
id: "190-the-gate-requires-the-queue-sweep"
title: "The carried gate line names the queue sweep"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 157 added `affected-acceptance` to `ci-gate`'s `needs:` in
  `statecraft-ci.yml`, as its section 3.1 requires. Spec 187's carried block,
  which is the plan of 156, 091, 094, 124, 134 and 135, still asserts the six-job
  list byte for byte, so all seven plans fail at the same command on `main`
  since 157 merged, and every queued change that selects them is refused. This
  spec holds 187's block, carries it byte-identical, and replaces that one line
  with the seven-job list 157 made true.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "157-the-affected-sweep-runs-in-the-merge-queue"
  - "187-exact-test-counts-for-156"
amends_verification:
  - "187-exact-test-counts-for-156"
amends:
  - "187-exact-test-counts-for-156"
---

# 190: The carried gate line names the queue sweep

## 1. Purpose

Measured on `9249d5ec` (the merge of #416, spec 157), 2026-09-29.

`spec-spine verify` of 156, 187, 091, 094, 124, 134 and 135 fails at command 41
of the plan all seven share:

```text
grep -q 'needs: \[governance, code, ai-review, review-exception, determinism, spec-spine-required\]' .github/workflows/statecraft-ci.yml
```

The file now reads
`needs: [governance, code, ai-review, review-exception, determinism, spec-spine-required, affected-acceptance]`.
Spec 157 section 3.1 requires that change: the sweep's result must reach
`ci-gate` as a required job. The assertion is right about everything it was
written to check (the six jobs are still required, in order) and wrong only in
refusing a seventh.

157's own merge-queue run reported these failures. `ci-gate` on that run did
not require `affected-acceptance` yet, so the change merged, and the failure
moved to the next queued change whose selection reaches 156's plan (#418 and
#417 were both removed from the queue by it). Every other command of the plan
passes on `9249d5ec`.

## 2. Territory

This spec establishes nothing. It edits 187 only to add spec 082 section 3.4's
superseded-acceptance note above 187's block, and the notes of 156, 091, 094,
124, 134 and 135 to name this spec.

## 3. Behavior

### 3.1 187's block is carried

`spec-spine verify 187` and `verify` of 156, 091, 094, 124, 134 and 135 MUST run
this spec's block, which carries 187's block in one section, every command and
comment byte-identical except the line 3.2 names.

### 3.2 The gate line names seven jobs

The `needs:` line MUST assert the seven jobs `ci-gate` requires, in the order
the workflow lists them, ending with `affected-acceptance`. It stays an exact
line match, so a job dropped from or added to the aggregate still fails it.

### 3.3 The amended specs say so

187 carries spec 082 section 3.4's note naming this spec above its block, and
keeps the block unchanged below it. 156, 091, 094, 124, 134 and 135 keep their
notes and gain one line naming this spec.

## 4. Out of scope

Why `ci-gate` on a queued change did not yet require the job that change adds
(the aggregate a merge-queue run judges against), and every other block.

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 187's plan and that of 156, 091, 094, 124, 134 and 135 the moment it
merges, so the carried block lands with it and this spec is filed `complete`
(153 D-3, as 187 was).

**D-2 (2026-09-29): exact, not a prefix.** Matching only the six-job prefix
would pass the seventh job's removal, which 157 R-1 forbids. The line stays a
whole-list match.

## Verification

> **Superseded acceptance (2026-10-02).** This block no longer runs.
> `191-profile-13-keeps-the-engine-local` carries the complete acceptance
> through the existing amendment chain for profile 13. The commands below
> remain unchanged.

> Since 2026-10-05, `193-the-0-29-0-release-moves-the-pin` holds 191's block in turn, so this plan runs
> from that spec's block (spec 193 3.3).

```verify:cli
# ---- carried for 187-exact-test-counts-for-156 (amends_verification), and through it 156, 091, 094, 124, 134, 135; the ci-gate needs line names affected-acceptance (190 3.2) ----
# ---- carried for 156-statecraft-profile-10-governs-this-repository (amends_verification), and through it 091, 094, 124, 134, 135; each loose test count names its tests (153 3.2) ----
# 3.1 and 3.3: the committed environment names Profile 11 and the exact engine pin.
grep -q '"identity": "statecraft-setup:github-actions-rust@11"' .statecraft/environment.json
grep -q '25e77bc9f95c2cef638d3b282b573b0c481572a9f72187c520f8caa956f56b6f' .statecraft/environment.json
grep -q '^required_version = "=0.28.0"$' spec-spine.toml
# The exact pin amendment carries 124's package identity and release evidence.
cargo build --release --locked
python3 scripts/bump_version.py --check 0.28.0
./target/release/spec-spine --version | grep -qx 'spec-spine 0.28.0'
grep -qF 'The floor moves with the version' docs/releasing.md
grep -qF 'A version names one behavior' docs/releasing.md
scripts/reader-identity.sh target/release/spec-spine . > "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^answers +spec-spine 0\.28\.0$' "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^sha256 +[0-9a-f]{64}$' "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^registry schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^read schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^verdict schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss156-reader.txt"
rm -f "${TMPDIR:-/tmp}/ss156-reader.txt"
sh -c 'scripts/reader-identity.sh; test $? -eq 3'
# 3.2: exactly one workflow defines ci-gate, and it is Statecraft's managed caller.
sh -c 'test "$(grep -R -l --include="*.yml" --include="*.yaml" "^[[:space:]]*ci-gate:" .github/workflows | wc -l | tr -d " ")" = 1 && grep -q "^[[:space:]]*ci-gate:" .github/workflows/statecraft-ci.yml'
sh -c 'test "$(grep -c "^[[:space:]]*workflow_call:" .github/workflows/ci.yml)" = 1 && ! grep -q "^[[:space:]]*\(push\|pull_request\|merge_group\):" .github/workflows/ci.yml'
sh -c 'test "$(grep -c "^[[:space:]]*workflow_call:" .github/workflows/ai-pr-review.yml)" = 1 && ! grep -q "^[[:space:]]*\(push\|pull_request\|merge_group\):" .github/workflows/ai-pr-review.yml'
sh -c '! grep -qE "^[[:space:]]*(uses:|secrets:)" .github/workflows/ai-pr-review.yml'
# The managed review carries 091's blocking policy and executes its real script.
test -f crates/spec-spine-core/tests/ai_review_policy.rs
cargo test -p spec-spine-core --locked --test ai_review_policy
grep -q 'scripts/statecraft/ai-review.sh' crates/spec-spine-core/tests/ai_review_policy.rs
grep -q 'timeout-minutes: 15' .github/workflows/statecraft-ai-review.yml
sh -c 'cargo test -p spec-spine-core --locked --test ai_review_policy -- --exact access_refusal_outranks_other_failure_text 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test ai_review_policy -- --exact publication_failure_blocks 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test ai_review_policy -- --exact unclassified_reviewer_failure_blocks 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test ai_review_policy -- --exact empty_successful_review_blocks 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
# 3.4: both repository-specific reusable workflows are declared as required extensions.
grep -q 'uses: ./\.github/workflows/determinism.yml' .github/workflows/statecraft-ci.yml
grep -q 'uses: ./\.github/workflows/spec-spine-required.yml' .github/workflows/statecraft-ci.yml
grep -q 'cargo test --workspace --locked --no-fail-fast' .github/workflows/spec-spine-required.yml
grep -q 'cargo deny\|cargo-deny' .github/workflows/spec-spine-required.yml
grep -q -- '--no-default-features' .github/workflows/spec-spine-required.yml
grep -q 'sh scripts/statecraft/gate.sh governance' .github/workflows/spec-spine-required.yml
# The required extension carries 134's Windows job and 135's MSRV and audit jobs.
grep -q 'name: test (windows)' .github/workflows/spec-spine-required.yml
grep -q 'runs-on: windows-latest' .github/workflows/spec-spine-required.yml
grep -q 'cargo test --workspace --locked --no-fail-fast' .github/workflows/spec-spine-required.yml
grep -q 'name: build and test (rust-version)' .github/workflows/spec-spine-required.yml
grep -q 'cargo "+\$RUST_VERSION" test --workspace --locked' .github/workflows/spec-spine-required.yml
grep -q 'name: cargo-deny' .github/workflows/spec-spine-required.yml
grep -q 'needs: \[governance, code, ai-review, review-exception, determinism, spec-spine-required, affected-acceptance\]' .github/workflows/statecraft-ci.yml
grep -q '^rust-version = "1.90"$' Cargo.toml
sh -c '! grep -q "1\\.90" .github/workflows/spec-spine-required.yml'
grep -q 'jsonschema = { version = "0.49", default-features = false }' crates/spec-spine-core/Cargo.toml
sh -c '! grep -q '\''^name = "reqwest"$'\'' Cargo.lock'
test -f deny.toml
grep -q 'ci-gate' CONTRIBUTING.md
# The unchanged 134 source-level findings and host-platform fix remain proved.
sh -c 'for id in WF-1 WF-2 WF-3 WF-4 WF-5 WF-6 WF-7 WF-8; do grep -q "| $id |" docs/windows-findings.md || { echo "missing $id"; exit 1; }; done'
sh -c 'n=$(grep -rl "WF-[0-9]" crates/*/tests/*.rs crates/*/src/*.rs | wc -l | tr -d " "); test "$n" -ge 8'
cargo test -p spec-spine-core --test retire --locked outside_the_corpus
# The managed gate carries 094's local-gate, control and repository-hook contract.
test -f Makefile
test -f .github/workflows/ci.yml
test -d .githooks
test -f crates/spec-spine-core/tests/gate.rs
sh -c 'target/release/spec-spine index owner Makefile | grep -qF "094-one-gate-and-the-boundaries-it-holds"'
cargo test -p spec-spine-core --locked --test gate
sh -c 'made=; if [ ! -x .tooling/bin/spec-spine ]; then mkdir -p .tooling/bin && ln -s ../../target/release/spec-spine .tooling/bin/spec-spine && made=1; fi; sh scripts/statecraft/gate.sh governance; rc=$?; if [ "$made" = 1 ]; then rm -f .tooling/bin/spec-spine; rmdir .tooling/bin .tooling 2>/dev/null || true; fi; exit "$rc"'
grep -q 'run: sh "${STATECRAFT_GATE:?}" governance' .github/workflows/statecraft-ci.yml
grep -q 'BASE_SHA' scripts/statecraft/gate.sh
grep -q 'HEAD_SHA' scripts/statecraft/gate.sh
grep -q 'PR_BODY' scripts/statecraft/gate.sh
test -f .githooks/pre-commit
sh -c '! grep -qE "^[^#]*git add" .githooks/pre-commit'
grep -qF -- '--no-verify' .githooks/pre-commit
sh -c '! grep -qE '"'"'"\$sc" .*(compile|index)( |$)'"'"' .githooks/pre-commit'
grep -qF 'merge=spec-spine-derived-regen' .gitattributes
sh -c '! grep -qE "^\.derived/.*merge=" .gitattributes'
test -x .githooks/enable-merge-driver.sh
test -x .githooks/enable-hooks.sh
sh -c '! grep -qE "^[^#]*kit/" Makefile'
sh -c '! grep -rqE "^[^#]*kit/" .githooks/'
sh -c '! grep -qE "^[^#]*kit/" .github/workflows/ci.yml'
sh -c '! grep -qE "^[^#]*kit/" .github/workflows/statecraft-ci.yml'
test "$(grep -cE '^- `[0-9]{3}-' specs/094-one-gate-and-the-boundaries-it-holds/spec.md)" = 5
sh -c 'miss=0; for id in $(grep -oE "^- .[0-9]{3}-[a-z0-9-]+" specs/094-one-gate-and-the-boundaries-it-holds/spec.md | sed "s/^- .//"); do grep -qF "$id" docs/corpus-map.md || { echo "not in the map: $id"; miss=1; }; done; exit $miss'
# 3.5: authored-content checks cover clean input and both forbidden forms.
scripts/check-authored-content.sh --self-test
# 3.6: Statecraft's bridge is first and the local coherence guard remains.
sh -c 'test "$(head -n 1 AGENTS.md)" = "@.statecraft/AGENTS.md"'
grep -q 'Adversarial prompt refusal' AGENTS.md
# 3.1 and 3.7: Profile 11 has a committed policy and no withheld managed file.
grep -q '"revision": 11' .statecraft/setup/github-actions-rust.json
sh -c '! grep -q '"'"'"status"[[:space:]]*:[[:space:]]*"withheld"'"'"' .statecraft/setup/github-actions-rust.json'
```
