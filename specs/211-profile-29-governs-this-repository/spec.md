---
id: "211-profile-29-governs-this-repository"
title: "Profile 29 governs this repository"
status: draft
implementation: complete
kind: tooling
created: "2026-11-26"
owner: "The spec-spine Authors"
risk: low
summary: >
  The repository runs Statecraft's github-actions-rust profile at revision 28.
  Revision 29 (statecraft-cli spec 073) gives the policy-isolation guard a
  new-crate class, so a new workspace crate that adds no dependency lands in
  one pull request; any registry or git package change stays policy. This
  spec renders it and carries spec 210's acceptance with the profile at
  revision 29.
amends:
  - "210-profile-28-governs-this-repository"
amends_sections: ["3"]
amends_verification:
  - "210-profile-28-governs-this-repository"
depends_on:
  - "210-profile-28-governs-this-repository"
extends:
  # 3.1: the render.
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/setup/github-actions-rust.json", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ci.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ai-review.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: "scripts/statecraft/", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/environment.json", nature: corrective }
  - { spec: "210-profile-28-governs-this-repository", unit: { kind: file, path: ".statecraft/REVIEW.md" }, nature: corrective }
  - { spec: "210-profile-28-governs-this-repository", unit: { kind: file, path: ".statecraft/setup/github-actions-rust.remote.json" }, nature: corrective }
---

# 211: Profile 29 governs this repository

## 1. Purpose

