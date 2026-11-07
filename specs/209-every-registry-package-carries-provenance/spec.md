---
id: "209-every-registry-package-carries-provenance"
title: "Every npm package carries provenance, and npm and crates.io publish through trusted publishing first"
status: draft
implementation: complete
kind: tooling
created: "2026-11-07"
owner: "The spec-spine Authors"
risk: medium
summary: >
  Spec 201 put the registry channels out of scope on the claim that "npm
  publishes with provenance". It does not: `publish-npm` publishes with a
  long-lived `NPM_TOKEN`, without `--provenance` and without an OIDC token,
  and `spec-spine@0.32.0` carries registry signatures but no attestation.
  This spec makes npm provenance required on every package the release
  publishes, moves npm and crates.io to trusted publishing (OIDC) with the
  stored token kept only as a fallback, and makes a version that is already
  public without provenance fail the run instead of being skipped.
amends:
  - "201-a-release-proves-itself-after-it-publishes"
amends_sections: ["4"]
depends_on:
  - "006-distribution"
  - "201-a-release-proves-itself-after-it-publishes"
extends:
  # The publish-npm and publish-crates jobs of 006's release workflow.
  - { spec: "006-distribution", unit: { kind: file, path: ".github/workflows/release.yml" }, nature: additive }
  # The npm and crates.io setup paragraphs of the release runbook.
  - { spec: "006-distribution", unit: { kind: file, path: "docs/releasing.md" }, nature: additive }
intent:
  goal: "Every npm package a spec-spine release publishes carries a verifiable SLSA provenance attestation, and no registry upload depends on a long-lived token when trusted publishing works."
  non_goals:
    - "Republishing or deprecating versions already on npm without provenance (0.32.0 and earlier stay as published)"
    - "Changing the PyPI channel, which already publishes through trusted publishing with PEP 740 attestations"
    - "A badge or attestation for crates.io, which records none"
---

# 209: Every npm package carries provenance, and npm and crates.io publish through trusted publishing first

## 1. Purpose

Measured on 2026-11-07:

- `npm view spec-spine@0.32.0 dist` lists `integrity` and two registry
  `signatures` and no `attestations`; the five `@spec-spine/cli-*` platform
  packages are the same. npm shows no provenance for any spec-spine version.
- `publish-npm` in `.github/workflows/release.yml` has no `permissions` block
  (the workflow default is `contents: read`, so no `id-token: write`), runs
  `npm publish "${dir}" --access public` without `--provenance`, and
  authenticates with the repository secret `NPM_TOKEN`.
- `publish-crates` authenticates with the repository secret
  `CARGO_REGISTRY_TOKEN`.
- Spec 201 §4 states "npm publishes with provenance", which was never true.

Provenance proves where a package was built and from which commit. It does
not prove the code is safe; review and the gate still do that. Its value
depends on it being present on every version, so a consumer can treat its
absence as a signal.

## 2. Territory

This spec establishes no new files. It amends spec 201 §4 (the claim about
npm) and is an additive extender of spec 006 for
`.github/workflows/release.yml` (the `publish-npm` and `publish-crates` jobs)
and `docs/releasing.md` (the npm and crates.io setup). 006 remains their owner.

## 3. Behavior

### 3.1 npm: trusted publishing first, provenance always

1. `publish-npm` MUST run in the `npm-publish` Environment with
   `permissions: { contents: read, id-token: write }`.
2. It MUST install npm 11.21.0 (trusted publishing needs npm 11.5.1 or later),
   pinned by the registry's SHA-512 integrity checked before installation.
3. Every `npm publish` MUST pass `--provenance`.
4. Authentication is npm trusted publishing (OIDC). When npm refuses the
   exchange (npm/cli#9969: npm rejects GitHub's immutable OIDC subject, which
   this repository uses), npm falls back to the `NPM_TOKEN` secret, written to
   a user `.npmrc` only when the secret exists. `--provenance` signs with the
   job's OIDC token on either path.
