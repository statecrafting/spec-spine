---
id: "201-a-release-proves-itself-after-it-publishes"
title: "A release ships its attestation bundles, requires signing, and verifies what it published"
status: approved
kind: "tooling"
created: "2026-10-21"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "006-distribution"
  - "019-release-supply-chain-artifacts"
amends:
  - "019-release-supply-chain-artifacts"
amends_sections: ["3", "4"]
extends:
  # The bundle upload, the signing precondition and the post-publish
  # verification job are new steps and one new job in 006's release workflow.
  - { spec: "006-distribution", unit: { kind: file, path: ".github/workflows/release.yml" }, nature: additive }
  # The offline verification command and what the verify job asserts.
  - { spec: "006-distribution", unit: { kind: file, path: "docs/releasing.md" }, nature: additive }
summary: >
  Spec 019 records each archive's build-provenance attestation only in GitHub's
  attestation store and leaves the bundle as "a later additive refinement".
  This spec is that refinement and two more. Each archive's Sigstore bundle
  (`<archive>.intoto.jsonl`) is attached to the GitHub Release next to the
  archive, so an archive verifies offline. Signing is required on every `v*`
  tag: a build that cannot obtain the OIDC token or whose attestation step
  yields no bundle fails before any publish job runs. A final job in
  `release.yml` downloads what the release actually published and verifies it:
  every checksum, every attestation against both its attached bundle and the
  store, and the presence of every archive's SBOM and bundle. A failed check
  fails the release workflow.
intent:
  goal: "A published release carries everything needed to verify it offline, and the release workflow itself proves the published assets verify."
  non_goals:
    - "Signing or verifying the crates.io, npm or PyPI channels, which keep their own provenance"
    - "An opt-out that publishes an unsigned release"
---

# 201: A release ships its attestation bundles, requires signing, and verifies what it published

## 1. Purpose

Spec 019 gave every release archive a CycloneDX SBOM and a SLSA
build-provenance attestation. Three gaps remain:

- The attestation lives only in GitHub's attestation store. 019 §4 put
  "attaching the raw attestation bundle as a release asset" out of scope as a
  later additive refinement. Without the bundle, verifying an archive needs the
  store to be reachable.
- Nothing checks the release after it publishes. The `publish` job uploads
  `dist/*` and the workflow ends; a missing SBOM, a checksum that does not
  match, or an attestation that does not verify is found by a consumer, if at
  all.
- Signing is required in practice (the attestation step needs the OIDC token)
  but not stated as a precondition. A build whose attestation produced nothing
  usable has no explicit refusal before the publish jobs.

## 2. Territory

This spec establishes no new files. It amends spec 019 (§3, which lists the
per-archive steps, and §4, which excluded the bundle) and is an additive
extender of spec 006:

- **`.github/workflows/release.yml`** (additive): the signing precondition and
  the bundle step in the `build` job, the bundle in the uploaded artifact, and
  a new `verify-release` job after `publish`.
- **`docs/releasing.md`** (additive): the offline verification command and what
  `verify-release` asserts.

006 remains the owner of both files.

## 3. Behavior

### 3.1 The bundle is a release asset

For each of the five triples, the `build` job MUST write the attestation
bundle produced by `actions/attest-build-provenance` (its `bundle-path`
output) to `spec-spine-<tag>-<triple>.<ext>.intoto.jsonl`, named after the
archive it attests, and include it in the uploaded `archive-<triple>`
artifact. The `publish` job attaches it with the archives, unchanged, since it
already globs `dist/*`. A consumer verifies offline with:

```sh
gh attestation verify spec-spine-<tag>-<triple>.<ext> \
  --bundle spec-spine-<tag>-<triple>.<ext>.intoto.jsonl \
  --repo statecrafting/spec-spine
```

`<ext>` is `tar.gz` for the four Unix triples and `zip` for
`x86_64-pc-windows-msvc`.

### 3.2 Signing is required

