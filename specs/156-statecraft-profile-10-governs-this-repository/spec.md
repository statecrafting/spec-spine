---
id: "156-statecraft-profile-10-governs-this-repository"
title: "Statecraft Profile 11 governs this repository"
status: draft
kind: "tooling"
created: "2026-09-27"
summary: >
  Statecraft is the adopted owner of the managed development environment, but
  this repository still carries its pre-adoption CI workflow, AI review and
  instruction surface. Profile 11 is adopted at one recorded identity, with
  its exact spec-spine pin and managed authority files, while the Windows,
  MSRV, cargo-deny, no-default-features, self-governance and determinism checks
  particular to this engine remain required behind the one ci-gate status.
implementation: complete
owner: "The spec-spine Authors"
risk: critical
depends_on:
  - "092-the-engine-ships-governance-not-an-environment"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "124-the-expansion-line-carries-its-own-version"
  - "150-a-change-runs-the-acceptance-it-can-break"
# 094 sections 3.1, 3.3 through 3.5 and 4.1 put the canonical local gate in the
# root Makefile and make .github/workflows/ci.yml its caller and aggregate.
# Section 3.2 below moves that canonical gate and caller to Statecraft's
# managed surface while preserving the one-status and explicit-control rules.
# 124 section 3.2 requires a one-directional floor. Section 3.3 below replaces
# it here with the exact pin Profile 11 requires for a qualified adoption.
# 091 sections 3.3, 3.6.1 and 3.8 bind the retired review workflow's credential,
# timeout and embedded-script mechanics. Section 3.2 below replaces those
# mechanics with Profile 11's managed review while preserving strict blocking.
amends:
  - "091-an-unclassified-review-failure-blocks-the-merge"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "098-a-citation-the-renumber-could-not-see"
  - "124-the-expansion-line-carries-its-own-version"
  - "134-the-suite-runs-on-windows"
  - "135-the-toolchain-floor-and-dependencies-are-checked"
amends_sections:
  - "3-3-class-2-secret-unset-unchanged"
  - "3-6-1-a-timeout-is-not-a-recognized-transient-signal"
  - "3-8-the-policy-test-executes-the-workflow-s-own-script"
  - "3-1-the-target"
  - "3-3-the-ownership-assertion-is-conditional-and-every-skip-is-announced"
  - "3-4-the-coupling-step-has-explicit-controls"
  - "3-5-the-gate-holds-on-a-corpus-with-no-code"
  - "4-1-the-pull-request-and-push-legs-call-the-target"
  - "5-resolved-decisions"
  - "3-2-the-floor-moves-with-the-version"
  - "3-1-a-windows-test-check-of-its-own"
  - "3-1-the-declared-floor-is-built-and-tested"
  - "3-2-the-dependencies-are-checked"
amends_verification:
  - "091-an-unclassified-review-failure-blocks-the-merge"
  - "094-one-gate-and-the-boundaries-it-holds"
  - "124-the-expansion-line-carries-its-own-version"
  - "134-the-suite-runs-on-windows"
  - "135-the-toolchain-floor-and-dependencies-are-checked"
establishes:
  - { kind: file, path: ".statecraft/AGENTS.md" }
  - { kind: file, path: ".statecraft/environment.json" }
  - { kind: file, path: ".statecraft/setup/github-actions-rust.json" }
  - { kind: file, path: ".github/workflows/statecraft-ci.yml" }
  - { kind: file, path: ".github/workflows/statecraft-ai-review.yml" }
  - { kind: file, path: ".github/workflows/spec-spine-required.yml" }
  - { kind: file, path: ".github/workflows/ai-pr-review.yml" }
  - { kind: directory, path: "scripts/statecraft/" }
  - { kind: file, path: "scripts/check-authored-content.sh" }
  - { kind: file, path: "README.md" }
