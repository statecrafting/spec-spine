---
id: "210-profile-28-governs-this-repository"
title: "Profile 28 governs this repository, and the merge queue is declared"
status: approved
implementation: complete
kind: tooling
created: "2026-11-07"
owner: "The spec-spine Authors"
risk: medium
summary: >
  The repository runs Statecraft's github-actions-rust profile at revision 13.
  Revision 28 adds five required jobs (preflight, supply-chain, workflow-lint,
  sg-scan, secret-scan), a policy-isolation guard, a reviewer guide, a remote
  desired-state record and a merge-queue rule for post-merge pushes. This spec
  adopts it in three pull requests: a preparation under revision 13 that
  declares the queue, gives every extra job the post_merge input and clears
  the workflow lint; a change that hashes the files the render adds; and the
  render itself. Spec 191's aggregate of seven jobs and its 15-minute review
  become twelve jobs and 20 minutes.
amends:
  - "191-profile-13-keeps-the-engine-local"
  - "208-the-0-32-0-release-moves-the-pin"
amends_sections: ["3"]
amends_verification:
  - "208-the-0-32-0-release-moves-the-pin"
depends_on:
  - "157-the-affected-sweep-runs-in-the-merge-queue"
  - "191-profile-13-keeps-the-engine-local"
  - "208-the-0-32-0-release-moves-the-pin"
  - "209-every-registry-package-carries-provenance"
establishes:
  # Written by the revision 28 render (3.1).
  - { kind: file, path: ".statecraft/REVIEW.md" }
  - { kind: file, path: ".statecraft/setup/github-actions-rust.remote.json" }
  # 3.4: what the workflow lint accepts (pull request 2, with its hash).
  - { kind: file, path: ".github/zizmor.yml" }
extends:
  # 2: the three files above enter [index] extra_hashed_inputs (L-008).
  - { spec: "061-shipped-is-not-the-same-as-working", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
  # 3.3: the queue declaration in the recorded parameters.
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/environment.json", nature: additive }
  # 3.3: every extra required job declares the post_merge input.
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/spec-spine-required.yml", nature: additive }
  - { spec: "157-the-affected-sweep-runs-in-the-merge-queue", unit: ".github/workflows/affected-acceptance.yml", nature: additive }
  - { spec: "203-the-determinism-build-restores-a-cargo-cache", unit: ".github/workflows/determinism.yml", nature: additive }
  # 3.4: the repository workflows pass the workflow lint.
  - { spec: "099-a-merged-acceptance-is-asked-again", unit: ".github/workflows/acceptance.yml", nature: corrective }
  - { spec: "006-distribution", unit: { kind: file, path: ".github/workflows/release.yml" }, nature: corrective }
  # 3.1: the render (pull request 3).
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/setup/github-actions-rust.json", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ci.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ai-review.yml", nature: corrective }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: "scripts/statecraft/", nature: corrective }
  # D-5: the two tests that read the rendered files.
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: "crates/spec-spine-core/tests/ai_review_policy.rs", nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: corrective }
---

# 210: Profile 28 governs this repository, and the merge queue is declared

## 1. Purpose

