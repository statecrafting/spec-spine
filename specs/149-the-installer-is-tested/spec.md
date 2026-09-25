---
id: "149-the-installer-is-tested"
title: "The installer is tested"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Spec 006 is one of the four specs the pre-release sweep still reports
  `exempt`, because `install.sh`, the tag-gated release pipeline and npm
  publication "run only in a release" (139 3.3). Two of its three channels do
  not: `install.sh` is a shell script whose every decision (platform, tag,
  checksum, attestation policy, install directory) can be exercised offline
  against a local fixture release, and the npm shim already has an offline
  unit test and smoke test. This spec adds an offline test of `install.sh`,
  gives 006 an executable acceptance that runs it and the npm shim's tests,
  and removes 006's line from the legacy ledger. The release pipeline's
  publication steps stay unexercised by it, and it says so.
implementation: pending
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-distribution"
  - "089-nothing-reruns-a-merged-acceptance"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
amends:
  - "006-distribution"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
amends_verification:
  - "006-distribution"
establishes:
  - { kind: file, path: "scripts/test-install.sh", planned: true }
extends:
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: corrective }
---

# 149: The installer is tested

## 1. Purpose

The sweep of `7ce4935f` ended `passed=143 failed=0 exempt=4`. The ledger's
reason for 006 is that its territory runs only in a release. That is true of
`release.yml`'s publish jobs, and not of `install.sh` or `npm/`:

- `install.sh` reaches the network only through `curl` or `wget`, and the
  platform only through `uname`. With both replaced by stubs on `PATH`, every
  branch runs offline.
- `npm/` already has `npm test` (the platform map) and `npm run smoke` (pack,
  install and launch through the shim), both offline, both run by the release
  qualification but by no acceptance block.

## 2. Territory

- `scripts/test-install.sh` (new): the offline installer test.
- `scripts/verify-sweep.sh`: 006's ledger line is removed.

## 3. Behavior

### 3.1 The installer runs against a fixture release

`scripts/test-install.sh` MUST build a fixture release directory holding one
archive (`spec-spine-<tag>-<triple>.tar.gz` containing a stub `spec-spine`
that prints a version) and its `.sha256` sidecar, put a `curl` stub on `PATH`
that serves those URLs from the directory (and fails any other), and run
`install.sh` with `SPEC_SPINE_VERSION`, `SPEC_SPINE_BIN_DIR` and
`SPEC_SPINE_SKIP_ATTESTATION=1`. It MUST assert, each as its own case with its
exit code:

1. a matching checksum installs the binary into `SPEC_SPINE_BIN_DIR`, exit 0,
   and the installed binary runs;
2. a sidecar that does not match refuses, exit 1, naming the checksum, and
   installs nothing;
3. an archive without the binary refuses, exit 1;
4. `uname` stubs reporting an unsupported OS and an unsupported architecture
   each refuse, exit 1, naming it;
5. `SPEC_SPINE_REQUIRE_ATTESTATION=1` with no `gh` on `PATH` refuses, exit 1;
6. `latest` resolves the tag from the stubbed releases API response.

The triple under test is the host's, from the same `uname` mapping the
installer uses, so the test runs on every sweep host.

### 3.2 006 has an executable acceptance

This spec's block replaces 006's (`amends_verification`): it runs
`scripts/test-install.sh`, `npm test` and `npm run smoke`.

### 3.3 The ledger shrinks (amends 139 3.3)

`006-distribution` MUST be removed from `legacy_ledger()`; `LEDGER_CLOSED_AT`
is unchanged (the ledger only shrinks). The sweep then reports 006 `passed`
or `failed`, never `exempt`.

## 4. Out of scope

- `release.yml`'s build, attestation and publish jobs, and the published
  channels: they run in a release and are witnessed by the release record's
  channel checks, not by acceptance.
- musl detection: it reads `/lib/ld-musl-*` and `ldd`, which a stub cannot
  redirect without editing the installer.

## 5. Resolved decisions

None yet.

## Verification

```verify:cli
sh scripts/test-install.sh
sh -c 'cd npm && npm test'
sh -c 'cd npm && npm run smoke'
sh -c '! grep -qx "006-distribution" scripts/verify-sweep.sh'
```