Every `v*` tag is a signed release. In the `build` job, before the archive is
attested, a step MUST fail the job when the OIDC token cannot be requested
(`ACTIONS_ID_TOKEN_REQUEST_URL` or `ACTIONS_ID_TOKEN_REQUEST_TOKEN` absent).
After the attestation step, a step MUST fail the job when the bundle file is
absent or empty. Every publish job already `needs: build`, so a build that
fails here publishes nothing on any channel.

### 3.3 The release verifies what it published

A `verify-release` job MUST run after `publish` (`needs: publish`) with
`contents: read` and `attestations: read`, the second because
`gh attestation verify` without `--bundle` reads the attestation store
through the API. It downloads the assets of the GitHub Release
for the tag (not the build artifacts) and fails unless, for each of the five
triples:

1. the archive, its `.sha256`, its `.cdx.json` and its `.intoto.jsonl` are all
   present;
2. the archive's SHA-256 equals its `.sha256` sidecar;
3. `gh attestation verify` passes against the attached bundle (`--bundle`);
4. `gh attestation verify` passes against the attestation store (no
   `--bundle`).

A failed check fails the release workflow run. It does not unpublish anything:
the run's failure is the signal, and the remedy is a maintainer's, per
`docs/releasing.md`.

### 3.4 Pinned like the rest

Every action the new steps use is pinned to a full commit SHA with a
`# vX.Y.Z` comment, as the rest of `release.yml` is.

## 4. Out of scope

- The crates.io, npm and PyPI channels. PyPI already publishes PEP 740
  attestations and npm publishes with provenance; their verification is
  unchanged.
- Committing any bundle or SBOM: both stay release-time artifacts.
- Re-running a failed `verify-release` against a re-uploaded asset. A tag that
  fails verification is handled by the release runbook, not automatically.
- An opt-out that ships an unsigned release (D-1).

## 5. Resolved decisions

**2026-10-27: ratified by the owner.** The owner confirmed every decision
below as written and directed ratification in this pull request; the owner
exception approved on its run is the ratification of record.

**D-1 (2026-10-21): no opt-out from signing.** A repository variable that
skipped the signing precondition would make "signed" a property of
configuration rather than of every release, and a forked or misconfigured
run would publish unsigned archives silently. Signing costs nothing extra
here: the OIDC token is already requested for the 019 attestation. If signing
cannot run, the release does not publish.

**D-2 (2026-10-21): no `amends_verification`.** Spec 019 has no
`## Verification` block, so there is nothing to carry; this spec's block is
the first acceptance over 019's steps.

**D-3 (2026-10-21): verification runs inside `release.yml`.** The repository
has no release script; a separate workflow triggered on `release: published`
would not fail the run that published, which is the run a maintainer watches.

## Verification

```verify:cli
# 3.1: the bundle is named after its archive and travels in the uploaded artifact.
grep -qF 'steps.attest.outputs.bundle-path' .github/workflows/release.yml
grep -qF 'spec-spine-*.intoto.jsonl' .github/workflows/release.yml
# 3.2: the signing precondition refuses a build with no OIDC token and no bundle.
grep -qF 'ACTIONS_ID_TOKEN_REQUEST_URL' .github/workflows/release.yml
grep -qF 'refusing to publish an unsigned release' .github/workflows/release.yml
# 3.3: the post-publish job reads the published release and verifies both ways:
# against the bundle (item 3) and against the store with no --bundle (item 4).
grep -qE '^  verify-release:$' .github/workflows/release.yml
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "needs: publish"'
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "gh release download"'
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "gh attestation verify .*--bundle"'
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep "gh attestation verify" | grep -vqF -- "--bundle"'
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "sha256"'
sh -c 'sed -n "/^  verify-release:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "attestations: read"'
# 3.4: every action in the workflow is SHA-pinned with a version comment.
sh -c '! grep -E "^[[:space:]]*-?[[:space:]]*uses:" .github/workflows/release.yml | grep -vqE "@[0-9a-f]{40} # v[0-9]+\.[0-9]+\.[0-9]+$"'
# The runbook carries the offline command.
grep -qF -- '--bundle spec-spine-<tag>-<triple>.<ext>.intoto.jsonl' docs/releasing.md
```