extends:
  # Statecraft inserts the bridge that 092 section 3.10 reserves to it.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: "AGENTS.md" }, nature: additive }
  # Profile 11 updates the repository-local delivery instructions as one set.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: directory, path: ".claude/skills/" }, nature: corrective }
  # The exact consumer pin deliberately replaces 124's moving floor.
  - { spec: "061-shipped-is-not-the-same-as-working", unit: { kind: file, path: "spec-spine.toml" }, nature: corrective }
  # The managed workflow replaces this repository-owned predecessor.
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: file, path: ".github/workflows/ci.yml" }, nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: file, path: "crates/spec-spine-core/tests/gate.rs" }, nature: corrective }
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: { kind: file, path: "crates/spec-spine-core/tests/ai_review_policy.rs" }, nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: "crates/spec-spine-core/tests/harness_skills.rs" }, nature: corrective }
  - { spec: "105-governed-scope-is-enabled-here", unit: { kind: file, path: "CLAUDE.md" }, nature: corrective }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/adoption-guide.md" }, nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: file, path: "Makefile" }, nature: corrective }
  - { spec: "135-the-toolchain-floor-and-dependencies-are-checked", unit: { kind: file, path: "crates/spec-spine-core/Cargo.toml" }, nature: corrective }
references:
  - { unit: { kind: file, path: "CODEOWNERS" }, role: "existing-authority-ownership" }
  - { unit: { kind: file, path: ".github/workflows/determinism.yml" }, role: "required-extension" }
  - { unit: { kind: file, path: ".github/workflows/acceptance.yml" }, role: "unchanged" }
  - { unit: { kind: file, path: "Makefile" }, role: "repository-owned-gate" }
intent:
  goal: "Adopt Statecraft Profile 11 without weakening this repository's governed checks or transferring its product gate to generated policy."
  non_goals:
    - "Ratifying this draft or publishing a spec-spine or Statecraft release."
    - "Changing repository settings, secrets, environments or branch protection."
    - "Replacing scheduled and pre-release acceptance sweeps."
---

# 156: Statecraft Profile 11 governs this repository

## 1. Purpose

### 1.1 The owner named in 092 has arrived

Spec 092 removed the development-environment distribution from spec-spine and
named Statecraft CLI as its sole owner. It deliberately left this repository's
own harness in place until Statecraft's global delivery concretely replaced it.
That replacement now exists as Statecraft's `github-actions-rust` Profile 11.

Measured on 2026-09-27 from Statecraft commit
`7c59640ba8f4c98819bf2b7bce54acd71e26c4f3`, its profile revision is 11 and its
profile identity is
`25e77bc9f95c2cef638d3b282b573b0c481572a9f72187c520f8caa956f56b6f`.
The source-built CLI passed its own governed gate. Its read-only plan against
this repository recognized the existing corpus and refused the managed files
because `spec-spine.toml` carries a version range rather than the exact pin the
profile requires. The refusal is a useful result: this is an adoption with an
explicit amendment, not an initializer run that quietly chooses policy.

### 1.2 Adoption must not narrow the gate

The current `.github/workflows/ci.yml` does more than a generic Rust build. It
runs the workspace on Windows, tests the declared minimum Rust version, runs
cargo-deny, proves `spec-spine-core` without default features, governs this
repository with the binary the candidate builds, and calls the cross-triple
determinism workflow. Spec 094 makes `ci-gate` the one required aggregate.

Profile 11 has a typed extension point for this exact case:
`ci.extra_required_jobs`. Each declared job calls a repository-owned reusable
workflow, is included in `ci-gate`'s needs, and blocks on failure, cancellation
or skip. The adoption uses that extension point rather than dropping checks or
maintaining a second aggregate.

### 1.3 The pin is intentionally different here

Spec 124 section 3.2 requires this repository's own `required_version` to be a
moving `>=` floor. Profile 11 requires an exact spec-spine pin before it will
render or qualify its managed files. Both cannot be true at once. This spec
amends 124 for this repository: while Profile 11 owns the environment, the
configuration carries an exact released engine version. A later package
version change and its Statecraft profile update move that pin together.

