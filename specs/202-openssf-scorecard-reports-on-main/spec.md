---
id: "202-openssf-scorecard-reports-on-main"
title: "OpenSSF Scorecard reports on main"
status: approved
kind: "tooling"
created: "2026-10-21"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "019-release-supply-chain-artifacts"
summary: >
  An unmanaged `scorecard.yml` runs the OpenSSF Scorecard action on every push
  to `main`, weekly, and on any branch-protection change, uploads its SARIF to
  code scanning and publishes the result to the OpenSSF API. Every action is
  pinned to a commit SHA. It is a report, not a gate: no pull request waits on
  it and `ci-gate` does not need it.
establishes:
  - ".github/workflows/scorecard.yml"
references:
  - { unit: { kind: file, path: ".github/workflows/statecraft-ci.yml" }, role: "ci-gate's needs list, which this workflow stays out of" }
  - { unit: { kind: file, path: ".github/dependabot.yml" }, role: "proposes SHA bumps for the pinned actions" }
---

# 202: OpenSSF Scorecard reports on main

## 1. Purpose

Spec 019 gives each release an SBOM and a provenance attestation. Nothing
measures the repository's own supply-chain posture over time: branch
protection, token permissions, pinned dependencies, signed releases, and the
other checks the OpenSSF Scorecard runs. Scorecard does, and its result in
code scanning makes a regression visible on the commit that caused it.

## 2. Territory

- **`.github/workflows/scorecard.yml`**: new, unmanaged. It is not in
  `.statecraft/environment.json`, and no Statecraft profile renders it.

## 3. Behavior

### 3.1 Triggers

The workflow runs on `push` to `main`, on a weekly `schedule`, and on
`branch_protection_rule`. It has no `pull_request` or `merge_group` trigger,
so it never appears among a pull request's checks. A newer push to `main`
cancels an older push run; scheduled and branch-protection runs are never
cancelled.

### 3.2 Least privilege

The workflow's default token is `read-all`. The one job names its scopes,
since a job-level block sets every unnamed scope to none: `contents: read`
and `actions: read` for the analysis, `security-events: write` to upload
SARIF, and `id-token: write` so the OpenSSF API can verify the run that
publishes. Checkout does not persist
credentials.

### 3.3 Output

The job runs `ossf/scorecard-action` with SARIF output and
`publish_results: true`, keeps the SARIF as a five-day artifact, and uploads
it to code scanning with `github/codeql-action/upload-sarif`.

### 3.4 Every action pinned

Each `uses:` names a full commit SHA with a `# vX.Y.Z` comment naming the
release it resolves to, read from that action's repository on 2026-10-21.
`dependabot.yml`'s `github-actions` entry proposes bumps.

### 3.5 Not a gate

`ci-gate`'s `needs` list does not include it, and no required check names
it. A low score is information for the owner; making a Scorecard check
required is a separate decision, and the managed workflow changes only
through the Statecraft profile.

## 4. Out of scope

- zizmor, an action-pin verifier and secret scanning. They arrive through
  the Statecraft profile.
- Acting on any finding Scorecard reports.
- A README badge.

## 5. Resolved decisions

**2026-10-27: ratified by the owner.** The owner confirmed every decision
below as written and directed ratification in this pull request; the owner
exception approved on its run is the ratification of record.

**D-1 (2026-10-21): a new spec, not an amendment of 019.** 019 is about the
release's artifacts; Scorecard reads the repository and runs outside the
release, so no 019 text changes.

**D-2 (2026-10-21): attribution.** The job shape follows the Scorecard
action's documented example and the Preloop project's `scorecard.yml`
(Apache-2.0). The file is re-implemented from that behaviour rather than
copied, its header credits both, and every pin was resolved again for this
repository.

## Verification

```verify:cli
# 3.1: triggers, and no pull-request trigger.
test -f .github/workflows/scorecard.yml
grep -qE '^  branch_protection_rule:' .github/workflows/scorecard.yml
grep -qE '^  schedule:' .github/workflows/scorecard.yml
grep -qF 'branches: [main]' .github/workflows/scorecard.yml
sh -c '! grep -qE "^  (pull_request|pull_request_target|merge_group):" .github/workflows/scorecard.yml'
# 3.2: least privilege.
grep -qE '^permissions: read-all$' .github/workflows/scorecard.yml
grep -qF 'persist-credentials: false' .github/workflows/scorecard.yml
grep -qE '^      contents: read' .github/workflows/scorecard.yml
grep -qE '^      actions: read' .github/workflows/scorecard.yml
grep -qE '^      security-events: write' .github/workflows/scorecard.yml
grep -qE '^      id-token: write' .github/workflows/scorecard.yml
# 3.3: the analysis publishes and uploads SARIF.
grep -qF 'uses: ossf/scorecard-action@' .github/workflows/scorecard.yml
grep -qF 'publish_results: true' .github/workflows/scorecard.yml
grep -qF 'uses: github/codeql-action/upload-sarif@' .github/workflows/scorecard.yml
# 3.4: every action is pinned to a commit SHA with its release named.
sh -c 'n=$(grep -cE "^[[:space:]]*-?[[:space:]]*uses:" .github/workflows/scorecard.yml); p=$(grep -cE "uses: [^ ]+@[0-9a-f]{40} # v[0-9]+\.[0-9]+\.[0-9]+$" .github/workflows/scorecard.yml); test "$n" -ge 4 && test "$n" = "$p"'
# 3.5: ci-gate does not need it.
sh -c '! grep -E "^[[:space:]]*needs:" .github/workflows/statecraft-ci.yml | grep -q scorecard'
```