5. The job no longer skips when `NPM_TOKEN` is absent. With neither trusted
   publishing nor the token, it fails.

### 3.2 npm: present means attested

Before each package, the job reads the version from the registry:

1. absent: it is published, and the job then waits, bounded, until the
   version is visible with a `dist.attestations.provenance.predicateType`
   that starts with `https://slsa.dev/provenance/`; otherwise it fails;
2. present with such an attestation: it is skipped, so rerunning a tag stays
   idempotent;
3. present without one: the job fails and republishes nothing. A version
   published outside the run cannot be recorded as released by it.

### 3.3 crates.io: trusted publishing first

`publish-crates` MUST run in the `crates-publish` Environment with
`id-token: write`, exchange the job's OIDC token through
`rust-lang/crates-io-auth-action` (pinned by commit), and publish with the
short-lived token it returns. When the exchange fails (no trusted publisher
configured for a crate), it falls back to `CARGO_REGISTRY_TOKEN`; with
neither, it fails.

### 3.4 Pinned like the rest

Every action the jobs use is pinned to a full commit SHA with a `# vX.Y.Z`
comment, as the rest of `release.yml` is.

## 4. Out of scope

- Removing `NPM_TOKEN` and `CARGO_REGISTRY_TOKEN`. That follows once npm fixes
  npm/cli#9969 and a release has published every crate and package through
  trusted publishing; it is a separate change.
- Verification of the registry channels inside `verify-release` (201 §3.3):
  3.2 already refuses an unattested npm version at publication.

## 5. Resolved decisions

**2026-11-07: owner direction.** The owner directed that npm provenance be
standardized across every package statecrafting releases (spec-spine, rustev
and later statecraft-cli), with OIDC trusted publishing first and a token
fallback while npm/cli#9969 is open, and that a version public without
provenance fail the release. rustev records the same rule as its decision
A-87. Ratification of this spec stays the owner's act.

**One-time owner setup before the next tag.** On npmjs.com, a trusted
publisher for `spec-spine` and each `@spec-spine/cli-*` package naming
`statecrafting/spec-spine`, workflow `release.yml`, environment
`npm-publish`; on crates.io, a trusted publisher for `spec-spine-types`,
`spec-spine-core` and `spec-spine-cli` naming the same repository, workflow
and environment `crates-publish`. `NPM_TOKEN` and `CARGO_REGISTRY_TOKEN` stay
as repository secrets until 3's fallback is removed.

## Verification

```verify:cli
# 3.1: publish-npm runs in npm-publish with an id-token, the pinned npm, --provenance, and no token gate.
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "environment: npm-publish"'
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "id-token: write"'
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "npm-11.21.0.tgz"'
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "npm publish .*--provenance"'
sh -c '! sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "steps.gate.outputs.present"'
# 3.2: present means attested; an unattested version fails.
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "slsa.dev/provenance/"'
sh -c 'sed -n "/^  publish-npm:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "present without provenance"'
# 3.3: publish-crates exchanges its OIDC token first.
sh -c 'sed -n "/^  publish-crates:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "environment: crates-publish"'
sh -c 'sed -n "/^  publish-crates:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -q "id-token: write"'
sh -c 'sed -n "/^  publish-crates:/,/^  [a-z-]*:$/p" .github/workflows/release.yml | grep -qE "rust-lang/crates-io-auth-action@[0-9a-f]{40} # v[0-9]+"'
# 3.4: every action in the workflow is SHA-pinned with a version comment.
sh -c '! grep -E "^[[:space:]]*-?[[:space:]]*uses:" .github/workflows/release.yml | grep -vqE "@[0-9a-f]{40} # v[0-9]+\.[0-9]+\.[0-9]+$"'
# The runbook names the trusted publisher setup and the provenance check.
grep -qF 'environment npm-publish' docs/releasing.md
grep -qF 'dist.attestations.provenance.predicateType' docs/releasing.md
```