Spec 191 adopted revision 13 of Statecraft's `github-actions-rust` profile.
The owner directed on 2026-11-07 that this repository move to revision 28
(statecraft-cli spec 072, #430) and skip revision 27, and declare its merge
queue as rustev and statecraft-cli do. Revision 13 cannot record that
declaration: `remote.merge_queue` arrives at revision 16, and from revision
24 a declared queue requires every extra required job to accept the boolean
`workflow_call` input `post_merge`, or the plan refuses.

Revision 28 also requires what this repository did not yet meet: zizmor, run
by the new workflow-lint job, reports findings in three repository-owned
workflows; and the render's two new files are governance inputs that no
content hash covers.

## 2. Territory

Pull request 1 (this one, under revision 13): the queue parameter in
`.statecraft/environment.json`; the `post_merge` input in `determinism.yml`,
`spec-spine-required.yml` and `affected-acceptance.yml`; the zizmor
corrections in `acceptance.yml`, `determinism.yml` and `release.yml`; the
zizmor configuration `.github/zizmor.yml`; and this spec, whose block asserts
what this pull request makes true (3.5).

Pull request 2: `spec-spine.toml` alone with this spec, adding the render's
two new files and `.github/zizmor.yml` to `[index] extra_hashed_inputs`, and
claiming `.github/zizmor.yml` here.

Pull request 3: the render by `init plan` and `init apply`, the two tests
that read the rendered scripts, the carried acceptance of spec 208 (3.5),
and this spec to `implementation: complete`.

## 3. Behavior

### 3.1 Revision 28 is the recorded profile

The recorded profile is `github-actions-rust` revision 28, identity
`ddb55e6ee7c1478b1c88463a3f2fa7a53b3f20aad492a7dfcae29cbbd4d21178`, with the
parameters spec 191 recorded and none removed. Every managed file is
recorded at revision 28, including `.statecraft/REVIEW.md` and
`.statecraft/setup/github-actions-rust.remote.json`, which the render
writes. The exact engine pin stays `=0.32.0`.

The two managed workflows the plan reports as customized take the profile's
bytes: `statecraft-ci.yml` (spec 191's adoption edits; their intent is
carried by 3.2 and 3.5 rather than by hand edits to the managed file) and
`statecraft-ai-review.yml` (a Dependabot pin bump; the profile's pin
replaces it).

### 3.2 The aggregate requires twelve jobs (amends 191 3)

Spec 191 3 requires "all seven jobs". Under revision 28, `ci-gate` needs
twelve: governance, preflight, code, supply-chain, workflow-lint, sg-scan,
secret-scan, ai-review, review-exception, and the three extra required jobs
191 kept (determinism, spec-spine-required, affected-acceptance). The managed
AI review allows 20 minutes (revision 20), not 15. Everything else 191 3
states is unchanged: `governance.require_ratified` stays false, and the
three extra jobs stay required.

### 3.3 The merge queue is declared

`.statecraft/environment.json` records `remote.merge_queue` as
`{required: true, merge_method: squash, build_concurrency: 5,
all_entries_must_pass: true}`, matching the branch protection the owner set
on 2026-11-07. The render records it in
`.statecraft/setup/github-actions-rust.remote.json`. `determinism.yml`,
`spec-spine-required.yml` and `affected-acceptance.yml` each declare the
`workflow_call` input `post_merge` (boolean, default false). They accept it
and change nothing on it: on a post-merge push they still run in full.

### 3.4 The repository workflows pass the workflow lint

zizmor 1.31.1, as revision 28 runs it (`zizmor --offline --format plain
.github/workflows`), reports no finding:

- every `actions/checkout` in `acceptance.yml`, `determinism.yml` and
  `release.yml` sets `persist-credentials: false`; no job in them pushes, and
  the acceptance sweep's `git fetch` reads a public repository;
- `acceptance.yml` passes event and step values to its scripts through
  `env:`, never by template expansion into a `run:` script, and refuses a
  dispatch `only` input outside the spec-id characters `[0-9a-z,-]`, so it
  cannot write a second step output;
- `.github/zizmor.yml` accepts two findings in `release.yml`, each with its
  reason: the integrity-checked npm tarball install (spec 209 3.1.2), and
  `softprops/action-gh-release`, whose replacement is a release-path change
  of its own.

### 3.5 Acceptance

Pull request 3 declares spec 208 in `amends_verification` and carries its
complete block. Five lines change: the profile identity and its digest, the
policy revision, the `ci-gate` needs line (3.2) and the review timeout
(3.2); lines for 3.1 join them. Until then this spec's block asserts 3.3 and
3.4, which pull request 1 makes true, and 208's block keeps running for the
chain it holds (D-6).

## 4. Out of scope

- Skipping work on a post-merge push in the three extra jobs. They accept
  `post_merge` and ignore it; using it is a later change.
- Replacing `softprops/action-gh-release` or the npm tarball install.
- The owner's remote settings, which this repository's files do not change.

## Decisions

**2026-11-08: ratified by the owner.** This pull request flips the spec
from `draft` to `approved`. Ratification takes effect with the owner's
approval of this pull request; the owner exception approved on its run is
the ratification of record (spec 198). The owner also directed that the
first push after #530, which revision 28 blocks once by design, is cleared
as the profile directs.

**D-1 (2026-11-07, owner).** Skip revision 27 for 28; sequence the move as
a preparation under revision 13, then the render; take the profile's bytes
for both customized workflows; no exemption and no amendment of spec 124.

**D-2 (2026-11-07).** The owner named `determinism.yml` and
`affected-acceptance.yml` for the `post_merge` input. The revision 28 plan
refuses unless every declared extra job accepts it, which includes
`spec-spine-required.yml`, so all three carry it.

**D-3 (2026-11-07).** The render adds two claimed files, and L-008 refuses
a claimed file no content hash covers. Pull request 1 may not touch
`spec-spine.toml` (D-1), and the render may not either: revision 28's guard,
which runs from the candidate during the adoption, refuses `spec-spine.toml`
beside any path other than its owning spec. So the hashing is its own pull
request between them, where the guard does not yet run.

**D-4 (2026-11-07).** zizmor's workflow-lint job is new at revision 22, and
a local run of the revision 28 render found two high, six medium, one low and
three informational findings in repository-owned workflows. 3.4 clears them
in pull request 1, where they are prerequisites of the render.

**D-5 (2026-11-07).** Against the render, `ai_review_policy.rs` fails three
of ten tests (its fixture predates revisions 20 and 26) and `gate.rs`'s
`no_workflow_step_restates_a_verb_the_one_gate_definition_runs` fails on
`cargo deny --locked check all` and a comment, neither a spec-spine verb.
Pull request 3 corrects both tests; `harness_skills.rs` passes unchanged.

**D-6 (2026-11-07).** Pull request 1 first carried 208's block here. The
merge queue refused it: an `amends_verification` edge moves the plan of every
spec in the chain at the moment it merges (187 D-1), so 187, which is
implemented, ran this block, whose revision 28 lines cannot hold before the
render, and failed at its first command. The carried block, the edge and the
chain's labels move to pull request 3, the change that makes the block true.

## Verification

```verify:cli
# ---- carried for 208-the-0-32-0-release-moves-the-pin (amends_verification), and through it 199, 195, 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the profile moves to revision 28 and the aggregate names its twelve jobs (210 3.5) ----
# ---- carried for 199-the-0-31-0-release-moves-the-pin (amends_verification), and through it 195, 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.32.0 together (208 3.1) ----
# ---- carried for 195-the-0-30-0-release-moves-the-pin (amends_verification), and through it 193, 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.31.0 together (199 3.1) ----
# ---- carried for 193-the-0-29-0-release-moves-the-pin (amends_verification), and through it 191, 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.30.0 together (195 3.1) ----
# ---- carried for 191-profile-13-keeps-the-engine-local (amends_verification), and through it 190, 187, 156, 091, 094, 124, 134, 135; the package version and the exact pin move to 0.29.0 together (193 3.1) ----
# ---- carried for 187-exact-test-counts-for-156 (amends_verification), and through it 156, 091, 094, 124, 134, 135; the ci-gate needs line names affected-acceptance (190 3.2) ----
# ---- carried for 156-statecraft-profile-10-governs-this-repository (amends_verification), and through it 091, 094, 124, 134, 135; each loose test count names its tests (153 3.2) ----
# 3.1 and 3.3: the committed environment names Profile 28 and the exact engine pin.
grep -q '"identity": "statecraft-setup:github-actions-rust@28"' .statecraft/environment.json
grep -q 'ddb55e6ee7c1478b1c88463a3f2fa7a53b3f20aad492a7dfcae29cbbd4d21178' .statecraft/environment.json
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
# 3.1 and 3.7: Profile 28 has a committed policy and no withheld managed file.
grep -q '"revision": 28' .statecraft/setup/github-actions-rust.json
sh -c '! grep -q '"'"'"status"[[:space:]]*:[[:space:]]*"withheld"'"'"' .statecraft/setup/github-actions-rust.json'
# 193 3.1: the package version and the exact pin name the same release, and the environment record agrees.
grep -q '"spec_spine": "0.32.0"' .statecraft/environment.json
sh -c 'test "$(sed -n "s/^required_version = \"=\\(.*\\)\"$/\\1/p" spec-spine.toml)" = "$(sed -n "s/^version = \"\\(.*\\)\"$/\\1/p" Cargo.toml | head -n 1)"'
# 210 3.1: every managed file is recorded at revision 28, and the render's new files exist.
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
```