## 2. Territory

This spec establishes the Statecraft environment record, policy, instruction
file, managed workflows and scripts. It establishes one
repository-owned reusable workflow for the engine-specific CI checks and one
authored-content checker. It adds Statecraft's import bridge to `AGENTS.md`,
changes the version constraint in `spec-spine.toml`, and reduces the old
top-level CI and review workflows to non-triggered endpoints (D-6). `CODEOWNERS`
stays repository-owned (D-3).

The root `Makefile` remains a repository-owned developer interface, but it is
no longer the canonical local or CI gate. `.github/workflows/determinism.yml`,
the scheduled acceptance workflow, release workflow, repository skills and
repository agents remain repository-owned.

## 3. Behavior

### 3.1 One recorded Profile 11 adoption

The committed Statecraft environment MUST name profile `github-actions-rust`,
revision 11, and profile identity
`25e77bc9f95c2cef638d3b282b573b0c481572a9f72187c520f8caa956f56b6f`.
The generated policy and managed files MUST be the result of the Statecraft CLI
built from commit `7c59640ba8f4c98819bf2b7bce54acd71e26c4f3`; they MUST NOT be
hand-authored approximations of that profile.

The profile parameters MUST declare:

- governance coverage enforcement on;
- authored-content checking through `scripts/check-authored-content.sh`,
  including pull-request title and body and commit messages;
- every commit gated;
- signed commits required;
- the default branch required as the pull-request base;
- unresolved ownership claims refused;
- `@bartekus` as the code owner;
- a review diff cap of 3000;
- `determinism` calling `.github/workflows/determinism.yml` and
  `spec-spine-required` calling
  `.github/workflows/spec-spine-required.yml` as extra required jobs.

The Statecraft plan after parameter selection MUST be free of withheld files
and conflicts before apply. Apply MUST use that plan's identity. A subsequent
plan on the applied tree MUST report no pending managed-file change.

### 3.2 Profile 11 owns the aggregate and the local gate

`.github/workflows/statecraft-ci.yml` MUST be the only workflow that defines a
job named `ci-gate`. It MUST run for pull requests, pushes to the default branch
and merge groups. Its aggregate MUST require the profile's governance, code,
AI-review and owner-exception jobs plus both declared extra jobs.

The Profile 11 `scripts/statecraft/gate.sh` MUST be the canonical definition of
the local and CI governance and code gates. The managed workflow MUST call that
script's named modes rather than restating their commands. The script MUST
retain the read-only governance floor and explicit coupling controls required
by spec 094, use only the exact repository-local engine pin, and expose the
profile's family exit contract.

The root `Makefile` remains a repository-owned developer and compatibility
interface. It does not define the Profile 11 gate and MUST NOT be treated as
the authority for what `ci-gate` runs. Local Profile 11 qualification MUST run
the managed governance and code modes, with the coupling mode run when frozen
base and head identities are available.

The old `.github/workflows/ci.yml` and `.github/workflows/ai-pr-review.yml`
MUST remain non-triggered compatibility endpoints because approved ownership
records still claim those paths. They MUST define only `workflow_call`, MUST
define no `ci-gate` job, and MUST retain no CI, review or secret-forwarding
behavior. Two `ci-gate` jobs MUST never coexist on the branch.

The managed review replaces spec 091's retired workflow mechanics. Its
repository-owned regression test MUST execute the managed
`scripts/statecraft/ai-review.sh`, not reconstruct its classifier. An absent
credential on a same-repository pull request is a refusal rather than a skip,
and the managed job carries a 15 minute bound. The load-bearing policy remains:
access refusal outranks a transient signal, an empty or malformed review
blocks, publication failure blocks, and every unclassified reviewer failure
blocks.

### 3.3 The profile pin is exact

