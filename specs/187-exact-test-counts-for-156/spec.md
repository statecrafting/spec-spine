---
id: "187-exact-test-counts-for-156"
title: "Exact test counts for 156"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 156's acceptance block carries four lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 156's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "151-carried-acceptance-tests-what-it-names"
  - "156-statecraft-profile-10-governs-this-repository"
amends_verification:
  - "156-statecraft-profile-10-governs-this-repository"
amends:
  - "156-statecraft-profile-10-governs-this-repository"
---

# 187: Exact test counts for 156

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 156's. 156 holds 091, 094, 124, 134 and 135, so their plans resolve here too; the superseded notes of 091, 094, 124, 134 and 135 name this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 156:29 | core `ai_review_policy` | `access_refusal` | 1 |
| 156:30 | core `ai_review_policy` | `publication_failure` | 1 |
| 156:31 | core `ai_review_policy` | `unclassified_reviewer_failure` | 1 |
| 156:32 | core `ai_review_policy` | `empty_successful_review` | 1 |

"Block line" counts lines inside 156's `verify:cli` fence. The names each
line ran:

- `ai_review_policy` `access_refusal`: `access_refusal_outranks_other_failure_text`.
- `ai_review_policy` `publication_failure`: `publication_failure_blocks`.
- `ai_review_policy` `unclassified_reviewer_failure`: `unclassified_reviewer_failure_blocks`.
- `ai_review_policy` `empty_successful_review`: `empty_successful_review_blocks`.

## 2. Territory

This spec establishes nothing. It edits 156 only to add spec 082 §3.4's
superseded-acceptance note above 156's block, and the notes of 091, 094, 124, 134 and 135 to name this spec.

## 3. Behavior

### 3.1 156's block is carried

`spec-spine verify 156` and `verify` of 091, 094, 124, 134 and 135 MUST run this
spec's block, which carries 156's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

156 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 091, 094, 124, 134 and 135 keep their notes and gain one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 156's plan and that of 091, 094, 124, 134 and 135 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

## Verification

> **Superseded acceptance (2026-09-29).** This block no longer runs.
> `190-the-gate-requires-the-queue-sweep` declares this spec in
> `amends_verification`, so `spec-spine verify 187` builds its plan from that
> spec's block, where these commands are carried with the `ci-gate` needs line
> naming `affected-acceptance`, which spec 157 added (spec 082 3.2 and 3.4).
>
> The commands below are kept verbatim and are not corrected (spec 037 3.1).

> **Superseded acceptance (2026-10-02).** This block no longer runs.
> `191-profile-13-keeps-the-engine-local` carries the complete acceptance
> through the existing amendment chain for profile 13. The commands below
> remain unchanged.

> Since 2026-10-05, `193-the-0-29-0-release-moves-the-pin` holds 191's block in turn, so this plan runs
> from that spec's block (spec 193 3.3).

```verify:cli
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
grep -q 'needs: \[governance, code, ai-review, review-exception, determinism, spec-spine-required\]' .github/workflows/statecraft-ci.yml
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