Spec 210 adopted revision 28 of Statecraft's `github-actions-rust` profile.
The owner directed on 2026-11-26 that every Statecraft-governed repository
move to revision 29 (statecraft-cli spec 073, #443). Under revision 28 a
workspace whose members are a glob cannot add a crate without an admin
merge: the manifest alone breaks the build, and beside its sources the guard
reads policy and other paths together.

## 2. Territory

One pull request: the render by `init plan` and `init apply` and the
carried acceptance of spec 210 (3.2), and the "Superseded acceptance" note
above 210's block. That note is the one edit to 210, and it is required:
`every_superseded_verification_block_says_so_in_its_own_document` in
`crates/spec-spine-core/tests/verify.rs` fails on a block an
`amends_verification` edge supersedes unless its own document says so. No
other spec is edited; the edge records the rest, and `registry
relationships` reports it.

## 3. Behavior

### 3.1 Revision 29 is the recorded profile

The recorded profile is `github-actions-rust` revision 29, identity
`9d5bb0e25975598a07bcb7ed7f0466a25dd725508cd050dbdbbec6f4950dcebe`, with the
parameters spec 210 recorded and none removed. Every managed file is
recorded at revision 29. `scripts/statecraft/guard-policy-files.sh` gains the
new-crate class: a `Cargo.toml` in a directory the merge base holds nothing
in, whose every dependency inherits from the workspace or is a path to a
workspace crate, and a `Cargo.lock` whose only change is that crate's
sourceless entry, are ignored like generated paths. Every other managed file
changes only its revision header and profile identity. The exact engine pin
stays `=0.32.0`, the `ci-gate` aggregate stays at spec 210's twelve jobs, and
the merge queue declaration is unchanged.

### 3.2 Acceptance

This spec declares spec 210 in `amends_verification` and carries its
complete block. Four lines change: the profile identity, its digest, the
policy revision and the comments naming the revision; one line for 3.1's
guard class joins them.

## 4. Out of scope

- Remote settings, which this repository's files do not change.
- The engine pin, which moves only with a release.

## Decisions

**D-1 (2026-11-26, owner).** Adopt revision 29 in every Statecraft-governed
repository. Revision 29 changes only the policy-isolation guard, so the move
is one pull request.

## Verification

```verify:cli
# ---- carried for 210-profile-28-governs-this-repository (amends_verification), and through it 208, 199, 195, 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the profile moves to revision 29 (211 3.1) ----
# ---- carried for 208-the-0-32-0-release-moves-the-pin (amends_verification), and through it 199, 195, 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the profile moves to revision 28 and the aggregate names its twelve jobs (210 3.5) ----
# ---- carried for 199-the-0-31-0-release-moves-the-pin (amends_verification), and through it 195, 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.32.0 together (208 3.1) ----
# ---- carried for 195-the-0-30-0-release-moves-the-pin (amends_verification), and through it 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.31.0 together (199 3.1) ----
# ---- carried for 193-the-0-29-0-release-moves-the-pin (amends_verification), and through it 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.30.0 together (195 3.1) ----
# ---- carried for 191-profile-13-keeps-the-engine-local (amends_verification), and through it 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.29.0 together (193 3.1) ----
# ---- carried for 187-exact-test-counts-for-156 (amends_verification), and through it 156, 091, 094, 124, 134, 135; the ci-gate needs line names affected-acceptance (190 3.2) ----
# ---- carried for 156-statecraft-profile-10-governs-this-repository (amends_verification), and through it 091, 094, 124, 134, 135; each loose test count names its tests (153 3.2) ----
# 3.1 and 3.3: the committed environment names Profile 29 and the exact engine pin.
grep -q '"identity": "statecraft-setup:github-actions-rust@29"' .statecraft/environment.json
grep -q '9d5bb0e25975598a07bcb7ed7f0466a25dd725508cd050dbdbbec6f4950dcebe' .statecraft/environment.json
grep -q '^required_version = "=0.32.0"$' spec-spine.toml
# The exact pin amendment carries 124's package identity and release evidence.
cargo build --release --locked
python3 scripts/bump_version.py --check 0.32.0
"$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/release/spec-spine" --version | grep -qx 'spec-spine 0.32.0'
grep -qF 'The floor moves with the version' docs/releasing.md
grep -qF 'A version names one behavior' docs/releasing.md
scripts/reader-identity.sh "$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/release/spec-spine" . > "${TMPDIR:-/tmp}/ss156-reader.txt"
grep -qE '^answers +spec-spine 0\.32\.0$' "${TMPDIR:-/tmp}/ss156-reader.txt"
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
grep -q 'timeout-minutes: 20' .github/workflows/statecraft-ai-review.yml
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
grep -q 'needs: \[governance, preflight, code, supply-chain, workflow-lint, sg-scan, secret-scan, ai-review, review-exception, determinism, spec-spine-required, affected-acceptance\]' .github/workflows/statecraft-ci.yml
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
# 3.1 and 3.7: Profile 29 has a committed policy and no withheld managed file.
grep -q '"revision": 29' .statecraft/setup/github-actions-rust.json
sh -c '! grep -q '"'"'"status"[[:space:]]*:[[:space:]]*"withheld"'"'"' .statecraft/setup/github-actions-rust.json'
# 193 3.1: the package version and the exact pin name the same release, and the environment record agrees.
grep -q '"spec_spine": "0.32.0"' .statecraft/environment.json
sh -c 'test "$(sed -n "s/^required_version = \"=\\(.*\\)\"$/\\1/p" spec-spine.toml)" = "$(sed -n "s/^version = \"\\(.*\\)\"$/\\1/p" Cargo.toml | head -n 1)"'
# 210 3.1 and 211 3.1: every managed file is recorded at revision 29, and the render's new files exist.
sh -c '! grep -q "github-actions-rust@13" .statecraft/environment.json'
test -f .statecraft/REVIEW.md
test -f .statecraft/setup/github-actions-rust.remote.json
test -f scripts/statecraft/guard-policy-files.sh
test -f scripts/statecraft/verify-action-pins.sh
grep -q '"mergeMethod": "squash"' .statecraft/setup/github-actions-rust.remote.json
grep -q '"allEntriesMustPass": true' .statecraft/setup/github-actions-rust.remote.json
sh scripts/statecraft/verify-action-pins.sh
# 210 3.3: the queue is declared, and every extra required job takes post_merge.
grep -q '"remote.merge_queue"' .statecraft/environment.json
grep -q '"merge_method": "squash"' .statecraft/environment.json
grep -q '"all_entries_must_pass": true' .statecraft/environment.json
sh -c 'for f in determinism spec-spine-required affected-acceptance; do sed -n "/^on:/,/^permissions:/p" .github/workflows/$f.yml | grep -q "^      post_merge:" || { echo "no post_merge input: $f"; exit 1; }; done'
# 210 3.4: no repository workflow interpolates an event value into a run script, and none persists the token.
sh -c '! grep -E "^ +(if |then|else|scripts/|echo |--out )" .github/workflows/acceptance.yml | grep -qF "\${{ github.event"'
sh -c '! grep -E "^ +(if |then|else|scripts/|echo |--out )" .github/workflows/acceptance.yml | grep -qF "\${{ steps.scope"'
sh -c 'for f in acceptance determinism release; do n=$(grep -c "uses: actions/checkout@" .github/workflows/$f.yml); m=$(grep -c "persist-credentials: false" .github/workflows/$f.yml); test "$n" = "$m" || { echo "$f: $n checkouts, $m without credentials"; exit 1; }; done'
test -f .github/zizmor.yml
grep -q '^  adhoc-packages:' .github/zizmor.yml
grep -q '^  superfluous-actions:' .github/zizmor.yml
grep -qF '*[!0-9a-z,-]*)' .github/workflows/acceptance.yml
# 211 3.1: the policy-isolation guard carries revision 29's new-crate class.
grep -q 'new crate (ignored)' scripts/statecraft/guard-policy-files.sh
```