`spec-spine.toml` `[meta] required_version` MUST equal `=0.28.0` for this
adoption. The binary at that version MUST be available from its public release
channel before the profile is applied. The Statecraft install script MUST read
the exact constraint and MUST refuse a range, an unqualified version or a
different version.

This replaces spec 124 section 3.2 only for the consumer pin in this repository.
Spec 124 sections 3.1 and 3.3 through 3.5 continue to govern package identity,
release steps, binary evidence and fixtures. A later spec-spine version bump
MUST update the exact pin and requalify the Statecraft plan in the same change.

### 3.4 The specialized Rust matrix remains required

`.github/workflows/spec-spine-required.yml` MUST be a reusable workflow and
MUST run all behavior from the predecessor CI workflow that Profile 11's code
job does not cover:

1. build, test and clippy for `spec-spine-core` with default features off, plus
   an assertion that tree-sitter is absent from that dependency graph;
2. the workspace test suite on Windows;
3. the workspace build and test at the `rust-version` declared in
   `Cargo.toml`;
4. cargo-deny;
5. the self-governance job using the candidate-built `spec-spine` binary and
   the event-specific coupling controls specified by 094 section 4.1.

The existing reusable determinism workflow MUST remain a separate extra job.
Both extra jobs MUST be required on pull-request, push and merge-group events.
A skipped extra job blocks the aggregate exactly as a failed or cancelled one
does.

### 3.5 Authored content rejects the two publication hazards

`scripts/check-authored-content.sh` MUST implement the interface Profile 11
declares for an authored-content checker. It MUST refuse:

- the Unicode em dash character in governed authored content, commit messages,
  and pull-request title or body;
- a URL formed by concatenating `https://codex.ai/` and `code/session_`,
  case-insensitively, in any repository content, commit message, pull-request
  title or body.

It MUST distinguish a finding from a usage error or execution failure using
the repository's exit contract. It MUST carry offline tests for clean input,
each forbidden form, filenames containing spaces, an empty range, and invalid
arguments. Generated derived shards and Git object data not selected by the
profile MUST NOT be searched as authored source.

### 3.6 The instruction bridge preserves local authority

Statecraft MUST write `.statecraft/AGENTS.md` and insert
`@.statecraft/AGENTS.md` as the first line of the root `AGENTS.md`, as spec 092
section 3.10 reserves to it. The imported instructions MUST NOT delete or
weaken the project rules that follow. In particular, the coherence guard,
owner delegation, governed artifact read rule, one-session-one-spec rule and
the repository's own gate remain binding.

### 3.7 Local qualification precedes remote activation

The implementation MUST finish with the Statecraft plan converged, the full
repository gate green, this spec's verification green, and the affected
acceptance sweep green on the exact branch head. Those results qualify the
committed adoption only.

Branch protection, merge queue settings, the
`statecraft-review-exception` Environment and its reviewers, Actions token
permissions, credentials, secrets, publication, merge and ratification are
owner actions outside this spec. The handoff MUST report each observed remote
obligation and whether it is already satisfied, without reading secret values.
Until those owner actions are complete, the profile is implemented locally but
not activated or qualified on GitHub.

## 4. Out of scope

- Ratifying this draft, merging its implementation or publishing any release.
- Reading, creating or changing secret values or invoking a paid AI provider.
- Applying repository settings, branch protection, merge queue or Environment
  changes.
- Replacing `.github/workflows/acceptance.yml`, the pre-release sweep or the
  local affected-acceptance requirement from spec 150.
- Replacing the repository's local skills and agents merely because the
  Statecraft instruction bridge exists.
- Removing the root `Makefile` or changing its repository-owned commands.

## 5. Resolved decisions

**D-1 (2026-09-27, owner): adopt Profile 11 through an amendment spec rather
than weakening its prerequisite or editing approved specs in place.** The
owner approved the critical-risk frontmatter naming amendments to specs 094
and 124. The implementation preserves the repository-specific CI matrix as
required Profile 11 extensions and stops before remote activation.

