---
id: "191-profile-13-keeps-the-engine-local"
title: "Profile 13 keeps the exact engine local"
status: draft
implementation: complete
kind: tooling
created: "2026-10-02"
summary: >
  Adopts Statecraft revision 13 without narrowing the engine-specific checks
  or retiring the repository-owned merge-queue acceptance sweep.
amends:
  - "156-statecraft-profile-10-governs-this-repository"
  - "157-the-affected-sweep-runs-in-the-merge-queue"
  - "190-the-gate-requires-the-queue-sweep"
amends_verification:
  - "190-the-gate-requires-the-queue-sweep"
depends_on:
  - "190-the-gate-requires-the-queue-sweep"
extends:
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/spec-spine-required.yml", nature: corrective }
  - { spec: "188-a-repository-pin-selects-the-engine", unit: "crates/spec-spine-launcher/tests/", nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_skills.rs", nature: corrective }
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: "crates/spec-spine-core/tests/ai_review_policy.rs", nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: "AGENTS.md", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/environment.json", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/setup/github-actions-rust.json", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ci.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ai-review.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: "scripts/statecraft/", nature: corrective }
---

# 191: Profile 13 keeps the exact engine local

## 1. Purpose

The owner requested the fleet upgrade on 2026-10-02. Approved spec 156
records revision 11, while the adopted producer now renders revision 13.
This amendment preserves the old contract and declares its replacement.

## 2. Territory

The managed profile, environment record and repository instructions change.
The specialized self-governance job stages its candidate-built engine at
`.bin/spec-spine` before invoking the same managed gate.
The review regression fixture supplies the three budget inputs now required
by the managed script, preserving its existing refusal and failure assertions. The harness gate inventory
also names revision 13's ownership query and distinguishes command verbs from
the query's positional path.
The launcher fixture compares the discovered repository with its canonical
path, including macOS's `/var` to `/private/var` alias, as the engine already
does. No product API or engine behavior changes. Statecraft `8f718e2` renders
all managed bytes with the previously recorded parameters.

## 3. Behavior

The recorded profile is `github-actions-rust` revision 13. The exact engine
pin remains `=0.28.0`; its regular executable lives at `.bin/spec-spine`.
The managed governance, coupling and code modes remain canonical. Signed
commits, authored-content checks, coverage and owner review remain required.

This repository builds draft specs and ratifies them in a separate PR.
Revision 11 does not refuse draft-owned implementation. The revision 13
parameter `governance.require_ratified` is therefore explicitly false here,
preserving that approved working model rather than silently enabling a
conflicting default. A status transition to approved still requires the
owner Environment review.

The determinism, specialized Rust matrix and affected-acceptance reusable
workflows remain extra required jobs. Revision 13 currently relocates the
engine and does not supply spec 157's proposed per-event sweep replacement.
Its interim affected-acceptance workflow therefore remains required until a
producer revision actually carries equivalent acceptance. The unchanged
aggregate requires all seven jobs, including that sweep.

## 4. Acceptance amendment

The complete block of spec 190 is carried below. Only the recorded profile
revision and identity, and the obsolete temporary .tooling installer fixture,
change. Compiled acceptance paths resolve Cargo's configured target directory
to support the required shared worktree target. Before invoking the managed gate,
acceptance stages the candidate-built engine at `.bin/spec-spine`. Isolated
merge-queue checkouts do not inherit ignored local installations. This keeps
acceptance on the exact candidate engine.

## Verification

```verify:cli
# ---- carried for 187-exact-test-counts-for-156 (amends_verification), and through it 156, 091, 094, 124, 134, 135; the ci-gate needs line names affected-acceptance (190 3.2) ----
# ---- carried for 156-statecraft-profile-10-governs-this-repository (amends_verification), and through it 091, 094, 124, 134, 135; each loose test count names its tests (153 3.2) ----
# 3.1 and 3.3: the committed environment names Profile 13 and the exact engine pin.
grep -q '"identity": "statecraft-setup:github-actions-rust@13"' .statecraft/environment.json
grep -q '6167a12e110b30ec27d37909344c7de0178ba477d5ad1ab91a0053430a89e17b' .statecraft/environment.json
grep -q '^required_version = "=0.28.0"$' spec-spine.toml
# The exact pin amendment carries 124's package identity and release evidence.
cargo build --release --locked
python3 scripts/bump_version.py --check 0.28.0
"$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/release/spec-spine" --version | grep -qx 'spec-spine 0.28.0'
grep -qF 'The floor moves with the version' docs/releasing.md
grep -qF 'A version names one behavior' docs/releasing.md
scripts/reader-identity.sh "$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/release/spec-spine" . > "${TMPDIR:-/tmp}/ss156-reader.txt"
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
sh -c '"$(cargo metadata --no-deps --format-version 1 | python3 -c '"'"'import json,sys; print(json.load(sys.stdin)["target_directory"])'"'"')/release/spec-spine" index owner Makefile | grep -qF "094-one-gate-and-the-boundaries-it-holds"'
cargo test -p spec-spine-core --locked --test gate
sh -c 'engine="$(cargo metadata --no-deps --format-version 1 | python3 -c '"'"'import json,sys; print(json.load(sys.stdin)["target_directory"])'"'"')/release/spec-spine"; mkdir -p .bin && cp "$engine" .bin/spec-spine.acceptance && chmod +x .bin/spec-spine.acceptance && mv .bin/spec-spine.acceptance .bin/spec-spine && sh scripts/statecraft/gate.sh governance'
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
# 3.1 and 3.7: Profile 13 has a committed policy and no withheld managed file.
grep -q '"revision": 13' .statecraft/setup/github-actions-rust.json
sh -c '! grep -q '"'"'"status"[[:space:]]*:[[:space:]]*"withheld"'"'"' .statecraft/setup/github-actions-rust.json'
```