**D-2 (2026-09-27, owner): the full-tree authored-content baseline is clean.** Three
legacy punctuation marks in spec 098 were replaced with semantically neutral
punctuation. The owner directs this amendment of spec 098's resolved-decision
prose because the repository-wide authored-content rule forbids those marks in
all governed content. No requirement, lifecycle field, citation or verification changed.
The forbidden URL is described here as two adjacent fragments so the governing
spec does not itself contain the content its checker refuses.

**D-3 (2026-09-27): the existing root `CODEOWNERS` remains repository-owned.**
It already assigns `@bartekus` to the whole repository and explicitly names the
CI, scripts and spec corpus surfaces. Statecraft therefore correctly leaves it
alone instead of rendering a duplicate under `.github/`.

**D-4 (2026-09-27, owner): adopting Profile 11 replaces the retired AI review
mechanics without weakening their blocking invariant.** The owner's Profile 11
approval includes its explicit missing-credential refusal, external managed
script and 15 minute job bound. Spec 091's core correction remains enforced:
an unclassified failure cannot become a successful review.

**D-5 (2026-09-27, owner): Profile 11 owns localhost gate behavior.** The owner
approved the Profile 11 gate model after its generated surface showed that the
managed workflow calls `scripts/statecraft/gate.sh` rather than `make gate`.
That script is the canonical local and CI gate, while `ci-gate` remains the one
required GitHub status and the root `Makefile` remains a repository-owned
developer and compatibility interface.

**D-6 (2026-09-27): legacy workflow paths remain compatibility endpoints.**
The first index after replacement exposed settled claims on `ci.yml`; deleting
it would require editing approved owning specs, which this build cannot do.
Profile 11 does not manage either legacy path. Both therefore remain as
non-triggered, inert compatibility workflows, with no second `ci-gate`, no
secrets and no copied implementation. This preserves the approved Profile 11 gate model and the
corpus's ownership history together.

**D-7 (2026-09-27): this amendment carries the acceptance for specs 091, 094,
124, 134 and 135.** Their original verification asserted the retired workflow
layout, the former Makefile-owned gate or the moving version floor. Profile 11
deliberately replaces those surfaces. The replacement checks below preserve
their blocking, explicit control, hook, merge-driver, ownership, version,
Windows, MSRV and dependency-audit invariants against the managed gate and
required workflow. Their original Verification sections retain their commands
and name this replacement block, as spec 082 requires.

**D-8 (2026-09-27): the Profile 11 aggregate diagnostic remains byte-exact.**
Review correctly found that the rendered `ci-gate` diagnostic can conflate an
unreadable base SHA with an ordinary failing dependency. This repository does
not hand-edit a managed artifact because doing so would break the exact profile
identity. The governance and code jobs already refuse an unreadable base, so
the aggregate cannot report success in that state. The definitive diagnostic
correction belongs in Statecraft Profile 12, followed by a fresh exact render.

## Verification

```verify:cli
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
sh -c 'out="$(cargo test -p spec-spine-core --locked --test ai_review_policy access_refusal 2>&1)" && printf "%s\n" "$out" | grep -qE "test result: ok\. [1-9][0-9]* passed"'
sh -c 'out="$(cargo test -p spec-spine-core --locked --test ai_review_policy publication_failure 2>&1)" && printf "%s\n" "$out" | grep -qE "test result: ok\. [1-9][0-9]* passed"'
sh -c 'out="$(cargo test -p spec-spine-core --locked --test ai_review_policy unclassified_reviewer_failure 2>&1)" && printf "%s\n" "$out" | grep -qE "test result: ok\. [1-9][0-9]* passed"'
sh -c 'out="$(cargo test -p spec-spine-core --locked --test ai_review_policy empty_successful_review 2>&1)" && printf "%s\n" "$out" | grep -qE "test result: ok\. [1-9][0-9]* passed"'
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
